//! Deep-zoom perturbation rendering stub.
//!
//! At extreme magnifications, neighboring pixels differ by amounts below f64
//! ulp spacing while the orbit values grow large. Perturbation theory tracks a
//! reference orbit at the viewport center and iterates a small delta for each
//! pixel: `δ_{n+1} = 2 z_n δ_n + δ_n² + Δc`. A future high-precision reference
//! orbit would unlock arbitrary-depth zoom; this module wires the selection
//! heuristic, reference-orbit cache, and delta iteration into the render path.

/// Viewport scale below which the perturbation renderer is selected.
pub const DEEP_ZOOM_SCALE_THRESHOLD: f64 = 1e-6;

/// Returns true when the viewport is zoomed deeply enough to prefer perturbation.
pub fn should_use_perturbation(scale: f64) -> bool {
    scale < DEEP_ZOOM_SCALE_THRESHOLD
}

/// Precomputed reference orbit for the viewport center `c*`.
#[derive(Debug, Clone)]
pub struct ReferenceOrbit {
    pub c_re: f64,
    pub c_im: f64,
    orbit_re: Vec<f64>,
    orbit_im: Vec<f64>,
}

impl ReferenceOrbit {
    /// Build the reference orbit `z*_{n+1} = z*² + c*` up to `max_iter` steps.
    pub fn build(c_re: f64, c_im: f64, max_iter: u32) -> Self {
        let cap = max_iter as usize + 1;
        let mut orbit_re = Vec::with_capacity(cap);
        let mut orbit_im = Vec::with_capacity(cap);
        orbit_re.push(0.0);
        orbit_im.push(0.0);

        let mut z_re = 0.0;
        let mut z_im = 0.0;
        for _ in 0..max_iter {
            let z_re2 = z_re * z_re;
            let z_im2 = z_im * z_im;
            if z_re2 + z_im2 > 4.0 {
                break;
            }
            z_im = 2.0 * z_re * z_im + c_im;
            z_re = z_re2 - z_im2 + c_re;
            orbit_re.push(z_re);
            orbit_im.push(z_im);
        }

        Self {
            c_re,
            c_im,
            orbit_re,
            orbit_im,
        }
    }

    pub fn len(&self) -> usize {
        self.orbit_re.len()
    }
}

/// Smooth escape count via first-order perturbation around `reference`.
pub fn perturbation_escape_time(
    c_re: f64,
    c_im: f64,
    reference: &ReferenceOrbit,
    max_iter: u32,
) -> f64 {
    let dc_re = c_re - reference.c_re;
    let dc_im = c_im - reference.c_im;

    let mut d_re = 0.0;
    let mut d_im = 0.0;
    let steps = max_iter.min(reference.len() as u32);

    for n in 0..steps {
        let z_re = reference.orbit_re[n as usize];
        let z_im = reference.orbit_im[n as usize];

        let combined_re = z_re + d_re;
        let combined_im = z_im + d_im;
        let mag2 = combined_re * combined_re + combined_im * combined_im;
        if mag2 > 4.0 {
            let log_zn = mag2.ln() / 2.0;
            let nu = (log_zn / 2.0_f64.ln()).ln() / 2.0_f64.ln();
            return n as f64 + 1.0 - nu;
        }

        let d_re2 = d_re * d_re;
        let d_im2 = d_im * d_im;

        d_re = 2.0 * (z_re * d_re - z_im * d_im) + (d_re2 - d_im2) + dc_re;
        d_im = 2.0 * (z_re * d_im + z_im * d_re) + 2.0 * d_re * d_im + dc_im;
    }

    max_iter as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mandelbrot::escape_time;

    #[test]
    fn threshold_selects_deep_zoom_only() {
        assert!(!should_use_perturbation(1e-5));
        assert!(should_use_perturbation(1e-7));
        assert!(should_use_perturbation(DEEP_ZOOM_SCALE_THRESHOLD * 0.5));
    }

    #[test]
    fn reference_orbit_starts_at_origin() {
        let orbit = ReferenceOrbit::build(-0.75, 0.1, 64);
        assert_eq!(orbit.orbit_re[0], 0.0);
        assert_eq!(orbit.orbit_im[0], 0.0);
        assert!(orbit.len() > 1);
    }

    #[test]
    fn perturbation_matches_direct_escape_at_center() {
        let c_re = -0.75;
        let c_im = 0.1;
        let max_iter = 256;
        let reference = ReferenceOrbit::build(c_re, c_im, max_iter);
        let direct = escape_time(c_re, c_im, max_iter);
        let perturbed = perturbation_escape_time(c_re, c_im, &reference, max_iter);
        assert!(
            (direct - perturbed).abs() < 1e-9,
            "center mismatch: direct={direct}, perturbed={perturbed}"
        );
    }

    #[test]
    fn perturbation_tracks_nearby_point() {
        let c_re = -0.75;
        let c_im = 0.1;
        let max_iter = 512;
        let reference = ReferenceOrbit::build(c_re, c_im, max_iter);
        let nearby_re = c_re + 1e-9;
        let nearby_im = c_im - 5e-10;
        let direct = escape_time(nearby_re, nearby_im, max_iter);
        let perturbed = perturbation_escape_time(nearby_re, nearby_im, &reference, max_iter);
        assert!(
            (direct - perturbed).abs() < 1e-5,
            "nearby mismatch: direct={direct}, perturbed={perturbed}"
        );
    }

    #[test]
    fn escaping_point_returns_fractional_count() {
        let reference = ReferenceOrbit::build(0.0, 0.0, 64);
        let escape = perturbation_escape_time(2.0, 2.0, &reference, 256);
        assert!(escape < 10.0);
        assert!(escape.fract() > 0.0);
    }
}
