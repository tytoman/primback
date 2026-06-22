use glam::Vec3;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::num::NonZero;
use std::time::Duration;

/// Supported synthesizer waveforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    Noise,
}

/// Musical pitch definitions mapping to frequencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pitch {
    C(i32),
    Cs(i32),
    Db(i32),
    D(i32),
    Ds(i32),
    Eb(i32),
    E(i32),
    F(i32),
    Fs(i32),
    Gb(i32),
    G(i32),
    Gs(i32),
    Ab(i32),
    A(i32),
    As(i32),
    Bb(i32),
    B(i32),
}

impl Pitch {
    /// Calculates the frequency of the pitch in Hz.
    pub fn freq(self) -> f32 {
        let (offset, octave) = match self {
            Pitch::C(oct) => (0, oct),
            Pitch::Cs(oct) | Pitch::Db(oct) => (1, oct),
            Pitch::D(oct) => (2, oct),
            Pitch::Ds(oct) | Pitch::Eb(oct) => (3, oct),
            Pitch::E(oct) => (4, oct),
            Pitch::F(oct) => (5, oct),
            Pitch::Fs(oct) | Pitch::Gb(oct) => (6, oct),
            Pitch::G(oct) => (7, oct),
            Pitch::Gs(oct) | Pitch::Ab(oct) => (8, oct),
            Pitch::A(oct) => (9, oct),
            Pitch::As(oct) | Pitch::Bb(oct) => (10, oct),
            Pitch::B(oct) => (11, oct),
        };
        let midi_note = 12 * (octave + 1) + offset;
        440.0 * 2.0f32.powf((midi_note - 69) as f32 / 12.0)
    }

    /// Transposes the pitch by the specified octave offset.
    pub fn octave(self, offset: i32) -> Self {
        match self {
            Pitch::C(oct) => Pitch::C(oct + offset),
            Pitch::Cs(oct) => Pitch::Cs(oct + offset),
            Pitch::Db(oct) => Pitch::Db(oct + offset),
            Pitch::D(oct) => Pitch::D(oct + offset),
            Pitch::Ds(oct) => Pitch::Ds(oct + offset),
            Pitch::Eb(oct) => Pitch::Eb(oct + offset),
            Pitch::E(oct) => Pitch::E(oct + offset),
            Pitch::F(oct) => Pitch::F(oct + offset),
            Pitch::Fs(oct) => Pitch::Fs(oct + offset),
            Pitch::Gb(oct) => Pitch::Gb(oct + offset),
            Pitch::G(oct) => Pitch::G(oct + offset),
            Pitch::Gs(oct) => Pitch::Gs(oct + offset),
            Pitch::Ab(oct) => Pitch::Ab(oct + offset),
            Pitch::A(oct) => Pitch::A(oct + offset),
            Pitch::As(oct) => Pitch::As(oct + offset),
            Pitch::Bb(oct) => Pitch::Bb(oct + offset),
            Pitch::B(oct) => Pitch::B(oct + offset),
        }
    }
}

/// Trait for converting values into frequency values.
pub trait IntoFreq {
    fn into_freq(self) -> f32;
}

impl IntoFreq for f32 {
    fn into_freq(self) -> f32 {
        self
    }
}

impl IntoFreq for Pitch {
    fn into_freq(self) -> f32 {
        self.freq()
    }
}

/// Represents a listener in 3D space for spatial audio calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioListener {
    /// The position of the listener in world coordinates.
    pub position: Vec3,
    /// The right vector of the listener (defining orientation).
    pub right: Vec3,
}

/// Parameters for synthesizer sound (waveform synthesis) playback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Synth {
    pub waveform: Waveform,
    pub frequency: f32,
    pub duration_secs: f32,
    pub volume: f32,
    pub position: Option<Vec3>,
    pub delay_secs: f32,
}

impl Synth {
    /// Creates a new `Synth` parameters with the given waveform and frequency.
    pub fn new(waveform: Waveform, frequency: impl IntoFreq) -> Self {
        Self {
            waveform,
            frequency: frequency.into_freq(),
            duration_secs: 0.1,
            volume: 1.0,
            position: None,
            delay_secs: 0.0,
        }
    }

    pub fn sine(frequency: impl IntoFreq) -> Self {
        Self::new(Waveform::Sine, frequency)
    }

    pub fn square(frequency: impl IntoFreq) -> Self {
        Self::new(Waveform::Square, frequency)
    }

    pub fn triangle(frequency: impl IntoFreq) -> Self {
        Self::new(Waveform::Triangle, frequency)
    }

