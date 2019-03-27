# rstetris
An implementation of the game [Tetris](https://en.wikipedia.org/wiki/Tetris), targeting [WebAssembly](https://webassembly.org/), written in [Rust](https://www.rust-lang.org/) with no external dependencies.

## How to play

1. Install the `wasm32-unknown-unknown` target with [rustup](https://rustup.rs/):
    ```
    rustup target add wasm32-unknown-unknown
    ```
2. Download and build the game:
    ```
    git clone https://github.com/dprien/rstetris.git
    cd rstetris
    cargo build --release --target=wasm32-unknown-unknown
    ```
3. Start a local webserver to serve the games' files:
    ```
    python -m http.server 8000 --bind 127.0.0.1 --directory web/
    ```
4. Point your favorite browser to [http://127.0.0.1:8000](http://127.0.0.1:8000).

## Controls
### Keyboard
* <kbd>Space</kbd> - Start a new game (on title or "Game Over" screen)
* <kbd>Esc</kbd> - Abort the current game
* <kbd>A</kbd> / <kbd>D</kbd> - Move left / right
* <kbd>S</kbd> / <kbd>W</kbd> - "Soft" / "Hard" drop
* <kbd>&leftarrow;</kbd> / <kbd>&rightarrow;</kbd> - Rotate counter-clockwise / clockwise

### Touch screen
* `Swipe up` - Start a new game (on title or "Game Over" screen)
* `Swipe left` / `right` - Move left / right
* `Swipe down` - "Soft" drop
* `Swipe up` - "Hard" drop
* `Tap` - Rotate clockwise

## Project goals & planned features
* [x] **No external dependencies**, like [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen), [web-sys](https://github.com/rustwasm/wasm-bindgen/tree/master/crates/web-sys) or [js-sys](https://github.com/rustwasm/wasm-bindgen/tree/master/crates/js-sys).
* [x] **No additional tooling**, like [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) or [npm](https://www.npmjs.com/get-npm).
* [x] **Tablet & smartphone support**, using [Touch events](https://developer.mozilla.org/en-US/docs/Web/API/Touch_events).
* [ ] **Multiple game modes**, like [40 lines](https://tetris.fandom.com/wiki/40_lines) or Endless.
* [ ] **Sound**, using the [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API).
* [ ] **Multiplayer**, either peer-to-peer (via [WebRTC's `RTCDataChannel`](https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel)) or server-client (via WebSockets).

## License

This project is licensed under the **GNU General Public License v3.0** (see [LICENSE](LICENSE) or https://www.gnu.org/licenses/gpl-3.0.html).
