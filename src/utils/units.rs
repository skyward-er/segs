use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PhisicalQuantity {
    pub metric_unit: UnitOfMeasure,
    pub value: f64,
}

// impl PhisicalQuantity {
//     pub fn to_string(&self) -> String {
//         match &self.prefix {
//             Some(prefix) => format!("{}{}", prefix.symbol(), self.metric_unit.symbol()),
//             None => self.metric_unit.symbol().to_string(),
//         }
//     }

//     pub fn scale(&self) -> f64 {
//         match &self.prefix {
//             Some(prefix) => prefix.value(),
//             None => 1.0,
//         }
//     }

//     pub fn increase_magnitude(&mut self) {
//         if let Some(prefix) = &self.prefix {
//             self.prefix = prefix.increase_magnitude();
//         } else {
//             self.prefix = Some(UnitPrefix::Kilo);
//         }
//     }

//     pub fn decrease_magnitude(&mut self) {
//         if let Some(prefix) = &self.prefix {
//             self.prefix = prefix.decrease_magnitude();
//         } else {
//             self.prefix = Some(UnitPrefix::Milli);
//         }
//     }
// }

// impl<T: AsRef<str>> From<T> for PhisicalQuantity {
//     fn from(s: T) -> Self {
//         s.as_ref().parse().log_unwrap()
//     }
// }

// impl FromStr for PhisicalQuantity {
//     type Err = ();

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         // First try to interpret the whole string as a measurement unit.
//         if let Some(unit) = TimeUnits::from_symbol(s) {
//             return Ok(PhisicalQuantity {
//                 metric_unit: unit,
//                 prefix: None,
//             });
//         }
//         // Otherwise, try to split the string into a prefix and a unit.
//         if s.len() < 2 {
//             return Err(());
//         }
//         let (prefix_str, rest) = s.split_at(1);
//         let prefix = UnitPrefix::from_symbol(prefix_str).ok_or(())?;
//         let metric_unit = TimeUnits::from_symbol(rest).ok_or(())?;
//         Ok(PhisicalQuantity {
//             metric_unit,
//             prefix: Some(prefix),
//         })
//     }
// }

#[derive(Debug, Clone)]
pub enum UnitOfMeasure {
    Time,
    Other,
}

// impl FromStr

#[derive(Debug, Clone)]
pub enum TimeUnits {
    Second,      // s
    Millisecond, // ms
    Microsecond, // us
    Nanosecond,  // ns
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
