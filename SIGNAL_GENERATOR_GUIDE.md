# Rusty Audio - Signal Generator Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Signal Types Overview](#signal-types-overview)
3. [Mathematical Background](#mathematical-background)
4. [Parameter Reference](#parameter-reference)
5. [Use Cases and Applications](#use-cases-and-applications)
6. [Testing Framework Integration](#testing-framework-integration)
7. [Advanced Topics](#advanced-topics)
8. [Safety Considerations](#safety-considerations)
9. [API Reference](#api-reference)
10. [Examples and Recipes](#examples-and-recipes)

---

## Introduction

The Rusty Audio Signal Generator is a comprehensive tool for generating test signals, calibration tones, and measurement signals. It provides precise control over various waveform types with sample-accurate generation at any supported sample rate.

### Key Features

- **Multiple Waveform Types**: Sine, square, sawtooth, triangle, white noise, pink noise
- **High Precision**: Frequency accurate to ±0.01%, amplitude to ±0.1 dB
- **Multi-tone Generation**: Up to 8 simultaneous tones with phase control
- **Sweep Generation**: Linear and logarithmic frequency sweeps
- **Modulation**: AM and FM synthesis capabilities
- **Real-time Generation**: Low-latency, lock-free audio generation

### Common Applications

- **Audio System Testing**: Frequency response, distortion measurement
- **Speaker Calibration**: Room correction, crossover alignment
- **Hearing Tests**: Audiometry, tinnitus matching
- **Education**: Demonstrating audio principles, acoustics
- **Music Production**: Reference tones, tuning, synthesis

---

## Signal Types Overview

### Basic Waveforms

| Waveform | Harmonic Content | Typical Use | Sound Character |
|----------|-----------------|-------------|-----------------|
| **Sine** | Fundamental only | Calibration, testing | Pure, smooth |
| **Square** | Odd harmonics | Digital synthesis | Harsh, hollow |
| **Sawtooth** | All harmonics | Analog synthesis | Bright, buzzy |
| **Triangle** | Odd harmonics (weak) | LFO, soft bass | Mellow, flute-like |

### Noise Types

| Noise Type | Spectrum | Power/Octave | Typical Use |
|------------|----------|--------------|-------------|
| **White** | Flat | 0 dB | Testing, masking |
| **Pink** | 1/f | -3 dB | Acoustics, mixing |
| **Brown** | 1/f² | -6 dB | Low frequency testing |
| **Blue** | f | +3 dB | Dithering |

---

## Mathematical Background

### Sine Wave Generation

The sine wave is the fundamental building block of all periodic signals:

```
y(t) = A × sin(2π × f × t + φ)
```

Where:
- `A` = Amplitude (0 to 1)
- `f` = Frequency (Hz)
- `t` = Time (seconds)
- `φ` = Phase offset (radians)

**Digital Implementation:**
```rust
fn generate_sine(sample_rate: f32, frequency: f32, amplitude: f32) -> impl Iterator<Item = f32> {
    let phase_increment = 2.0 * PI * frequency / sample_rate;
    let mut phase = 0.0;

    std::iter::from_fn(move || {
        let sample = amplitude * phase.sin();
        phase += phase_increment;
        if phase >= 2.0 * PI {
            phase -= 2.0 * PI; // Wrap phase to prevent accumulation errors
        }
        Some(sample)
    })
}
```

### Square Wave Generation

Square waves contain only odd harmonics with amplitudes inversely proportional to harmonic number:

```
y(t) = (4A/π) × Σ[sin(2π(2n-1)ft)/(2n-1)] for n=1 to ∞
```

**Efficient Band-Limited Implementation:**
```rust
fn generate_square(sample_rate: f32, frequency: f32, amplitude: f32, duty_cycle: f32) -> impl Iterator<Item = f32> {
    let period = sample_rate / frequency;
    let high_samples = (period * duty_cycle) as usize;
    let mut sample_count = 0;

    std::iter::from_fn(move || {
        let sample = if sample_count < high_samples {
            amplitude
        } else {
            -amplitude
        };

        sample_count += 1;
        if sample_count >= period as usize {
            sample_count = 0;
        }

        Some(sample)
    })
}
```

### Sawtooth Wave Generation

Sawtooth waves contain all harmonics with amplitudes inversely proportional to harmonic number:

```
y(t) = (2A/π) × Σ[(-1)^(n+1) × sin(2πnft)/n] for n=1 to ∞
```

**Anti-Aliased BLIT Implementation:**
```rust
fn generate_sawtooth_blit(sample_rate: f32, frequency: f32, amplitude: f32) -> impl Iterator<Item = f32> {
    let phase_increment = frequency / sample_rate;
    let mut phase = 0.0;

    std::iter::from_fn(move || {
        // Band-limited impulse train (BLIT) method
        let sample = amplitude * (2.0 * phase - 1.0);
        phase += phase_increment;
        if phase >= 1.0 {
            phase -= 1.0;
        }

        // Apply anti-aliasing filter
        Some(apply_polyblep(sample, phase, phase_increment))
    })
}
```

### Triangle Wave Generation

Triangle waves have a gentler harmonic rolloff than square waves:

```
y(t) = (8A/π²) × Σ[(-1)^((n-1)/2) × sin(2π(2n-1)ft)/(2n-1)²] for odd n
```

**Direct Synthesis Implementation:**
```rust
fn generate_triangle(sample_rate: f32, frequency: f32, amplitude: f32) -> impl Iterator<Item = f32> {
    let period = sample_rate / frequency;
    let mut phase = 0.0;
    let phase_increment = 4.0 * frequency / sample_rate;

    std::iter::from_fn(move || {
        let sample = if phase < 2.0 {
            amplitude * (phase - 1.0)
        } else {
            amplitude * (3.0 - phase)
        };

        phase += phase_increment;
        if phase >= 4.0 {
            phase -= 4.0;
        }

        Some(sample)
    })
}
```

### White Noise Generation

White noise has equal power at all frequencies. Generated using high-quality PRNG:

```rust
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

fn generate_white_noise(amplitude: f32) -> impl Iterator<Item = f32> {
    let mut rng = ChaCha20Rng::from_entropy();

    std::iter::from_fn(move || {
        let sample = rng.gen_range(-1.0..=1.0) * amplitude;
        Some(sample)
    })
}
```

### Pink Noise Generation

Pink noise has equal power per octave (-3dB/octave rolloff):

```rust
fn generate_pink_noise(amplitude: f32) -> impl Iterator<Item = f32> {
    // Paul Kellet's refined pink noise filter
    let mut b0 = 0.0;
    let mut b1 = 0.0;
    let mut b2 = 0.0;
    let mut b3 = 0.0;
    let mut b4 = 0.0;
    let mut b5 = 0.0;
    let mut b6 = 0.0;

    let mut rng = ChaCha20Rng::from_entropy();

    std::iter::from_fn(move || {
        let white = rng.gen_range(-1.0..=1.0);

        b0 = 0.99886 * b0 + white * 0.0555179;
        b1 = 0.99332 * b1 + white * 0.0750759;
        b2 = 0.96900 * b2 + white * 0.1538520;
        b3 = 0.86650 * b3 + white * 0.3104856;
        b4 = 0.55000 * b4 + white * 0.5329522;
        b5 = -0.7616 * b5 - white * 0.0168980;

        let pink = (b0 + b1 + b2 + b3 + b4 + b5 + b6 + white * 0.5362) * 0.11;
        b6 = white * 0.115926;

        Some(pink * amplitude)
    })
}
```

---

## Parameter Reference

### Frequency Parameters

| Parameter | Range | Default | Resolution | Notes |
|-----------|-------|---------|------------|-------|
| **Frequency** | 0.1 Hz - 22.05 kHz | 440 Hz | 0.01 Hz | Limited by Nyquist |
| **Fine Tune** | ±100 cents | 0 cents | 1 cent | 1 cent = 1/100 semitone |
| **Octave** | -5 to +5 | 0 | 1 | Multiply/divide by 2 |
| **Detune** | ±50 Hz | 0 Hz | 0.1 Hz | For beating effects |

### Amplitude Parameters

| Parameter | Range | Default | Resolution | Notes |
|-----------|-------|---------|------------|-------|
| **Amplitude** | 0 to 100% | 50% | 0.1% | Linear scale |
| **dB Level** | -∞ to 0 dB | -6 dB | 0.1 dB | Logarithmic scale |
| **Pan** | -100 to +100 | 0 | 1 | L/R balance |
| **Envelope** | ADSR | Off | - | Attack/Decay/Sustain/Release |

### Waveform-Specific Parameters

#### Square Wave
| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| **Duty Cycle** | 1% to 99% | 50% | Pulse width |
| **Edge Time** | 0 to 10ms | 0 | Rise/fall time |

#### Sawtooth Wave
| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| **Direction** | Up/Down | Up | Ramp direction |
| **Symmetry** | 0 to 100% | 50% | Ramp symmetry |

#### Triangle Wave
| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| **Symmetry** | 1% to 99% | 50% | Peak position |

#### Noise
| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| **Color** | White/Pink/Brown/Blue | White | Spectral shape |
| **Bandwidth** | 10 Hz to 20 kHz | Full | Filter range |
| **Seed** | 0 to 2^32-1 | Random | PRNG seed |

### Sweep Parameters

| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| **Start Freq** | 0.1 Hz to 22 kHz | 20 Hz | Initial frequency |
| **End Freq** | 0.1 Hz to 22 kHz | 20 kHz | Final frequency |
| **Duration** | 10ms to 60s | 1s | Sweep time |
| **Type** | Linear/Log/Exponential | Log | Sweep curve |
| **Repeat** | Off/On/Count | Off | Repetition mode |

---

## Use Cases and Applications

### Audio System Testing

#### Frequency Response Measurement
```rust
// Generate logarithmic sweep for impulse response measurement
let sweep = SignalGenerator::sweep()
    .start_frequency(20.0)
    .end_frequency(20000.0)
    .duration(Duration::from_secs(10))
    .sweep_type(SweepType::Logarithmic)
    .amplitude(0.5)
    .build();
```

**Purpose**: Measure system frequency response
**Method**: Play sweep, record output, deconvolve
**Result**: Impulse response and frequency response

#### THD+N Measurement
```rust
// Generate pure 1kHz sine for distortion measurement
let test_tone = SignalGenerator::sine()
    .frequency(1000.0)
    .amplitude(0.707) // -3dBFS
    .duration(Duration::from_secs(30))
    .build();
```

**Purpose**: Measure total harmonic distortion + noise
**Method**: FFT analysis of recorded signal
**Result**: THD+N percentage

### Speaker Calibration

#### Room Mode Detection
```rust
// Generate slow sweep through bass frequencies
let bass_sweep = SignalGenerator::sweep()
    .start_frequency(20.0)
    .end_frequency(200.0)
    .duration(Duration::from_secs(30))
    .sweep_type(SweepType::Linear)
    .build();
```

**Purpose**: Identify room resonances
**Method**: Measure SPL during sweep
**Result**: Room mode frequencies

#### Phase Alignment
```rust
// Generate impulse for time alignment
let impulse = SignalGenerator::impulse()
    .amplitude(1.0)
    .pre_delay(Duration::from_millis(100))
    .build();
```

**Purpose**: Align multiple speakers
**Method**: Measure arrival times
**Result**: Delay compensation values

### Hearing Tests

#### Pure Tone Audiometry
```rust
// Generate test tones at audiometric frequencies
const AUDIOMETRIC_FREQS: [f32; 11] = [
    125.0, 250.0, 500.0, 750.0, 1000.0,
    1500.0, 2000.0, 3000.0, 4000.0, 6000.0, 8000.0
];

for freq in AUDIOMETRIC_FREQS {
    let tone = SignalGenerator::sine()
        .frequency(freq)
        .amplitude_db(-40.0) // Start at 40 dB HL
        .duration(Duration::from_secs(3))
        .envelope(Envelope::raised_cosine(100.0)) // 100ms ramps
        .build();
}
```

#### Tinnitus Matching
```rust
// Generate adjustable tone for tinnitus frequency matching
let matching_tone = SignalGenerator::sine()
    .frequency_range(2000.0..=12000.0)
    .amplitude_range(0.0..=0.5)
    .modulation(Modulation::tremolo(4.0, 0.2)) // Optional modulation
    .build();
```

### Music Production

#### Tuning Reference
```rust
// Generate A440 tuning reference
let tuning_ref = SignalGenerator::sine()
    .frequency(440.0)
    .amplitude(0.5)
    .build();

// Generate equal temperament scale
let semitone_ratio = 2.0_f32.powf(1.0 / 12.0);
for semitone in 0..12 {
    let freq = 440.0 * semitone_ratio.powi(semitone);
    // Generate tone at freq
}
```

#### Synthesizer Testing
```rust
// Generate complex waveform for synthesis
let complex = SignalGenerator::multi_tone()
    .add_harmonic(100.0, 1.0, 0.0)      // Fundamental
    .add_harmonic(200.0, 0.5, PI/4.0)   // 2nd harmonic
    .add_harmonic(300.0, 0.33, PI/2.0)  // 3rd harmonic
    .add_harmonic(400.0, 0.25, 0.0)     // 4th harmonic
    .build();
```

---

## Testing Framework Integration

### Unit Test Signal Generation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Generate deterministic test signal
    fn test_signal(samples: usize) -> Vec<f32> {
        SignalGenerator::sine()
            .frequency(1000.0)
            .amplitude(1.0)
            .phase(0.0)
            .sample_rate(44100.0)
            .generate(samples)
    }

    #[test]
    fn test_filter_response() {
        let input = test_signal(1024);
        let mut filter = LowPassFilter::new(2000.0);
        let output = filter.process(&input);

        // Verify filter behavior
        assert!(output.len() == input.len());
        assert!(calculate_rms(&output) < calculate_rms(&input));
    }
}
```

### Performance Benchmarking

```rust
use criterion::{black_box, criterion_group, Criterion};

fn benchmark_generators(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_generators");

    group.bench_function("sine_1024", |b| {
        let gen = SignalGenerator::sine().frequency(1000.0).build();
        b.iter(|| {
            gen.generate(black_box(1024))
        });
    });

    group.bench_function("white_noise_1024", |b| {
        let gen = SignalGenerator::white_noise().build();
        b.iter(|| {
            gen.generate(black_box(1024))
        });
    });

    group.finish();
}
```

### Automated Testing Sequences

```rust
pub struct TestSequence {
    steps: Vec<TestStep>,
}

pub struct TestStep {
    signal: Box<dyn Generator>,
    duration: Duration,
    expected_response: ResponseCriteria,
}

impl TestSequence {
    pub fn frequency_response() -> Self {
        Self {
            steps: vec![
                TestStep {
                    signal: Box::new(SineGenerator::new(100.0, 0.5)),
                    duration: Duration::from_secs(1),
                    expected_response: ResponseCriteria::amplitude_range(0.4..=0.6),
                },
                // More test steps...
            ],
        }
    }

    pub async fn run(&self) -> TestResults {
        let mut results = TestResults::new();

        for step in &self.steps {
            let response = self.execute_step(step).await;
            results.add(step.validate(response));
        }

        results
    }
}
```

---

## Advanced Topics

### Anti-Aliasing Techniques

#### PolyBLEP (Polynomial Band-Limited Step)

```rust
fn polyblep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt + 1.0;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

fn generate_antialiased_square(frequency: f32, sample_rate: f32) -> impl Iterator<Item = f32> {
    let phase_increment = frequency / sample_rate;
    let mut phase = 0.0;

    std::iter::from_fn(move || {
        let mut value = if phase < 0.5 { 1.0 } else { -1.0 };

        // Apply PolyBLEP at discontinuities
        value += polyblep(phase, phase_increment);
        value -= polyblep((phase + 0.5) % 1.0, phase_increment);

        phase += phase_increment;
        if phase >= 1.0 {
            phase -= 1.0;
        }

        Some(value)
    })
}
```

#### Wavetable Synthesis

```rust
pub struct WavetableOscillator {
    tables: Vec<Vec<f32>>,
    table_size: usize,
    sample_rate: f32,
}

impl WavetableOscillator {
    pub fn new(sample_rate: f32) -> Self {
        let table_size = 2048;
        let tables = Self::generate_mipmap_tables(table_size);

        Self {
            tables,
            table_size,
            sample_rate,
        }
    }

    fn generate_mipmap_tables(size: usize) -> Vec<Vec<f32>> {
        let num_tables = (size as f32).log2() as usize;
        let mut tables = Vec::with_capacity(num_tables);

        for mip_level in 0..num_tables {
            let num_harmonics = size >> (mip_level + 1);
            let table = Self::generate_bandlimited_table(size, num_harmonics);
            tables.push(table);
        }

        tables
    }

    pub fn generate(&self, frequency: f32, phase: f32) -> f32 {
        // Select appropriate mipmap level based on frequency
        let table_index = self.select_table(frequency);
        let table = &self.tables[table_index];

        // Interpolate between samples
        let position = phase * self.table_size as f32;
        let index = position as usize % self.table_size;
        let fraction = position.fract();

        let s1 = table[index];
        let s2 = table[(index + 1) % self.table_size];

        s1 + (s2 - s1) * fraction // Linear interpolation
    }
}
```

### Modulation Techniques

#### Amplitude Modulation (AM)

```rust
pub struct AmplitudeModulator {
    carrier_freq: f32,
    modulator_freq: f32,
    modulation_depth: f32,
}

impl AmplitudeModulator {
    pub fn generate(&self, t: f32) -> f32 {
        let carrier = (2.0 * PI * self.carrier_freq * t).sin();
        let modulator = 1.0 + self.modulation_depth * (2.0 * PI * self.modulator_freq * t).sin();
        carrier * modulator
    }
}
```

#### Frequency Modulation (FM)

```rust
pub struct FrequencyModulator {
    carrier_freq: f32,
    modulator_freq: f32,
    modulation_index: f32,
}

impl FrequencyModulator {
    pub fn generate(&self, t: f32) -> f32 {
        let modulator = self.modulation_index * (2.0 * PI * self.modulator_freq * t).sin();
        (2.0 * PI * self.carrier_freq * t + modulator).sin()
    }
}
```

### Phase Coherent Multi-Tone Generation

```rust
pub struct MultiToneGenerator {
    tones: Vec<ToneParams>,
    sample_rate: f32,
    phases: Vec<f32>,
}

pub struct ToneParams {
    frequency: f32,
    amplitude: f32,
    initial_phase: f32,
}

impl MultiToneGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            tones: Vec::new(),
            sample_rate,
            phases: Vec::new(),
        }
    }

    pub fn add_tone(&mut self, freq: f32, amp: f32, phase: f32) {
        self.tones.push(ToneParams {
            frequency: freq,
            amplitude: amp,
            initial_phase: phase,
        });
        self.phases.push(phase);
    }

    pub fn generate_sample(&mut self) -> f32 {
        let mut sample = 0.0;

        for (i, tone) in self.tones.iter().enumerate() {
            sample += tone.amplitude * self.phases[i].sin();
            self.phases[i] += 2.0 * PI * tone.frequency / self.sample_rate;

            // Wrap phase to prevent numerical errors
            if self.phases[i] >= 2.0 * PI {
                self.phases[i] -= 2.0 * PI;
            }
        }

        // Apply soft clipping if necessary
        self.soft_clip(sample)
    }

    fn soft_clip(&self, x: f32) -> f32 {
        if x.abs() <= 0.5 {
            x
        } else {
            (2.0 / 3.0) * (x.signum() * (1.0 - (1.0 - x.abs()).powi(3)))
        }
    }
}
```

---

## Safety Considerations

### Volume Limiting

**Maximum Safe Levels:**

| Context | Maximum Level | Notes |
|---------|--------------|-------|
| Headphones | -12 dBFS | Protect hearing |
| Speakers | -6 dBFS | Prevent clipping |
| Recording | -3 dBFS | Leave headroom |
| Testing | -20 dBFS | Start low |

**Implementation:**
```rust
pub struct SafetyLimiter {
    max_amplitude: f32,
    attack_time: f32,
    release_time: f32,
    current_gain: f32,
}

impl SafetyLimiter {
    pub fn process(&mut self, sample: f32) -> f32 {
        let envelope = sample.abs();

        if envelope > self.max_amplitude {
            // Fast attack
            let target_gain = self.max_amplitude / envelope;
            self.current_gain += (target_gain - self.current_gain) * self.attack_time;
        } else {
            // Slow release
            self.current_gain += (1.0 - self.current_gain) * self.release_time;
        }

        sample * self.current_gain
    }
}
```

### DC Offset Prevention

```rust
pub struct DcBlocker {
    x1: f32,
    y1: f32,
    r: f32,
}

impl DcBlocker {
    pub fn new() -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            r: 0.995, // Cutoff around 3.5 Hz at 44.1 kHz
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = input - self.x1 + self.r * self.y1;
        self.x1 = input;
        self.y1 = output;
        output
    }
}
```

### Startup/Shutdown Ramping

```rust
pub struct RampGenerator {
    ramp_time: Duration,
    sample_rate: f32,
}

impl RampGenerator {
    pub fn generate_fade_in(&self) -> Vec<f32> {
        let samples = (self.ramp_time.as_secs_f32() * self.sample_rate) as usize;
        (0..samples)
            .map(|i| {
                let t = i as f32 / samples as f32;
                // Raised cosine fade
                0.5 * (1.0 - (PI * (1.0 - t)).cos())
            })
            .collect()
    }

    pub fn generate_fade_out(&self) -> Vec<f32> {
        let samples = (self.ramp_time.as_secs_f32() * self.sample_rate) as usize;
        (0..samples)
            .map(|i| {
                let t = i as f32 / samples as f32;
                // Raised cosine fade
                0.5 * (1.0 + (PI * t).cos())
            })
            .collect()
    }
}
```

---

## API Reference

### Core Generator Trait

```rust
pub trait Generator: Send + Sync {
    /// Generate next sample
    fn generate_sample(&mut self) -> f32;

    /// Generate buffer of samples
    fn generate_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.generate_sample();
        }
    }

    /// Reset generator state
    fn reset(&mut self);

    /// Get current parameters
    fn parameters(&self) -> GeneratorParams;
}
```

### Builder Pattern API

```rust
pub struct SignalGeneratorBuilder {
    waveform: WaveformType,
    frequency: f32,
    amplitude: f32,
    phase: f32,
    sample_rate: f32,
    // ... more fields
}

impl SignalGeneratorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn waveform(mut self, waveform: WaveformType) -> Self {
        self.waveform = waveform;
        self
    }

    pub fn frequency(mut self, freq: f32) -> Self {
        self.frequency = freq.clamp(0.1, self.sample_rate / 2.0);
        self
    }

    pub fn amplitude(mut self, amp: f32) -> Self {
        self.amplitude = amp.clamp(0.0, 1.0);
        self
    }

    pub fn build(self) -> Box<dyn Generator> {
        match self.waveform {
            WaveformType::Sine => Box::new(SineGenerator::new(
                self.frequency,
                self.amplitude,
                self.phase,
            )),
            WaveformType::Square => Box::new(SquareGenerator::new(
                self.frequency,
                self.amplitude,
                0.5, // Default duty cycle
            )),
            // ... other waveforms
        }
    }
}
```

### Convenience Functions

```rust
/// Generate test tone at standard frequency
pub fn test_tone_1khz(duration: Duration) -> Vec<f32> {
    SignalGenerator::sine()
        .frequency(1000.0)
        .amplitude(0.5)
        .duration(duration)
        .generate()
}

