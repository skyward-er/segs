use std::{ops::Range, sync::Arc};

use aho_corasick::AhoCorasick;
use egui::{
    Button, CursorIcon, Id, Response, ScrollArea, Sense, TextEdit, TextFormat, Ui, Vec2,
    text::LayoutJob, vec2,
};
use smallvec::SmallVec;

/// Horizontal padding inside each entry row (applied left and right of the
/// galley) when sizing the popup and painting the text.
const ENTRY_HPAD: f32 = 6.0;
/// Floor for the popup width — keeps the filter row from collapsing when no
/// entry is wider than this.
const MIN_POPUP_WIDTH: f32 = 220.0;
/// Accent color used to highlight matched substrings in entry labels.
const MATCH_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 153, 0);

/// A dropdown that opens a popup with a text-filter on top and a scrollable
/// list of matching options. Match ranges are highlighted in the option label.
///
/// Returns `true` if `current` was changed by the user this frame.
///
/// Inspired by the `show_layout_filter` widget on the `v2-rewrite` branch
/// (`crates/segs/src/ui/components/mode_toggle.rs`), adapted to plain egui.
pub fn filtered_select<T>(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    label: &str,
    current: &mut T,
    options: &[T],
    name_of: impl Fn(&T) -> String,
) -> bool
where
    T: Clone + PartialEq,
{
    let id = ui.make_persistent_id(id_salt);
    let popup_id = id.with("popup");
    let filter_id = id.with("filter");

    let current_label = format!("{label}: {}", name_of(current));
    let button_resp = ui.add_sized(
        [ui.available_width(), 22.0],
        Button::new(current_label).truncate(),
    );
    if button_resp.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
        // Reset the filter every time the popup is opened so the user starts
        // from a clean state.
        ui.memory_mut(|m| m.data.insert_temp::<String>(filter_id, String::new()));
    }

    let mut changed = false;
    egui::popup_below_widget(
        ui,
        popup_id,
        &button_resp,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            let mut filter: String = ui
                .memory(|m| m.data.get_temp::<String>(filter_id))
                .unwrap_or_default();

            // Reuse a cached finder + match list when the filter is unchanged
            // and the option set hasn't changed.
            let names: Vec<String> = options.iter().map(&name_of).collect();
            let matches = matches_with_cache(ui, id, &filter, &names);

            // Pre-layout the matched entries so we know how wide the popup
            // needs to be. Galleys are cached internally by egui keyed on
            // the layout job, so this is cheap to redo each frame.
            let text_color = ui.visuals().text_color();
            let mut entries: Vec<(usize, Arc<egui::Galley>)> = matches
                .iter()
                .map(|(idx, m)| {
                    let job = format_match_job(ui, &names[*idx], m, text_color);
                    let galley = ui.fonts(|f| f.layout_job(job));
                    (*idx, galley)
                })
                .collect();

            let max_entry_w = entries
                .iter()
                .map(|(_, g)| g.size().x)
                .fold(0.0_f32, f32::max);
            let screen_w = ui.ctx().screen_rect().width();
            let popup_w = (max_entry_w + ENTRY_HPAD * 2.0)
                .max(MIN_POPUP_WIDTH)
                .min((screen_w * 0.9).max(MIN_POPUP_WIDTH));
            ui.set_min_width(popup_w);
            ui.set_max_width(popup_w);

            let filter_resp = ui.add(
                TextEdit::singleline(&mut filter)
                    .hint_text("Filter…")
                    .desired_width(ui.available_width()),
            );
            filter_resp.request_focus();
            ui.memory_mut(|m| m.data.insert_temp(filter_id, filter.clone()));

            ui.separator();

            ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                if entries.is_empty() {
                    ui.add_space(4.0);
                    ui.colored_label(
                        ui.visuals().weak_text_color(),
                        "No matching fields",
                    );
                    return;
                }
                for (idx, galley) in entries.drain(..) {
                    let selected = current == &options[idx];
                    let resp = entry_ui(ui, galley, popup_w, selected);
                    if resp.clicked() {
                        *current = options[idx].clone();
                        changed = true;
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );

    changed
}

fn matches_with_cache(
    ui: &Ui,
    id: Id,
    filter: &str,
    names: &[String],
) -> Arc<Vec<(usize, TextMatch)>> {
    let finder_id = id.with("text_finder");
    let matches_id = id.with("matches_list");
    let count_id = id.with("matches_count");

    let cached_finder: Option<Arc<TextFinder>> = ui.memory(|m| m.data.get_temp(finder_id));
    let cached_matches: Option<Arc<Vec<(usize, TextMatch)>>> =
        ui.memory(|m| m.data.get_temp(matches_id));
    let cached_count: Option<usize> = ui.memory(|m| m.data.get_temp(count_id));

    let cache_valid = cached_finder
        .as_ref()
        .is_some_and(|f| f.has_same_pattern(filter))
        && cached_count == Some(names.len())
        && cached_matches.is_some();

    if cache_valid {
        return cached_matches.unwrap();
    }

    let finder = Arc::new(TextFinder::new(filter));
    let matches: Vec<(usize, TextMatch)> = finder.iter_find_in(names).collect();
    let matches = Arc::new(matches);
    ui.memory_mut(|m| {
        m.data.insert_temp(finder_id, finder);
        m.data.insert_temp(matches_id, matches.clone());
        m.data.insert_temp(count_id, names.len());
    });
    matches
}

fn entry_ui(ui: &mut Ui, galley: Arc<egui::Galley>, width: f32, selected: bool) -> Response {
    let height = (galley.size().y + 4.0).max(20.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, selected);
        let bg_fill = if selected || response.hovered() {
            visuals.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, 3.0, bg_fill);

        let text_pos = rect.left_center()
            - vec2(0.0, galley.size().y / 2.0)
            + Vec2::new(ENTRY_HPAD, 0.0);
        ui.painter().galley(text_pos, galley, visuals.text_color());
    }

    response
}

