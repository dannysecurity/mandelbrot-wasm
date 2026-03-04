#!/usr/bin/env bash
# Compile the Rust crate to WebAssembly and emit browser-ready JS bindings in pkg/.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RELEASE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      RELEASE=1
      shift
      ;;
    -h|--help)
      cat <<'EOF'
Usage: scripts/build.sh [--release]

Build mandelbrot-wasm for the browser with wasm-pack.

  --release   Enable release optimizations (opt-level = "s", LTO)

Outputs land in pkg/:
  mandelbrot_wasm_bg.wasm   compiled module
  mandelbrot_wasm.js        wasm-bindgen glue + ES module exports
  mandelbrot_wasm_bg.js     low-level WASM loader
  mandelbrot_wasm.d.ts      TypeScript declarations (optional tooling)

Serve the repo root and open index.html after building:
  python3 -m http.server 8080
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1 (try --help)" >&2
      exit 1
      ;;
  esac
done

if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
  echo "Adding wasm32-unknown-unknown target…"
  rustup target add wasm32-unknown-unknown
fi

PACK_ARGS=(build --target web --out-dir pkg)
if [[ "$RELEASE" -eq 1 ]]; then
  PACK_ARGS+=(--release)
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required. Install it from https://rustwasm.github.io/wasm-pack/installer/" >&2
  exit 1
fi

echo "Running: wasm-pack ${PACK_ARGS[*]}"
wasm-pack "${PACK_ARGS[@]}"

echo
echo "Build complete. Start a static server from the project root:"
echo "  python3 -m http.server 8080"
echo "Then open http://localhost:8080"
