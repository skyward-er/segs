use egui::{ImageSource, RichText, Theme, Ui, Window};
use serde::{Deserialize, Serialize};

use crate::{
    ccsds::TelemetryPacket,
    error::ErrInstrument,
    mavlink::reflection::{IndexedField, all_fields},
    ui::cache::ChangeTracker,
};

macro_rules! load_dark_sprite {
    ($str:expr) => {
        egui::include_image!(concat!("../../../../../../icons/pid_symbols/dark/", $str))
    };
}

macro_rules! load_light_sprite {
    ($str:expr) => {
        egui::include_image!(concat!("../../../../../../icons/pid_symbols/light/", $str))
    };
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct MotorValve {
    subscribed_field: Option<IndexedField>,
    pub variant: MotorValveVariant,

    #[serde(skip)]
    pub is_subs_window_visible: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum MotorValveVariant {
    TwoWay(TwoWayInternal),
    ThreeWay(ThreeWayInternal),
}

impl Default for MotorValveVariant {
    fn default() -> Self {
        MotorValveVariant::TwoWay(TwoWayInternal::default())
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct TwoWayInternal {
    last_value: Option<TwoWayStates>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct ThreeWayInternal {
    last_value: Option<ThreeWayStates>,
    invert: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
enum TwoWayStates {
    Open,
    Closed,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
enum ThreeWayStates {
    ActiveLeft,
    ActiveRight,
}

impl MotorValve {
    pub fn default_two_way() -> Self {
        Self { variant: MotorValveVariant::TwoWay(TwoWayInternal::default()), ..Default::default() }
    }

    pub fn default_three_way() -> Self {
        Self {
            variant: MotorValveVariant::ThreeWay(ThreeWayInternal::default()),
            ..Default::default()
        }
    }

    pub fn update(&mut self, packet: &TelemetryPacket) {
        if let Some(field) = &self.subscribed_field {
            if let Ok(value) = field.extract_as_u64(packet) {
                let value = value as u8;
                match &mut self.variant {
                    MotorValveVariant::TwoWay(internal) => {
                        internal.last_value = match value {
                            0 => Some(TwoWayStates::Closed),
                            1 => Some(TwoWayStates::Open),
                            _ => None,
                        };
                    }
                    MotorValveVariant::ThreeWay(internal) => {
                        internal.last_value = match (value, internal.invert) {
                            (0, false) => Some(ThreeWayStates::ActiveLeft),
                            (1, false) => Some(ThreeWayStates::ActiveRight),
                            (0, true) => Some(ThreeWayStates::ActiveRight),
                            (1, true) => Some(ThreeWayStates::ActiveLeft),
                            _ => None,
                        };
                    }
                }
            }
        }
    }

    pub fn reset_subscriptions(&mut self) {
        self.subscribed_field = None;
        match &mut self.variant {
            MotorValveVariant::TwoWay(internal) => internal.last_value = None,
            MotorValveVariant::ThreeWay(internal) => internal.last_value = None,
        }
    }

    pub fn subscriptions_ui(&mut self, ui: &mut Ui) {
        let change_tracker = ChangeTracker::record_initial_state(&self.subscribed_field);
        Window::new("Subscriptions")
            .id(ui.auto_id_with("subs_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_subs_window_visible)
            .show(ui.ctx(), |ui| {
                subscription_window(ui, &mut self.variant, &mut self.subscribed_field)
            });
        if change_tracker.has_changed(&self.subscribed_field) {
            match &mut self.variant {
                MotorValveVariant::TwoWay(internal) => internal.last_value = None,
                MotorValveVariant::ThreeWay(internal) => internal.last_value = None,
            }
        }
    }

    pub fn get_sprite(&self, theme: Theme) -> ImageSource {
        match (&self.variant, theme) {
            (MotorValveVariant::TwoWay(internal), Theme::Dark) => match internal.last_value {
                None => load_dark_sprite!("motor_valve.svg"),
                Some(TwoWayStates::Open) => load_dark_sprite!("motor_valve_green.svg"),
                Some(TwoWayStates::Closed) => load_dark_sprite!("motor_valve_red.svg"),
            },
            (MotorValveVariant::TwoWay(internal), Theme::Light) => match internal.last_value {
                None => load_light_sprite!("motor_valve.svg"),
                Some(TwoWayStates::Open) => load_light_sprite!("motor_valve_green.svg"),
                Some(TwoWayStates::Closed) => load_light_sprite!("motor_valve_red.svg"),
            },
            (MotorValveVariant::ThreeWay(internal), Theme::Dark) => match internal.last_value {
                None => load_dark_sprite!("three_way_valve.svg"),
                Some(ThreeWayStates::ActiveRight) => {
                    load_dark_sprite!("three_way_valve_active_right.svg")
                }
                Some(ThreeWayStates::ActiveLeft) => {
                    load_dark_sprite!("three_way_valve_active_left.svg")
                }
            },
            (MotorValveVariant::ThreeWay(internal), Theme::Light) => match internal.last_value {
                None => load_light_sprite!("three_way_valve.svg"),
                Some(ThreeWayStates::ActiveRight) => {
                    load_light_sprite!("three_way_valve_active_right.svg")
                }
                Some(ThreeWayStates::ActiveLeft) => {
                    load_light_sprite!("three_way_valve_active_left.svg")
                }
            },
        }
    }
}

fn subscription_window(
    ui: &mut Ui,
    variant: &mut MotorValveVariant,
    field: &mut Option<IndexedField>,
) {
    if let MotorValveVariant::ThreeWay(internal) = variant {
        ui.checkbox(&mut internal.invert, "Invert configuration");
        ui.add_sized([250., 10.], egui::Separator::default());
    }

    // Show fields with states (state fields are most useful for valve subscriptions)
    let fields: Vec<IndexedField> = all_fields()
        .into_iter()
        .filter(|f| f.field().has_states())
        .collect();

    if fields.is_empty() {
        ui.label(
            RichText::new("No state fields available (registry not loaded)")
                .underline()
                .strong(),
        );
        return;
    }

    let field = field.get_or_insert(fields[0].to_owned());
    egui::ComboBox::from_label("Field")
        .selected_text(&field.field().name)
        .show_ui(ui, |ui| {
            for f in fields.iter() {
                ui.selectable_value(field, f.to_owned(), &f.field().name);
            }
        });
}
