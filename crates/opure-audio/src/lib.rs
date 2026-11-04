#![no_std]
#![deny(warnings)]

//! # OPURE-AUDIO: Zero-Heap Scottish Formant Speech Synthesizer
//!
//! Real-time vocal tract formant synthesis in pure Q16.16 fixed-point math:
//! - Klatt 3-formant resonator with glottal pulse excitation.
//! - Standard 22,050 Hz 16-bit mono PCM rendering with zero dynamic heap allocations.
//! - In-memory RIFF WAV header serialization.
//! - Phonetic targets for OPURE OS's Scottish diagnostic telemetry voice.

pub mod formant;
pub mod phonemes;
pub mod synthesizer;

pub use formant::*;
pub use phonemes::*;
pub use synthesizer::*;

/// Predefined Scottish phrase sequences
pub const PHRASE_BRAW_LADDIE: &[Phoneme] = &[
    Phoneme::AA, Phoneme::IY, Phoneme::S, Phoneme::Silence,
    Phoneme::R, Phoneme::AA, Phoneme::Silence,
    Phoneme::AE, Phoneme::IY, Phoneme::Silence,
];

pub const PHRASE_HOOTS_MON: &[Phoneme] = &[
    Phoneme::OW, Phoneme::UW, Phoneme::T, Phoneme::S, Phoneme::Silence,
    Phoneme::AA, Phoneme::Silence,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_pcm_generation() {
        let mut synth = ScottishSpeechSynthesizer::new();
        let mut pcm_buffer = [0i16; 1024];

        let samples = synth.synthesize_phoneme_sequence(PHRASE_BRAW_LADDIE, 64, &mut pcm_buffer);
        assert_eq!(samples, 640);

        // Verify audio is non-silent
        let mut has_non_zero = false;
        for &s in &pcm_buffer[..samples] {
            if s != 0 {
                has_non_zero = true;
                break;
            }
        }
        assert!(has_non_zero, "Generated audio should contain non-zero PCM samples");
    }

    #[test]
    fn test_wav_header_serialization() {
        let mut header = [0u8; 44];
        ScottishSpeechSynthesizer::write_wav_header(22050, &mut header);

        assert_eq!(&header[0..4], b"RIFF");
        assert_eq!(&header[8..12], b"WAVE");
        assert_eq!(&header[12..16], b"fmt ");
        assert_eq!(&header[36..40], b"data");

        // Data length check (22050 * 2 = 44100 bytes)
        let data_size = u32::from_le_bytes(header[40..44].try_into().unwrap());
        assert_eq!(data_size, 44100);
    }
}
