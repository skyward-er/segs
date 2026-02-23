//! CSS-like cascading style system for egui.
//!
//! # Design
//!
//! Styles are stored on a thread-local stack. Reading the current style
//! is a plain `Vec::last()` — no locks, no hashmap, no `ctx.data` access.
//! Scoped overrides push/pop the stack at scope boundaries only, not per
//! widget.
//!
//! Two root styles are maintained — one for dark mode, one for light mode.
//! Call `AppStyle::sync(ui)` once at the top of each frame to push the
//! correct root onto the stack based on egui's active theme. The stack is
//! then ready for the entire frame.
//!
//! # Usage
//!
//! ```rust
//! // 1. Define your dark and light base styles and set them up once:
//! AppStyle::setup(Style::dark(), Style::light());
//!
//! // 2. At the top of your root UI function, sync with egui's active theme:
//! AppStyle::sync(ui);
//!
//! // 3. Use anywhere in UI code via the StyleExt trait:
//! use crate::style::StyleExt as _;
//!
//! fn my_widget(ui: &mut egui::Ui) {
//!     let s = ui.app_style();
//!     ui.label(egui::RichText::new("Hello").color(s.primary));
//!
//!     ui.with_style_override(|s| s.primary = egui::Color32::RED, |ui| {
//!         let s = ui.app_style();
//!         ui.label(egui::RichText::new("Red").color(s.primary));
//!     });
//! }
//! ```
//!
//! # Assumptions
//!
//! egui UI traversal is always single-threaded — `&mut Ui` is `!Send`,
//! so two threads can never walk the same UI tree simultaneously.
//! This lets us use a plain thread-local stack with zero synchronization.

use std::{cell::RefCell, sync::Arc};

use egui::{Context, InnerResponse, Ui};

use super::Style;

// Convenience: `Style::default()` follows egui's default theme (dark).
impl Default for Style {
    fn default() -> Self {
        Self::dark()
    }
}

// -----------------------------------------------------------------------------
// Internal state: two root styles (dark + light) and the per-frame stack.
//
// Both live in the same thread_local block so they are always in sync and
// accessed with a single TLS read.
//
// STACK invariant: always has at least one entry during a UI pass (pushed by
// `AppStyle::sync` at frame start, popped after the root UI closure returns).
// -----------------------------------------------------------------------------

struct State {
    dark: Arc<Style>,
    light: Arc<Style>,
    stack: Vec<Arc<Style>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            dark: Arc::new(Style::dark()),
            light: Arc::new(Style::light()),
            stack: Vec::new(),
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

// -----------------------------------------------------------------------------
// Public API: AppStyle
// -----------------------------------------------------------------------------

/// Entry point for the app style system.
pub struct AppStyle;

impl AppStyle {
    /// Install the application's dark and light base styles.
    /// Call once at app startup, before any UI code runs.
    pub fn setup(dark: Style, light: Style) {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.dark = Arc::new(dark);
            state.light = Arc::new(light);
        });
    }

    /// Synchronise the stack root with egui's currently active theme.
    ///
    /// Call this **once, at the very top of your root UI function**, before
    /// any widgets are added. It pushes the correct base style (dark or light)
    /// onto the stack, making it visible to all descendants for this frame.
    ///
    /// The corresponding pop happens automatically via the returned
    /// `FrameGuard` which you must keep alive for the duration of the
    /// frame:
    ///
    /// ```rust
    /// fn ui(&mut self, ctx: &egui::Context) {
    ///     let _style_guard = AppStyle::sync(ctx);
    ///     // ... all your UI code here ...
    /// }
    /// ```
    #[must_use = "keep the guard alive for the entire frame"]
    pub fn sync(ctx: &Context) -> FrameGuard {
        let root = STATE.with(|s| {
            let state = s.borrow();
            match ctx.theme() {
                egui::Theme::Dark => Arc::clone(&state.dark),
                egui::Theme::Light => Arc::clone(&state.light),
            }
        });
        Self::push(root);
        FrameGuard
    }

    /// Read the currently active style.
    /// Delegated to by `ui.app_style()`.
    #[inline]
    pub fn current() -> Arc<Style> {
        STATE.with(|s| {
            s.borrow()
                .stack
                .last()
                .cloned()
                // Fallback: if sync() was not called yet, return the dark base.
                .unwrap_or_else(|| Arc::new(Style::dark()))
        })
    }

    /// Update the dark base style.
    /// Takes effect at the next `sync()` call (i.e. next frame).
    pub fn set_dark(style: Style) {
        STATE.with(|s| s.borrow_mut().dark = Arc::new(style));
    }

    /// Update the light base style.
    /// Takes effect at the next `sync()` call (i.e. next frame).
    pub fn set_light(style: Style) {
        STATE.with(|s| s.borrow_mut().light = Arc::new(style));
    }

    // ---- Internal stack operations ------------------------------------------

    #[inline]
    fn push(style: Arc<Style>) {
        STATE.with(|s| s.borrow_mut().stack.push(style));
    }

    #[inline]
    fn pop() {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            // Always leave at least one entry so current() never returns None
            // during a frame (the root pushed by sync()).
            if state.stack.len() > 1 {
                state.stack.pop();
            }
        });
    }

    #[inline]
    fn mutate_top(f: impl FnOnce(&mut Style)) {
        STATE.with(|s| {
            if let Some(top) = s.borrow_mut().stack.last_mut() {
                f(Arc::make_mut(top));
            }
        });
    }
}