    pub fn sawtooth(frequency: impl IntoFreq) -> Self {
        Self::new(Waveform::Sawtooth, frequency)
    }

    pub fn noise() -> Self {
        Self::new(Waveform::Noise, 0.0)
    }

    /// Coin collection sound (bright high-pitched pulse)
    pub fn coin() -> Self {
        Self::square(Pitch::B(5)).duration(0.08).volume(0.4)
    }

    /// Jump sound (slightly longer soft low-mid tone)
    pub fn jump() -> Self {
        Self::triangle(Pitch::E(3)).duration(0.15).volume(0.5)
    }

    /// Damage sound (short noise)
    pub fn damage() -> Self {
        Self::noise().duration(0.1).volume(0.6)
    }

    /// Explosion sound (longer noise)
    pub fn explosion() -> Self {
        Self::noise().duration(0.35).volume(0.8)
    }

    /// UI click / decision sound
    pub fn click() -> Self {
        Self::sine(Pitch::E(4)).duration(0.05).volume(0.4)
    }

    pub fn duration(mut self, secs: f32) -> Self {
        if secs < 0.0 {
            log::warn!("Synth duration must be non-negative. Clamping to 0.0. Got: {}", secs);
            self.duration_secs = 0.0;
        } else {
            self.duration_secs = secs;
        }
        self
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }

    pub fn delay(mut self, secs: f32) -> Self {
        if secs < 0.0 {
            log::warn!("Synth delay must be non-negative. Clamping to 0.0. Got: {}", secs);
            self.delay_secs = 0.0;
        } else {
            self.delay_secs = secs;
        }
        self
    }
}

/// Calculates left and right volumes (panning and distance attenuation) from relative 3D coordinates of camera and sound source.
fn calculate_3d_volumes(emitter_pos: Vec3, listener_pos: Vec3, listener_right: Vec3) -> (f32, f32) {
    let to_emitter = emitter_pos - listener_pos;
    let distance = to_emitter.length();

    // 1. Distance attenuation (reference distance = 1.0, attenuation rolloff = 0.5)
    let ref_distance = 1.0;
    let rolloff = 0.5;
    let attenuation = ref_distance / (ref_distance + rolloff * distance);

    // 2. Left/Right panning calculation
    let to_emitter_dir = to_emitter.normalize_or_zero();
    let dot = to_emitter_dir.dot(listener_right).clamp(-1.0, 1.0); // -1.0 (left) to 1.0 (right)

    // Constant Power Panning
    let angle = (dot + 1.0) * (std::f32::consts::PI / 4.0); // 0 to PI/2
    let left_gain = angle.cos();
    let right_gain = angle.sin();

    (left_gain * attenuation, right_gain * attenuation)
}

/// Waveform synthesis source supporting stereo output.
pub struct SynthSource {
    waveform: Waveform,
    frequency: f32,
    sample_rate: u32,
    num_samples: usize,
    current_sample: usize,
    left_volume: f32,
    right_volume: f32,
    last_value: f32,
}

impl SynthSource {
    pub fn new(
        waveform: Waveform,
        frequency: f32,
        duration: Duration,
        left_volume: f32,
        right_volume: f32,
    ) -> Self {
        let sample_rate = 44100;
        // Stereo output doubles the total number of samples
        let num_samples = (sample_rate as f32 * duration.as_secs_f32()) as usize * 2;
        Self {
            waveform,
            frequency,
            sample_rate,
            num_samples,
            current_sample: 0,
            left_volume,
            right_volume,
            last_value: 0.0,
        }
    }
}

impl Iterator for SynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.num_samples {
            return None;
        }

        let is_left = self.current_sample % 2 == 0;

        let value = if is_left {
            let mono_sample_idx = self.current_sample / 2;
            let t = mono_sample_idx as f32 / self.sample_rate as f32;
            let raw_val = match self.waveform {
                Waveform::Sine => (t * self.frequency * 2.0 * std::f32::consts::PI).sin(),
                Waveform::Square => {
                    if (t * self.frequency).fract() < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Waveform::Triangle => {
                    let x = (t * self.frequency).fract();
                    if x < 0.25 {
                        x * 4.0
                    } else if x < 0.75 {
                        2.0 - x * 4.0
                    } else {
                        x * 4.0 - 4.0
                    }
                }
                Waveform::Sawtooth => (t * self.frequency).fract() * 2.0 - 1.0,
                Waveform::Noise => {
                    let mut x = mono_sample_idx as u32 + 1;
                    x ^= x << 13;
                    x ^= x >> 17;
                    x ^= x << 5;
                    (x as f32 / u32::MAX as f32) * 2.0 - 1.0
                }
            };
            self.last_value = raw_val;
            raw_val * self.left_volume
        } else {
            self.last_value * self.right_volume
        };

        self.current_sample += 1;
        Some(value)
    }
}

