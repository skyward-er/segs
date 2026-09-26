use std::sync::Arc;

use egui::{Align, FontId, Galley, Ui, Vec2};

use crate::ui::components::auto_size;

const MIN_AUTO_SIZE: i32 = 12;
const MAX_AUTO_SIZE: i32 = 96;

/// Returns centered, wrapped labels sharing one fitted font size within the supplied text area.
/// Oversized words are ellipsized at fixed size or the 12-point automatic minimum; callers clip height.
/// Automatic fitting uses shaped reference measurements and at most seven arithmetic-only passes.
pub(super) fn layout(
    ui: &Ui,
    states: &[(i64, &str)],
    available: Vec2,
    auto_size: bool,
    text_size: i64,
) -> Vec<Arc<Galley>> {
    // Measure each word once, preserving native measurements for explicitly fixed text
    let measurement_size = if auto_size {
        auto_size::REFERENCE_SIZE
    } else {
        text_size.clamp(2, 500) as f32
    };
    let font = FontId::monospace(measurement_size);
    let space = auto_size::layout(ui, " ".to_owned(), font.clone(), Align::LEFT, false).size();
    let names: Vec<_> = states
        .iter()
        .map(|&(value, name)| {
            tokenize(
                ui,
                if name.is_empty() {
                    value.to_string()
                } else {
                    name.to_owned()
                },
                &font,
                space.y,
            )
        })
        .collect();

    // Search only numeric widths and line counts, leaving room for raster-size rounding
    let available = available.max(Vec2::ZERO);
    let fitting = if auto_size { available * 0.95 } else { available };
    let size = if auto_size {
        let mut minimum = MIN_AUTO_SIZE;
        let mut maximum = MAX_AUTO_SIZE;
        let mut fitted = MIN_AUTO_SIZE;
        while minimum <= maximum {
            let candidate = (minimum + maximum) / 2;
            let reference_area = fitting * (measurement_size / candidate as f32);
            if names.iter().all(|name| name.fits(reference_area, space.x)) {
                fitted = candidate;
                minimum = candidate + 1;
            } else {
                maximum = candidate - 1;
            }
        }
        fitted as f32
    } else {
        measurement_size
    };

    // Construct the final strings once and rasterize only the chosen bucket or fixed size
    let reference_width = fitting.x * measurement_size / size;
    let needs_ellipsis = names.iter().any(|name| {
        name.tokens
            .iter()
            .any(|token| matches!(token, Token::Word { measured, .. } if measured.size().x > reference_width))
    });
    let ellipsis_width = if needs_ellipsis {
        auto_size::layout(ui, "…".to_owned(), font, Align::LEFT, false).size().x
    } else {
        0.
    };
    let mut labels: Vec<_> = names
        .iter()
        .map(|name| {
            let output = compose(name, reference_width, space.x, ellipsis_width);
            auto_size::layout(ui, output, FontId::monospace(size), Align::Center, auto_size)
        })
        .collect();

    // Correct final rounding uniformly without changing the shared size or minimum-size policy
    if auto_size {
        let scale = labels.iter().fold(1_f32, |scale, label| {
            let bounds = label.size();
            let width_scale = if bounds.x > 0. { available.x / bounds.x } else { 1. };
            let height_scale = if bounds.y > 0. { available.y / bounds.y } else { 1. };
            scale.min(width_scale).min(height_scale)
        });
        let scale = scale.max(MIN_AUTO_SIZE as f32 / size);
        if scale < 1. {
            labels = labels
                .into_iter()
                .map(|label| auto_size::scaled(label, scale))
                .collect();
        }
    }
    labels
}

