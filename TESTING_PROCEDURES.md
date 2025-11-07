# Rusty Audio - Manual Testing Procedures

## Overview

This document provides comprehensive manual testing procedures for the Rusty Audio car stereo-style interface, with a focus on HiDPI displays, landscape optimization, and accessibility features.

## Test Environment Setup

### Hardware Requirements
- **Primary**: Windows PC with HiDPI display (1.25x scaling recommended)
- **Secondary**: Multiple resolution monitors for cross-resolution testing
- **Audio**: External speakers or headphones for audio testing
- **Input**: Keyboard and mouse for interaction testing

### Software Requirements
- Windows 10/11 with HiDPI display scaling enabled
- Latest Rust toolchain (1.70+)
- Audio test files in supported formats (MP3, WAV, FLAC, OGG, M4A)

### Display Configuration Testing Matrix

| Resolution | Scaling | Orientation | Priority |
|------------|---------|-------------|----------|
| 1920x1080 | 1.0x | Landscape | High |
| 1920x1080 | 1.25x | Landscape | Critical |
| 2560x1440 | 1.25x | Landscape | High |
| 2560x1440 | 1.5x | Landscape | Medium |
| 3440x1440 | 1.25x | Landscape | Medium |
| 1366x768 | 1.0x | Landscape | Low |

## Test Execution Procedures

### 1. Application Launch Testing

#### Test Case 1.1: Initial Startup
**Objective**: Verify application launches correctly with proper HiDPI scaling.

**Steps**:
1. Ensure Windows display scaling is set to 125%
2. Launch `rusty-audio.exe` from command line or file explorer
3. Observe application window appearance and sizing

**Expected Results**:
- ✅ Application opens in landscape mode (1200x800 default)
- ✅ Text and UI elements are crisp and properly scaled
- ✅ Window is centered on screen
- ✅ All buttons and controls are clearly visible
- ✅ No visual artifacts or blurry text

**Validation Criteria**:
- Startup time < 3 seconds
- No console errors or warnings
- UI elements scale proportionally to system DPI

#### Test Case 1.2: Window Resizing
**Objective**: Verify responsive layout adaptation during window resizing.

**Steps**:
1. Launch application in default size (1200x800)
2. Drag window corners to resize to minimum size (800x600)
3. Drag to resize to maximum screen size
4. Test intermediate sizes: 1000x700, 1400x900, 1600x1000

**Expected Results**:
- ✅ UI layout adapts smoothly to different sizes
- ✅ No UI elements are cut off or overlapping
- ✅ Tab buttons remain accessible
- ✅ Control panels adjust appropriately
- ✅ Text remains readable at all sizes

### 2. Interface Navigation Testing

#### Test Case 2.1: Tab Navigation
**Objective**: Verify all tabs are accessible and display correctly.

**Steps**:
1. Click each tab in sequence: Playback → Effects → EQ → Generator → Settings
2. Verify content loads in each tab
3. Test tab switching with keyboard shortcuts (if available)
4. Return to Playback tab

**Expected Results**:
- ✅ All tabs respond to clicks within 100ms
- ✅ Tab content loads completely before displaying
- ✅ Active tab is clearly highlighted
- ✅ Tab labels are readable and properly sized
- ✅ No visual glitches during tab transitions

#### Test Case 2.2: Car Stereo Style Interface Verification
**Objective**: Confirm the interface maintains car stereo aesthetics.

**Steps**:
1. Observe overall visual design and color scheme
2. Check button styling and layout
3. Verify landscape-optimized arrangement
4. Test theme selector functionality

**Expected Results**:
- ✅ Interface resembles modern car stereo design
- ✅ Landscape layout is intuitive and ergonomic
- ✅ Color scheme is automotive-appropriate
- ✅ Button sizes are touch-friendly (minimum 40x30px)
- ✅ Visual hierarchy guides user attention effectively

### 3. Audio Functionality Testing

#### Test Case 3.1: File Loading and Playback
**Objective**: Verify audio file loading and basic playback controls.

