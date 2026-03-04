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

## Build pipeline

The explorer is a static site: Rust sources compile to WebAssembly, `wasm-pack` emits JavaScript bindings, and the browser loads both from `pkg/`.

```
src/*.rs  ──cargo (wasm32)──►  target/wasm32-unknown-unknown/
                                        │
                                 wasm-pack build
                                        ▼
                              pkg/mandelbrot_wasm_bg.wasm
                              pkg/mandelbrot_wasm.js
                                        │
index.html ──imports──► www/app.js ─────┘
                              │
                              ▼
                     canvas blit via web-sys (zero-copy)
```

### Quick start

From the repository root:

```bash
./scripts/build.sh
python3 -m http.server 8080
```

Open http://localhost:8080 in your browser.

### Pipeline steps

| Step | Command | What it does |
|------|---------|--------------|
| 1. WASM target | `rustup target add wasm32-unknown-unknown` | Installs the cross-compilation triple (once per toolchain). |
| 2. Compile + bind | `wasm-pack build --target web --out-dir pkg` | Builds the `cdylib`, runs `wasm-bindgen`, and writes ES-module glue into `pkg/`. |
| 3. Serve | `python3 -m http.server 8080` | Serves `index.html`, `www/app.js`, and generated `pkg/` artifacts over HTTP (required for `fetch`ing the `.wasm` file). |

The helper script runs steps 1–2 and prints the serve command:

```bash
./scripts/build.sh          # debug build (faster iteration)
./scripts/build.sh --release  # size-optimized output (matches Cargo `[profile.release]`)
```

### `pkg/` outputs

`wasm-pack` regenerates this directory on every build (it is gitignored):

| File | Role |
|------|------|
| `mandelbrot_wasm_bg.wasm` | Compiled WebAssembly module with the renderer |
| `mandelbrot_wasm.js` | ES module exporting `Explorer`, `init`, and palette helpers |
| `mandelbrot_wasm_bg.js` | Low-level loader that instantiates the WASM module |
| `mandelbrot_wasm.d.ts` | TypeScript declarations for editor tooling |

`www/app.js` imports `../pkg/mandelbrot_wasm.js`, calls `init()` to fetch/instantiate the module, binds the page canvas with `Explorer.bind_canvas()`, and presents frames through `render_to_canvas()` — the RGBA buffer is viewed directly from WASM linear memory via `web-sys`, avoiding a JS-side pixel copy.

### Native tests (no browser)

Core math and palette logic run under `cargo test` on the host triple — the same code paths compiled into the WASM module:

```bash
cargo test
```

Use this while iterating on `src/mandelbrot.rs` or `src/palette.rs` before rebuilding `pkg/`.

## Development

Release builds enable size optimizations (`opt-level = "s"`, LTO). Pass `--release` to `scripts/build.sh` or `wasm-pack` when measuring download size or profiling frame time.

## Project layout

```
├── src/
│   ├── lib.rs          # wasm-bindgen Explorer API
│   ├── canvas.rs       # web-sys canvas presenter (zero-copy blit)
│   ├── mandelbrot.rs   # viewport, escape-time, render loop
│   └── palette.rs      # color theme definitions
├── www/app.js          # canvas UI and input handling
├── index.html          # explorer page
└── scripts/
    ├── build.sh        # wasm-pack build helper
    └── commit-at.sh
```

## Controls

| Input | Action |
|-------|--------|
| Drag | Pan |
| Scroll wheel | Zoom at cursor |
| Palette dropdown | Change color theme |
| Keys `1`–`4` | Quick-switch palettes |
| Iterations slider | Increase/decrease detail |
| Reset view | Return to default framing |

## License

MIT
