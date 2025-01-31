pub fn is_default<T: Default + PartialEq>(x: &T) -> bool {
    *x == T::default()
}

pub fn is_zero(x: &f32) -> bool {
    *x == 0.0
}
