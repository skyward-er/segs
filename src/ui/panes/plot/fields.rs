use serde::{Deserialize, Serialize};

use crate::{
    APP_START_TIMESTAMP_ORIGIN,
    mavlink::{TimedMessage, reflection::IndexedField},
    utils::units::{TimeUnits, UnitOfMeasure},
};

#[derive(Clone, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum XPlotField {
    MsgReceiptTimestamp,
    Field(IndexedField),
}

impl XPlotField {
    pub fn unit(&self) -> UnitOfMeasure {
        match self {
            XPlotField::MsgReceiptTimestamp => UnitOfMeasure::Time(TimeUnits::Millisecond),
            XPlotField::Field(field) => UnitOfMeasure::from(field.field().unit.as_ref()),
        }
    }

    pub fn name(&self) -> String {
        match self {
            XPlotField::MsgReceiptTimestamp => "receival timestamp".to_string(),
            XPlotField::Field(field) => field.field().name.clone(),
        }
    }

    pub fn extract_from_message(&self, message: &TimedMessage) -> Result<f64, String> {
        match self {
            XPlotField::MsgReceiptTimestamp => {
                Ok((message.time - *APP_START_TIMESTAMP_ORIGIN).as_millis() as f64)
            }
            XPlotField::Field(field) => field.extract_as_f64(&message.message),
        }
    }
}

impl From<IndexedField> for XPlotField {
    fn from(field: IndexedField) -> Self {
        Self::Field(field)
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum YPlotField {
    Field(IndexedField),
}

impl YPlotField {
    pub fn unit(&self) -> UnitOfMeasure {
        match self {
            YPlotField::Field(field) => UnitOfMeasure::from(field.field().unit.as_ref()),
        }
    }

    pub fn name(&self) -> String {
        match self {
            YPlotField::Field(field) => field.field().name.clone(),
        }
    }

    pub fn extract_from_message(&self, message: &TimedMessage) -> Result<f64, String> {
        match self {
            YPlotField::Field(field) => field.extract_as_f64(&message.message),
        }
    }
}

impl From<IndexedField> for YPlotField {
    fn from(field: IndexedField) -> Self {
        Self::Field(field)
    }
}
