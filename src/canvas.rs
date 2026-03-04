use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

/// Browser canvas target for presenting RGBA pixel buffers from WASM memory.
pub struct CanvasPresenter {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl CanvasPresenter {
    pub fn attach(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2d context unavailable"))?
            .dyn_into::<CanvasRenderingContext2d>()?;
        Ok(Self { canvas, ctx })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.canvas.width(), self.canvas.height())
    }

    /// Blit an RGBA buffer into the bound canvas using a zero-copy memory view.
    pub fn blit_rgba(&self, buffer: &[u8], width: u32, height: u32) -> Result<(), JsValue> {
        if !buffer_matches_dimensions(buffer, width, height) {
            return Err(JsValue::from_str(&format!(
                "buffer length {} does not match {width}x{height} RGBA",
                buffer.len()
            )));
        }

        let image = ImageData::new_with_u8_clamped_array_and_sh(Clamped(buffer), width, height)?;
        self.ctx.put_image_data(&image, 0.0, 0.0)?;
        Ok(())
    }
}

/// Returns true when `buffer` holds exactly `width * height` RGBA pixels.
pub fn buffer_matches_dimensions(buffer: &[u8], width: u32, height: u32) -> bool {
    buffer.len() == pixel_count(width, height)
}

pub fn pixel_count(width: u32, height: u32) -> usize {
    (width as u64 * height as u64 * 4) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_count_scales_with_area() {
        assert_eq!(pixel_count(800, 600), 800 * 600 * 4);
        assert_eq!(pixel_count(1, 1), 4);
    }

    #[test]
    fn buffer_validation_rejects_mismatch() {
        let buf = vec![0u8; 16];
        assert!(buffer_matches_dimensions(&buf, 2, 2));
        assert!(!buffer_matches_dimensions(&buf, 3, 3));
    }

    #[test]
    fn zero_sized_canvas_has_empty_buffer() {
        assert_eq!(pixel_count(0, 600), 0);
        assert!(buffer_matches_dimensions(&[], 0, 0));
    }
}