/// Generate calibration signal
pub fn calibration_signal() -> Vec<f32> {
    SignalGenerator::sine()
        .frequency(1000.0)
        .amplitude_db(-20.0) // -20 dBFS
        .duration(Duration::from_secs(10))
        .generate()
}

/// Generate SMPTE test signal
pub fn smpte_test_signal() -> Vec<f32> {
    SignalGenerator::multi_tone()
        .add_tone(60.0, 0.4)    // Low frequency
        .add_tone(7000.0, 0.4)  // High frequency
        .generate()
}
```

---

## Examples and Recipes

### Example 1: Generate Chord

```rust
// Generate C major triad
fn generate_c_major_chord() -> Vec<f32> {
    let sample_rate = 44100.0;
    let duration = Duration::from_secs(2);

    let mut generator = MultiToneGenerator::new(sample_rate);
    generator.add_tone(261.63, 0.33, 0.0); // C4
    generator.add_tone(329.63, 0.33, 0.0); // E4
    generator.add_tone(392.00, 0.33, 0.0); // G4

    let num_samples = (duration.as_secs_f32() * sample_rate) as usize;
    let mut buffer = vec![0.0; num_samples];

    for sample in &mut buffer {
        *sample = generator.generate_sample();
    }

    buffer
}
```

### Example 2: Binaural Beats

```rust
// Generate binaural beats for meditation
fn generate_binaural_beats(base_freq: f32, beat_freq: f32) -> (Vec<f32>, Vec<f32>) {
    let sample_rate = 44100.0;
    let duration = Duration::from_secs(60);

    let left = SignalGenerator::sine()
        .frequency(base_freq)
        .amplitude(0.5)
        .sample_rate(sample_rate)
        .duration(duration)
        .generate();

    let right = SignalGenerator::sine()
        .frequency(base_freq + beat_freq)
        .amplitude(0.5)
        .sample_rate(sample_rate)
        .duration(duration)
        .generate();

    (left, right)
}

