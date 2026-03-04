/// Named color themes for mapping escape iterations to RGBA pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    Classic,
    Fire,
    Ocean,
    Grayscale,
}

impl Palette {
    pub const ALL: [Palette; 4] = [
        Palette::Classic,
        Palette::Fire,
        Palette::Ocean,
        Palette::Grayscale,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Palette::Classic => "Classic",
            Palette::Fire => "Fire",
            Palette::Ocean => "Ocean",
            Palette::Grayscale => "Grayscale",
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }

    /// Map a normalized value in `[0, 1]` to an RGBA color.
    pub fn sample(self, t: f64) -> [u8; 4] {
        let t = t.clamp(0.0, 1.0);
        let [r, g, b] = match self {
            Palette::Classic => classic(t),
            Palette::Fire => fire(t),
            Palette::Ocean => ocean(t),
            Palette::Grayscale => grayscale(t),
        };
        [r, g, b, 255]
    }
}

fn classic(t: f64) -> [u8; 3] {
    let r = (0.5 + 0.5 * (2.0 * std::f64::consts::PI * (t * 3.0 + 0.0)).sin()) * 255.0;
    let g = (0.5 + 0.5 * (2.0 * std::f64::consts::PI * (t * 3.0 + 0.33)).sin()) * 255.0;
    let b = (0.5 + 0.5 * (2.0 * std::f64::consts::PI * (t * 3.0 + 0.67)).sin()) * 255.0;
    [r as u8, g as u8, b as u8]
}

fn fire(t: f64) -> [u8; 3] {
    let r = (255.0 * t.powf(0.4)).min(255.0);
    let g = (255.0 * (t * 1.2 - 0.1).max(0.0).powf(0.8)).min(255.0);
    let b = (255.0 * (t * 2.0 - 0.8).max(0.0).powf(1.5)).min(255.0);
    [r as u8, g as u8, b as u8]
}

fn ocean(t: f64) -> [u8; 3] {
    let r = (40.0 + 60.0 * (std::f64::consts::PI * t).sin().abs()) as u8;
    let g = (80.0 + 120.0 * t) as u8;
    let b = (120.0 + 135.0 * t.sqrt()) as u8;
    [r, g, b]
}

fn grayscale(t: f64) -> [u8; 3] {
    let v = (t * 255.0) as u8;
    [v, v, v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_names_are_unique() {
        let names: Vec<_> = Palette::ALL.iter().map(|p| p.name()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len());
    }

    #[test]
    fn samples_are_opaque() {
        for palette in Palette::ALL {
            assert_eq!(palette.sample(0.5)[3], 255);
        }
    }

    #[test]
    fn from_index_round_trips() {
        for (index, palette) in Palette::ALL.iter().enumerate() {
            assert_eq!(Palette::from_index(index), *palette);
        }
        assert_eq!(Palette::from_index(99), Palette::ALL[99 % Palette::ALL.len()]);
    }
}
