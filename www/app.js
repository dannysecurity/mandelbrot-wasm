import { Explorer, ExplorerHost, canvasPoint } from "./explorer-host.js";

const canvas = document.getElementById("canvas");
const stage = document.querySelector(".stage");
const status = document.getElementById("status");
const paletteSelect = document.getElementById("palette");
const iterationsInput = document.getElementById("iterations");
const resetButton = document.getElementById("reset");

let host;
let dragging = false;
let lastX = 0;
let lastY = 0;
let hashTimer = null;
let activeTouches = new Map();
let lastPinchDistance = 0;

function explorer() {
  return host.explorer;
}

function fillPaletteOptions() {
  paletteSelect.replaceChildren();
  const count = explorer().palette_count();
  for (let index = 0; index < count; index += 1) {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = Explorer.palette_name_at(index);
    paletteSelect.appendChild(option);
  }
}

function updateStatus() {
  const renderMode = explorer().uses_perturbation_rendering()
    ? "perturbation"
    : "direct";
  const orbitLen = explorer().reference_orbit_length();
  const rebases = explorer().perturbation_rebase_count();
  const deepZoomMeta =
    renderMode === "perturbation"
      ? ` · orbit ${orbitLen} · rebases ${rebases}`
      : "";
  status.textContent =
    `center ${explorer().center_re().toFixed(6)} + ${explorer().center_im().toFixed(6)}i · ` +
    `scale ${explorer().scale().toExponential(3)} · ${renderMode}${deepZoomMeta} · ${explorer().palette_name()} · ` +
    `${explorer().max_iterations()} iterations · ${explorer().width()}×${explorer().height()}px`;
}

function scheduleHashUpdate() {
  if (hashTimer !== null) {
    clearTimeout(hashTimer);
  }
  hashTimer = setTimeout(() => {
    hashTimer = null;
    const re = explorer().center_re();
    const im = explorer().center_im();
    const scale = explorer().scale();
    location.hash = `${re.toFixed(6)},${im.toFixed(6)},${scale.toExponential(3)}`;
  }, 250);
}

function applyViewportFromHash() {
  const raw = location.hash.slice(1);
  if (!raw) {
    return;
  }
  const [re, im, scale] = raw.split(",").map(Number);
  if (![re, im, scale].every(Number.isFinite)) {
    return;
  }
  explorer().set_viewport(re, im, scale);
}

function scheduleRender() {
  host.scheduleRender();
}

function setPalette(index) {
  explorer().set_palette(index);
  paletteSelect.value = String(explorer().palette_index());
  scheduleRender();
}

function handlePanDelta(clientX, clientY) {
  const point = canvasPoint(canvas, clientX, clientY);
  const dx = point.x - lastX;
  const dy = point.y - lastY;
  lastX = point.x;
  lastY = point.y;
  explorer().pan(dx, dy);
  scheduleRender();
  scheduleHashUpdate();
}

function handlePinchZoom() {
  if (activeTouches.size !== 2) {
    lastPinchDistance = 0;
    return;
  }
  const [first, second] = [...activeTouches.values()];
  const distance = Math.hypot(second.x - first.x, second.y - first.y);
  if (lastPinchDistance > 0 && distance > 0) {
    const factor = distance / lastPinchDistance;
    const focus = canvasPoint(
      canvas,
      (first.clientX + second.clientX) / 2,
      (first.clientY + second.clientY) / 2,
    );
    explorer().zoom(factor, focus.x, focus.y);
    scheduleRender();
    scheduleHashUpdate();
  }
  lastPinchDistance = distance;
}

function wireInput() {
  canvas.addEventListener("mousedown", (event) => {
    dragging = true;
    const point = canvasPoint(canvas, event.clientX, event.clientY);
    lastX = point.x;
    lastY = point.y;
  });

  window.addEventListener("mouseup", () => {
    dragging = false;
  });

  canvas.addEventListener("mousemove", (event) => {
    if (!dragging) return;
    handlePanDelta(event.clientX, event.clientY);
  });

  canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const factor = event.deltaY < 0 ? 1.15 : 1 / 1.15;
      const point = canvasPoint(canvas, event.clientX, event.clientY);
      explorer().zoom(factor, point.x, point.y);
      scheduleRender();
      scheduleHashUpdate();
    },
    { passive: false },
  );

  canvas.addEventListener(
    "touchstart",
    (event) => {
      event.preventDefault();
      for (const touch of event.changedTouches) {
        activeTouches.set(touch.identifier, {
          clientX: touch.clientX,
          clientY: touch.clientY,
          x: touch.clientX,
          y: touch.clientY,
        });
      }
      if (activeTouches.size === 1) {
        const touch = [...activeTouches.values()][0];
        const point = canvasPoint(canvas, touch.clientX, touch.clientY);
        lastX = point.x;
        lastY = point.y;
      }
      if (activeTouches.size === 2) {
        lastPinchDistance = 0;
        handlePinchZoom();
      }
    },
    { passive: false },
  );

  canvas.addEventListener(
    "touchmove",
    (event) => {
      event.preventDefault();
      for (const touch of event.changedTouches) {
        const existing = activeTouches.get(touch.identifier);
        if (!existing) continue;
        activeTouches.set(touch.identifier, {
          clientX: touch.clientX,
          clientY: touch.clientY,
          x: touch.clientX,
          y: touch.clientY,
        });
      }
      if (activeTouches.size === 1) {
        const touch = [...activeTouches.values()][0];
        handlePanDelta(touch.clientX, touch.clientY);
      } else if (activeTouches.size === 2) {
        handlePinchZoom();
      }
    },
    { passive: false },
  );

  const endTouch = (event) => {
    for (const touch of event.changedTouches) {
      activeTouches.delete(touch.identifier);
    }
    if (activeTouches.size < 2) {
      lastPinchDistance = 0;
    }
  };

  canvas.addEventListener("touchend", endTouch);
  canvas.addEventListener("touchcancel", endTouch);

  paletteSelect.addEventListener("change", () => {
    setPalette(Number(paletteSelect.value));
  });

  window.addEventListener("keydown", (event) => {
    const digit = Number(event.key);
    if (digit >= 1 && digit <= explorer().palette_count()) {
      setPalette(digit - 1);
    }
  });

  iterationsInput.addEventListener("input", () => {
    explorer().set_max_iterations(Number(iterationsInput.value));
    scheduleRender();
  });

  resetButton.addEventListener("click", () => {
    explorer().reset_view();
    scheduleRender();
    scheduleHashUpdate();
  });

  window.addEventListener("hashchange", () => {
    applyViewportFromHash();
    scheduleRender();
  });

  host.observeResize(stage);
}

async function boot() {
  host = await ExplorerHost.create(canvas, {
    onPresent(explorer) {
      try {
        explorer.render_to_canvas();
        updateStatus();
      } catch (error) {
        status.textContent = `Render failed: ${error}`;
        console.error(error);
      }
    },
  });
  applyViewportFromHash();

  fillPaletteOptions();
  paletteSelect.value = String(explorer().palette_index());
  wireInput();
  scheduleRender();
}

boot().catch((error) => {
  status.textContent = `Failed to load WASM: ${error}`;
  console.error(error);
});
