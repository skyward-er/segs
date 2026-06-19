use std::sync::Arc;

use egui::{Context, Id, IdMap, Key, KeyboardShortcut, Modifiers, mutex::Mutex};

/// Contains all keyboard shortcuts added by the UI.
///
/// [`ShortcutHandler`] is used to register shortcuts and consume them.
pub struct ShortcutHandler {
    /// The egui context. Needed to consume shortcuts.
    ctx: Context,

    current_term: u32,
    active_leases: IdMap<(u32, Box<dyn ShortcutLease>)>,

    shortcut_state: ShortcutAppState,
}

impl ShortcutHandler {
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            current_term: 0,
            active_leases: IdMap::default(),
            shortcut_state: ShortcutAppState::default(),
        }
    }

    pub fn move_term(&mut self) {
        self.current_term = self.current_term.wrapping_add(1);

        // first remove all leases that are no longer active and call their `once_ended` method
        let mut ids_to_remove = Vec::new();
        for (id, (term, lease)) in self.active_leases.iter_mut() {
            if self.current_term.wrapping_add(term.wrapping_neg()) > 1 {
                lease.once_ended(&mut self.shortcut_state);
                ids_to_remove.push(*id);
            }
        }
        for id in ids_to_remove {
            self.active_leases.remove(&id);
        }

        // then call `while_active` for all active leases
        for (_, (_, lease)) in self.active_leases.iter_mut() {
            lease.while_active(&mut self.shortcut_state);
        }
    }

    pub fn capture_actions<A>(
        &mut self,
        id: impl Into<Id>,
        lease: Box<dyn ShortcutLease>,
        function: impl FnOnce(&ShortcutAppState) -> Vec<(Modifiers, Key, A)>,
    ) -> Option<A> {
        self.active_leases
            .insert(id.into(), (self.current_term, lease));
        let mut captured_action: Option<A> = None;
        let actions = function(&self.shortcut_state);
        for (modifier, key, action) in actions {
            let shortcut = KeyboardShortcut::new(modifier, key);
            captured_action = captured_action.or(self
                .ctx
                .input_mut(|i| i.consume_shortcut(&shortcut).then_some(action)));
        }

        captured_action
    }

}

#[derive(Debug, Clone, Default)]
pub struct ShortcutAppState {
    pub is_command_switch_active: bool,
}

pub trait ShortcutLease: Send + Sync {
    /// Called when the lease is active.
    #[allow(unused_variables)]
    fn while_active(&mut self, state: &mut ShortcutAppState) {}

    /// Called when the lease ends.
    #[allow(unused_variables)]
    fn once_ended(&mut self, state: &mut ShortcutAppState) {}
}

impl ShortcutLease for () {}

pub trait ShortcutHandlerExt {
    fn shortcuts(&self) -> Arc<Mutex<ShortcutHandler>>;
}

impl ShortcutHandlerExt for Context {
    fn shortcuts(&self) -> Arc<Mutex<ShortcutHandler>> {
        self.memory_mut(|w| {
            if let Some(arc) = w.data.get_temp("shortcut_handler".into()) {
                arc
            } else {
                let handler = Arc::new(Mutex::new(ShortcutHandler::new(self.clone())));
                w.data
                    .insert_temp("shortcut_handler".into(), handler.clone());
                handler
            }
        })
    }
}
