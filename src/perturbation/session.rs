use super::reference::{F64OrbitBackend, OrbitBackend, ReferenceOrbit};

/// Cached reference-orbit state reused across frames during deep zoom.
#[derive(Debug, Clone)]
pub struct PerturbationSession {
    cached: Option<ReferenceOrbit>,
    rebase_count: u32,
    /// Rebuild the reference orbit when the center moves more than this in either axis.
    rebase_epsilon: f64,
}

impl Default for PerturbationSession {
    fn default() -> Self {
        Self::new()
    }
}

impl PerturbationSession {
    pub fn new() -> Self {
        Self {
            cached: None,
            rebase_count: 0,
            rebase_epsilon: 1e-15,
        }
    }

    pub fn rebase_count(&self) -> u32 {
        self.rebase_count
    }

    pub fn reference_orbit_len(&self) -> usize {
        self.cached.as_ref().map_or(0, ReferenceOrbit::len)
    }

    /// Return a reference orbit for `(c_re, c_im)`, rebuilding when the center drifts.
    pub fn orbit_for(&mut self, c_re: f64, c_im: f64, max_iter: u32) -> &ReferenceOrbit {
        let needs_rebase = match &self.cached {
            None => true,
            Some(orbit) => {
                (orbit.c_re - c_re).abs() > self.rebase_epsilon
                    || (orbit.c_im - c_im).abs() > self.rebase_epsilon
                    || orbit.len() <= 1
            }
        };

        if needs_rebase {
            self.cached = Some(F64OrbitBackend::build_reference(c_re, c_im, max_iter));
            self.rebase_count = self.rebase_count.saturating_add(1);
        }

        self.cached.as_ref().expect("orbit cache populated above")
    }

    pub fn invalidate(&mut self) {
        self.cached = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_has_no_cached_orbit() {
        let session = PerturbationSession::new();
        assert_eq!(session.reference_orbit_len(), 0);
        assert_eq!(session.rebase_count(), 0);
    }

    #[test]
    fn invalidate_clears_cache() {
        let mut session = PerturbationSession::new();
        session.orbit_for(-0.5, 0.0, 64);
        assert!(session.reference_orbit_len() > 0);
        session.invalidate();
        assert_eq!(session.reference_orbit_len(), 0);
    }
}
