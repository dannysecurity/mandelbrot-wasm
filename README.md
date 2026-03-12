# mandelbrot-wasm

Interactive Mandelbrot set explorer rendered in WebAssembly with Rust.

Pan and zoom through the fractal, switch color palettes, and adjust iteration depth — all computed in WASM and drawn to an HTML canvas.

## Features

- **WASM renderer** — escape-time Mandelbrot with smooth coloring
- **Deep-zoom perturbation subsystem** — below scale `1e-6`, rendering switches to reference-orbit delta iteration with series-window bootstrapping, delta stability bailout, and per-frame orbit caching (`src/perturbation/`); status bar shows `perturbation`, orbit length, and rebase count
- **Pan / zoom** — drag to pan, scroll wheel to zoom toward the cursor; touch drag and pinch on mobile
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

A stripped-down canvas demo lives at http://localhost:8080/minimal.html — it uses the same WASM bindings through `www/explorer-host.js` with only pan, zoom, and a status line.

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

`www/app.js` imports `./explorer-host.js`, which loads `../pkg/mandelbrot_wasm.js`, calls `init()` to fetch/instantiate the module, binds the page canvas with `Explorer.bind_canvas()`, and presents frames through `render_to_canvas()` — the RGBA buffer is viewed directly from WASM linear memory via `web-sys`, avoiding a JS-side pixel copy.

### Minimal canvas host

`www/explorer-host.js` is the shared WASM-to-canvas bridge. It wraps module boot, canvas binding, `requestAnimationFrame` scheduling, responsive sizing, and pointer pan/zoom so host pages stay thin:

```js
import { ExplorerHost } from "./explorer-host.js";

const host = await ExplorerHost.create(canvas, {
  onPresent(explorer) {
    explorer.render_to_canvas();
  },
});
host.wirePointerPanZoom();
host.scheduleRender();
```

| Page | Script | What it adds beyond the host |
|------|--------|------------------------------|
| `minimal.html` | `www/minimal.js` | Status line only |
| `index.html` | `www/app.js` | Palettes, iterations, touch, URL hash |

### Native tests (no browser)

Core math and palette logic run under `cargo test` on the host triple — the same code paths compiled into the WASM module:

```bash
cargo test
```

Use this while iterating on `src/mandelbrot.rs`, `src/perturbation/`, or `src/palette.rs` before rebuilding `pkg/`.

### Development loop

Not every edit requires a full WASM rebuild:

| Changed files | What to run |
|---------------|-------------|
| `src/*.rs` | `cargo test`, then `./scripts/build.sh`, then refresh the browser |
| `www/*.js`, `index.html`, `minimal.html` | Refresh the browser only |
| `Cargo.toml` (deps or release profile) | `./scripts/build.sh` (add `--release` when tuning size) |

Run `./scripts/build.sh --help` for script options. The crate declares both `cdylib` and `rlib` in `Cargo.toml`: `cdylib` is what `wasm-pack` links for the browser, while `rlib` lets `cargo test` compile the same sources on your host triple without a WASM target.

### Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `Failed to load WASM` / missing `pkg/` import | Build has not been run | `./scripts/build.sh` from the repo root |
| Page works from a server but not when opened as a file | Browsers block `fetch()` of `.wasm` over `file://` | Serve over HTTP (`python3 -m http.server 8080`) |
| `wasm-pack is required` | Tool not on `PATH` | Install from the [wasm-pack installer](https://rustwasm.github.io/wasm-pack/installer/) |
| Stale fractal after editing Rust | Old artifacts in `pkg/` | Re-run `./scripts/build.sh` and hard-refresh the tab |

## Development

Release builds enable size optimizations (`opt-level = "s"`, LTO). Pass `--release` to `scripts/build.sh` or `wasm-pack` when measuring download size or profiling frame time.

## Project layout

```
├── src/
│   ├── lib.rs              # wasm-bindgen Explorer API
│   ├── canvas.rs           # web-sys canvas presenter (zero-copy blit)
│   ├── mandelbrot.rs       # viewport, escape-time, render loop
│   ├── perturbation/       # deep-zoom reference-orbit perturbation subsystem
│   │   ├── mod.rs          # threshold heuristic and integration tests
│   │   ├── reference.rs    # reference orbit + OrbitBackend trait stub
│   │   ├── delta.rs        # quadratic delta iteration with bailout
│   │   ├── series.rs       # linear series-approximation window stub
│   │   ├── stability.rs    # delta magnitude and glitch heuristics
│   │   └── session.rs      # cached reference-orbit session / rebasing
│   └── palette.rs          # color theme definitions
├── www/
│   ├── explorer-host.js  # shared WASM-to-canvas bridge
│   ├── minimal.js        # minimal demo UI
│   └── app.js            # full explorer UI and input handling
├── index.html          # full explorer page
├── minimal.html        # minimal WASM canvas demo
└── scripts/
    ├── build.sh        # wasm-pack build helper
    └── commit-at.sh
```

## Controls

| Input | Action |
|-------|--------|
| Drag | Pan |
| Scroll wheel | Zoom at cursor |
| Touch drag | Pan (one finger) |
| Pinch | Zoom toward gesture center |
| Palette dropdown | Change color theme |
| Keys `1`–`4` | Quick-switch palettes |
| Iterations slider | Increase/decrease detail |
| Reset view | Return to default framing |
| URL hash | Share or bookmark the current viewport (`#re,im,scale`) |

## License

MIT
