import init, { Explorer } from "../pkg/mandelbrot_wasm.js";

const canvas = document.getElementById("canvas");
const status = document.getElementById("status");
const paletteSelect = document.getElementById("palette");
const iterationsInput = document.getElementById("iterations");
const resetButton = document.getElementById("reset");

let explorer;
let dragging = false;
let lastX = 0;
let lastY = 0;
let renderQueued = false;

function fillPaletteOptions() {
  paletteSelect.replaceChildren();
  const count = explorer.palette_count();
  for (let index = 0; index < count; index += 1) {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = Explorer.palette_name_at(index);
    paletteSelect.appendChild(option);
  }
}

function updateStatus() {
  status.textContent =
    `center ${explorer.center_re().toFixed(6)} + ${explorer.center_im().toFixed(6)}i · ` +
    `scale ${explorer.scale().toExponential(3)} · ${explorer.palette_name()} · ` +
    `${explorer.max_iterations()} iterations · ${explorer.width()}×${explorer.height()}px`;
}

function presentFrame() {
  renderQueued = false;
  try {
    explorer.render_to_canvas();
    updateStatus();
  } catch (error) {
    status.textContent = `Render failed: ${error}`;
    console.error(error);
  }
}

function scheduleRender() {
  if (renderQueued) return;
  renderQueued = true;
  requestAnimationFrame(presentFrame);
}

function setPalette(index) {
  explorer.set_palette(index);
  paletteSelect.value = String(explorer.palette_index());
  scheduleRender();
}

function syncCanvasSize() {
  const width = canvas.clientWidth || canvas.width;
  const height = canvas.clientHeight || canvas.height;
  if (width === explorer.width() && height === explorer.height()) {
    return;
  }
  canvas.width = width;
  canvas.height = height;
  explorer.resize(width, height);
}

function wireInput() {
  canvas.addEventListener("mousedown", (event) => {
    dragging = true;
    lastX = event.offsetX;
    lastY = event.offsetY;
  });

  window.addEventListener("mouseup", () => {
    dragging = false;
  });

  canvas.addEventListener("mousemove", (event) => {
    if (!dragging) return;
    const dx = event.offsetX - lastX;
    const dy = event.offsetY - lastY;
    lastX = event.offsetX;
    lastY = event.offsetY;
    explorer.pan(dx, dy);
    scheduleRender();
  });

  canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const factor = event.deltaY < 0 ? 1.15 : 1 / 1.15;
      explorer.zoom(factor, event.offsetX, event.offsetY);
      scheduleRender();
    },
    { passive: false },
  );

  paletteSelect.addEventListener("change", () => {
    setPalette(Number(paletteSelect.value));
  });

  window.addEventListener("keydown", (event) => {
    const digit = Number(event.key);
    if (digit >= 1 && digit <= explorer.palette_count()) {
      setPalette(digit - 1);
    }
  });

  iterationsInput.addEventListener("input", () => {
    explorer.set_max_iterations(Number(iterationsInput.value));
    scheduleRender();
  });

  resetButton.addEventListener("click", () => {
    explorer.reset_view();
    scheduleRender();
  });

  window.addEventListener("resize", () => {
    syncCanvasSize();
    scheduleRender();
  });
}

async function boot() {
  await init();

  syncCanvasSize();
  explorer = new Explorer(canvas.width, canvas.height);
  explorer.bind_canvas(canvas);

  fillPaletteOptions();
  paletteSelect.value = String(explorer.palette_index());
  wireInput();
  scheduleRender();
}

boot().catch((error) => {
  status.textContent = `Failed to load WASM: ${error}`;
  console.error(error);
});
