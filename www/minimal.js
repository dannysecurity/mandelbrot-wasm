import { ExplorerHost } from "./explorer-host.js";

const canvas = document.getElementById("canvas");
const stage = document.querySelector(".stage");
const status = document.getElementById("status");

function formatStatus(explorer) {
  const mode = explorer.uses_perturbation_rendering() ? "perturbation" : "direct";
  return (
    `center ${explorer.center_re().toFixed(4)} + ${explorer.center_im().toFixed(4)}i · ` +
    `scale ${explorer.scale().toExponential(2)} · ${mode} · ` +
    `${explorer.width()}×${explorer.height()}px · ${explorer.buffer_byte_length()} bytes`
  );
}

async function boot() {
  const host = await ExplorerHost.create(canvas, {
    onPresent(explorer) {
      explorer.render_to_canvas();
      status.textContent = formatStatus(explorer);
    },
  });
  host.wirePointerPanZoom({
    onViewChange: () => {
      status.textContent = formatStatus(host.explorer);
    },
  });
  host.observeResize(stage);
  host.scheduleRender();
}

boot().catch((error) => {
  status.textContent = `Failed to load WASM: ${error}`;
  console.error(error);
});