// Example: Alpha waves (10 Hz beats)
let (left, right) = generate_binaural_beats(200.0, 10.0);
```

### Example 3: Chirp Signal

```rust
// Generate exponential chirp for system identification
fn generate_chirp(start_freq: f32, end_freq: f32, duration: Duration) -> Vec<f32> {
    let sample_rate = 44100.0;
    let num_samples = (duration.as_secs_f32() * sample_rate) as usize;
    let k = (end_freq / start_freq).ln() / duration.as_secs_f32();

    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let freq = start_freq * (k * t).exp();
            let phase = 2.0 * PI * start_freq * ((k * t).exp() - 1.0) / k;
            0.5 * phase.sin()
        })
        .collect()
}
```

### Example 4: DTMF Tone

```rust
// Generate DTMF (phone dial) tone
fn generate_dtmf(digit: char) -> Option<Vec<f32>> {
    let (low_freq, high_freq) = match digit {
        '1' => (697.0, 1209.0),
        '2' => (697.0, 1336.0),
        '3' => (697.0, 1477.0),
        '4' => (770.0, 1209.0),
        '5' => (770.0, 1336.0),
        '6' => (770.0, 1477.0),
        '7' => (852.0, 1209.0),
        '8' => (852.0, 1336.0),
        '9' => (852.0, 1477.0),
        '*' => (941.0, 1209.0),
        '0' => (941.0, 1336.0),
        '#' => (941.0, 1477.0),
        _ => return None,
    };

    let mut generator = MultiToneGenerator::new(44100.0);
    generator.add_tone(low_freq, 0.5, 0.0);
    generator.add_tone(high_freq, 0.5, 0.0);

    let duration = Duration::from_millis(200);
    let num_samples = (duration.as_secs_f32() * 44100.0) as usize;
    let mut buffer = vec![0.0; num_samples];

    for sample in &mut buffer {
        *sample = generator.generate_sample();
    }

    Some(buffer)
}
```

### Example 5: Custom Waveform

```rust
// Generate custom waveform from harmonic series
fn generate_custom_timbre(fundamental: f32, harmonics: &[(f32, f32)]) -> Vec<f32> {
    let sample_rate = 44100.0;
    let duration = Duration::from_secs(1);

    let mut generator = MultiToneGenerator::new(sample_rate);

    // Add fundamental
    generator.add_tone(fundamental, 0.5, 0.0);

    // Add harmonics
    for (harmonic_number, amplitude) in harmonics {
        generator.add_tone(fundamental * harmonic_number, *amplitude, 0.0);
    }

    let num_samples = (duration.as_secs_f32() * sample_rate) as usize;
    let mut buffer = vec![0.0; num_samples];

    for sample in &mut buffer {
        *sample = generator.generate_sample();
    }

    buffer
}

