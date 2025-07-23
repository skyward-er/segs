use tracing::error;

/// Trait to instrument common error handling for Result & Option types
pub trait ErrInstrument {
    type Inner;

    fn log_expect(self, msg: &str) -> Self::Inner;
    fn log_unwrap(self) -> Self::Inner;
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
                panic!("{msg}: {e:?}");
            }
        }
    }

    fn log_unwrap(self) -> Self::Inner {
        match self {
            Ok(t) => t,
            Err(e) => {
                error!("Called unwrap on an Err value: {:?}", e);
                panic!("Called unwrap on an Err value: {e:?}");
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
                panic!("{msg}");
            }
        }
    }

    fn log_unwrap(self) -> Self::Inner {
        match self {
            Some(t) => t,
            None => {
                error!("Called unwrap on a None value");
                panic!("Called unwrap on a None value");
            }
        }
    }
}
