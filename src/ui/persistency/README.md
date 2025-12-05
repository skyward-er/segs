# Layout persistency

## Rationale

For the typical Skyward project, the launch is divided into two main parts: refueling and launch. During the refueling phase, the rocket is connected to a ground support equipment that allows the onboard tanks to be filled with propellant. After refueling, the launch phase involves configuring and monitoring the rocket itself. These different phases require monitoring and controlling very distinct parameters of different devices.

The mission control software is thus required to adapt its interface to these multiple phases. Moreover, the software is also used during test campaigns (e.g., of new rocket engines) where the design of the interface is not yet finalized.

The goal of SEGS is to provide runtime-customizable mission control software. The application allows customization of the interface with multiple **panes** (i.e., widgets) and the ability to compose these panes using splitters, tabs, and windows.

For the user to take advantage of this customizability, there must be a way to save layouts persistently and to load them at runtime with minimal effort.

## Layout representation

The code represents the layout with the `AppState` struct. There are two essential elements to keep track of:

- The layout of panes (i.e., their organization into splitters, tabs, windows, etc.)
- Each pane's individual state (e.g., its configuration, subscriptions, colors, etc.)

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AppState {
    pub panes_tree: Tree<Pane>, // egui_tiles tree data structure that stores the layout
    pub next_pane_id: PaneId, // Just stores the id for the next pane that will be added
    #[cfg(feature = "conrig")]
    pub command_switch_window: CommandSwitchWindow,
}
```

The layout is implemented by the `egui_tiles` library, which uses a tree data structure where tiles are stored. A **tile** can either be a container (such as a splitter, tab, or grid) or a pane. Each **tile** is identified by a unique ID.

We will not focus on the layout as it is implemented externally by `egui_tiles`. The important thing to know is that the `egui_tiles::Tree` struct is serializable and deserializable.

All the panes are listed in the `segs::ui::panes::PaneKind` enum, where each element provides a string name via the [`EnumMessage`](https://docs.rs/strum/latest/strum/derive.EnumMessage.html) macro. This enum enables the population of the **widget gallery**, which displays a list of available widgets in the UI for the user.

The struct defined to store a pane state has two requirements:

- Implement `Serialize` and `Deserialize` traits in order to be stored on disk
- Implement the `Eq` trait such that the current app state can be compared against the stored layouts

### PartialEq vs Eq

The difference between `PartialEq` and `Eq` is that `Eq` requires **reflexivity** (i.e., 'a == a'). In contrast, both requires **symmetricity** (i.e. 'a == b => b == a') and **transitivity** (i.e. 'a == b && b == c => a == c`).

| Property         | `PartialEq` | `Eq` | Description                        |
| ---------------- | ----------- | ---- | ---------------------------------- |
| **Reflexivity**  |             | ✅   | $a = a$                            |
| **Symmetricity** | ✅          | ✅   | $a = b \implies b = a$             |
| **Transitivity** | ✅          | ✅   | $a = b \land b = c \implies a = c$ |

The main problem is with floating point numbers that do not have a full equivalence relation, because `NaN != NaN`.

## Usage flowchart

![](/docs/images/App layout state machine.png)
