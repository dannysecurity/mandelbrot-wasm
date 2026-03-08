use super::reference::ReferenceOrbit;

/// Number of early iterations handled by the series-approximation window.
pub const SERIES_WINDOW_DEPTH: u32 = 4;

/// Returns the configured series window depth (stub for future SSA tuning).
pub fn series_window_depth() -> u32 {
    SERIES_WINDOW_DEPTH
}

/// Taylor-window stub for the first few perturbation steps when |Δc| is tiny.
///
/// For small `dc`, the quadratic δ² term is negligible in the opening iterations,
/// so δ_n ≈ Δc · (2 z*_{n-1})...(2 z*_0). This mirrors the linearized series
/// used in series approximation algorithms before full delta iteration resumes.
#[derive(Debug, Clone, Copy)]
pub struct SeriesWindow {
    pub depth: u32,
}

impl Default for SeriesWindow {
    fn default() -> Self {
        Self {
            depth: SERIES_WINDOW_DEPTH,
        }
    }
}

impl SeriesWindow {
    /// Advance δ through the linearized window: δ_{n+1} ≈ 2 z*_n δ_n + Δc.
    pub fn advance_linear(
        &self,
        reference: &ReferenceOrbit,
        dc_re: f64,
        dc_im: f64,
        start_n: u32,
    ) -> (f64, f64, u32) {
        let mut d_re = 0.0;
        let mut d_im = 0.0;
        let end = start_n.saturating_add(self.depth).min(reference.len() as u32);

        for n in start_n..end {
            let (z_re, z_im) = reference.z_at(n as usize);
            let combined_re = z_re + d_re;
            let combined_im = z_im + d_im;
            if combined_re * combined_re + combined_im * combined_im > 4.0 {
                return (d_re, d_im, n);
            }

            d_re = 2.0 * (z_re * d_re - z_im * d_im) + dc_re;
            d_im = 2.0 * (z_re * d_im + z_im * d_re) + dc_im;
        }

        (d_re, d_im, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mandelbrot::escape_time;
    use crate::perturbation::perturbation_escape_time;

    #[test]
    fn series_window_depth_is_positive() {
        assert!(series_window_depth() > 0);
    }

    #[test]
    fn linear_window_matches_full_perturbation_for_tiny_offset() {
        let c_re = -0.75;
        let c_im = 0.1;
        let max_iter = 256;
        let reference = ReferenceOrbit::build(c_re, c_im, max_iter);
        let dc_re = 1e-14;
        let dc_im = -2e-14;

        let window = SeriesWindow::default();
        let (d_re, d_im, n) = window.advance_linear(&reference, dc_re, dc_im, 0);
        assert_eq!(n, SERIES_WINDOW_DEPTH);

        let full = perturbation_escape_time(c_re + dc_re, c_im + dc_im, &reference, max_iter);
        let direct = escape_time(c_re + dc_re, c_im + dc_im, max_iter);
        assert!((direct - full).abs() < 1e-5);
        assert!(d_re.abs() < 1e-6 && d_im.abs() < 1e-6);
    }
}
