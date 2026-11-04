//! Real-Time Scottish Formant Speech Synthesizer

use crate::formant::{FormantFilter, GlottalExciter, SAMPLE_RATE};
use crate::phonemes::Phoneme;

/// Zero-Allocation Speech Synthesizer for OPURE OS
pub struct ScottishSpeechSynthesizer {
    pub filter1: FormantFilter,
    pub filter2: FormantFilter,
    pub filter3: FormantFilter,
    pub exciter: GlottalExciter,
    pub noise_lfsr: u32,
}

impl ScottishSpeechSynthesizer {
    pub const fn new() -> Self {
        Self {
            filter1: FormantFilter::new(),
            filter2: FormantFilter::new(),
            filter3: FormantFilter::new(),
            exciter: GlottalExciter::new(125), // 125 Hz base Scottish male pitch
            noise_lfsr: 0xACE1ACE1,
        }
    }

    /// Generates next noise sample for unvoiced fricatives.
    #[inline(always)]
    fn next_noise(&mut self) -> i32 {
        let bit = ((self.noise_lfsr >> 0) ^ (self.noise_lfsr >> 2) ^ (self.noise_lfsr >> 3) ^ (self.noise_lfsr >> 5)) & 1;
        self.noise_lfsr = (self.noise_lfsr >> 1) | (bit << 31);
        ((self.noise_lfsr & 0xFFFF) as i32) - 32768
    }

    /// Sets acoustic formant parameters for an active phoneme.
    pub fn set_phoneme(&mut self, phoneme: Phoneme) {
        let t = phoneme.get_targets();
        self.filter1.set_parameters(t.f1, t.b1);
        self.filter2.set_parameters(t.f2, t.b2);
        self.filter3.set_parameters(t.f3, t.b3);
    }

    /// Synthesizes a sequence of phonemes into raw 16-bit PCM buffer.
    pub fn synthesize_phoneme_sequence(
        &mut self,
        sequence: &[Phoneme],
        samples_per_phoneme: usize,
        out_pcm: &mut [i16],
    ) -> usize {
        let total_samples = (sequence.len() * samples_per_phoneme).min(out_pcm.len());
        let mut sample_idx = 0;

        for &phoneme in sequence {
            self.set_phoneme(phoneme);
            let target = phoneme.get_targets();

            let mut count = 0;
            while count < samples_per_phoneme && sample_idx < total_samples {
                let excitation = if target.is_voiced {
                    self.exciter.step()
                } else {
                    self.next_noise() / 4
                };

                let y1 = self.filter1.step(excitation);
                let y2 = self.filter2.step(excitation);
                let y3 = self.filter3.step(excitation);

                let mixed = (y1 + (y2 / 2) + (y3 / 4)).clamp(-32768, 32767) as i16;
                out_pcm[sample_idx] = mixed;

                sample_idx += 1;
                count += 1;
            }
        }

        total_samples
    }

    /// Formats a 44-byte standard RIFF WAV header over an unmanaged byte slice.
    pub fn write_wav_header(pcm_sample_count: usize, out_header: &mut [u8; 44]) {
        let data_size = (pcm_sample_count * 2) as u32;
        let file_size = data_size + 36;
        let sample_rate = SAMPLE_RATE;
        let byte_rate = sample_rate * 2; // 16-bit mono

        out_header[0..4].copy_from_slice(b"RIFF");
        out_header[4..8].copy_from_slice(&file_size.to_le_bytes());
        out_header[8..12].copy_from_slice(b"WAVE");
        out_header[12..16].copy_from_slice(b"fmt ");
        out_header[16..20].copy_from_slice(&16u32.to_le_bytes()); // subchunk size (16 for PCM)
        out_header[20..22].copy_from_slice(&1u16.to_le_bytes());  // PCM format = 1
        out_header[22..24].copy_from_slice(&1u16.to_le_bytes());  // Mono = 1 channel
        out_header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
        out_header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
        out_header[32..34].copy_from_slice(&2u16.to_le_bytes());  // Block align = 2
        out_header[34..36].copy_from_slice(&16u16.to_le_bytes()); // Bits per sample = 16
        out_header[36..40].copy_from_slice(b"data");
        out_header[40..44].copy_from_slice(&data_size.to_le_bytes());
    }
}
