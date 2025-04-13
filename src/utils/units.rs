use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitOfMeasure {
    Time(TimeUnits),
    Other(String),
    Adimensional,
}

impl<T: AsRef<str>> From<Option<T>> for UnitOfMeasure {
    fn from(s: Option<T>) -> Self {
        let s = s.as_ref();
        match s {
            Some(s) if s.as_ref().is_empty() => UnitOfMeasure::Adimensional,
            Some(s) => {
                if let Ok(unit) = TimeUnits::from_str(s.as_ref()) {
                    UnitOfMeasure::Time(unit)
                } else {
                    UnitOfMeasure::Other(s.as_ref().to_string())
                }
            }
            None => UnitOfMeasure::Adimensional,
        }
    }
}

impl Display for UnitOfMeasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitOfMeasure::Time(unit) => write!(f, "{}", unit),
            UnitOfMeasure::Other(unit) => write!(f, "{}", unit),
            UnitOfMeasure::Adimensional => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeUnits {
    Second,      // s
    Millisecond, // ms
    Microsecond, // us
    Nanosecond,  // ns
}

impl TimeUnits {
    pub fn scale(&self) -> f64 {
        match self {
            TimeUnits::Second => 1.0,
            TimeUnits::Millisecond => 1e-3,
            TimeUnits::Microsecond => 1e-6,
            TimeUnits::Nanosecond => 1e-9,
        }
    }
}

impl FromStr for TimeUnits {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s" => Ok(TimeUnits::Second),
            "ms" => Ok(TimeUnits::Millisecond),
            "us" => Ok(TimeUnits::Microsecond),
            "ns" => Ok(TimeUnits::Nanosecond),
            _ => Err(()),
        }
    }
}

impl Display for TimeUnits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnits::Second => write!(f, "s"),
            TimeUnits::Millisecond => write!(f, "ms"),
            TimeUnits::Microsecond => write!(f, "µs"),
            TimeUnits::Nanosecond => write!(f, "ns"),
        }
    }
}
