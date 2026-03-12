use crate::fractal_core::{has_escaped, mandelbrot_step};

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
            let mag2 = z_re * z_re + z_im * z_im;
            if has_escaped(mag2) {
                break;
            }
            (z_re, z_im) = mandelbrot_step(z_re, z_im, c_re, c_im);
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

    pub fn z_at(&self, n: usize) -> (f64, f64) {
        (self.orbit_re[n], self.orbit_im[n])
    }

    #[cfg(test)]
    pub(crate) fn synthetic(c_re: f64, c_im: f64, orbit_re: Vec<f64>, orbit_im: Vec<f64>) -> Self {
        Self {
            c_re,
            c_im,
            orbit_re,
            orbit_im,
        }
    }
}

/// Trait stub for a future high-precision reference-orbit backend (MPFR, soft-float).
pub trait OrbitBackend {
    fn build_reference(c_re: f64, c_im: f64, max_iter: u32) -> ReferenceOrbit;
}

/// Default f64 reference-orbit backend used by the explorer today.
pub struct F64OrbitBackend;

impl OrbitBackend for F64OrbitBackend {
    fn build_reference(c_re: f64, c_im: f64, max_iter: u32) -> ReferenceOrbit {
        ReferenceOrbit::build(c_re, c_im, max_iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_orbit_starts_at_origin() {
        let orbit = ReferenceOrbit::build(-0.75, 0.1, 64);
        let (z_re, z_im) = orbit.z_at(0);
        assert_eq!(z_re, 0.0);
        assert_eq!(z_im, 0.0);
        assert!(orbit.len() > 1);
    }

    #[test]
    fn reference_orbit_truncates_on_escape() {
        let orbit = ReferenceOrbit::build(2.0, 2.0, 256);
        assert!(orbit.len() < 256);
        let (z_re, z_im) = orbit.z_at(orbit.len() - 1);
        assert!(z_re * z_re + z_im * z_im > 4.0 || orbit.len() == 1);
    }

    #[test]
    fn f64_backend_matches_direct_build() {
        let direct = ReferenceOrbit::build(-0.5, 0.0, 128);
        let via_trait = F64OrbitBackend::build_reference(-0.5, 0.0, 128);
        assert_eq!(direct.len(), via_trait.len());
    }
}
