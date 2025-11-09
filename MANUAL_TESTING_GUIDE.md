# Manual Testing Guide - Phase 4.1 Audio Recording

## Overview

This document provides step-by-step instructions for manually testing the complete audio recording workflow in rusty-audio. These tests require actual audio hardware (microphone) and should be performed after the automated UI tests pass.

## Prerequisites

### Hardware Requirements
- Microphone or audio input device
- Audio output device (headphones/speakers) for monitoring
- Windows PC with audio drivers installed

### Software Requirements
- rusty-audio binary compiled: `cargo build --release`
- Audio device drivers properly installed
- Microphone permissions granted to the application

## Test Suite

### Test 1: Device Enumeration

**Objective:** Verify that the application can detect and list available audio input devices.

**Steps:**
1. Launch rusty-audio: `cargo run --release`
2. Navigate to the Recording panel (🎙️ icon or tab)
3. Locate the "🎤 Input Device" section
4. Click the device dropdown

**Expected Results:**
- ✓ Dropdown shows list of available input devices
- ✓ Default input device is marked with "🎤 (Default)"
- ✓ Non-default devices shown with "🎵" icon
- ✓ If no devices available, shows "⚠️ No input devices found"

**Pass/Fail:** _________

**Notes:**
```
Number of devices detected: ________
Default device name: ________________
```

---

### Test 2: Device Selection

**Objective:** Verify device selection updates the recorder connection.

**Steps:**
1. From Test 1, with device dropdown open
2. Select a non-default input device
3. Observe the status message below the dropdown

**Expected Results:**
- ✓ Selected device name appears in dropdown
- ✓ Status changes to "✓ Device connected and ready" (green)
- ✓ No errors logged to console

**Pass/Fail:** _________

**Notes:**
```
Selected device: ________________
```

---

### Test 3: Recording State Transitions

**Objective:** Test the complete recording state machine.

**Steps:**
1. With device selected, observe initial state shows "⚪ Idle"
2. Click "⏺ Record" button
3. Speak into microphone for 5 seconds
4. Click "⏸ Pause" button
5. Wait 3 seconds (silent)
6. Click "▶ Resume" button  
7. Speak for 3 more seconds
8. Click "⏹ Stop" button

**Expected Results:**
- ✓ State transitions: Idle → Recording → Paused → Recording → Stopped
- ✓ Record button turns red (🔴) when recording
- ✓ Status text updates correctly for each state
- ✓ Duration timer increments only when recording (not when paused)
- ✓ "Recorded" buffer duration increases appropriately

**Pass/Fail:** _________

**Notes:**
```
Final duration: ________________
Final buffer size: ________________
```

---

### Test 4: Level Meters

**Objective:** Verify real-time audio level metering.

**Steps:**
1. Start recording (⏺ Record)
2. Remain silent for 2 seconds - observe meters
3. Speak at normal volume for 3 seconds - observe meters
4. Speak very loudly (not shouting) for 2 seconds - observe meters
5. Stop recording (⏹ Stop)

**Expected Results:**
- ✓ During silence: meters show minimal activity (-60 dB or lower)
- ✓ During normal speech: meters show green zone (-18 to -6 dB)
- ✓ During loud speech: meters show yellow/red zone (-6 to 0 dB)
- ✓ Peak indicators respond faster than RMS indicators
- ✓ Level meters show activity on correct channel (Ch 1 for mono, Ch 1 & Ch 2 for stereo)
- ✓ Meters decay smoothly after audio stops

**Pass/Fail:** _________

**Notes:**
```
Peak levels observed: ________________
RMS levels observed: ________________
```

---

### Test 5: Clip Detection

**Objective:** Test clipping indicator functionality.

**Steps:**
1. Start recording
2. Speak VERY LOUDLY or tap/blow on microphone to cause clipping
3. Observe the right edge of level meters for red clip indicator
4. Click "Clear Clips" button

**Expected Results:**
- ✓ Red clip indicator appears when audio exceeds ~0.99 amplitude
- ✓ Clip indicator persists until "Clear Clips" is clicked
- ✓ "Clear Clips" button successfully resets all clip indicators

**Pass/Fail:** _________

**Notes:**
```
Clip detected: Yes / No
```

---

### Test 6: Monitoring Modes

**Objective:** Test different monitoring configurations.

**Steps:**
1. Start recording
2. Set monitoring mode to "🔇 Off" - speak into mic
3. Set monitoring mode to "⚡ Direct" - speak into mic
4. Adjust gain slider to 50% - speak into mic
5. Set monitoring mode to "🎛️ Routed" - speak into mic
6. Set monitoring mode back to "🔇 Off"
7. Stop recording

**Expected Results:**
- ✓ Off mode: No audio heard in headphones/speakers
- ✓ Direct mode: Audio heard with minimal latency
- ✓ Gain adjustment affects monitoring volume appropriately
- ✓ Routed mode: Audio heard (may have higher latency)
- ✓ Gain slider shows percentage (0-100%)
- ✓ Description text updates for each mode

