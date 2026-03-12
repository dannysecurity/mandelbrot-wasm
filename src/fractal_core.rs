//! Shared Mandelbrot quadratic map iteration for escape-time rendering.
//!
//! Both the direct per-pixel path and the reference-orbit builder use the same
//! `z_{n+1} = z_n² + c` step and escape disc so perturbation stays aligned
//! with the standard escape-time loop.

/// Squared escape radius: points with `|z|² > ESCAPE_RADIUS_SQ` are outside.
pub const ESCAPE_RADIUS_SQ: f64 = 4.0;

/// One iteration of the Mandelbrot map in rectangular form.
#[inline]
pub fn mandelbrot_step(z_re: f64, z_im: f64, c_re: f64, c_im: f64) -> (f64, f64) {
    let z_re2 = z_re * z_re;
    let z_im2 = z_im * z_im;
    (z_re2 - z_im2 + c_re, 2.0 * z_re * z_im + c_im)
}

/// Returns true when `|z|²` lies outside the escape disc.
#[inline]
pub fn has_escaped(mag2: f64) -> bool {
    mag2 > ESCAPE_RADIUS_SQ
}

/// Renormalized smooth escape count at the given iteration index.
///
/// `mag2` is `|z|²` measured when escape is first detected.
pub fn smooth_escape_count(iter: u32, mag2: f64) -> f64 {
    let log_zn = mag2.ln() / 2.0;
    let nu = (log_zn / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
    iter as f64 + 1.0 - nu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_step_stays_at_origin() {
        let (z_re, z_im) = mandelbrot_step(0.0, 0.0, 0.0, 0.0);
        assert_eq!(z_re, 0.0);
        assert_eq!(z_im, 0.0);
    }

    #[test]
    fn escape_disc_is_strictly_outside_radius_two() {
        assert!(!has_escaped(4.0));
        assert!(has_escaped(4.0001));
    }

    #[test]
    fn smooth_count_is_fractional_on_escape() {
        let count = smooth_escape_count(3, 4.5);
        assert!(count > 3.0);
        assert!(count < 4.0);
        assert!(count.fract() > 0.0);
    }
}