// -----------------------------------------------------------------------------
// RAII guards
// -----------------------------------------------------------------------------

/// Returned by `AppStyle::sync`. Pops the frame root on drop.
/// Keep alive for the entire frame (i.e. bind to `let _guard = ...`).
pub struct FrameGuard;

impl Drop for FrameGuard {
    #[inline]
    fn drop(&mut self) {
        // Pop all the way back to empty — any leaked scope guards would have
        // already panicked (PopGuard is dropped before FrameGuard in LIFO order),
        // so by the time we get here the stack should already be at depth 1.
        STATE.with(|s| s.borrow_mut().stack.clear());
    }
}

/// RAII guard for `with_style_override` / `with_style` scopes.
/// Pops exactly one entry on drop. Panic-safe.
struct PopGuard;

impl Drop for PopGuard {
    #[inline]
    fn drop(&mut self) {
        AppStyle::pop();
    }
}

// -----------------------------------------------------------------------------
// Extension trait - the public UI API.
//
// Import with `use crate::style::StyleExt as _;` and the methods become
// available on every `&mut egui::Ui`, mirroring egui's own `ui.style()` API.
// -----------------------------------------------------------------------------

pub trait CtxStyleExt {
    /// Read the current app style.
    ///
    /// Returns an `Arc` - deref it for field access.
    /// Read once per scope, use the result for all widgets in that scope:
    ///
    /// ```rust
    /// let s = ui.app_style();
    /// ui.label(RichText::new("a").color(s.primary));
    /// ui.label(RichText::new("b").color(s.on_surface));
    /// ```
    fn app_style(&self) -> Arc<Style>;

    /// Mutate the current style in place (clone-on-write).
    /// Affects all widgets added to this `Ui` after this call.
    /// Does not affect sibling or parent scopes.
    fn app_style_mut(&mut self, f: impl FnOnce(&mut Style));
}

pub trait UiStyleExt: CtxStyleExt {
    /// Scoped style override — inherits current style, applies overrides.
    ///
    /// All widgets inside `add_contents` see the modified style.
    /// The previous style is restored automatically on exit, even on panic.
    /// Mirrors a CSS class applied to a container.
    fn with_style_override<R>(
        &mut self,
        override_fn: impl FnOnce(&mut Style),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R>;

    /// Full style replacement for the duration of `add_contents`.
    /// No inheritance — the given style is used as-is.
    fn with_style<R>(
        &mut self,
        style: impl Into<Arc<Style>>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R>;

    /// Restore the theme's base style (dark or light root) for the duration
    /// of `add_contents`, discarding any intermediate overrides.
    fn with_base_style<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;
}

impl CtxStyleExt for Context {
    #[inline]
    fn app_style(&self) -> Arc<Style> {
        AppStyle::current()
    }

    #[inline]
    fn app_style_mut(&mut self, f: impl FnOnce(&mut Style)) {
        AppStyle::mutate_top(f);
    }
}

impl CtxStyleExt for Ui {
    #[inline]
    fn app_style(&self) -> Arc<Style> {
        AppStyle::current()
    }

    #[inline]
    fn app_style_mut(&mut self, f: impl FnOnce(&mut Style)) {
        AppStyle::mutate_top(f);
    }
}

impl UiStyleExt for Ui {
    fn with_style_override<R>(
        &mut self,
        override_fn: impl FnOnce(&mut Style),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        // Clone the current Arc (atomic increment), then apply overrides.
        // Arc::make_mut clones Style only if the Arc is shared.
        let mut inherited = AppStyle::current();
        override_fn(Arc::make_mut(&mut inherited));
        AppStyle::push(inherited);
        let _guard = PopGuard;
        self.scope(add_contents)
    }

    fn with_style<R>(
        &mut self,
        style: impl Into<Arc<Style>>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        AppStyle::push(style.into());
        let _guard = PopGuard;
        self.scope(add_contents)
    }

    fn with_base_style<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // Re-read the theme root (dark or light) from STATE directly,
        // bypassing whatever overrides are currently on the stack.
        let base = STATE.with(|s| {
            let state = s.borrow();
            match self.ctx().theme() {
                egui::Theme::Dark => Arc::clone(&state.dark),
                egui::Theme::Light => Arc::clone(&state.light),
            }
        });
        AppStyle::push(base);
        let _guard = PopGuard;
        self.scope(add_contents)
    }
}