**Pass/Fail:** _________

**Notes:**
```
Direct mode latency: ________________
Routed mode latency: ________________
```

---

### Test 7: WAV Export

**Objective:** Verify recording can be saved and played back.

**Steps:**
1. Record 10 seconds of speech (counting 1 to 10)
2. Stop recording
3. Under "💾 File Management", select format "WAV (32-bit float)"
4. Click "💾 Save Recording..." button
5. Save file as `test-recording.wav` in Downloads folder
6. Open saved file in external audio player (Windows Media Player, VLC, Audacity)
7. Play back recording

**Expected Results:**
- ✓ Save button is enabled after recording
- ✓ File dialog opens successfully
- ✓ File saves without errors
- ✓ File size is reasonable (~1.7 MB for 10 sec stereo @ 48kHz)
- ✓ Playback in external player matches recorded audio
- ✓ Audio quality is good (no distortion, noise, or artifacts)
- ✓ Complete recording captured (can hear counting 1-10)

**Pass/Fail:** _________

**Notes:**
```
File size: ________________
Playback quality: ________________
```

---

### Test 8: Buffer Management

**Objective:** Test buffer clear and duration tracking.

**Steps:**
1. Record 5 seconds of audio
2. Note the "Recorded: X.Xs" buffer duration
3. Click "🗑️ Clear Buffer" button
4. Observe buffer duration

**Expected Results:**
- ✓ Buffer duration resets to "Recorded: 0.0s"
- ✓ Level meters drop to zero
- ✓ Previous audio data is cleared
- ✓ Can start new recording after clearing

**Pass/Fail:** _________

---

### Test 9: Duration Tracking

**Objective:** Verify accurate duration calculation with pauses.

**Steps:**
1. Start recording, speak for 5 seconds
2. Pause recording, wait 10 seconds (silent)
3. Resume recording, speak for 5 more seconds
4. Stop recording
5. Note both "Duration" and "Recorded" times

**Expected Results:**
- ✓ Duration excludes pause time (~10 seconds total)
- ✓ Recorded buffer time matches duration (~10 seconds)
- ✓ Timer pauses when recording is paused
- ✓ Timer resumes counting when recording resumes

**Pass/Fail:** _________

**Notes:**
```
Expected duration: ~10 seconds
Actual duration: ________________
Recorded buffer: ________________
```

---

### Test 10: Long Recording Stress Test

**Objective:** Test stability during extended recording.

**Steps:**
1. Select input device
2. Start recording
3. Speak intermittently for 60 seconds (count to 60)
4. Monitor level meters throughout
5. Stop and save recording as `long-test.wav`
6. Play back to verify complete recording

**Expected Results:**
- ✓ Application remains responsive throughout 60-second recording
- ✓ Level meters continue to update smoothly
- ✓ No audio dropouts or glitches
- ✓ Complete 60 seconds captured in file
- ✓ Memory usage remains stable (check Task Manager)
- ✓ No crashes or freezes

**Pass/Fail:** _________

**Notes:**
```
Memory at start: ________________
Memory at end: ________________
Issues observed: ________________
```

---

## Summary Report

### Test Results

| Test | Pass | Fail | Notes |
|------|------|------|-------|
| 1. Device Enumeration | ☐ | ☐ | |
| 2. Device Selection | ☐ | ☐ | |
| 3. State Transitions | ☐ | ☐ | |
| 4. Level Meters | ☐ | ☐ | |
| 5. Clip Detection | ☐ | ☐ | |
| 6. Monitoring Modes | ☐ | ☐ | |
| 7. WAV Export | ☐ | ☐ | |
| 8. Buffer Management | ☐ | ☐ | |
| 9. Duration Tracking | ☐ | ☐ | |
| 10. Long Recording | ☐ | ☐ | |

**Total Passed:** _____ / 10

**Overall Assessment:** ☐ Ready for Production  ☐ Needs Fixes  ☐ Major Issues

---

## Known Issues & Workarounds

_(Document any issues found during testing)_

1. Issue:
   - Workaround:

2. Issue:
   - Workaround:

---

## Environment Information

**Test Date:** ______________

**Tester Name:** ______________

**System Configuration:**
- OS: Windows _____ (Build _______)
- CPU: ______________
- RAM: ______________
- Audio Interface: ______________
- Microphone Model: ______________
- rusty-audio Version: ______________
- Commit Hash: ______________

---

## Automated Test Status

Before performing manual tests, verify automated tests pass:

```bash
cargo test --test egui_kittest_tests
```

**Automated Tests:** ☐ Passing (23/23)  ☐ Failing

---

## Next Steps

After completing this manual testing:

1. ☐ Document all issues in GitHub Issues
2. ☐ Create bug reports for failures
3. ☐ Update CHANGELOG.md with test results  
4. ☐ Proceed to Phase 5 (MIDI support) if all tests pass
5. ☐ Fix critical issues before production release

---

## Additional Notes

_(Any additional observations or comments)_