**Preparation**: Prepare test files in multiple formats:
- `test_file.mp3` (128kbps, 44.1kHz)
- `test_file.wav` (16-bit, 44.1kHz)
- `test_file.flac` (lossless)
- `test_file.ogg` (Vorbis)

**Steps**:
1. Click "📁 Open" button
2. Select each test file format one at a time
3. Verify metadata display (title, artist, album, year)
4. Test playback controls: Play, Pause, Stop
5. Test loop toggle functionality

**Expected Results**:
- ✅ All supported formats load successfully
- ✅ Metadata displays correctly (or "Unknown" if not available)
- ✅ Playback starts within 2 seconds of clicking play
- ✅ Audio quality is clear and undistorted
- ✅ All playback controls respond immediately

#### Test Case 3.2: Volume and Audio Controls
**Objective**: Test volume control and audio processing features.

**Steps**:
1. Load an audio file and start playback
2. Test volume slider from 0% to 100%
3. Verify volume safety warnings at high levels (>80%)
4. Test volume keyboard shortcuts (↑/↓ arrows)
5. Verify audio output levels are appropriate

**Expected Results**:
- ✅ Volume changes smoothly without clicking or distortion
- ✅ Safety warning appears when volume exceeds 80%
- ✅ Volume can be set to exactly 0% (silent) and 100% (maximum)
- ✅ Keyboard shortcuts respond within 50ms
- ✅ No audio clipping or distortion at any volume level

#### Test Case 3.3: Progress Bar and Seeking
**Objective**: Verify progress tracking and seeking functionality.

**Steps**:
1. Load a long audio file (>1 minute)
2. Start playback and observe progress bar movement
3. Click different positions on progress bar to seek
4. Use keyboard arrows for seeking (←/→ for 5-second jumps)
5. Test seeking while paused

**Expected Results**:
- ✅ Progress bar updates smoothly during playback
- ✅ Clicking on progress bar seeks to correct position
- ✅ Seeking completes within 1 second
- ✅ Keyboard seeking works in 5-second increments
- ✅ Time display shows correct current/total duration

### 4. Equalizer and Effects Testing

#### Test Case 4.1: Equalizer Functionality
**Objective**: Test 8-band equalizer controls and audio processing.

**Steps**:
1. Navigate to EQ tab
2. Load an audio file with diverse frequency content
3. Adjust each EQ band individually from -40dB to +40dB
4. Test "Reset All" button functionality
5. Create different EQ curves (bass boost, treble boost, V-shape)

**Expected Results**:
- ✅ Each EQ band affects audio within expected frequency range
- ✅ EQ knobs respond smoothly to mouse input
- ✅ Gain values display accurately (-40.0dB to +40.0dB)
- ✅ "Reset All" returns all bands to 0dB
- ✅ Audio changes are audible and appropriate for each frequency band

#### Test Case 4.2: Spectrum Analyzer
**Objective**: Verify real-time spectrum analysis display.

**Steps**:
1. Navigate to Effects tab
2. Load audio file and start playback
3. Observe spectrum analyzer during different types of audio
4. Test different visualization modes (Linear, Logarithmic, Bars, Waveform)
5. Verify spectrum updates in real-time

**Expected Results**:
- ✅ Spectrum analyzer shows appropriate frequency content
- ✅ Visualization updates smoothly at ~30fps minimum
- ✅ Different modes display distinct visual representations
- ✅ Amplitude levels correspond to audio volume
- ✅ No visual glitches or performance issues

### 5. Signal Generator Testing

#### Test Case 5.1: Signal Generation
**Objective**: Test built-in signal generator functionality.

**Steps**:
1. Navigate to Generator tab
2. Test different waveform types (sine, square, sawtooth, triangle)
3. Test frequency range (20Hz to 20kHz)
4. Test amplitude controls
5. Generate test signals and verify audio output

**Expected Results**:
- ✅ All waveform types generate correctly
- ✅ Frequency can be set accurately across full range
- ✅ Generated signals are mathematically correct
- ✅ No artifacts or distortion in generated signals
- ✅ Signal generation doesn't interfere with file playback

### 6. Accessibility Testing

