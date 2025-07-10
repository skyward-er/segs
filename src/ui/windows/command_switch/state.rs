use serde::{Deserialize, Serialize};

/// State of the command switch window, controlling visibility and interaction.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum VisibileState {
    /// The command switch window is hidden.
    #[default]
    Hidden,
    /// The command switch window is showing the command catalog. Configurable
    /// commands can be added or edited.
    CommandCatalog,
    /// The command switch window is showing the settings for a specific command.
    CommandSettings,
    /// The command switch window is showing the command switch interface.
    /// Commands can be executed directly from here.
    CommandSwitch,
    /// The command switch window is showing the settings for a configurable command.
    ConfigurableCommandDialog,
}

impl VisibileState {
    /// Check if the current state is in CommandCatalog or CommandSettings mode.
    fn is_catalog(&self) -> bool {
        matches!(
            self,
            VisibileState::CommandCatalog | VisibileState::CommandSettings
        )
    }

    /// Check if the current state is in CommandSwitch or ConfigurableCommandDialog mode.
    fn is_command_switch(&self) -> bool {
        matches!(
            self,
            VisibileState::CommandSwitch | VisibileState::ConfigurableCommandDialog
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StateManager {
    state: VisibileState,
    previous_state: VisibileState,
}

impl StateManager {
    pub fn switch_command(&mut self) {
        match self.state {
            // if hidden, restore previous state
            VisibileState::Hidden => {
                if self.previous_state.is_command_switch() {
                    std::mem::swap(&mut self.previous_state, &mut self.state);
                } else {
                    self.state = VisibileState::CommandSwitch;
                }
            }
            // if command switch, hide it
            VisibileState::CommandSwitch | VisibileState::ConfigurableCommandDialog => {
                self.previous_state = self.state;
                self.state = VisibileState::Hidden;
            }
            // if command catalog or command settings, switch to command switch
            VisibileState::CommandCatalog | VisibileState::CommandSettings => {
                self.previous_state = self.state;
                self.state = VisibileState::CommandSwitch;
            }
        }
    }

    pub fn switch_catalog(&mut self) {
        match self.state {
            // if hidden, restore previous state
            VisibileState::Hidden => {
                if self.previous_state.is_catalog() {
                    std::mem::swap(&mut self.previous_state, &mut self.state);
                } else {
                    self.state = VisibileState::CommandCatalog
                }
            }
            // if command catalog, hide it
            VisibileState::CommandCatalog | VisibileState::CommandSettings => {
                self.previous_state = self.state;
                self.state = VisibileState::Hidden;
            }
            // if command switch, switch to command catalog
            VisibileState::CommandSwitch | VisibileState::ConfigurableCommandDialog => {
                self.previous_state = self.state;
                self.state = VisibileState::CommandCatalog;
            }
        }
    }

    pub fn hide(&mut self) {
        if self.state != VisibileState::Hidden {
            self.previous_state = self.state;
            self.state = VisibileState::Hidden;
        }
    }

    /// Check if the current state is in CommandCatalog or CommandSettings mode.
    pub fn is_catalog(&self) -> bool {
        self.state.is_catalog()
    }

    /// Check if the current state is in CommandSwitch or ConfigurableCommandDialog mode.
    pub fn is_command_switch(&self) -> bool {
        self.state.is_command_switch()
    }
}
