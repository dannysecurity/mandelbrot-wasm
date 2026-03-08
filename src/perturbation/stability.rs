/// Outcome of a delta-stability check during perturbation iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilityOutcome {
    /// Delta remains small enough relative to the reference orbit.
    Stable,
    /// |δ| grew too large; perturbation series is no longer trustworthy.
    DeltaBailout,
    /// Reference orbit magnitude is extreme; a high-precision rebase is advised.
    GlitchSuspected,
}

/// Heuristic limits for perturbation delta iteration.
#[derive(Debug, Clone, Copy)]
pub struct DeltaStability {
    /// Bail out when |δ| exceeds this multiple of |z*|.
    pub max_delta_ratio: f64,
    /// Flag a glitch when |z*| exceeds this magnitude.
    pub glitch_orbit_mag: f64,
}

impl Default for DeltaStability {
    fn default() -> Self {
        Self {
            max_delta_ratio: 1e4,
            glitch_orbit_mag: 1e12,
        }
    }
}

impl DeltaStability {
    pub fn check(&self, z_re: f64, z_im: f64, d_re: f64, d_im: f64) -> StabilityOutcome {
        let z_mag = (z_re * z_re + z_im * z_im).sqrt();
        if z_mag > self.glitch_orbit_mag {
            return StabilityOutcome::GlitchSuspected;
        }

        let d_mag = (d_re * d_re + d_im * d_im).sqrt();
        if z_mag > 0.0 && d_mag > z_mag * self.max_delta_ratio {
            return StabilityOutcome::DeltaBailout;
        }

        StabilityOutcome::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_when_delta_is_tiny() {
        let policy = DeltaStability::default();
        assert_eq!(policy.check(1.0, 0.0, 1e-12, 0.0), StabilityOutcome::Stable);
    }

    #[test]
    fn bailout_when_delta_dominates() {
        let policy = DeltaStability::default();
        assert_eq!(
            policy.check(1.0, 0.0, 2e4, 0.0),
            StabilityOutcome::DeltaBailout
        );
    }

    #[test]
    fn glitch_when_orbit_is_huge() {
        let policy = DeltaStability::default();
        assert_eq!(
            policy.check(1e13, 0.0, 1e-9, 0.0),
            StabilityOutcome::GlitchSuspected
        );
    }
}