#### Test Case 6.1: Keyboard Navigation
**Objective**: Verify comprehensive keyboard accessibility.

**Steps**:
1. Use Tab key to navigate through all interactive elements
2. Test Enter/Space for button activation
3. Test arrow keys for slider and knob controls
4. Verify focus indicators are clearly visible
5. Test Escape key for dialog dismissal

**Expected Results**:
- ✅ All interactive elements are keyboard accessible
- ✅ Tab order is logical and intuitive
- ✅ Focus indicators are clearly visible with high contrast
- ✅ Keyboard shortcuts work consistently
- ✅ No elements are unreachable via keyboard

#### Test Case 6.2: High Contrast Mode
**Objective**: Test high contrast accessibility features.

**Steps**:
1. Enable high contrast mode (if available)
2. Navigate through all application areas
3. Test readability of all text elements
4. Verify button and control visibility
5. Test color-dependent information display

**Expected Results**:
- ✅ All text has sufficient contrast ratio (4.5:1 minimum)
- ✅ Interactive elements are clearly distinguishable
- ✅ No information is conveyed by color alone
- ✅ Focus indicators remain visible in high contrast mode

#### Test Case 6.3: Volume Safety Features
**Objective**: Verify hearing protection features.

**Steps**:
1. Gradually increase volume to test warning thresholds
2. Test emergency volume reduction (if available)
3. Verify volume level announcements
4. Test volume limits and safety overrides

**Expected Results**:
- ✅ Warning appears at 80% volume level
- ✅ Emergency volume reduction works when triggered
- ✅ Volume safety indicators are clearly visible
- ✅ Audio never exceeds safe levels without explicit user override

### 7. HiDPI and Scaling Testing

#### Test Case 7.1: Multi-DPI Testing
**Objective**: Verify application behavior across different DPI settings.

**Test Matrix**:
| DPI Setting | Scale Factor | Window Behavior | UI Scaling |
|-------------|--------------|-----------------|------------|
| 96 DPI | 100% | Test responsiveness | Verify clarity |
| 120 DPI | 125% | Test responsiveness | Verify clarity |
| 144 DPI | 150% | Test responsiveness | Verify clarity |
| 192 DPI | 200% | Test responsiveness | Verify clarity |

**Steps for each DPI setting**:
1. Change Windows display scaling
2. Restart application
3. Verify UI element sizing and clarity
4. Test all major functionality
5. Check for any visual artifacts

**Expected Results**:
- ✅ UI scales appropriately for each DPI setting
- ✅ Text remains crisp and readable
- ✅ Icons and graphics scale without artifacts
- ✅ Interaction areas maintain appropriate sizes
- ✅ No functionality is lost at any scale factor

#### Test Case 7.2: Multi-Monitor Testing
**Objective**: Test behavior across monitors with different DPI settings.

**Steps**:
1. Configure multi-monitor setup with different DPI settings
2. Move application window between monitors
3. Test full-screen mode on each monitor
4. Verify scaling adjustments during monitor transitions

**Expected Results**:
- ✅ Application adapts when moved between monitors
- ✅ Scaling adjusts appropriately for each monitor's DPI
- ✅ No visual glitches during monitor transitions
- ✅ Full-screen mode works correctly on all monitors

### 8. Performance and Stability Testing

#### Test Case 8.1: Extended Usage Testing
**Objective**: Verify application stability during extended use.

**Steps**:
1. Run application for 2+ hours continuously
2. Play multiple audio files in sequence
3. Switch between tabs frequently
4. Adjust controls regularly
5. Monitor memory usage and performance

**Expected Results**:
- ✅ No memory leaks or excessive memory usage
- ✅ Performance remains consistent over time
- ✅ No crashes or freezes during extended use
- ✅ Audio quality remains stable
- ✅ UI responsiveness doesn't degrade

#### Test Case 8.2: Stress Testing
**Objective**: Test application behavior under stress conditions.

**Steps**:
1. Load very large audio files (>100MB)
2. Rapidly adjust multiple EQ bands simultaneously
3. Switch tabs rapidly in succession
4. Test with high CPU load from other applications
5. Test with limited available memory

