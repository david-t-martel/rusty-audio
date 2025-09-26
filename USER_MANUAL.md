# Rusty Audio - User Manual

## Welcome to Rusty Audio

Rusty Audio is a professional-grade audio player and signal processing application built with Rust and egui. It provides high-quality audio playback, advanced effects processing, parametric equalization, and comprehensive signal generation capabilities.

## Table of Contents

1. [Getting Started](#getting-started)
2. [System Requirements](#system-requirements)
3. [Installation](#installation)
4. [First-Time Setup](#first-time-setup)
5. [User Interface Overview](#user-interface-overview)
6. [Feature Documentation](#feature-documentation)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Troubleshooting Guide](#troubleshooting-guide)
9. [Audio Safety Guidelines](#audio-safety-guidelines)
10. [Frequently Asked Questions](#frequently-asked-questions)

---

## Getting Started

### Quick Start Guide

1. **Launch the Application**
   - Windows: Double-click `rusty-audio.exe`
   - Linux: Run `./rusty-audio` from terminal
   - macOS: Open `RustyAudio.app`

2. **Load Your First Audio File**
   - Press `Ctrl+O` or click the "Open File" button
   - Select an audio file (MP3, WAV, FLAC, OGG, AAC)
   - The file will load automatically

3. **Play Audio**
   - Press `Space` or click the Play button
   - Adjust volume with Up/Down arrow keys
   - Seek with Left/Right arrow keys

### User Interface Layout

The application is organized into five main tabs:
- **Playback**: Main audio controls and visualization
- **Effects**: Audio effects processing
- **EQ**: 8-band parametric equalizer
- **Generator**: Signal generation tools
- **Settings**: Application preferences and themes

---

## System Requirements

### Minimum Requirements
- **OS**: Windows 10/11, Ubuntu 20.04+, macOS 11+
- **Processor**: Dual-core 2.0 GHz
- **RAM**: 4 GB
- **Storage**: 100 MB available space
- **Audio**: Any WASAPI-compatible device (Windows), PulseAudio (Linux), CoreAudio (macOS)

### Recommended Requirements
- **Processor**: Quad-core 2.5 GHz or better
- **RAM**: 8 GB or more
- **Storage**: 500 MB for optimal performance
- **Audio**: Low-latency audio interface for professional use
- **Display**: 1920x1080 or higher for best experience

### Supported Audio Formats
- **Lossless**: WAV, FLAC, ALAC, AIFF
- **Compressed**: MP3, AAC, OGG Vorbis, Opus
- **Sample Rates**: 8 kHz to 192 kHz
- **Bit Depths**: 8-bit, 16-bit, 24-bit, 32-bit float

---

## Installation

### Windows Installation

1. **Download the Installer**
   - Visit the releases page
   - Download `rusty-audio-setup.exe`

2. **Run the Installer**
   - Right-click and select "Run as administrator"
   - Follow the installation wizard
   - Choose installation directory (default: `C:\Program Files\Rusty Audio`)

3. **Audio Driver Configuration**
   - The installer will detect your audio devices
   - Select your preferred output device
   - Configure sample rate and buffer size

### Linux Installation

#### Ubuntu/Debian
```bash
# Install dependencies
sudo apt update
sudo apt install libasound2-dev libpulse-dev

# Download and extract
wget https://github.com/yourusername/rusty-audio/releases/latest/rusty-audio-linux.tar.gz
tar -xzf rusty-audio-linux.tar.gz
cd rusty-audio

# Make executable and run
chmod +x rusty-audio
./rusty-audio
```

#### Arch Linux
```bash
# Install from AUR
yay -S rusty-audio
```

### macOS Installation

1. **Download the DMG**
   - Download `RustyAudio.dmg` from releases

2. **Install the Application**
   - Open the DMG file
   - Drag RustyAudio to Applications folder
   - First run: Right-click and select "Open" to bypass Gatekeeper

3. **Grant Audio Permissions**
   - System Preferences → Security & Privacy → Microphone
   - Enable RustyAudio if recording features are needed

---

## First-Time Setup

### Audio Configuration

1. **Select Audio Output Device**
   - Go to Settings tab → Audio
   - Choose your preferred output device
   - Test with the "Test Audio" button

2. **Configure Buffer Size**
   - Lower values (128-256) for lower latency
   - Higher values (512-2048) for stability
   - Recommended: 512 samples for general use

3. **Set Sample Rate**
   - Match your audio interface's native rate
   - Common rates: 44.1 kHz, 48 kHz, 96 kHz
   - Higher rates for professional work

### Theme Selection

1. **Choose Visual Theme**
   - Settings → Appearance → Theme
   - Available themes:
     - Catppuccin Mocha (Dark, default)
     - Catppuccin Latte (Light)
     - Dracula (Dark, high contrast)
     - High Contrast (Accessibility)

2. **Adjust UI Scale**
   - Settings → Appearance → UI Scale
   - Range: 50% to 200%
   - Use Ctrl+Plus/Minus for quick adjustment

### Keyboard Shortcuts Setup

1. **View Current Shortcuts**
   - Press F1 or go to Settings → Shortcuts
   - Print or save the reference card

2. **Customize Shortcuts** (Coming in v2.0)
   - Settings → Shortcuts → Customize
   - Click on action and press new key combination
   - Reset to defaults option available

---

## User Interface Overview

### Main Window Components

```
┌─────────────────────────────────────────────────────────┐
│  File  Edit  View  Tools  Help                         │
├─────────────────────────────────────────────────────────┤
│  [Playback] [Effects] [EQ] [Generator] [Settings]      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌───────────────────────────────────────────────┐    │
│  │          Spectrum Visualizer                   │    │
│  │     ▁▃▅▇█▇▅▃▁ ▃▅▇█▇▅▃ ▁▃▅▇█▇▅▃▁           │    │
│  └───────────────────────────────────────────────┘    │
│                                                         │
│  ┌─── Now Playing ──────────────────────────────┐     │
│  │ Title:  Song Name                            │     │
│  │ Artist: Artist Name                          │     │
│  │ Album:  Album Name                           │     │
│  │ [Album Art]  Duration: 3:45 / 5:20          │     │
│  └──────────────────────────────────────────────┘     │
│                                                         │
│  ┌─── Playback Controls ────────────────────────┐     │
│  │  [⏮] [⏸] [⏯] [⏭]  Volume: ▓▓▓▓▓░░░ 65%    │     │
│  │  Progress: ▓▓▓▓▓▓▓░░░░░░░░░ 3:45 / 5:20    │     │
│  │  [🔁 Loop] [🔀 Shuffle]                      │     │
│  └──────────────────────────────────────────────┘     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Status Bar Information

The status bar at the bottom displays:
- Current audio format and sample rate
- CPU and memory usage
- Audio buffer health indicator
- Error messages and notifications

---

## Feature Documentation

### Playback Tab

#### Basic Playback Controls

**Play/Pause Button**
- Click or press `Space` to toggle playback
- Visual indicator shows current state
- Animated pulse effect when playing

**Stop Button**
- Click or press `S` to stop playback
- Returns to beginning of track
- Clears audio buffer

**Volume Control**
- Drag slider or use Up/Down arrow keys
- Range: 0% to 100%
- Double-click to reset to 50%
- Scroll wheel for fine adjustment

**Seek Bar**
- Click to jump to position
- Drag for scrubbing
- Left/Right arrows: ±5 seconds
- Page Up/Down: ±30 seconds

#### Advanced Playback Features

**Loop Mode**
- Press `L` to toggle loop
- Options: Off, Single Track, All Tracks
- Visual indicator in control bar

**Playback Speed**
- Range: 0.25x to 4.0x
- Preserves pitch by default
- Option for pitch-shifted speed change

**A-B Repeat**
- Set point A at current position
- Set point B to create loop region
- Visual markers on progress bar

### Effects Tab

#### Available Effects

**Reverb**
- Room Size: 0-100%
- Damping: 0-100%
- Wet/Dry Mix: 0-100%
- Presets: Hall, Room, Chamber, Cathedral

**Delay**
- Delay Time: 1ms to 2000ms
- Feedback: 0-95%
- Mix: 0-100%
- Sync to BPM option

**Compressor**
- Threshold: -60dB to 0dB
- Ratio: 1:1 to 20:1
- Attack: 0.1ms to 100ms
- Release: 10ms to 1000ms
- Makeup Gain: 0dB to 24dB

**Distortion**
- Drive: 0-100%
- Tone: 0-100%
- Output Level: 0-100%
- Types: Soft Clip, Hard Clip, Tube

#### Effects Chain Management

1. **Enable/Disable Effects**
   - Click power button on each effect
   - Bypass all with master bypass

2. **Reorder Effects**
   - Drag and drop to reorder
   - Signal flows top to bottom

3. **Save/Load Presets**
   - Save current settings as preset
   - Quick access to factory presets
   - Import/export preset files

### EQ Tab

#### 8-Band Parametric Equalizer

**Frequency Bands:**
1. Sub-Bass: 20-60 Hz
2. Bass: 60-250 Hz
3. Low-Mid: 250-500 Hz
4. Mid: 500-2000 Hz
5. Upper-Mid: 2-4 kHz
6. Presence: 4-6 kHz
7. Brilliance: 6-16 kHz
8. Air: 16-20 kHz

**Controls per Band:**
- Frequency: Center frequency adjustment
- Gain: -24dB to +24dB
- Q Factor: 0.1 to 10 (bandwidth)
- Enable/Disable toggle

**Global Controls:**
- Master bypass
- Reset all bands
- Input/Output gain
- Visual spectrum overlay

**EQ Presets:**
- Flat (Default)
- Bass Boost
- Vocal Enhance
- Bright
- Warm
- Loudness
- Custom presets

### Generator Tab

#### Signal Types

**Sine Wave**
- Frequency: 20 Hz to 20 kHz
- Amplitude: 0-100%
- Phase: 0-360°
- Pure tone generation

**Square Wave**
- Frequency: 20 Hz to 20 kHz
- Duty Cycle: 1-99%
- Amplitude: 0-100%
- Harmonic-rich signal

**Sawtooth Wave**
- Frequency: 20 Hz to 20 kHz
- Amplitude: 0-100%
- Direction: Rising/Falling
- Full harmonic spectrum

**Triangle Wave**
- Frequency: 20 Hz to 20 kHz
- Amplitude: 0-100%
- Symmetry: 1-99%
- Soft harmonics

**White Noise**
- Amplitude: 0-100%
- Filter: Low-pass/High-pass/Band-pass
- Cutoff frequency control

**Pink Noise**
- Amplitude: 0-100%
- -3dB/octave spectrum
- Reference calibration option

#### Advanced Generator Features

**Frequency Sweep**
- Start: 20 Hz to 20 kHz
- End: 20 Hz to 20 kHz
- Duration: 100ms to 60s
- Type: Linear/Logarithmic

**Multi-Tone**
- Up to 8 simultaneous tones
- Independent frequency/amplitude
- Phase relationships
- Harmonic series presets

**Modulation**
- AM (Amplitude Modulation)
- FM (Frequency Modulation)
- Carrier and modulator controls
- Modulation depth and rate

### Settings Tab

#### Audio Settings

**Output Device**
- List of available devices
- Default device option
- Exclusive mode toggle
- Device sample rate info

**Buffer Settings**
- Size: 64 to 4096 samples
- Count: 2 to 8 buffers
- Latency display in ms
- Optimization suggestions

**Processing**
- Dithering: On/Off
- Bit depth conversion
- Sample rate conversion quality
- DC offset removal

#### Appearance Settings

**Theme**
- Catppuccin variants
- High contrast modes
- Custom color picker
- Import/export themes

**UI Scale**
- 50% to 200%
- DPI awareness
- Font size adjustment
- Icon size options

**Visualizer**
- FFT size: 256 to 8192
- Update rate: 15-144 fps
- Color schemes
- Peak hold options

#### Performance Settings

**CPU Usage**
- Multi-threading: On/Off
- Core affinity
- Priority level
- Power saving mode

**Memory**
- Cache size limit
- Preload amount
- Garbage collection
- Memory usage display

**Graphics**
- Hardware acceleration
- VSync: On/Off
- Frame rate limit
- Reduced motion mode

---

## Keyboard Shortcuts

### Global Shortcuts (Work in All Tabs)

| Shortcut | Action | Description |
|----------|--------|-------------|
| **Space** | Play/Pause | Toggle playback |
| **Enter** | Play/Pause | Alternative toggle |
| **S** | Stop | Stop playback and return to start |
| **L** | Loop | Toggle loop mode |
| **M** | Mute | Toggle mute |
| **Ctrl+O** | Open | Open file dialog |
| **Ctrl+S** | Save | Save current preset |
| **Ctrl+Q** | Quit | Exit application |
| **F1** | Help | Show help/shortcuts |
| **F11** | Fullscreen | Toggle fullscreen |
| **Escape** | Cancel | Cancel current operation |

### Playback Navigation

| Shortcut | Action | Description |
|----------|--------|-------------|
| **←/→** | Seek ±5s | Jump 5 seconds |
| **Shift+←/→** | Seek ±1s | Fine seek 1 second |
| **Page Up/Down** | Seek ±30s | Jump 30 seconds |
| **Home** | Beginning | Go to start |
| **End** | End | Go to end |
| **↑/↓** | Volume ±5% | Adjust volume |
| **Shift+↑/↓** | Volume ±1% | Fine volume adjust |

### Tab Navigation

| Shortcut | Action | Description |
|----------|--------|-------------|
| **Ctrl+1** | Playback Tab | Switch to Playback |
| **Ctrl+2** | Effects Tab | Switch to Effects |
| **Ctrl+3** | EQ Tab | Switch to EQ |
| **Ctrl+4** | Generator Tab | Switch to Generator |
| **Ctrl+5** | Settings Tab | Switch to Settings |
| **Tab** | Next Control | Focus next control |
| **Shift+Tab** | Previous Control | Focus previous control |

### EQ Shortcuts (When in EQ Tab)

| Shortcut | Action | Description |
|----------|--------|-------------|
| **1-8** | Select Band | Select EQ band 1-8 |
| **Q/A** | Gain ±1dB | Increase/decrease gain |
| **W/S** | Q ±0.1 | Adjust Q factor |
| **E/D** | Freq ±10Hz | Adjust frequency |
| **R** | Reset Band | Reset selected band |
| **T** | Reset All | Reset all bands |

### Effects Shortcuts (When in Effects Tab)

| Shortcut | Action | Description |
|----------|--------|-------------|
| **E** | Toggle Effect | Enable/disable current |
| **B** | Bypass All | Bypass effects chain |
| **P** | Next Preset | Cycle through presets |
| **Shift+P** | Previous Preset | Previous preset |
| **C** | Copy Settings | Copy effect settings |
| **V** | Paste Settings | Paste effect settings |

### Generator Shortcuts (When in Generator Tab)

| Shortcut | Action | Description |
|----------|--------|-------------|
| **G** | Generate | Start/stop generation |
| **W** | Wave Type | Cycle wave types |
| **F** | Frequency Mode | Toggle freq input mode |
| **A** | Amplitude Mode | Toggle amp input mode |
| **N** | Add Tone | Add multi-tone |
| **Delete** | Remove Tone | Remove selected tone |

### Accessibility Shortcuts

| Shortcut | Action | Description |
|----------|--------|-------------|
| **Ctrl+Plus** | Zoom In | Increase UI scale |
| **Ctrl+Minus** | Zoom Out | Decrease UI scale |
| **Ctrl+0** | Reset Zoom | Reset to 100% |
| **Alt+H** | High Contrast | Toggle high contrast |
| **Alt+M** | Reduce Motion | Toggle animations |
| **Alt+C** | Color Mode | Cycle color blind modes |

---

## Troubleshooting Guide

### Common Issues and Solutions

#### Audio Playback Issues

**No Sound Output**
1. Check volume is not muted or at 0%
2. Verify correct output device in Settings
3. Check Windows/system volume mixer
4. Restart audio service:
   - Windows: `services.msc` → Windows Audio
   - Linux: `systemctl restart pulseaudio`
   - macOS: `sudo killall coreaudiod`

**Crackling or Distorted Audio**
1. Increase buffer size in Settings → Audio
2. Close other audio applications
3. Disable audio enhancements in system settings
4. Update audio drivers
5. Try exclusive mode (Windows)

**Audio Stuttering**
1. Check CPU usage (should be <50%)
2. Disable visualizer temporarily
3. Close background applications
4. Increase process priority
5. Disable power saving modes

#### File Loading Issues

**"Unsupported Format" Error**
1. Verify file extension is supported
2. Check file isn't corrupted
3. Try converting with external tool
4. Update codec libraries

**"Access Denied" Error**
1. Check file permissions
2. Move file to accessible location
3. Run as administrator (Windows)
4. Check if file is in use

**Metadata Not Displaying**
1. File may lack metadata tags
2. Use external tagger (MP3Tag, etc.)
3. Check encoding of metadata
4. Try different file format

#### Performance Issues

**High CPU Usage**
1. Reduce visualizer FFT size
2. Disable unused effects
3. Lower UI refresh rate
4. Close other applications
5. Check for background processes

**High Memory Usage**
1. Clear file cache in Settings
2. Reduce preload buffer
3. Restart application
4. Check for memory leaks
5. Update to latest version

**Slow UI Response**
1. Disable animations in Settings
2. Reduce UI scale
3. Switch to simpler theme
4. Update graphics drivers
5. Disable hardware acceleration

#### Generator Issues

**No Signal Output**
1. Check amplitude is not 0%
2. Verify generator is enabled
3. Check routing to output
4. Verify frequency is in range
5. Test with sine wave first

**Distorted Generated Signal**
1. Reduce amplitude below 80%
2. Check for clipping indicators
3. Adjust output gain
4. Verify sample rate match
5. Use lower frequencies

### Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Failed to initialize audio" | Driver issue | Reinstall audio drivers |
| "Buffer underrun detected" | Performance issue | Increase buffer size |
| "Sample rate mismatch" | Device conflict | Match device sample rate |
| "File access error" | Permission issue | Check file permissions |
| "Memory allocation failed" | Out of memory | Close other applications |
| "Device not found" | Device disconnected | Reconnect audio device |

### Getting Help

**Support Resources:**
1. GitHub Issues: Report bugs and request features
2. Discord Server: Community support
3. Documentation Wiki: Detailed guides
4. Email Support: support@rustyaudio.app

**When Reporting Issues:**
- Application version
- Operating system
- Audio device info
- Steps to reproduce
- Error messages
- Log files (Settings → Debug → Export Logs)

---

## Audio Safety Guidelines

### Volume Safety

#### Recommended Listening Levels

**Safe Listening Practices:**
- Keep volume below 85 dB for extended listening
- Take 15-minute breaks every hour
- Use the 60/60 rule: 60% volume for 60 minutes
- Lower volume in quiet environments

**Volume Limiting Features:**
1. **Soft Limiter** (Enabled by default)
   - Prevents sudden volume spikes
   - Gradual compression above 90%
   - Protects hearing and equipment

2. **Hard Limiter**
   - Absolute maximum at 0 dBFS
   - Prevents digital clipping
   - Brick-wall limiting

3. **Reference Level Calibration**
   - Set reference level for your system
   - SPL meter integration
   - K-System metering option

### Hearing Protection

**Warning Signs of Dangerous Levels:**
- Need to raise voice to be heard
- Ringing in ears after listening
- Temporary hearing reduction
- Ear fatigue or discomfort

**Safe Practices:**
- Start at low volume and increase gradually
- Use high-quality headphones/speakers
- Avoid prolonged high-volume exposure
- Regular hearing checks recommended

### Equipment Protection

**Speaker/Headphone Safety:**
- Check impedance matching
- Start with volume at zero
- Gradual volume increases
- Monitor for distortion
- Use appropriate amplification

**Signal Generation Safety:**
- Always start at low amplitude
- Check frequency range compatibility
- Avoid DC offset signals
- Monitor output levels
- Use protection circuits

---

## Frequently Asked Questions

### General Questions

**Q: Is Rusty Audio free to use?**
A: Yes, Rusty Audio is open-source and free for personal and commercial use under the MIT license.

**Q: Can I use Rusty Audio for professional audio work?**
A: Yes, Rusty Audio supports professional audio formats and sample rates up to 192 kHz / 32-bit.

**Q: Does Rusty Audio support VST plugins?**
A: VST support is planned for version 2.0. Currently, built-in effects are available.

**Q: Can I use Rusty Audio as my default media player?**
A: Yes, you can set file associations in your operating system settings.

### Technical Questions

**Q: What audio backend does Rusty Audio use?**
A: Rusty Audio uses CPAL (Cross-Platform Audio Library) which interfaces with:
- WASAPI (Windows)
- PulseAudio/ALSA (Linux)
- CoreAudio (macOS)

**Q: Why is CPU usage high during playback?**
A: This may be due to:
- Real-time spectrum analysis
- High-quality sample rate conversion
- Multiple active effects
- Visualizer rendering

**Q: Can I use ASIO drivers on Windows?**
A: ASIO support requires additional configuration. See the GitHub wiki for ASIO setup guide.

**Q: How accurate is the signal generator?**
A: The signal generator is accurate to within ±0.01% for frequency and ±0.1 dB for amplitude.

### Feature Questions

**Q: Can I record audio with Rusty Audio?**
A: Recording features are planned for version 2.0. Currently, Rusty Audio is playback-only.

**Q: Is there a playlist feature?**
A: Basic playlist support is available. Advanced playlist management coming in version 1.5.

**Q: Can I stream audio from network sources?**
A: Network streaming is on the roadmap for version 2.0.

**Q: Does Rusty Audio support gapless playback?**
A: Yes, gapless playback is supported for sequential tracks in compatible formats.

### Troubleshooting Questions

**Q: Why won't my file play?**
A: Check that:
1. File format is supported
2. File isn't corrupted
3. You have read permissions
4. Codecs are properly installed

**Q: How do I reset all settings?**
A: Delete the configuration file:
- Windows: `%APPDATA%\RustyAudio\config.toml`
- Linux: `~/.config/rusty-audio/config.toml`
- macOS: `~/Library/Preferences/com.rustyaudio.config.toml`

**Q: Why is there latency in the signal generator?**
A: Latency depends on buffer size. Reduce buffer size in Settings → Audio for lower latency.

---

## Appendices

### Appendix A: Supported File Formats

| Format | Extension | Codec | Max Sample Rate | Max Bit Depth |
|--------|-----------|-------|-----------------|----------------|
| WAV | .wav | PCM, Float | 384 kHz | 32-bit float |
| FLAC | .flac | FLAC | 192 kHz | 24-bit |
| MP3 | .mp3 | MPEG-1/2 Layer 3 | 48 kHz | 320 kbps |
| AAC | .m4a, .aac | AAC-LC, HE-AAC | 96 kHz | 320 kbps |
| OGG | .ogg | Vorbis, Opus | 192 kHz | 500 kbps |
| AIFF | .aiff | PCM | 192 kHz | 32-bit |

### Appendix B: System Audio APIs

| Platform | Primary API | Fallback API | Exclusive Mode |
|----------|-------------|--------------|----------------|
| Windows 10/11 | WASAPI | DirectSound | Yes |
| Linux | PulseAudio | ALSA | No |
| macOS | CoreAudio | - | Yes |

### Appendix C: Keyboard Shortcut Quick Card

Print this section for desk reference:

```
ESSENTIAL SHORTCUTS
━━━━━━━━━━━━━━━━━━
Space      Play/Pause
S          Stop
L          Loop
M          Mute
↑/↓        Volume
←/→        Seek
Ctrl+O     Open File
F1         Help

NAVIGATION
━━━━━━━━━━
Ctrl+1-5   Switch Tabs
Tab        Next Control
Home/End   Track Start/End
Page ↑/↓   Skip ±30s

ACCESSIBILITY
━━━━━━━━━━━━━
Ctrl +/-   Zoom
Alt+H      High Contrast
Alt+C      Color Blind Mode
```

---

*End of User Manual - Version 1.0*

*For the latest documentation and updates, visit:*
- GitHub: [github.com/yourusername/rusty-audio](https://github.com/yourusername/rusty-audio)
- Documentation: [docs.rustyaudio.app](https://docs.rustyaudio.app)
- Support: [support@rustyaudio.app](mailto:support@rustyaudio.app)