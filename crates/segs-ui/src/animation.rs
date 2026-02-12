use egui::{Context, Id, emath::lerp};

pub trait AnimationExt {
    fn animate_value_with_easing(&self, id: Id, target_value: f32, easing: fn(f32) -> f32) -> f32;
    fn animate_value_with_time_and_easing(
        &self,
        id: Id,
        target_value: f32,
        duration: f32,
        easing: fn(f32) -> f32,
    ) -> f32;
}

impl AnimationExt for Context {
    fn animate_value_with_easing(&self, id: Id, target_value: f32, easing: fn(f32) -> f32) -> f32 {
        animate_value_with_easing(self, id, target_value, easing)
    }

    fn animate_value_with_time_and_easing(
        &self,
        id: Id,
        target_value: f32,
        duration: f32,
        easing: fn(f32) -> f32,
    ) -> f32 {
        animate_value_with_time_and_easing(self, id, target_value, duration, easing)
    }
}

/// Animates a value towards a target using a specified easing function and the
/// default animation duration.
pub fn animate_value_with_easing(ctx: &Context, id: Id, target_value: f32, easing: fn(f32) -> f32) -> f32 {
    let animation_time = ctx.style().animation_time;
    animate_value_with_time_and_easing(ctx, id, target_value, animation_time, easing)
}

/// Animates a value towards a target using a specified easing function and
/// duration.
pub fn animate_value_with_time_and_easing(
    ctx: &Context,
    id: Id,
    target_value: f32,
    duration: f32,
    easing: impl Fn(f32) -> f32,
) -> f32 {
    let now = ctx.input(|i| i.time);

    // We store: (Origin Value, Target Value, Start Time)
    // Note: We do NOT store 'current' value. We calculate it fresh every frame.
    let (mut origin, mut active_target, mut start_time) =
        ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || (target_value, target_value, now)));

    // Detect if the user requested a NEW target value
    if target_value != active_target {
        // 1. Calculate where we are RIGHT NOW based on the OLD animation
        // This ensures a smooth handoff from the old movement to the new one.
        let previous_elapsed = (now - start_time) as f32;
        let previous_t = (previous_elapsed / duration).clamp(0.0, 1.0);
        let current_value_at_switch = lerp(origin..=active_target, easing(previous_t));

        // 2. Update state:
        // The current position becomes the NEW origin.
        // The requested value becomes the NEW target.
        // The timer resets to NOW.
        origin = current_value_at_switch;
        active_target = target_value;
        start_time = now;

        // Save the updated state immediately
        ctx.data_mut(|d| d.insert_temp(id, (origin, active_target, start_time)));
    }

    // Standard Animation Calculation
    let elapsed = (now - start_time) as f32;

    // Ensure t is exactly 0.0 to 1.0
    let t = (elapsed / duration).clamp(0.0, 1.0);

    // Apply easing
    let eased_t = easing(t);

    // Interpolate between the FIXED origin and the FIXED target
    let current_value = lerp(origin..=active_target, eased_t);

    // Request repaint if we are still moving
    if t < 1.0 {
        ctx.request_repaint();
    }

    current_value
}
