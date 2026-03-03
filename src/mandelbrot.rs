use crate::palette::Palette;

/// View rectangle in the complex plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub center_re: f64,
    pub center_im: f64,
    pub scale: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center_re: -0.5,
            center_im: 0.0,
            scale: 3.0,
        }
    }
}

impl Viewport {
    pub fn pan(&mut self, dx_pixels: f64, dy_pixels: f64, width: u32, height: u32) {
        let w = width.max(1) as f64;
        let h = height.max(1) as f64;
        self.center_re -= (dx_pixels / w) * self.scale * 2.0;
        self.center_im += (dy_pixels / h) * self.scale * 2.0;
    }

    pub fn zoom(&mut self, factor: f64, focus_x: f64, focus_y: f64, width: u32, height: u32) {
        let w = width.max(1) as f64;
        let h = height.max(1) as f64;
        let aspect = w / h;

        let fx = self.center_re + (focus_x / w - 0.5) * self.scale * 2.0 * aspect;
        let fy = self.center_im + (focus_y / h - 0.5) * self.scale * 2.0;

        self.scale = (self.scale / factor).clamp(1e-14, 1e6);

        self.center_re = fx - (focus_x / w - 0.5) * self.scale * 2.0 * aspect;
        self.center_im = fy - (focus_y / h - 0.5) * self.scale * 2.0;
    }

    fn map_pixel(&self, x: u32, y: u32, width: u32, height: u32) -> (f64, f64) {
        let w = width.max(1) as f64;
        let h = height.max(1) as f64;
        let aspect = w / h;
        let re = self.center_re + (x as f64 / w - 0.5) * self.scale * 2.0 * aspect;
        let im = self.center_im + (y as f64 / h - 0.5) * self.scale * 2.0;
        (re, im)
    }
}

/// Smooth escape-time iteration count for point `c`.
pub fn escape_time(c_re: f64, c_im: f64, max_iter: u32) -> f64 {
    let mut z_re = 0.0;
    let mut z_im = 0.0;
    let mut iter = 0u32;

    while iter < max_iter {
        let z_re2 = z_re * z_re;
        let z_im2 = z_im * z_im;
        if z_re2 + z_im2 > 4.0 {
            let mag: f64 = z_re2 + z_im2;
            let log_zn = mag.ln() / 2.0_f64;
            let nu = (log_zn / 2.0_f64.ln()).ln() / 2.0_f64.ln();
            return iter as f64 + 1.0 - nu;
        }
        z_im = 2.0 * z_re * z_im + c_im;
        z_re = z_re2 - z_im2 + c_re;
        iter += 1;
    }

    max_iter as f64
}

/// Render the Mandelbrot set into an RGBA buffer.
pub fn render(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    viewport: Viewport,
    max_iter: u32,
    palette: Palette,
) {
    assert_eq!(buffer.len(), (width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let (c_re, c_im) = viewport.map_pixel(x, y, width, height);
            let escape = escape_time(c_re, c_im, max_iter);
            let t = if escape >= max_iter as f64 {
                0.0
            } else {
                (escape / max_iter as f64).fract()
            };
            let color = palette.sample(t);
            let idx = ((y * width + x) * 4) as usize;
            buffer[idx..idx + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_in_the_set() {
        assert_eq!(escape_time(0.0, 0.0, 256), 256.0);
    }

    #[test]
    fn point_outside_escapes_quickly() {
        assert!(escape_time(2.0, 2.0, 256) < 10.0);
    }

    #[test]
    fn render_fills_buffer() {
        let mut buf = vec![0u8; 4 * 4 * 4];
        render(
            &mut buf,
            4,
            4,
            Viewport::default(),
            64,
            Palette::Classic,
        );
        assert!(buf.iter().any(|&b| b > 0));
    }

    #[test]
    fn zoom_clamps_extremes() {
        let mut vp = Viewport::default();
        vp.zoom(1e-20, 0.0, 0.0, 100, 100);
        assert!(vp.scale >= 1e-14);
        vp.zoom(1e-20, 0.0, 0.0, 100, 100);
        assert!(vp.scale <= 1e6);
    }
}
