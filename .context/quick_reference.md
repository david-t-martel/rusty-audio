# Rusty Audio - Quick Reference

## 🚨 Critical Safety Features
- **Emergency Volume Reduction**: Press `Escape` key
- **Volume Limits**: Max 85dB SPL equivalent
- **Gradual Changes**: 100ms transitions
- **Safety Indicators**: Color-coded warnings

## 📁 Key Files
- **Accessibility**: `src/ui/accessibility.rs`
- **Safety System**: `src/ui/volume_safety.rs`
- **Signal Generator**: `src/ui/signal_generator.rs`
- **Error Handling**: `src/ui/error_handling.rs`
- **Enhanced Controls**: `src/ui/enhanced_controls.rs`

## 🛠️ Build Commands
```bash
cargo build --release           # Production build
cargo run                       # Development run
cargo test --all-features       # Run all tests
cargo test accessibility        # Test accessibility
```

## ⚠️ Before Making Changes
1. **Check existing files** - NO duplicate/enhanced versions
2. **Test accessibility** - Screen reader must work
3. **Verify safety** - Volume limits must be enforced
4. **Run tests** - All must pass before commit

## 🎯 Current Focus
- ✅ Accessibility framework (COMPLETE)
- ✅ Safety systems (COMPLETE)
- ✅ Signal generator (COMPLETE)
- ✅ Error handling (COMPLETE)
- 🚧 Documentation polish
- 🚧 Warning cleanup

## 🔑 Key Patterns
```rust
// Accessibility
impl AccessibleControl for Widget {
    fn aria_label(&self) -> String
    fn keyboard_handler(&mut self, key: Key) -> bool
}

// Safety
fn set_volume(&mut self, value: f32) {
    let safe = self.safety_check(value);
    self.gradual_change(safe);
}

// Error Recovery
AudioError::DeviceNotFound {
    recovery: RecoveryAction::SelectDevice
}
```

## 📊 Performance Targets
- Rendering: 60+ FPS
- Audio Latency: <10ms
- Memory: ~50MB baseline
- CPU: <15% active

## 🚫 Never Do This
- Create `enhanced_*.rs` files
- Create `simple_*.rs` files
- Create `*_v2.rs` files
- Skip safety checks
- Ignore accessibility
- Direct volume sets without safety

## ✅ Always Do This
- Use `AccessibilityManager`
- Include ARIA labels
- Test with screen reader
- Implement safety checks
- Provide error recovery
- Update existing files

## 🎵 Signal Types Available
1. Sine Wave
2. Square Wave
3. Triangle Wave
4. Sawtooth Wave
5. White Noise
6. Pink Noise
7. Chirp
8. Multi-tone

## 🔊 Volume Safety Levels
- 🟢 0-50%: Safe (Green)
- 🟡 50-70%: Moderate (Yellow)
- 🟠 70-85%: Caution (Orange)
- 🔴 85-100%: Warning (Red)

## 📝 Documentation Structure
```
documentation/
├── USER_GUIDE.md         # End-user guide
├── ACCESSIBILITY_GUIDE.md # A11y features
├── DEVELOPER_GUIDE.md    # Dev reference
└── API_REFERENCE.md      # API docs
```

## 🎮 Keyboard Shortcuts
- `Escape`: Emergency volume reduction
- `Tab`: Navigate controls
- `Space`: Play/Pause
- `Arrow Keys`: Adjust values
- `F1`: Help
- `Alt+H`: High contrast mode

## 🔄 Agent Workflow
1. **Review**: architect-reviewer identifies issues
2. **Design**: ui-ux-designer creates specifications
3. **Build**: frontend-developer implements
4. **Document**: docs-architect creates guides
5. **Test**: All safety and accessibility verified

---
*Remember: Safety First, Accessibility Always, No Duplication Ever*