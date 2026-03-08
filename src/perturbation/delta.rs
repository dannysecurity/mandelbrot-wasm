use super::reference::ReferenceOrbit;
use super::series::SeriesWindow;
use super::stability::{DeltaStability, StabilityOutcome};

/// Smooth escape count via first-order perturbation around `reference`.
///
/// Uses the series window for early iterations, then full quadratic delta
/// iteration with stability bailout when |δ| grows too large.
pub fn perturbation_escape_time(
    c_re: f64,
    c_im: f64,
    reference: &ReferenceOrbit,
    max_iter: u32,
) -> f64 {
    let dc_re = c_re - reference.c_re;
    let dc_im = c_im - reference.c_im;
    let stability = DeltaStability::default();
    let series = SeriesWindow::default();

    let (mut d_re, mut d_im, start_n) = series.advance_linear(reference, dc_re, dc_im, 0);
    let steps = max_iter.min(reference.len() as u32);

    for n in start_n..steps {
        let (z_re, z_im) = reference.z_at(n as usize);

        match stability.check(z_re, z_im, d_re, d_im) {
            StabilityOutcome::Stable => {}
            StabilityOutcome::DeltaBailout | StabilityOutcome::GlitchSuspected => {
                return max_iter as f64;
            }
        }

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

    #[test]
    fn glitch_suspected_returns_max_iter() {
        let reference = ReferenceOrbit::synthetic(0.0, 0.0, vec![0.0, 1e13], vec![0.0, 0.0]);
        let escape = perturbation_escape_time(0.0, 1e-20, &reference, 64);
        assert_eq!(escape, 64.0);
    }
}
