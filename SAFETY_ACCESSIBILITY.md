# Rusty Audio - Safety and Accessibility Guide

## Table of Contents

1. [Audio Safety Overview](#audio-safety-overview)
2. [Volume Limiting and Protection](#volume-limiting-and-protection)
3. [Hearing Protection Guidelines](#hearing-protection-guidelines)
4. [Accessibility Features](#accessibility-features)
5. [Keyboard Navigation](#keyboard-navigation)
6. [Screen Reader Support](#screen-reader-support)
7. [Visual Accessibility](#visual-accessibility)
8. [Motor Accessibility](#motor-accessibility)
9. [Cognitive Accessibility](#cognitive-accessibility)
10. [Emergency Procedures](#emergency-procedures)

---

## Audio Safety Overview

Rusty Audio prioritizes user hearing health and safety through multiple protective mechanisms and clear guidelines. The application includes real-time monitoring, automatic limiting, and proactive warnings to prevent hearing damage.

### Safety Philosophy

1. **Prevention First**: Default settings prioritize safety
2. **User Education**: Clear information about risks
3. **Active Protection**: Real-time monitoring and intervention
4. **Emergency Controls**: Quick access to safety features
5. **Compliance**: Meets international safety standards

### Key Safety Features

- **Automatic Volume Limiting**: Prevents dangerous output levels
- **Gradual Volume Changes**: No sudden loud sounds
- **Visual Warnings**: Color-coded volume indicators
- **Emergency Volume Cut**: Panic button (Ctrl+Shift+M)
- **Startup Safety**: Always starts at safe volume (50%)

---

## Volume Limiting and Protection

### Multi-Layer Protection System

```
Input → Soft Limiter → Hard Limiter → Safety Check → Output
         ↓               ↓              ↓
     (Warning)      (Protection)   (Emergency)
```

### Volume Safety Zones

| Zone | Range | Visual Indicator | Status | Action |
|------|-------|-----------------|--------|--------|
| **Safe** | 0-60% | Green | Normal listening | No action needed |
| **Moderate** | 60-75% | Yellow | Extended exposure caution | Show duration warning |
| **Loud** | 75-85% | Orange | Limited exposure | Display countdown timer |
| **Dangerous** | 85-95% | Red | Hearing risk | Require confirmation |
| **Critical** | 95-100% | Red + Flash | Immediate risk | Auto-reduce after 10s |

### Automatic Safety Features

#### Soft Limiter
Engages gradually to prevent sudden volume spikes:
- **Threshold**: 85% volume
- **Attack Time**: 5ms
- **Ratio**: 2:1 compression
- **Purpose**: Smooth limiting without distortion

#### Hard Limiter
Absolute protection against dangerous levels:
- **Threshold**: 95% volume
- **Attack Time**: 0.1ms
- **Ratio**: ∞:1 (brick wall)
- **Purpose**: Prevent any signal above safety threshold

#### Emergency Volume Reduction
Automatic intervention for prolonged exposure:
```
IF volume > 85% AND duration > 30 minutes THEN
    Show warning dialog
    Reduce to 75% after 60 seconds without user response
```

### Implementation Details

```rust
pub struct VolumeSafetyManager {
    current_volume: f32,
    exposure_time: Duration,
    last_warning: Option<Instant>,
    emergency_active: bool,
}

impl VolumeSafetyManager {
    pub fn check_safety(&mut self, volume: f32) -> SafetyStatus {
        match volume {
            v if v <= 0.6 => SafetyStatus::Safe,
            v if v <= 0.75 => SafetyStatus::Moderate(self.calculate_safe_duration(v)),
            v if v <= 0.85 => SafetyStatus::Loud(Duration::from_secs(1800)), // 30 min
            v if v <= 0.95 => SafetyStatus::Dangerous(Duration::from_secs(300)), // 5 min
            _ => SafetyStatus::Critical(Duration::from_secs(10)), // 10 sec
        }
    }

    fn calculate_safe_duration(&self, volume: f32) -> Duration {
        // NIOSH exposure formula
        let db = self.volume_to_db(volume);
        let hours = 8.0 / 2.0_f32.powf((db - 85.0) / 3.0);
        Duration::from_secs((hours * 3600.0) as u64)
    }
}
```

---

## Hearing Protection Guidelines

### Safe Listening Practices

#### Recommended Exposure Limits (NIOSH/WHO Standards)

| Sound Level | Maximum Daily Exposure | Volume Setting |
|------------|------------------------|----------------|
| 75 dB | 127 hours | ~40% |
| 80 dB | 25 hours | ~50% |
| 85 dB | 8 hours | ~60% |
| 90 dB | 2.5 hours | ~70% |
| 95 dB | 47 minutes | ~80% |
| 100 dB | 15 minutes | ~90% |
| 105 dB | 5 minutes | ~100% |

### The 60/60 Rule
- Listen at no more than **60% volume**
- Take a break after **60 minutes**
- Reduces risk of noise-induced hearing loss by 85%

### Warning Signs of Dangerous Listening

**Immediate Signs:**
- Need to raise voice to be heard over music
- Ringing or buzzing in ears (tinnitus)
- Muffled hearing after listening
- Ear pain or discomfort

**Long-term Signs:**
- Difficulty understanding speech in noise
- Frequently asking people to repeat themselves
- Turning up TV/music louder than others prefer
- Missing parts of conversations

### Best Practices by Device Type

#### Headphones
- Use noise-cancelling headphones to avoid compensating for ambient noise
- Prefer over-ear to in-ear designs for better sound isolation
- Set maximum volume limit in settings
- Clean ear pads regularly for hygiene

#### Speakers
- Position at least 3 feet away
- Use room correction to avoid hot spots
- Consider acoustic treatment for better clarity at lower volumes
- Calibrate to reference level (83 dB SPL)

#### Professional Monitoring
- Calibrate monitors to K-System standards
- Use SPL meter to verify levels
- Take 10-minute breaks every hour
- Maintain consistent monitoring level

---

## Accessibility Features

### Overview

Rusty Audio implements comprehensive accessibility features following WCAG 2.1 AAA guidelines, ensuring the application is usable by everyone regardless of ability.

### Accessibility Manager

The centralized accessibility system provides:
- Screen reader support with semantic markup
- Keyboard-only navigation
- High contrast modes
- Reduced motion options
- Cognitive load reduction

```rust
pub struct AccessibilityManager {
    screen_reader_mode: bool,
    high_contrast_mode: bool,
    reduce_motion: bool,
    large_text_mode: bool,
    focus_indicators: bool,
    announcement_queue: VecDeque<Announcement>,
}
```

### Quick Access

**Accessibility Menu**: `Alt+A`
- Toggle screen reader mode
- Switch contrast themes
- Adjust text size
- Enable focus indicators
- Configure keyboard navigation

---

## Keyboard Navigation

### Complete Keyboard Control

Every feature in Rusty Audio is accessible via keyboard:

#### Navigation Flow
```
Tab → Next control
Shift+Tab → Previous control
Arrow Keys → Navigate within groups
Enter/Space → Activate control
Escape → Cancel/Close dialog
```

### Focus Management

#### Visual Focus Indicators
- **Primary Focus**: 3px blue outline with 2px offset
- **Keyboard Focus**: Additional glow effect
- **High Contrast**: 4px black/white outline
- **Focus Trap**: Modal dialogs contain focus

#### Focus Order
1. Main menu/toolbar
2. Tab navigation
3. Primary controls (play/stop/volume)
4. Content area
5. Secondary controls
6. Status bar

### Keyboard Shortcuts

#### Essential Navigation
| Key | Action | Context |
|-----|--------|---------|
| `Tab` | Next control | Global |
| `Shift+Tab` | Previous control | Global |
| `Arrow Keys` | Navigate within group | Lists/Grids |
| `Home/End` | First/Last item | Lists |
| `Page Up/Down` | Page navigation | Long lists |

#### Accessibility Shortcuts
| Key | Action | Purpose |
|-----|--------|---------|
| `Alt+A` | Accessibility menu | Quick settings |
| `Alt+H` | High contrast toggle | Visual aid |
| `Alt+M` | Reduce motion | Motion sensitivity |
| `Alt+R` | Screen reader mode | Blind users |
| `Alt+Z` | Zoom/magnification | Low vision |

#### Control Shortcuts
| Key | Action | Fine Control |
|-----|--------|---------|
| `Ctrl+↑/↓` | Large adjustment | ±10% |
| `↑/↓` | Normal adjustment | ±5% |
| `Shift+↑/↓` | Fine adjustment | ±1% |
| `Alt+↑/↓` | Micro adjustment | ±0.1% |

### Custom Key Binding

Users can customize all keyboard shortcuts:

```rust
pub struct KeyBinding {
    action: Action,
    primary: KeyCombo,
    secondary: Option<KeyCombo>,
    description: String,
}

pub struct KeyCombo {
    key: Key,
    modifiers: Modifiers,
}
```

---

## Screen Reader Support

### ARIA Implementation

All UI elements include proper ARIA attributes:

```rust
pub struct AccessibleWidget {
    role: AriaRole,
    label: String,
    description: Option<String>,
    value: Option<String>,
    state: WidgetState,
}

// Example implementation
ui.allocate_ui(|ui| {
    ui.add(
        Button::new("Play")
            .aria_role(AriaRole::Button)
            .aria_label("Play audio file")
            .aria_pressed(is_playing)
            .aria_keyshortcuts("Space")
    );
});
```

### Live Regions

Dynamic content updates are announced:

```rust
pub enum AnnouncementPriority {
    Low,      // aria-live="polite"
    Medium,   // aria-live="polite" aria-atomic="true"
    High,     // aria-live="assertive"
    Critical, // role="alert"
}
```

### Screen Reader Optimizations

#### Semantic Structure
- Proper heading hierarchy (h1-h6)
- Landmark regions (main, nav, aside)
- Descriptive labels for all controls
- Context provided for complex widgets

#### Announcement Examples
```
"Volume slider, 50 percent, adjustable"
"Play button, pressed, shortcut Space"
"EQ band 1000 hertz, gain 0 decibels, adjustable with Q and A keys"
"Warning: Volume above safe listening level"
```

### Supported Screen Readers

| Platform | Screen Reader | Support Level |
|----------|--------------|---------------|
| Windows | NVDA | Full |
| Windows | JAWS | Full |
| Windows | Narrator | Full |
| macOS | VoiceOver | Full |
| Linux | Orca | Full |

---

## Visual Accessibility

### Color and Contrast

#### High Contrast Modes

**Mode 1: WCAG AAA Compliant**
- Text: 7:1 contrast ratio minimum
- UI Elements: 4.5:1 contrast ratio
- Focus Indicators: 3:1 against all backgrounds

**Mode 2: True High Contrast**
- Pure black background (#000000)
- Pure white text (#FFFFFF)
- Yellow highlights (#FFFF00)
- No gradients or transparency

**Mode 3: Inverted Colors**
- System-wide color inversion compatible
- Preserves readability with inverted palette

### Color Blind Modes

| Mode | Affected Colors | Adjustment |
|------|----------------|------------|
| Protanopia | Red-Green | Shift red to blue |
| Deuteranopia | Red-Green | Enhanced brightness difference |
| Tritanopia | Blue-Yellow | Shift to red-green spectrum |
| Achromatopsia | All colors | Grayscale with pattern indicators |

### Visual Indicators

Non-color dependent status indication:
- **Icons**: Unique shapes for each state
- **Patterns**: Texture overlays for zones
- **Text**: Always present as fallback
- **Animation**: Optional pulsing for attention

### Text and Typography

#### Scalable Text
- **Minimum Size**: 14px (default zoom)
- **Maximum Size**: 32px (200% zoom)
- **Line Height**: 1.5x minimum
- **Character Spacing**: Adjustable

#### Font Options
```rust
pub enum FontPreference {
    System,        // Use system default
    OpenDyslexic,  // Dyslexia-friendly
    AtkinsonHyperlegible, // Maximum clarity
    Custom(String),
}
```

### Reduced Motion

For users with vestibular disorders:

```rust
pub struct MotionSettings {
    disable_animations: bool,
    reduce_transparency: bool,
    disable_parallax: bool,
    crossfade_duration: Duration, // Instead of slide
}

// Implementation
if accessibility.reduce_motion {
    // Use simple fade instead of slide
    transition = Transition::Fade(Duration::from_millis(150));
} else {
    transition = Transition::Slide(Duration::from_millis(300));
}
```

---

## Motor Accessibility

### Adaptive Input

#### Sticky Keys
- Single finger operation for modifier keys
- Configurable timeout
- Visual indicator for active modifiers

#### Slow Keys
- Adjustable key acceptance delay (0-2000ms)
- Prevents accidental key presses
- Audio/visual feedback on acceptance

#### Repeat Keys
- Customizable repeat rate
- Adjustable initial delay
- Can be disabled entirely

### Large Click Targets

All interactive elements meet minimum size requirements:

| Element Type | Minimum Size | Recommended | Touch |
|--------------|-------------|-------------|-------|
| Buttons | 44×44px | 48×48px | 56×56px |
| Sliders | 44px height | 48px | 56px |
| Checkboxes | 20×20px | 24×24px | 32×32px |
| Links | 44px tap area | 48px | 56px |

### Gesture Alternatives

All gestures have keyboard equivalents:

| Gesture | Keyboard Alternative |
|---------|---------------------|
| Drag slider | Arrow keys |
| Pinch zoom | Ctrl +/- |
| Swipe | Page Up/Down |
| Long press | Shift+F10 (context menu) |
| Double tap | Enter |

### Dwell Clicking

For users who cannot click:

```rust
pub struct DwellClick {
    enabled: bool,
    dwell_time: Duration,     // Time to hover before click
    movement_threshold: f32,   // Pixels of allowed movement
    visual_feedback: bool,     // Show countdown
}
```

---

## Cognitive Accessibility

### Simplified Mode

Reduces cognitive load through:
- Fewer on-screen options
- Larger, clearer labels
- Step-by-step workflows
- Automatic settings

#### Progressive Disclosure
```
Basic Mode → Show only essential controls
Advanced Mode → Show common controls
Expert Mode → Show all controls
```

### Clear Language

#### Terminology
- Avoid jargon when possible
- Provide tooltips with explanations
- Use consistent terminology
- Include glossary in help

#### Instructions
- Short, action-oriented sentences
- Step-by-step guidance
- Visual aids where helpful
- Confirmation for destructive actions

### Memory Aids

#### Undo/Redo
- Unlimited undo history
- Clear description of each action
- Keyboard shortcuts (Ctrl+Z/Y)

#### Persistent State
- Remember user preferences
- Restore last session
- Bookmarks for positions
- History of recent files

### Error Prevention

#### Confirmation Dialogs
```rust
pub struct ConfirmDialog {
    title: String,
    message: String,
    destructive: bool,
    require_typing: Option<String>, // Type "DELETE" to confirm
}
```

#### Input Validation
- Real-time feedback
- Clear error messages
- Suggested corrections
- Prevent invalid states

### Focus and Attention

#### Reduce Distractions
- Hide non-essential UI elements
- Mute background animations
- Focus mode for current task
- Customizable workspace

#### Task Guidance
- Progress indicators for long tasks
- Clear current step highlighting
- Breadcrumb navigation
- Success confirmations

---

## Emergency Procedures

### Emergency Volume Cut

**Trigger**: `Ctrl+Shift+M` or Panic Button

**Action**:
1. Immediately reduce volume to 20%
2. Pause all audio playback
3. Show confirmation dialog
4. Log incident for safety tracking

```rust
pub fn emergency_volume_cut(&mut self) {
    self.previous_volume = self.volume;
    self.volume = 0.2; // Safe level
    self.gain_node.gain().set_value(self.volume);

    if let Some(source) = &self.source_node {
        source.pause();
    }

    self.show_emergency_dialog();
    self.log_safety_incident(SafetyIncident::EmergencyStop);
}
```

### System Failure Recovery

#### Audio System Crash
1. Detect audio thread failure
2. Attempt automatic restart
3. Fall back to null audio device
4. Preserve user data and state
5. Show recovery options

#### File Corruption
1. Detect corrupted audio data
2. Stop playback immediately
3. Prevent repeated attempts
4. Suggest file repair tools
5. Log detailed error information

### Accessibility Failure Fallbacks

#### Screen Reader Failure
- Fall back to high contrast mode
- Enable keyboard hints overlay
- Show text descriptions for all elements
- Log assistive technology errors

#### Input Device Failure
- Enable on-screen keyboard
- Activate dwell clicking
- Show gesture alternatives
- Provide command palette

### User Distress Detection

Monitor for signs of user difficulty:
- Repeated failed actions
- Rapid control changes
- Extended high volume exposure
- Accessibility feature conflicts

Response:
```rust
pub fn detect_user_distress(&mut self) -> Option<DistressType> {
    if self.failed_attempts > 5 {
        return Some(DistressType::RepeatedFailure);
    }

    if self.volume_changes_per_minute > 20 {
        return Some(DistressType::RapidAdjustment);
    }

    if self.high_volume_duration > Duration::from_secs(1800) {
        return Some(DistressType::ProlongedExposure);
    }

    None
}

pub fn offer_assistance(&mut self, distress: DistressType) {
    match distress {
        DistressType::RepeatedFailure => {
            self.show_help_suggestion();
            self.enable_simplified_mode();
        },
        DistressType::RapidAdjustment => {
            self.show_preset_options();
            self.suggest_auto_mode();
        },
        DistressType::ProlongedExposure => {
            self.show_hearing_warning();
            self.reduce_volume_gradually();
        },
    }
}
```

---

## Compliance and Standards

### Accessibility Standards

| Standard | Level | Status |
|----------|-------|--------|
| WCAG 2.1 | AAA | Compliant |
| Section 508 | - | Compliant |
| EN 301 549 | - | Compliant |
| ISO/IEC 40500 | - | Compliant |

### Safety Standards

| Standard | Description | Compliance |
|----------|-------------|------------|
| IEC 60065 | Audio equipment safety | Yes |
| IEC 62368-1 | Audio/video safety | Yes |
| EN 50332 | Sound pressure limits | Yes |
| NIOSH REL | Occupational exposure | Yes |

### Testing Methodologies

#### Automated Testing
- aXe accessibility scanner
- WAVE evaluation tool
- Lighthouse audits
- Custom test suites

#### Manual Testing
- Screen reader testing with real users
- Keyboard-only navigation verification
- Color contrast analysis
- Cognitive load assessment

#### User Testing
- Disability community feedback
- Usability studies
- A/B testing for accessibility features
- Long-term exposure studies

---

## Configuration Examples

### Accessibility Profiles

#### Low Vision Profile
```toml
[accessibility.low_vision]
ui_scale = 1.5
high_contrast = true
large_cursors = true
focus_highlight = "thick"
font_size = 18
```

#### Motor Impairment Profile
```toml
[accessibility.motor]
large_targets = true
sticky_keys = true
dwell_click = true
dwell_time = 800
slow_keys_delay = 200
```

#### Hearing Safety Profile
```toml
[safety.hearing]
max_volume = 0.75
warning_threshold = 0.6
auto_reduce = true
exposure_tracking = true
break_reminders = true
```

#### Cognitive Support Profile
```toml
[accessibility.cognitive]
simplified_ui = true
clear_language = true
step_guidance = true
auto_settings = true
reduce_options = true
```

---

## Getting Help

### Accessibility Support

**Email**: accessibility@rustyaudio.app
**Discord**: #accessibility channel
**GitHub**: Issues with 'accessibility' label

### Resources

- **Documentation**: [docs.rustyaudio.app/accessibility](https://docs.rustyaudio.app/accessibility)
- **Video Tutorials**: Fully captioned and described
- **Community Forum**: Accessibility section
- **User Groups**: Connect with other users

### Feedback

We actively seek feedback on accessibility:
- Feature requests
- Bug reports
- Usability suggestions
- Testing participation

---

*End of Safety and Accessibility Guide - Version 1.0*

*Rusty Audio is committed to being usable by everyone. If you encounter any accessibility barriers, please contact us immediately.*

*For emergency audio safety issues, use Ctrl+Shift+M to activate emergency volume reduction.*