// Example: Clarinet-like timbre (odd harmonics)
let clarinet = generate_custom_timbre(440.0, &[
    (3.0, 0.3),   // 3rd harmonic
    (5.0, 0.15),  // 5th harmonic
    (7.0, 0.08),  // 7th harmonic
    (9.0, 0.04),  // 9th harmonic
]);
```

---

## Troubleshooting

### Common Issues

| Problem | Cause | Solution |
|---------|-------|----------|
| Clicking/popping | No fade in/out | Apply envelope |
| Distortion | Amplitude too high | Reduce to <0.9 |
| Aliasing | Frequency too high | Use band-limited waveforms |
| DC offset | Asymmetric waveform | Apply DC blocker |
| Phase cancellation | Identical signals | Add slight detuning |

### Debugging Tips

```rust
// Debug signal properties
pub fn analyze_signal(signal: &[f32]) {
    println!("Samples: {}", signal.len());
    println!("Min: {:.3}", signal.iter().fold(f32::INFINITY, |a, &b| a.min(b)));
    println!("Max: {:.3}", signal.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));
    println!("RMS: {:.3}", calculate_rms(signal));
    println!("Peak: {:.3}", signal.iter().map(|x| x.abs()).fold(0.0, f32::max));
    println!("DC: {:.6}", signal.iter().sum::<f32>() / signal.len() as f32);
}

fn calculate_rms(signal: &[f32]) -> f32 {
    let sum_squares: f32 = signal.iter().map(|x| x * x).sum();
    (sum_squares / signal.len() as f32).sqrt()
}
```

---

*End of Signal Generator Guide - Version 1.0*

*For additional resources and updates:*
- API Documentation: `cargo doc --open`
- Source Code: [github.com/yourusername/rusty-audio](https://github.com/yourusername/rusty-audio)
- Signal Processing Theory: [dspguide.com](http://www.dspguide.com)