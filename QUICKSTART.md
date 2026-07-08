# SEGS — Quick start

SEGS is a ground-station GUI for receiving CCSDS telemetry and sending commands, configured with COSMOS definition files.

## Launching

```
segs --tlm-def <fsw_tlm.txt> --cmd-def <fsw_cmd.txt> --ethernet <ip>:<send_port>:<recv_port>
```

- `--tlm-def` / `--cmd-def`: COSMOS telemetry/command definitions. Without them you'll get an empty registry — useful only for layout editing.
- `--ethernet <ip>:<send>:<recv>` or `--serial <port>:<baud>`: data source. You can also leave this off and configure it later from the **Sources** button.

If a definition file fails to parse, the error is printed to stderr; check there first if telemetry/commands look empty.

## The workspace

The window is a tree of resizable **panes**. Each pane shows one widget. You assemble your view by splitting and replacing panes.

| Pane type | What it does |
|---|---|
| **Plot 2D** | Live time-series of any plottable telemetry field. |
| **Messages Viewer** | Tabular dump of selected fields from incoming packets. |
| **Command** | A single button that sends a configured command. |
| **PID** | Piping & instrumentation diagram editor with live values. |
| **Default** | Empty placeholder — start here, then replace via the Gallery. |

Right-click a pane for its context menu (settings, replace, etc.).

## Pane shortcuts (Shortcuts ⌨ button in the bottom bar)

| Keys | Action |
|---|---|
| `Ctrl + H` / `Ctrl + V` | Split the hovered pane horizontally / vertically. |
| `Ctrl + R` | Replace the hovered pane via the Widget Gallery. |
| `Ctrl + W` | Close the hovered pane. |
| `Shift + Esc` | Maximize the hovered pane. |
| `Esc` | Exit a maximized pane. |
| `/` | Open the command switch (if commands are loaded). |

## Bottom bar

- **Sources 🔌** — open/close the data connection (Ethernet or Serial). The little LED on the left blinks while packets arrive.
- **Layouts 💾** — save the current pane tree under a name; reload or import a saved layout. The layout entry turns **green** when the on-disk file matches the current state, **yellow** when it has unsaved changes.
- **Shortcuts ⌨** — cheat sheet of the shortcuts above.

## Configuring a Plot

1. Add a Plot pane (Widget Gallery via `Ctrl + R`, or right-click an empty pane).
2. Right-click → **Source Data Settings…**
3. Pick the **X axis** (default: message receipt timestamp) and one or more **Y axes** using the filterable dropdowns — type to filter, matches are highlighted in orange.
4. Each Y axis row has its own color, line width, and 🗑 to remove.

## Sending a Command

1. Add a Command pane.
2. Right-click → **Settings…** to pick a command from the COSMOS catalog and fill in its parameters.
3. Click the pane to send. Or use the `/` command switch for keyboard-driven sending.

## Saving your work

Layouts and connection state persist across runs. To share a setup, open the **Layouts** manager, save it under a name, and the JSON file lands in your layouts directory — copy it to another machine and **Import** it there.
