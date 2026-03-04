mod mandelbrot;
mod palette;

pub use mandelbrot::{escape_time, render, Viewport};
pub use palette::Palette;

use wasm_bindgen::prelude::*;

/// WASM-facing renderer state for the interactive explorer.
#[wasm_bindgen]
pub struct Explorer {
    viewport: Viewport,
    width: u32,
    height: u32,
    max_iter: u32,
    palette: Palette,
    buffer: Vec<u8>,
}

#[wasm_bindgen]
impl Explorer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Explorer {
        let pixel_count = (width * height * 4) as usize;
        Explorer {
            viewport: Viewport::default(),
            width,
            height,
            max_iter: 256,
            palette: Palette::Classic,
            buffer: vec![0; pixel_count],
        }
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
        self.viewport
            .pan(dx, dy, self.width, self.height);
    }

    pub fn zoom(&mut self, factor: f64, focus_x: f64, focus_y: f64) {
        self.viewport
            .zoom(factor, focus_x, focus_y, self.width, self.height);
    }

    pub fn render_frame(&mut self) {
        render(
            &mut self.buffer,
            self.width,
            self.height,
            self.viewport,
            self.max_iter,
            self.palette,
        );
    }

    pub fn pixels(&self) -> Vec<u8> {
        self.buffer.clone()
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
