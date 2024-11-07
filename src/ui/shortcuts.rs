use egui::{Context, Key, KeyboardShortcut, Modifiers};

pub fn map_to_action<A: Clone>(ctx: &Context, pairs: &[((Modifiers, Key), A)]) -> Option<A> {
    ctx.input_mut(|i| {
        pairs.iter().find_map(|((modifier, key), action)| {
            i.consume_shortcut(&KeyboardShortcut::new(*modifier, *key))
                .then_some(action.to_owned())
        })
    })
}
