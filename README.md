<div align="center">
  <img src="./assets/logo.png" alt="Logo" width="400"/>
  <br>
  <img src="https://img.shields.io/badge/version-v0.1.0-blue.svg?logo=rust">
  <img src="https://img.shields.io/badge/development-alpha-red">
</div>

# Skyward Enhanced Ground Software

Skyward Enhanced Ground Software (in short **SEGS**) is a team effort that aims to provide a renewed and enhanced ground software experience for our operators. It is currently in development and is not ready for production use, contributions are welcome.


---

**Table of contents**:
- [Installation](#installation)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [Functional Requirements](#functional-requirements)

---

## Installation

Since we are in the early stages of development, we do not provide a binary release yet. However, you can build the project from source.

<!-- TODO: ADD BINARY RELEASE CI TASK -->

<!-- TODO: ADD USAGE SECTION ONCE WE REACH BETA -->

<!-- TODO: UNCOMMENT THE FOLLOWING ONCE WE REACH BETA -->

<!-- ## Support

You can reach out to the project team for support, either through the issue tracker or the slack channel (`#segs-support`). -->

<!-- TODO: CREATE segs-support chnnale -->

## Contributing

If you want to contribute to the project, you can read the [CONTRIBUTING.md](./CONTRIBUTING.md) file for more information.


## Roadmap

- [ ] **2024-11-23 - v0.1.0**: Alpha release, minimal features and layout to test the architecture in a real-world scenario.
  - [ ] All Modularity requirements (R01, R01.1, R01.2, R02, R03) should be met
  - [ ] Persistency R04 and R04.2 should be met
  - [ ] Data sources (R05 and R17) for mockup testing before real-world test
  - [ ] Plotting widgets (R07) should be implemented
  - [ ] Persistency (R18) to log all received messages and capture the working of SEGS


## Functional Requirements

In this section we refer to the precise wording defined in [RFC2119 IETF](https://www.ietf.org/rfc/rfc2119.txt) for the requirements. Additional terminology is defined in the [Glossary](./GLOSSARY.md).

- [ ] **Modularity**
  - [ ] **R01**: It **MUST** be fully modular: the layout (set of widgets) must be configurable *WRA* according to individual needs.
  - [ ] **R01.1**: It **MAY** be implemented as a Tiling Manager with vertical and horizontal splitters to facilitate the migration from SkywardHub
  - [ ] **R01.2**: It **MUST** support multiple windows: you must be able to split the layout into multiple distinct windows
  - [ ] **R02**: The content displayed by the widgets **MUST** be configurable *WRA*.
  - [ ] **R03**: The data source of the widgets' content **MUST** be configurable *WRA*.
- [ ] **Persistency and configuration management**
  - [ ] **R04**: It **MUST** allow exporting and importing layout configurations in a serialized format: the entire layout must be serializable to a file (in JSON, XML, YAML, or any other format) to save, manage and load different configurations without needing to reset the system each time.
  - [ ] **R04.1**: The serialization format **SHOULD** be easily interpreted by a human (human-friendly).
  - [ ] **R04.2**: The serialization format **SHOULD** be uniquely tied to the Layout, in a declarative manner (the layout can be expressed and composed directly by using this format).
- [ ] **Data Sources configurability**
  - [ ] **R05**: It **MUST** be able to receive Mavlink messages from one or more Serial and UDP connections.
  - [ ] **R05.1**: It **MAY** provide auto detection of GSBs connected to the same LAN of a specific network card
  - [ ] **R05.2**: It **MAY** provide filtering of serial ports showing only GSBs.
  - [ ] **R05.3**: It **MUST** allow to manually specify which UDP socket/serial port to use for a connection
- [ ] **Widget requirements**
  - [ ] **R06**: It **MUST** provide a widget for visualizing arbitrary fields from raw Mavlink messages in a textual format.
  - [ ] **R07**: It **MUST** provide a plotting widget (both 2D and 3D).
  - [ ] **R07.1**: It **MUST** be adjustable in terms of colors, scales and series to plot *WRA*.
  - [ ] **R08**: It **MAY** provide a map widget (where the satellite map is displayed).
  - [ ] **R09**: It **MUST** provide a state visualizer widget.
  - [ ] **R09.1**: It **SHOULD** be completely configurable by the user *WRA*.
  - [ ] **R10**: It **MUST** provide a widget displaying outgoing tele-commands and their corresponding reply in an organized manner.
  - [ ] **R10.1**: It **MAY** provide an easy way to inspect previously sent tele-commands and their reply.
  - [ ] **R11**: Every widget **SHOULD** display the time elapsed since its data source was last updated (aka. last reception of a message).
  - [ ] **R12**: It **MAY** provide a timer widget (e.g. a countdown timer).
  - [ ] **R13**: It **MUST** provide a widget capable of sending Mavlink commands with and without parameters (replacing legacy Command Pads and Compact Command Pads).
  - [ ] **R13.1**: It **SHOULD** provide a setting to send the command periodically
- [ ] **User Interface and Interaction**
  - [ ] **R14**: It **MUST** support light and dark color themes.
  - [ ] **R14.1**: t **MAY** support other user-configurable themes *WRA*.
  - [ ] **R15**: It **MUST NOT** send commands without the user's will (recognition of patterns and response with request of more information or sending of commands is not allowed).
  - [ ] **R16**: It **SHOULD** account for different sizes of the monitor (responsive design)
- [ ] **Reproducibility**
  - [ ] **R17**: It **MAY** provide a way to replay a stream of Mavlink messages from a file for testing.
  - [ ] **R18**: It **SHOULD** log all received messages since the application started in a persistent manner.
  - [ ] **R18.1**: It **MAY** directly decode the message.
