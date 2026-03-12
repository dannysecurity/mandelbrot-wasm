import init, { Explorer } from "../pkg/mandelbrot_wasm.js";

/** Map CSS/client coordinates to canvas backing-store pixels. */
export function canvasPoint(canvas, clientX, clientY) {
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  return {
    x: (clientX - rect.left) * scaleX,
    y: (clientY - rect.top) * scaleY,
  };
}

/** Coalesce render requests into one requestAnimationFrame callback. */
export class RenderScheduler {
  constructor(present) {
    this.present = present;
    this.queued = false;
  }

  schedule() {
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(() => {
      this.queued = false;
      this.present();
    });
  }
}

/**
 * Thin host that wires the WASM Explorer to a canvas element.
 * Handles module init, canvas binding, sizing, and frame presentation.
 */
export class ExplorerHost {
  constructor(explorer, canvas, { onPresent } = {}) {
    this.explorer = explorer;
    this.canvas = canvas;
    this.onPresent = onPresent;
    this.scheduler = new RenderScheduler(() => this.presentFrame());
  }

  static async create(canvas, options = {}) {
    await init();
    const explorer = new Explorer(canvas.width, canvas.height);
    explorer.bind_canvas(canvas);
    const host = new ExplorerHost(explorer, canvas, options);
    host.syncCanvasSize();
    return host;
  }

  scheduleRender() {
    this.scheduler.schedule();
  }

  presentFrame() {
    if (this.onPresent) {
      return this.onPresent(this.explorer);
    }
    return this.explorer.render_to_canvas();
  }

  syncCanvasSize() {
    const width = Math.max(1, Math.floor(this.canvas.clientWidth || this.canvas.width));
    const height = Math.max(1, Math.floor(this.canvas.clientHeight || this.canvas.height));
    if (width === this.explorer.width() && height === this.explorer.height()) {
      return false;
    }
    this.canvas.width = width;
    this.canvas.height = height;
    this.explorer.resize(width, height);
    return true;
  }

  observeResize(element) {
    const observer = new ResizeObserver(() => {
      if (this.syncCanvasSize()) {
        this.scheduleRender();
      }
    });
    observer.observe(element);
    return observer;
  }

  wirePointerPanZoom({ onViewChange } = {}) {
    let dragging = false;
    let lastX = 0;
    let lastY = 0;

    const notify = () => {
      onViewChange?.();
      this.scheduleRender();
    };

    const panDelta = (clientX, clientY) => {
      const point = canvasPoint(this.canvas, clientX, clientY);
      const dx = point.x - lastX;
      const dy = point.y - lastY;
      lastX = point.x;
      lastY = point.y;
      this.explorer.pan(dx, dy);
      notify();
    };

    this.canvas.addEventListener("mousedown", (event) => {
      dragging = true;
      const point = canvasPoint(this.canvas, event.clientX, event.clientY);
      lastX = point.x;
      lastY = point.y;
    });

    window.addEventListener("mouseup", () => {
      dragging = false;
    });

    this.canvas.addEventListener("mousemove", (event) => {
      if (!dragging) return;
      panDelta(event.clientX, event.clientY);
    });

    this.canvas.addEventListener(
      "wheel",
      (event) => {
        event.preventDefault();
        const factor = event.deltaY < 0 ? 1.15 : 1 / 1.15;
        const point = canvasPoint(this.canvas, event.clientX, event.clientY);
        this.explorer.zoom(factor, point.x, point.y);
        notify();
      },
      { passive: false },
    );
  }
}

export { Explorer };
