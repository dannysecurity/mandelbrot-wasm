use crate::palette::Palette;
use crate::perturbation::{
    perturbation_escape_time, should_use_perturbation, PerturbationSession,
};

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
    /// Build a viewport with scale clamped to the same limits used by zoom.
    pub fn with_clamped(center_re: f64, center_im: f64, scale: f64) -> Self {
        Self {
            center_re,
            center_im,
            scale: scale.clamp(1e-14, 1e6),
        }
    }

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

/// Whether the given viewport would use perturbation-based rendering.
pub fn uses_perturbation(viewport: Viewport) -> bool {
    should_use_perturbation(viewport.scale)
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
    render_with_session(buffer, width, height, viewport, max_iter, palette, None);
}

/// Render with an optional perturbation session for reference-orbit reuse.
pub fn render_with_session(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    viewport: Viewport,
    max_iter: u32,
    palette: Palette,
    session: Option<&mut PerturbationSession>,
) {
    assert_eq!(buffer.len(), (width * height * 4) as usize);

    if should_use_perturbation(viewport.scale) {
        render_perturbation(buffer, width, height, viewport, max_iter, palette, session);
        return;
    }

    for y in 0..height {
        for x in 0..width {
            let (c_re, c_im) = viewport.map_pixel(x, y, width, height);
            let escape = escape_time(c_re, c_im, max_iter);
            write_pixel(buffer, width, x, y, escape, max_iter, palette);
        }
    }
}

fn render_perturbation(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    viewport: Viewport,
    max_iter: u32,
    palette: Palette,
    session: Option<&mut PerturbationSession>,
) {
    let mut local_session;
    let reference = match session {
        Some(s) => s.orbit_for(viewport.center_re, viewport.center_im, max_iter),
        None => {
            local_session = PerturbationSession::new();
            local_session.orbit_for(viewport.center_re, viewport.center_im, max_iter)
        }
    };

    for y in 0..height {
        for x in 0..width {
            let (c_re, c_im) = viewport.map_pixel(x, y, width, height);
            let escape = perturbation_escape_time(c_re, c_im, reference, max_iter);
            write_pixel(buffer, width, x, y, escape, max_iter, palette);
        }
    }
}

fn write_pixel(
    buffer: &mut [u8],
    width: u32,
    x: u32,
    y: u32,
    escape: f64,
    max_iter: u32,
    palette: Palette,
) {
    // The renormalized escape count is smooth in its fractional part; using
    // that directly avoids banding from scaling by max_iter.
    let t = if escape >= max_iter as f64 {
        0.0
    } else {
        escape.fract()
    };
    let color = palette.sample(t);
    let idx = ((y * width + x) * 4) as usize;
    buffer[idx..idx + 4].copy_from_slice(&color);
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
    fn smooth_escape_uses_fractional_part() {
        let c_re = -0.75;
        let c_im = 0.1;
        let max_iter = 512;
        let escape = escape_time(c_re, c_im, max_iter);
        assert!(escape < max_iter as f64);
        assert!(escape.fract() > 0.0);
        assert!(escape.fract() < 1.0);
    }

    #[test]
    fn zoom_clamps_extremes() {
        let mut vp = Viewport::default();
        vp.zoom(1e-20, 0.0, 0.0, 100, 100);
        assert!(vp.scale >= 1e-14);
        vp.zoom(1e-20, 0.0, 0.0, 100, 100);
        assert!(vp.scale <= 1e6);
    }

    #[test]
    fn pan_shifts_center() {
        let mut vp = Viewport::default();
        let before = (vp.center_re, vp.center_im);
        vp.pan(50.0, -25.0, 200, 100);
        assert_ne!(vp.center_re, before.0);
        assert_ne!(vp.center_im, before.1);
    }

    #[test]
    fn with_clamped_limits_scale() {
        let vp = Viewport::with_clamped(0.0, 0.0, 1e-20);
        assert_eq!(vp.scale, 1e-14);
        let vp = Viewport::with_clamped(0.0, 0.0, 1e20);
        assert_eq!(vp.scale, 1e6);
    }

    #[test]
    fn deep_viewport_selects_perturbation() {
        let shallow = Viewport::default();
        assert!(!uses_perturbation(shallow));
        let deep = Viewport::with_clamped(-0.75, 0.1, 1e-8);
        assert!(uses_perturbation(deep));
    }

    #[test]
    fn perturbation_render_fills_buffer() {
        let mut buf = vec![0u8; 4 * 4 * 4];
        render(
            &mut buf,
            4,
            4,
            Viewport::with_clamped(-0.75, 0.1, 1e-8),
            128,
            Palette::Classic,
        );
        assert!(buf.iter().any(|&b| b > 0));
    }

    #[test]
    fn session_reuses_reference_orbit_across_renders() {
        let mut buf = vec![0u8; 4 * 4 * 4];
        let mut session = PerturbationSession::new();
        let viewport = Viewport::with_clamped(-0.75, 0.1, 1e-8);
        render_with_session(
            &mut buf,
            4,
            4,
            viewport,
            128,
            Palette::Classic,
            Some(&mut session),
        );
        assert_eq!(session.rebase_count(), 1);
        render_with_session(
            &mut buf,
            4,
            4,
            viewport,
            128,
            Palette::Classic,
            Some(&mut session),
        );
        assert_eq!(session.rebase_count(), 1);
        assert!(session.reference_orbit_len() > 1);
    }
}
