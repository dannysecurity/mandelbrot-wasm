//! Deep-zoom perturbation rendering subsystem.
//!
//! At extreme magnifications, neighboring pixels differ by amounts below f64
//! ulp spacing while the orbit values grow large. Perturbation theory tracks a
//! reference orbit at the viewport center and iterates a small delta for each
//! pixel: `δ_{n+1} = 2 z_n δ_n + δ_n² + Δc`. A future high-precision reference
//! orbit would unlock arbitrary-depth zoom; this module wires the selection
//! heuristic, reference-orbit cache, delta iteration, stability bailout, and
//! series-approximation hooks into the render path.

mod delta;
mod reference;
mod series;
mod session;
mod stability;

pub use delta::perturbation_escape_time;
pub use series::series_window_depth;
pub use session::PerturbationSession;

/// Viewport scale below which the perturbation renderer is selected.
pub const DEEP_ZOOM_SCALE_THRESHOLD: f64 = 1e-6;

/// Returns true when the viewport is zoomed deeply enough to prefer perturbation.
pub fn should_use_perturbation(scale: f64) -> bool {
    scale < DEEP_ZOOM_SCALE_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mandelbrot::escape_time;
    use crate::perturbation::reference::ReferenceOrbit;

    #[test]
    fn threshold_selects_deep_zoom_only() {
        assert!(!should_use_perturbation(1e-5));
        assert!(should_use_perturbation(1e-7));
        assert!(should_use_perturbation(DEEP_ZOOM_SCALE_THRESHOLD * 0.5));
    }

    #[test]
    fn threshold_boundary_is_strictly_less_than() {
        assert!(!should_use_perturbation(DEEP_ZOOM_SCALE_THRESHOLD));
        assert!(should_use_perturbation(nextafter_below(
            DEEP_ZOOM_SCALE_THRESHOLD
        )));
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
    fn perturbation_agrees_with_direct_on_nearby_grid() {
        let center_re = -0.75;
        let center_im = 0.1;
        let max_iter = 512;
        let reference = ReferenceOrbit::build(center_re, center_im, max_iter);
        let offsets = [
            (0.0, 0.0),
            (1e-9, 0.0),
            (-5e-10, 2e-10),
            (3e-10, -1e-9),
        ];
        for (dr, di) in offsets {
            let c_re = center_re + dr;
            let c_im = center_im + di;
            let direct = escape_time(c_re, c_im, max_iter);
            let perturbed = perturbation_escape_time(c_re, c_im, &reference, max_iter);
            assert!(
                (direct - perturbed).abs() < 1e-5,
                "offset ({dr}, {di}): direct={direct}, perturbed={perturbed}"
            );
        }
    }

    #[test]
    fn session_reuses_orbit_when_center_unchanged() {
        let mut session = PerturbationSession::new();
        let first = session.orbit_for(-0.75, 0.1, 256);
        let first_len = first.len();
        let second = session.orbit_for(-0.75, 0.1, 256);
        assert_eq!(second.len(), first_len);
        assert_eq!(session.rebase_count(), 1);
    }

    #[test]
    fn session_rebases_when_center_shifts() {
        let mut session = PerturbationSession::new();
        session.orbit_for(-0.75, 0.1, 256);
        session.orbit_for(-0.7500001, 0.1000001, 256);
        assert_eq!(session.rebase_count(), 2);
    }

    fn nextafter_below(x: f64) -> f64 {
        let bits = x.to_bits();
        if x == 0.0 {
            -f64::MIN_POSITIVE
        } else if bits & (1u64 << 63) != 0 {
            f64::from_bits(bits + 1)
        } else {
            f64::from_bits(bits - 1)
        }
    }
}