**Expected Results**:
- ✅ Application handles large files gracefully
- ✅ Rapid interactions don't cause instability
- ✅ Performance degrades gracefully under stress
- ✅ Error handling prevents crashes
- ✅ Recovery from stress conditions is automatic

### 9. Error Handling and Recovery Testing

#### Test Case 9.1: File Error Handling
**Objective**: Test handling of problematic audio files.

**Test Files**:
- Corrupted MP3 file
- Unsupported audio format
- Very large file (>500MB)
- File with unusual metadata
- Zero-byte file

**Steps**:
1. Attempt to load each problematic file
2. Observe error messages and recovery behavior
3. Verify application remains stable after errors
4. Test error dialog functionality

**Expected Results**:
- ✅ Clear, helpful error messages for each error type
- ✅ Application doesn't crash with invalid files
- ✅ Recovery options are provided where possible
- ✅ Error dialogs are dismissible and informative

#### Test Case 9.2: Permission and Access Testing
**Objective**: Test behavior with file permission issues.

**Steps**:
1. Attempt to load file from restricted directory
2. Test with file opened by another application
3. Test with read-only files
4. Test network file access scenarios

**Expected Results**:
- ✅ Permission errors are handled gracefully
- ✅ Helpful suggestions provided for permission issues
- ✅ Application doesn't crash due to access errors
- ✅ Alternative file selection options are offered

## Test Report Template

### Test Execution Summary

**Test Date**: _________
**Tester**: _________
**Application Version**: _________
**OS/Hardware**: _________

### Results Summary

| Test Category | Total Tests | Passed | Failed | Pass Rate |
|---------------|-------------|--------|--------|-----------|
| Launch & Navigation | | | | |
| Audio Functionality | | | | |
| Equalizer & Effects | | | | |
| Signal Generator | | | | |
| Accessibility | | | | |
| HiDPI & Scaling | | | | |
| Performance | | | | |
| Error Handling | | | | |
| **TOTAL** | | | | |

### Critical Issues Found

| Issue ID | Severity | Description | Steps to Reproduce | Workaround |
|----------|----------|-------------|-------------------|------------|
| | | | | |

### Performance Metrics

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Startup Time | <3s | | |
| Tab Switch Time | <100ms | | |
| Audio Load Time | <2s | | |
| Seek Response Time | <1s | | |
| Frame Rate (HiDPI) | 30fps | | |

### Recommendations

#### High Priority
- [ ] Issue 1
- [ ] Issue 2

#### Medium Priority
- [ ] Issue 3
- [ ] Issue 4

#### Low Priority
- [ ] Issue 5
- [ ] Issue 6

### Overall Assessment

**Car Stereo Interface**: ⭐⭐⭐⭐⭐ (Rate 1-5)
**HiDPI Support**: ⭐⭐⭐⭐⭐ (Rate 1-5)
**Accessibility**: ⭐⭐⭐⭐⭐ (Rate 1-5)
**Performance**: ⭐⭐⭐⭐⭐ (Rate 1-5)
**Overall Quality**: ⭐⭐⭐⭐⭐ (Rate 1-5)

**Ready for Release**: ✅ Yes / ❌ No

**Additional Comments**:
_________________________________________________
_________________________________________________

---

## Automated Test Integration

This manual testing should be complemented with the automated testing framework:

```bash
# Run comprehensive automated tests
cargo test --release

# Run UI-specific tests
cargo test ui_tests --release

# Run visual regression tests
cargo test visual_regression --release

# Run performance benchmarks
cargo bench
```

## Continuous Integration Checklist

- [ ] All manual test cases pass
- [ ] Automated test suite passes (>95% success rate)
- [ ] Visual regression tests pass (<2% difference)
- [ ] Performance benchmarks meet targets
- [ ] No critical accessibility issues
- [ ] HiDPI scaling verified on multiple devices
- [ ] Car stereo interface design approved
- [ ] Documentation updated

---

**Document Version**: 1.0
**Last Updated**: December 2024
**Next Review**: Quarterly or with major releases