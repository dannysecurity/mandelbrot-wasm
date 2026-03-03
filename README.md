# mandelbrot-wasm

Interactive Mandelbrot set explorer rendered in WebAssembly with Rust.

Pan and zoom through the fractal, switch color palettes, and adjust iteration depth — all computed in WASM and drawn to an HTML canvas.

## Features

- **WASM renderer** — escape-time Mandelbrot with smooth coloring
- **Pan / zoom** — drag to pan, scroll wheel to zoom toward the cursor
- **Palette themes** — Classic, Fire, Ocean, and Grayscale
- **Adjustable detail** — iteration slider from 64 to 1024

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
- A static file server for local development

## Build

Add the WebAssembly target once:

```bash
rustup target add wasm32-unknown-unknown
```

Build the WASM package:

```bash
wasm-pack build --target web --out-dir pkg
```

Serve the project root (the page loads `index.html` and imports from `pkg/`):

```bash
python3 -m http.server 8080
```

Open http://localhost:8080 in your browser.

## Development

Run native unit tests (core math and palettes, no browser required):

```bash
cargo test
```

Release builds enable size optimizations (`opt-level = "s"`, LTO).

## Project layout

```
├── src/
│   ├── lib.rs          # wasm-bindgen Explorer API
│   ├── mandelbrot.rs   # viewport, escape-time, render loop
│   └── palette.rs      # color theme definitions
├── www/app.js          # canvas UI and input handling
├── index.html          # explorer page
└── scripts/commit-at.sh
```

## Controls

| Input | Action |
|-------|--------|
| Drag | Pan |
| Scroll wheel | Zoom at cursor |
| Palette dropdown | Change color theme |
| Iterations slider | Increase/decrease detail |
| Reset view | Return to default framing |

## License

MIT