impl Source for SynthSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.num_samples - self.current_sample)
    }

    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap() // Stereo
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(self.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.num_samples as f32 / (self.sample_rate as f32 * 2.0),
        ))
    }
}

struct DelayedSynth {
    delay: f32,
    synth: Synth,
    listener: Option<AudioListener>,
}

/// The audio subsystem that handles sound effect playback, volume controls, and spatial audio.
pub struct AudioSystem {
    _stream: Option<MixerDeviceSink>,
    // Volume settings
    master_volume: f32,
    se_volume: f32,
    // Delayed synth playback queue
    delayed_synths: Vec<DelayedSynth>,
}

impl AudioSystem {
    /// Creates a new `AudioSystem` and opens the default output device.
    pub fn new() -> Self {
        match DeviceSinkBuilder::open_default_sink() {
            Ok(mut stream) => {
                stream.log_on_drop(false);
                Self {
                    _stream: Some(stream),
                    master_volume: 1.0,
                    se_volume: 1.0,
                    delayed_synths: Vec::new(),
                }
            }
            Err(e) => {
                log::warn!("Failed to initialize audio output stream: {}", e);
                Self {
                    _stream: None,
                    master_volume: 1.0,
                    se_volume: 1.0,
                    delayed_synths: Vec::new(),
                }
            }
        }
    }

    /// Plays a synthesizer sound.
    pub fn play_synth(&mut self, mut synth: Synth, listener: Option<AudioListener>) {
        if synth.duration_secs < 0.0 {
            log::warn!("synth.duration_secs is negative ({}). Clamping to 0.0.", synth.duration_secs);
            synth.duration_secs = 0.0;
        }
        if synth.delay_secs < 0.0 {
            log::warn!("synth.delay_secs is negative ({}). Clamping to 0.0.", synth.delay_secs);
            synth.delay_secs = 0.0;
        }

        if synth.delay_secs > 0.0 {
            let delay = synth.delay_secs;
            synth.delay_secs = 0.0;
            self.delayed_synths.push(DelayedSynth {
                delay,
                synth,
                listener,
            });
            return;
        }

        let stream = match &self._stream {
            Some(s) => s,
            None => return,
        };

        let duration = Duration::from_secs_f64(synth.duration_secs as f64);
        let base_vol = synth.volume.clamp(0.0, 1.0);

        let source = if let (Some(pos), Some(l)) = (synth.position, listener) {
            let (left_vol, right_vol) = calculate_3d_volumes(pos, l.position, l.right);
            SynthSource::new(
                synth.waveform,
                synth.frequency,
                duration,
                left_vol * base_vol,
                right_vol * base_vol,
            )
        } else {
            SynthSource::new(
                synth.waveform,
                synth.frequency,
                duration,
                base_vol,
                base_vol,
            )
        };

        let player = Player::connect_new(stream.mixer());
        player.set_volume(self.master_volume * self.se_volume);
        player.append(source);
        player.detach();
    }

    // Volume management APIs
    /// Sets the master volume scale.
    pub fn set_master_volume(&mut self, volume: f32) {
        if volume < 0.0 || volume > 1.0 {
            log::warn!("Master volume should be in range [0.0, 1.0]. Clamping to [0.0, 1.0]. Got: {}", volume);
        }
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Sets the sound effects volume scale.
    pub fn set_se_volume(&mut self, volume: f32) {
        if volume < 0.0 || volume > 1.0 {
            log::warn!("SE volume should be in range [0.0, 1.0]. Clamping to [0.0, 1.0]. Got: {}", volume);
        }
        self.se_volume = volume.clamp(0.0, 1.0);
    }

    /// Gets the master volume scale.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Gets the sound effects volume scale.
    pub fn se_volume(&self) -> f32 {
        self.se_volume
    }

    /// Updates the audio system state, playing any pending delayed sounds.
    pub fn update(&mut self, dt: f32) {
        // 1. Update and play synths
        let mut ready_synths = Vec::new();
        self.delayed_synths.retain_mut(|ds| {
            ds.delay -= dt;
            if ds.delay <= 0.0 {
                ready_synths.push((ds.synth, ds.listener));
                false
            } else {
                true
            }
        });
        for (synth, listener) in ready_synths {
            self.play_synth(synth, listener);
        }
    }
}

