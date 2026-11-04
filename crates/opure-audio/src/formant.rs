//! Fixed-Point Formant Resonator & Glottal Excitation Engine
//!
//! Implements a 2-pole digital resonator filter in pure Q16.16 fixed-point math,
//! simulating human vocal tract resonances (F1, F2, F3 formants) with 0 heap bytes.

pub const SAMPLE_RATE: u32 = 22050;

/// 2-Pole Digital Resonator in Q16.16 Fixed Point
#[derive(Debug, Clone, Copy)]
pub struct FormantFilter {
    pub y1: i32,
    pub y2: i32,
    pub a: i32,  // Q16.16 input gain
    pub b1: i32, // Q16.16 feedback coeff 1
    pub b2: i32, // Q16.16 feedback coeff 2
}

impl FormantFilter {
    pub const fn new() -> Self {
        Self {
            y1: 0,
            y2: 0,
            a: 65536,
            b1: 0,
            b2: 0,
        }
    }

    /// Sets formant frequency and bandwidth in Hertz using fixed-point approximation.
    pub fn set_parameters(&mut self, freq_hz: u32, bandwidth_hz: u32) {
        // Approximate pole radius r = exp(-pi * B / Fs) in Q16.16
        // 1 - (pi * B / Fs) where pi/Fs ~ 3.14159 / 22050 ~ 0.00014247
        let factor = ((bandwidth_hz as i64 * 9) / 65536) as i32; // scaled
        let r = 65536 - factor.clamp(100, 32000);

        // Approximate theta = 2 * pi * F / Fs
        // theta scaled: (freq_hz * 65536 * 2 * 31416) / (22050 * 10000) ~ (freq_hz * 1867) / 100
        let theta_step = ((freq_hz as i64 * 1867) / 100) as i32;
        let cos_theta = approximate_cos_q16(theta_step);

        // b1 = 2 * r * cos(theta) in Q16.16
        self.b1 = (((2 * r as i64) * cos_theta as i64) >> 16) as i32;
        // b2 = -r^2 in Q16.16
        self.b2 = -(((r as i64 * r as i64) >> 16) as i32);
        // Gain a ~ (1 - r) in Q16.16
        self.a = (65536 - r).clamp(1000, 65536);
    }

    /// Filters a single excitation sample x (in 16-bit range).
    #[inline(always)]
    pub fn step(&mut self, x: i32) -> i32 {
        let x_scaled = ((x as i64 * self.a as i64) >> 16) as i32;
        let y0 = x_scaled
            + (((self.b1 as i64 * self.y1 as i64) >> 16) as i32)
            + (((self.b2 as i64 * self.y2 as i64) >> 16) as i32);

        self.y2 = self.y1;
        self.y1 = y0.clamp(-2147483647, 2147483647);
        y0
    }
}

/// Approximate cosine function in Q16.16 (-65536 to 65536).
#[inline(always)]
fn approximate_cos_q16(angle_q16: i32) -> i32 {
    let pi_q16 = 205887; // pi in Q16
    let two_pi_q16 = 411774;
    let mut x = angle_q16 % two_pi_q16;
    if x < 0 {
        x += two_pi_q16;
    }
    if x > pi_q16 {
        x = two_pi_q16 - x;
    }
    // Parabolic approximation: 1 - (2*x/pi)^2 * 0.5 ~ (pi^2 - 4*x^2 + 4*x*pi)/(pi^2)
    let norm = (x as i64 * 65536) / pi_q16 as i64; // 0 to 65536
    let quad = 65536 - (((norm * norm) >> 15) as i32);
    quad.clamp(-65536, 65536)
}

/// Glottal Pulse Waveform Generator
pub struct GlottalExciter {
    pub phase: u32,
    pub pitch_step: u32,
}

impl GlottalExciter {
    pub const fn new(pitch_hz: u32) -> Self {
        let step = (pitch_hz * 65536) / SAMPLE_RATE;
        Self {
            phase: 0,
            pitch_step: step,
        }
    }

    pub fn set_pitch(&mut self, pitch_hz: u32) {
        self.pitch_step = (pitch_hz * 65536) / SAMPLE_RATE;
    }

    /// Generates next excitation impulse sample.
    #[inline(always)]
    pub fn step(&mut self) -> i32 {
        self.phase = self.phase.wrapping_add(self.pitch_step);
        let cycle_phase = (self.phase % 65536) as i32;

        // Rosenberg glottal pulse approximation: open phase rise + rapid close
        if cycle_phase < 26214 {
            // Rise phase
            (cycle_phase * 16384) / 26214
        } else if cycle_phase < 39321 {
            // Fall phase
            16384 - (((cycle_phase - 26214) * 16384) / 13107)
        } else {
            // Closed phase
            0
        }
    }
}