fn format_match_job(
    _ui: &Ui,
    text: &str,
    matches: &TextMatch,
    regular_color: egui::Color32,
) -> LayoutJob {
    let font = egui::FontId::proportional(13.0);
    let regular = TextFormat {
        font_id: font.clone(),
        color: regular_color,
        ..Default::default()
    };
    let highlight = TextFormat {
        font_id: font,
        color: MATCH_COLOR,
        ..Default::default()
    };

    let mut job = LayoutJob::default();
    let mut last_index = 0;
    for range in matches.0.iter() {
        if range.start > last_index {
            job.append(&text[last_index..range.start], 0.0, regular.clone());
        }
        job.append(&text[range.clone()], 0.0, highlight.clone());
        last_index = range.end;
    }
    if last_index < text.len() {
        job.append(&text[last_index..], 0.0, regular);
    }
    job
}

#[derive(Debug, Clone)]
struct TextFinder {
    pattern: String,
    matcher: Option<AhoCorasick>,
}

impl TextFinder {
    fn new(pattern: impl AsRef<str>) -> Self {
        let pattern = pattern.as_ref().to_lowercase();
        let matcher = if pattern.is_empty() {
            None
        } else {
            AhoCorasick::new([pattern.clone()]).ok()
        };
        Self { pattern, matcher }
    }

    fn has_same_pattern(&self, pattern: impl AsRef<str>) -> bool {
        self.pattern == pattern.as_ref().to_lowercase()
    }

    fn iter_find_in<'a, I, S>(&'a self, haystack: I) -> impl Iterator<Item = (usize, TextMatch)> + 'a
    where
        I: IntoIterator<Item = S> + 'a,
        S: AsRef<str> + 'a,
    {
        // Empty pattern → match everything in original order with no
        // highlights.
        if self.pattern.is_empty() {
            return Box::new(
                haystack
                    .into_iter()
                    .enumerate()
                    .map(|(i, _)| (i, TextMatch(SmallVec::new()))),
            ) as Box<dyn Iterator<Item = (usize, TextMatch)>>;
        }

        let matcher = self.matcher.as_ref();
        let mut matches: Vec<(usize, TextMatch)> = haystack
            .into_iter()
            .enumerate()
            .filter_map(|(i, text)| {
                let matcher = matcher?;
                let ranges: SmallVec<[Range<usize>; 3]> = matcher
                    .find_iter(&text.as_ref().to_lowercase())
                    .map(|m| m.range())
                    .collect();
                if ranges.is_empty() {
                    None
                } else {
                    Some((i, TextMatch(ranges)))
                }
            })
            .collect();
        // Sort by earliest match position so the most relevant hits float to
        // the top.
        matches.sort_by(|(_, a), (_, b)| a.cmp(b));
        Box::new(matches.into_iter())
    }
}

#[derive(Debug, Clone)]
struct TextMatch(SmallVec<[Range<usize>; 3]>);

impl PartialEq for TextMatch {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for TextMatch {}

impl Ord for TextMatch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            match a.start.cmp(&b.start) {
                std::cmp::Ordering::Equal => continue,
                ord => return ord,
            }
        }
        self.0.len().cmp(&other.0.len())
    }
}

impl PartialOrd for TextMatch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
