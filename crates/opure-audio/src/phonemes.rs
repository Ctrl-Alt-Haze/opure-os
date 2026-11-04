//! Phonetic Acoustic Targets for Scottish Formant Synthesis

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phoneme {
    Silence,
    AA, // "braw" (F1: 730, F2: 1090, F3: 2440)
    AE, // "malt" (F1: 660, F2: 1720, F3: 2410)
    IY, // "laddie" (F1: 270, F2: 2290, F3: 3010)
    EH, // "crivens" (F1: 530, F2: 1840, F3: 2480)
    UW, // "smooth" (F1: 300, F2: 870, F3: 2240)
    OW, // "hoots" (F1: 570, F2: 840, F3: 2410)
    ER, // "purrin" (F1: 490, F2: 1350, F3: 1690)
    R,  // trilled Scottish 'r'
    S,  // sibilant fricative
    T,  // stop consonant
}

pub struct FormantTarget {
    pub f1: u32,
    pub f2: u32,
    pub f3: u32,
    pub b1: u32,
    pub b2: u32,
    pub b3: u32,
    pub is_voiced: bool,
}

impl Phoneme {
    pub fn get_targets(&self) -> FormantTarget {
        match self {
            Self::Silence => FormantTarget { f1: 250, f2: 500, f3: 1500, b1: 100, b2: 150, b3: 200, is_voiced: false },
            Self::AA => FormantTarget { f1: 730, f2: 1090, f3: 2440, b1: 90, b2: 110, b3: 170, is_voiced: true },
            Self::AE => FormantTarget { f1: 660, f2: 1720, f3: 2410, b1: 80, b2: 100, b3: 150, is_voiced: true },
            Self::IY => FormantTarget { f1: 270, f2: 2290, f3: 3010, b1: 60, b2: 90, b3: 150, is_voiced: true },
            Self::EH => FormantTarget { f1: 530, f2: 1840, f3: 2480, b1: 70, b2: 100, b3: 160, is_voiced: true },
            Self::UW => FormantTarget { f1: 300, f2: 870, f3: 2240, b1: 70, b2: 100, b3: 160, is_voiced: true },
            Self::OW => FormantTarget { f1: 570, f2: 840, f3: 2410, b1: 80, b2: 90, b3: 150, is_voiced: true },
            Self::ER => FormantTarget { f1: 490, f2: 1350, f3: 1690, b1: 80, b2: 100, b3: 140, is_voiced: true },
            Self::R  => FormantTarget { f1: 350, f2: 1200, f3: 1800, b1: 100, b2: 120, b3: 160, is_voiced: true },
            Self::S  => FormantTarget { f1: 200, f2: 3000, f3: 4500, b1: 300, b2: 400, b3: 600, is_voiced: false },
            Self::T  => FormantTarget { f1: 300, f2: 1800, f3: 3200, b1: 200, b2: 300, b3: 400, is_voiced: false },
        }
    }
}
