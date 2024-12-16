use tracing::error;

/// Trait to instrument common error handling for Result & Option types
pub trait ErrInstrument {
    type Inner;

    fn log_expect(self, msg: &str) -> Self::Inner;
}

impl<T, E> ErrInstrument for Result<T, E>
where
    E: std::fmt::Debug,
{
    type Inner = T;

    fn log_expect(self, msg: &str) -> Self::Inner {
        match self {
            Ok(t) => t,
            Err(e) => {
                error!("{}: {:?}", msg, e);
                panic!("{}: {:?}", msg, e);
            }
        }
    }
}

impl<T> ErrInstrument for Option<T> {
    type Inner = T;

    fn log_expect(self, msg: &str) -> Self::Inner {
        match self {
            Some(t) => t,
            None => {
                error!("{}", msg);
                panic!("{}", msg);
            }
        }
    }
}