impl NameTokens {
    /// Returns whether every word and the greedy wrapped line count fit in reference-size coordinates.
    fn fits(&self, available: Vec2, space: f32) -> bool {
        // Simulate wrapping without constructing strings or requesting any font layouts
        let mut line_width = 0.;
        let mut occupied = false;
        let mut lines = 1;
        for token in &self.tokens {
            match token {
                Token::Newline => {
                    lines += 1;
                    line_width = 0.;
                    occupied = false;
                }
                Token::Word { measured, .. } => {
                    let width = measured.size().x;
                    if width > available.x {
                        return false;
                    }
                    if occupied {
                        if line_width + space + width > available.x {
                            lines += 1;
                            line_width = 0.;
                        } else {
                            line_width += space;
                        }
                    }
                    line_width += width;
                    occupied = true;
                }
            }
        }
        lines as f32 * self.line_height <= available.y
    }
}

/// Returns a name with shaped word measurements and explicit newlines from one character scan.
/// Other whitespace is collapsed; the retained galleys are unscaled for accurate ellipsis metrics.
fn tokenize(ui: &Ui, text: String, font: &FontId, line_height: f32) -> NameTokens {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut line_height = line_height;

    // Flush each complete word at whitespace or the end-of-string sentinel
    for (index, character) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
        if character.is_whitespace() {
            if let Some(start) = start.take() {
                let range = start..index;
                let measured = auto_size::layout(ui, text[range.clone()].to_owned(), font.clone(), Align::LEFT, false);
                line_height = line_height.max(measured.size().y);
                tokens.push(Token::Word { range, measured });
            }
            if character == '\n' {
                tokens.push(Token::Newline);
            }
        } else {
            start.get_or_insert(index);
        }
    }
    NameTokens {
        text,
        tokens,
        line_height,
    }
}

/// Returns the final greedy-wrapped label using reference-size widths and already shaped words.
/// Oversized words occupy their own truncated line; explicit newlines are always preserved.
fn compose(name: &NameTokens, available_width: f32, space: f32, ellipsis_width: f32) -> String {
    let mut output = String::with_capacity(name.text.len());
    let mut line_width = 0.;
    let mut occupied = false;
    let mut oversized = false;

    // Write the chosen wrapping once, eliding only words that cannot fit on their own
    for token in &name.tokens {
        let Token::Word { range, measured } = token else {
            output.push('\n');
            line_width = 0.;
            occupied = false;
            oversized = false;
            continue;
        };
        let word = &name.text[range.clone()];
        let width = measured.size().x;
        if occupied {
            if oversized || width > available_width || line_width + space + width > available_width {
                output.push('\n');
                line_width = 0.;
            } else {
                output.push(' ');
                line_width += space;
            }
        }
        oversized = width > available_width;
        if oversized {
            append_elided(&mut output, word, measured, available_width, ellipsis_width);
        } else {
            output.push_str(word);
            line_width += width;
        }
        occupied = true;
    }
    output
}

/// Appends a fitting prefix plus ellipsis using one scan of already measured glyph positions.
/// Writes nothing if the ellipsis cannot fit; byte boundaries always follow complete Unicode characters.
fn append_elided(output: &mut String, word: &str, measured: &Galley, width: f32, ellipsis_width: f32) {
    if ellipsis_width > width {
        return;
    }
    let mut end = 0;
    if let Some(row) = measured.rows.first() {
        for ((index, character), glyph) in word.char_indices().zip(&row.glyphs) {
            if glyph.max_x() + ellipsis_width > width {
                break;
            }
            end = index + character.len_utf8();
        }
    }
    output.push_str(&word[..end]);
    output.push('…');
}

/// One name with measurements shared by all arithmetic fitting passes in the current frame.
struct NameTokens {
    /// Original name or integer fallback, providing storage for word ranges.
    text: String,
    /// Ordered measured words and explicit line breaks, without redundant whitespace tokens.
    tokens: Vec<Token>,
    /// Conservative line height in reference-size logical points, including fallback glyphs.
    line_height: f32,
}

/// A shaped whitespace-delimited word or an explicit line break.
enum Token {
    /// One word whose dimensions and glyph positions are shared with egui's layout cache.
    Word {
        /// UTF-8 byte range containing the complete word in its owning name.
        range: std::ops::Range<usize>,
        /// Unscaled shaped word at the measurement size.
        measured: Arc<Galley>,
    },
    /// A newline preserved even when consecutive or at a name's boundary.
    Newline,
}
