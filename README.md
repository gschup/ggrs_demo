# [![GGRS LOGO](./ggrs_logo.png)](https://github.com/gschup/ggrs/)

## GGRS & Matchbox & Macroquad Demo

Try it out yourself!

### Basic Local Setup Instructions

1. Install matchbox server:
```sh
cargo install matchbox_server
```
2. Launch matchbox server in a separate terminal window:
```sh
matchbox_server
```
3. Run the game in two different terminal windows:
```sh
cargo run
```
4. The game's code is configured to target default port local host matchbox_server. Thus once you type in the same lobby number (ex. 1234) in both game clients, they should connect via the matchbox server and you will have ggrs + macroquad working locally.

### WASM/Web
Follow instructions in `build-wasm.sh`.

## Licensing

this project is dual-licensed under either

- [MIT License](./LICENSE-MIT): Also available [online](http://opensource.org/licenses/MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE): Also available [online](http://www.apache.org/licenses/LICENSE-2.0)

at your option.
