use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use egui::{Align, Color32, FontId, Galley, Ui, Vec2, text::LayoutJob};

use crate::ui::components::centered_value::{self, FontMetrics};

/// Returns centered, wrapped labels sharing one fitted font size within the supplied text area.
/// Oversized words are ellipsized at fixed size or the 12-point automatic minimum; callers clip height.
/// Cache hits do no tokenization or measurement; misses use at most seven fitting passes and one fallback.
pub(super) fn layout(
    ui: &Ui,
    states: &[(i64, &str)],
    available: Vec2,
    auto_size: bool,
    text_size: i64,
) -> Arc<Vec<Arc<Galley>>> {
    // Cache complete wrapped layouts using only geometry, configuration, and font inputs
    let mut hasher = DefaultHasher::new();
    states.hash(&mut hasher);
    available.x.to_bits().hash(&mut hasher);
    available.y.to_bits().hash(&mut hasher);
    auto_size.hash(&mut hasher);
    text_size.hash(&mut hasher);
    let hash = hasher.finish();
    let fonts = centered_value::font_metrics(ui);
    let id = ui.id().with("state_machine_wrapped_labels");
    if let Some(cache) = ui.data(|data| data.get_temp::<LayoutCache>(id))
        && cache.hash == hash
        && cache.fonts == fonts
    {
        return cache.labels;
    }

    // Tokenize once per cache miss and share the tokens across the bounded fitting search
    let names: Vec<_> = states
        .iter()
        .map(|&(value, name)| {
            tokenize(if name.is_empty() {
                value.to_string()
            } else {
                name.to_owned()
            })
        })
        .collect();
    let labels = if auto_size {
        let mut minimum = 12;
        let mut maximum = 96;
        let mut fitted = None;
        // Searching 85 integer sizes requires at most seven complete fitting passes
        while minimum <= maximum {
            let candidate = (minimum + maximum) / 2;
            if let Some(labels) = compose(ui, &names, candidate as f32, available, false) {
                fitted = Some(labels);
                minimum = candidate + 1;
            } else {
                maximum = candidate - 1;
            }
        }
        // Successful candidate galleys are already final and need no additional measurement
        fitted.unwrap_or_else(|| compose(ui, &names, 12., available, true).expect("ellipsis layout always completes"))
    } else {
        compose(ui, &names, text_size.clamp(2, 500) as f32, available, true).expect("ellipsis layout always completes")
    };
    let labels = Arc::new(labels);
    ui.data_mut(|data| {
        data.insert_temp(
            id,
            LayoutCache {
                hash,
                fonts,
                labels: Arc::clone(&labels),
            },
        )
    });
    labels
}

/// Splits a name into word ranges and explicit newlines in a single character scan.
/// Other whitespace is collapsed and punctuation remains part of its word.
fn tokenize(text: String) -> NameTokens {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, character) in text.char_indices() {
        if character.is_whitespace() {
            if let Some(start) = start.take() {
                tokens.push(Token::Word(start..index));
            }
            if character == '\n' {
                tokens.push(Token::Newline);
            }
        } else {
            start.get_or_insert(index);
        }
    }
    if let Some(start) = start {
        tokens.push(Token::Word(start..text.len()));
    }
    NameTokens { text, tokens }
}

/// Returns centered galleys from one linear pass over each label's tokens.
/// Without ellipsis, returns `None` immediately when a word or completed label cannot fit.
/// With ellipsis, oversized words occupy their own truncated line and height is left to caller clipping.
fn compose(ui: &Ui, names: &[NameTokens], size: f32, available: Vec2, ellipsize: bool) -> Option<Vec<Arc<Galley>>> {
    // Measure shared spacing once per candidate rather than once per label or word
    let font = FontId::monospace(size);
    let space = measure(ui, " ".to_owned(), &font, Align::LEFT).size().x;
    let ellipsis = ellipsize.then(|| measure(ui, "…".to_owned(), &font, Align::LEFT).size().x);
    let mut labels = Vec::with_capacity(names.len());
    for name in names {
        let mut output = String::with_capacity(name.text.len());
        let mut line_width = 0.;
        let mut occupied = false;
        let mut oversized = false;
        // Each token is measured once and only the current line's width is retained
        for token in &name.tokens {
            let Token::Word(range) = token else {
                output.push('\n');
                line_width = 0.;
                occupied = false;
                oversized = false;
                continue;
            };
            let word = &name.text[range.clone()];
            let measured = measure(ui, word.to_owned(), &font, Align::LEFT);
            let width = measured.size().x;
            if width > available.x && !ellipsize {
                return None;
            }
            if occupied {
                if oversized || width > available.x || line_width + space + width > available.x {
                    output.push('\n');
                    line_width = 0.;
                } else {
                    output.push(' ');
                    line_width += space;
                }
            }
            oversized = width > available.x;
            if oversized {
                append_elided(
                    &mut output,
                    word,
                    &measured,
                    available.x,
                    ellipsis.expect("ellipsis width was measured"),
                );
            } else {
                output.push_str(word);
                line_width += width;
            }
            occupied = true;
        }

        // Explicit line breaks are final; egui must never introduce its own word splits
        let galley = measure(ui, output, &font, Align::Center);
        if !ellipsize && (galley.size().x > available.x || galley.size().y > available.y) {
            return None;
        }
        labels.push(galley);
    }
    Some(labels)
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

/// Returns a galley honoring only explicit newlines, with the requested horizontal alignment.
fn measure(ui: &Ui, text: String, font: &FontId, alignment: Align) -> Arc<Galley> {
    let mut job = LayoutJob::simple(text, font.clone(), Color32::WHITE, f32::INFINITY);
    job.halign = alignment;
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

/// One owned name tokenized once for all candidate sizes in a cache miss.
struct NameTokens {
    /// Original name or integer fallback, providing storage for word ranges.
    text: String,
    /// Ordered words and explicit line breaks, without redundant whitespace tokens.
    tokens: Vec<Token>,
}

/// A whitespace-delimited word or an explicit line break.
enum Token {
    /// UTF-8 byte range containing one complete word in its owning name.
    Word(std::ops::Range<usize>),
    /// A newline preserved even when consecutive or at a name's boundary.
    Newline,
}

/// Wrapped label geometry independent of telemetry and transition status.
#[derive(Clone)]
struct LayoutCache {
    /// Fingerprint of state contents, available area, and sizing settings.
    hash: u64,
    /// Font environment used to build these galleys.
    fonts: Arc<FontMetrics>,
    /// Center-aligned galleys in sorted state order, ready for recoloring and painting.
    labels: Arc<Vec<Arc<Galley>>>,
}
