mod canvas;
mod mandelbrot;
mod palette;

pub use canvas::{buffer_matches_dimensions, pixel_count, CanvasPresenter};
pub use mandelbrot::{escape_time, render, Viewport};
pub use palette::Palette;

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

/// WASM-facing renderer state for the interactive explorer.
#[wasm_bindgen]
pub struct Explorer {
    viewport: Viewport,
    width: u32,
    height: u32,
    max_iter: u32,
    palette: Palette,
    buffer: Vec<u8>,
    canvas: Option<CanvasPresenter>,
}

#[wasm_bindgen]
impl Explorer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Explorer {
        let (width, height) = normalize_dimensions(width, height);
        Explorer {
            viewport: Viewport::default(),
            width,
            height,
            max_iter: 256,
            palette: Palette::Classic,
            buffer: vec![0; pixel_count(width, height)],
            canvas: None,
        }
    }

    /// Attach a DOM canvas so frames can be presented without copying through JS.
    pub fn bind_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        let presenter = CanvasPresenter::attach(canvas)?;
        let (width, height) = presenter.dimensions();
        self.resize_internal(width, height);
        self.canvas = Some(presenter);
        Ok(())
    }

    pub fn is_canvas_bound(&self) -> bool {
        self.canvas.is_some()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_internal(width, height);
    }

    pub fn reset_view(&mut self) {
        self.viewport = Viewport::default();
    }

    pub fn set_palette(&mut self, index: u32) {
        self.palette = Palette::from_index(index as usize);
    }

    pub fn set_max_iterations(&mut self, max_iter: u32) {
        self.max_iter = max_iter.clamp(32, 4096);
    }

    pub fn max_iterations(&self) -> u32 {
        self.max_iter
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.viewport.pan(dx, dy, self.width, self.height);
    }

    pub fn zoom(&mut self, factor: f64, focus_x: f64, focus_y: f64) {
        self.viewport
            .zoom(factor, focus_x, focus_y, self.width, self.height);
    }

    /// Render into the internal RGBA buffer only (no canvas presentation).
    pub fn render_frame(&mut self) {
        self.render_into_buffer();
    }

    /// Render and, when a canvas is bound, blit directly from WASM linear memory.
    pub fn render_to_canvas(&mut self) -> Result<(), JsValue> {
        self.render_into_buffer();
        if let Some(canvas) = &self.canvas {
            canvas.blit_rgba(&self.buffer, self.width, self.height)?;
        }
        Ok(())
    }

    /// Copy of the RGBA buffer for hosts that prefer manual ImageData wiring.
    pub fn pixels(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn palette_name(&self) -> String {
        self.palette.name().to_string()
    }

    pub fn palette_index(&self) -> u32 {
        Palette::ALL
            .iter()
            .position(|&p| p == self.palette)
            .unwrap_or(0) as u32
    }

    pub fn palette_count(&self) -> u32 {
        Palette::ALL.len() as u32
    }

    pub fn palette_name_at(index: u32) -> String {
        Palette::from_index(index as usize).name().to_string()
    }

    pub fn center_re(&self) -> f64 {
        self.viewport.center_re
    }

    pub fn center_im(&self) -> f64 {
        self.viewport.center_im
    }

    pub fn scale(&self) -> f64 {
        self.viewport.scale
    }
}

impl Explorer {
    fn render_into_buffer(&mut self) {
        render(
            &mut self.buffer,
            self.width,
            self.height,
            self.viewport,
            self.max_iter,
            self.palette,
        );
    }

    fn resize_internal(&mut self, width: u32, height: u32) {
        let (width, height) = normalize_dimensions(width, height);
        if width == self.width && height == self.height {
            return;
        }
        self.width = width;
        self.height = height;
        self.buffer.resize(pixel_count(width, height), 0);
    }
}

fn normalize_dimensions(width: u32, height: u32) -> (u32, u32) {
    (width.max(1), height.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_starts_without_canvas() {
        let explorer = Explorer::new(64, 48);
        assert!(!explorer.is_canvas_bound());
        assert_eq!(explorer.width(), 64);
        assert_eq!(explorer.height(), 48);
        assert_eq!(explorer.buffer.len(), pixel_count(64, 48));
    }

    #[test]
    fn resize_reallocates_buffer() {
        let mut explorer = Explorer::new(10, 10);
        explorer.resize(20, 15);
        assert_eq!(explorer.width(), 20);
        assert_eq!(explorer.height(), 15);
        assert_eq!(explorer.buffer.len(), pixel_count(20, 15));
    }

    #[test]
    fn zero_dimensions_clamp_to_one_pixel() {
        let explorer = Explorer::new(0, 0);
        assert_eq!(explorer.width(), 1);
        assert_eq!(explorer.height(), 1);
    }
}
