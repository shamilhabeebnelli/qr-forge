# qrcodegenerator

Fast, private QR code generator that runs **entirely in the browser**. The QR
engine is written in Rust, compiled to WebAssembly, and wrapped in a small
static site. QR generation and file exports happen on your device. Using external share or opening a generated link is an explicit external action.

**Live site:** https://qrcodegenerator.denelli.in/

## Features

- **Text / link** QR codes
- **WiFi** network QR codes (WPA/WEP/open, hidden networks) so guests can join
  by scanning
- **WhatsApp** click-to-chat QR codes (`wa.me` links with a prefilled message)
- **UPI payment** QR codes with payee name, amount in INR and a note
- Error correction level (L/M/Q/H), custom colors and quiet-zone control
- Save as photo (**PNG** 512/1024/2048 px) or **SVG**, save as **PDF**, **print**, **share** to other apps, or copy to clipboard
- Client-side generation, no accounts or analytics. Offline use requires the page and WebAssembly assets to remain loaded; no service worker or installable offline cache is included.

## Tech stack

| Piece | Choice |
| --- | --- |
| QR engine | Rust + [`qrcode`](https://crates.io/crates/qrcode) crate |
| Browser bridge | `wasm-bindgen` via `wasm-pack` |
| Frontend | Vanilla HTML/CSS/JS, no framework |
| Hosting | GitHub Pages, built and deployed by GitHub Actions |

## How it works

`src/lib.rs` holds the pure-Rust core: payload builders for each QR type
(WIFI:, upi://pay, wa.me) and two renderers (SVG string, and a compact
module matrix for crisp canvas/PNG export). Thin `#[wasm_bindgen]` wrappers
expose them to JavaScript; `site/app.js` wires them to the UI.

The workflow in `.github/workflows/deploy.yml` runs the Rust test suite,
builds the WASM package with `wasm-pack`, and publishes `site/` to GitHub
Pages on every push to `main`.

The GitHub repository is named `qrcodegenerator`. The internal Rust crate and generated WebAssembly files retain the `qr-forge` / `qr_forge` names so the existing browser imports remain compatible.

## Build locally

```bash
rustup target add wasm32-unknown-unknown
cargo test                              # run the core test suite
wasm-pack build --release --target web --out-dir site/pkg
cd site && python3 -m http.server 8080  # then open http://localhost:8080
```

## License

MIT
