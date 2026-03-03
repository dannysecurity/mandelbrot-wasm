import init, { Explorer } from "../pkg/mandelbrot_wasm.js";

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const status = document.getElementById("status");
const paletteSelect = document.getElementById("palette");
const iterationsInput = document.getElementById("iterations");
const resetButton = document.getElementById("reset");

const PALETTES = ["Classic", "Fire", "Ocean", "Grayscale"];

let explorer;
let dragging = false;
let lastX = 0;
let lastY = 0;

function fillPaletteOptions() {
  PALETTES.forEach((name, index) => {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = name;
    paletteSelect.appendChild(option);
  });
}

function drawFrame() {
  explorer.render_frame();
  const pixels = explorer.pixels();
  const image = new ImageData(
    new Uint8ClampedArray(pixels),
    canvas.width,
    canvas.height,
  );
  ctx.putImageData(image, 0, 0);
  status.textContent =
    `center ${explorer.center_re().toFixed(6)} + ${explorer.center_im().toFixed(6)}i · ` +
    `scale ${explorer.scale().toExponential(3)} · ${explorer.palette_name()}`;
}

function scheduleRender() {
  requestAnimationFrame(drawFrame);
}

async function boot() {
  fillPaletteOptions();
  await init();
  explorer = new Explorer(canvas.width, canvas.height);
  drawFrame();

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
    explorer.set_palette(Number(paletteSelect.value));
    scheduleRender();
  });

  iterationsInput.addEventListener("input", () => {
    explorer.set_max_iterations(Number(iterationsInput.value));
    scheduleRender();
  });

  resetButton.addEventListener("click", () => {
    explorer.reset_view();
    scheduleRender();
  });
}

boot().catch((error) => {
  status.textContent = `Failed to load WASM: ${error}`;
  console.error(error);
});
