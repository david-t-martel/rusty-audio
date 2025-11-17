// tests/theme_system_tests.rs
//! Comprehensive tests for theme system and color calculations

use rusty_audio::ui::theme::{AppTheme, ThemeColors};

// ============================================================================
// THEME ENUMERATION TESTS
// ============================================================================

#[test]
fn test_all_themes_available() {
    let themes = AppTheme::all();
    assert_eq!(themes.len(), 7, "Should have exactly 7 themes");

    let expected = vec![
        AppTheme::Dark,
        AppTheme::Light,
        AppTheme::Mocha,
        AppTheme::Macchiato,
        AppTheme::Frappe,
        AppTheme::Latte,
        AppTheme::HighContrast,
    ];

    for (i, expected_theme) in expected.iter().enumerate() {
        assert_eq!(&themes[i], expected_theme, "Theme at index {} mismatch", i);
    }
}

#[test]
fn test_theme_display_names() {
    let test_cases = vec![
        (AppTheme::Dark, "Dark"),
        (AppTheme::Light, "Light"),
        (AppTheme::Mocha, "Mocha"),
        (AppTheme::Macchiato, "Macchiato"),
        (AppTheme::Frappe, "Frappe"),
        (AppTheme::Latte, "Latte"),
        (AppTheme::HighContrast, "High Contrast"),
    ];

    for (theme, expected_name) in test_cases {
        assert_eq!(theme.to_string(), expected_name);
    }
}

// ============================================================================
// THEME COLOR TESTS
// ============================================================================

#[test]
fn test_dark_theme_colors() {
    let theme = AppTheme::Dark;
    let colors = theme.colors();

    // Dark theme should have dark background
    assert!(is_dark_color(colors.bg_primary), "Background should be dark");

    // Text should be light on dark background
    assert!(is_light_color(colors.text_primary), "Text should be light");

    // Verify contrast ratio
    let contrast = calculate_contrast_ratio(colors.bg_primary, colors.text_primary);
    assert!(contrast > 4.5, "Dark theme contrast ratio should be > 4.5, got {}", contrast);
}

#[test]
fn test_light_theme_colors() {
    let theme = AppTheme::Light;
    let colors = theme.colors();

    // Light theme should have light background
    assert!(is_light_color(colors.bg_primary), "Background should be light");

    // Text should be dark on light background
    assert!(is_dark_color(colors.text_primary), "Text should be dark");

    // Verify contrast ratio
    let contrast = calculate_contrast_ratio(colors.bg_primary, colors.text_primary);
    assert!(contrast > 4.5, "Light theme contrast ratio should be > 4.5, got {}", contrast);
}

#[test]
fn test_high_contrast_theme() {
    let theme = AppTheme::HighContrast;
    let colors = theme.colors();

    // High contrast should have maximum contrast ratio
    let contrast = calculate_contrast_ratio(colors.bg_primary, colors.text_primary);
    assert!(
        contrast > 12.0,
        "High contrast theme should have ratio > 12.0, got {}",
        contrast
    );

    // Should use pure black and white
    assert!(
        colors.bg_primary == [0, 0, 0] || colors.bg_primary == [255, 255, 255],
        "High contrast background should be pure black or white"
    );
}

#[test]
fn test_accent_colors_distinct() {
    for theme in AppTheme::all() {
        let colors = theme.colors();

        // Accent color should be distinct from background
        let bg_contrast = calculate_contrast_ratio(colors.bg_primary, colors.accent);
        assert!(
            bg_contrast > 3.0,
            "{} theme: accent should contrast with background (ratio: {})",
            theme,
            bg_contrast
        );

        // Accent color should be distinct from text
        let text_contrast = calculate_contrast_ratio(colors.text_primary, colors.accent);
        assert!(
            text_contrast > 2.0,
            "{} theme: accent should be distinct from text (ratio: {})",
            theme,
            text_contrast
        );
    }
}

// ============================================================================
// SPECTRUM GRADIENT TESTS
// ============================================================================

#[test]
fn test_spectrum_gradient_smooth() {
    for theme in AppTheme::all() {
        let colors = theme.colors();

        // Should have at least 3 gradient colors (low, mid, high)
        assert!(
            colors.spectrum_low != colors.spectrum_mid,
            "{} theme: low and mid spectrum colors should differ",
            theme
        );
        assert!(
            colors.spectrum_mid != colors.spectrum_high,
            "{} theme: mid and high spectrum colors should differ",
            theme
        );
    }
}

#[test]
fn test_spectrum_gradient_blue_to_red() {
    // Default gradient should go blue → cyan → red
    let theme = AppTheme::Dark;
    let colors = theme.colors();

    // Low should be bluish
    assert!(
        colors.spectrum_low[2] > colors.spectrum_low[0],
        "Low spectrum should be bluish (B > R)"
    );

    // Mid should be cyan-ish (blue + green)
    assert!(
        colors.spectrum_mid[1] > 100 && colors.spectrum_mid[2] > 100,
        "Mid spectrum should be cyan-ish"
    );

    // High should be reddish
    assert!(
        colors.spectrum_high[0] > colors.spectrum_high[2],
        "High spectrum should be reddish (R > B)"
    );
}

#[test]
fn test_spectrum_gradient_interpolation() {
    let theme = AppTheme::Dark;
    let colors = theme.colors();

    // Test interpolation at different points
    let low = colors.spectrum_low;
    let mid = colors.spectrum_mid;
    let high = colors.spectrum_high;

    // Interpolate at 25% (between low and mid)
    let color_25 = interpolate_gradient(&colors, 0.25);
    assert!(
        is_between_colors(color_25, low, mid),
        "25% should be between low and mid"
    );

    // Interpolate at 50% (at mid)
    let color_50 = interpolate_gradient(&colors, 0.5);
    assert!(
        colors_equal(color_50, mid),
        "50% should match mid color"
    );

    // Interpolate at 75% (between mid and high)
    let color_75 = interpolate_gradient(&colors, 0.75);
    assert!(
        is_between_colors(color_75, mid, high),
        "75% should be between mid and high"
    );
}

// ============================================================================
// EGUI VISUALS CONVERSION TESTS
// ============================================================================

#[test]
fn test_to_egui_visuals_dark() {
    let theme = AppTheme::Dark;
    let visuals = theme.to_egui_visuals();

    // Should be dark mode
    assert!(visuals.dark_mode, "Dark theme should set dark_mode = true");

    // Background should match theme colors
    let bg_color = visuals.widgets.noninteractive.bg_fill;
    assert_egui_color_matches(bg_color, theme.colors().bg_primary);
}

#[test]
fn test_to_egui_visuals_light() {
    let theme = AppTheme::Light;
    let visuals = theme.to_egui_visuals();

    // Should be light mode
    assert!(!visuals.dark_mode, "Light theme should set dark_mode = false");

    // Background should match theme colors
    let bg_color = visuals.widgets.noninteractive.bg_fill;
    assert_egui_color_matches(bg_color, theme.colors().bg_primary);
}

#[test]
fn test_egui_widget_colors() {
    for theme in AppTheme::all() {
        let visuals = theme.to_egui_visuals();
        let colors = theme.colors();

        // Interactive widgets should use accent color
        let button_bg = visuals.widgets.inactive.bg_fill;
        // Should be darker/lighter than background
        let bg = visuals.widgets.noninteractive.bg_fill;
        assert_ne!(button_bg, bg, "{} theme: button should differ from background", theme);

        // Hovered widgets should be brighter
        let hovered_bg = visuals.widgets.hovered.bg_fill;
        assert_ne!(hovered_bg, button_bg, "{} theme: hovered state should differ", theme);
    }
}

// ============================================================================
// ACCESSIBILITY TESTS
// ============================================================================

#[test]
fn test_wcag_aa_compliance() {
    // All themes should meet WCAG AA standards (4.5:1 for normal text)
    for theme in AppTheme::all() {
        let colors = theme.colors();
        let contrast = calculate_contrast_ratio(colors.bg_primary, colors.text_primary);

        assert!(
            contrast >= 4.5,
            "{} theme fails WCAG AA: contrast ratio {} < 4.5",
            theme,
            contrast
        );
    }
}

#[test]
fn test_high_contrast_wcag_aaa() {
    // High contrast theme should meet WCAG AAA (7:1)
    let theme = AppTheme::HighContrast;
    let colors = theme.colors();
    let contrast = calculate_contrast_ratio(colors.bg_primary, colors.text_primary);

    assert!(
        contrast >= 7.0,
        "High contrast theme should meet WCAG AAA (7:1), got {}",
        contrast
    );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn is_dark_color(color: [u8; 3]) -> bool {
    let luminance = relative_luminance(color);
    luminance < 0.5
}

fn is_light_color(color: [u8; 3]) -> bool {
    !is_dark_color(color)
}

fn relative_luminance(color: [u8; 3]) -> f32 {
    let r = (color[0] as f32 / 255.0).powf(2.2);
    let g = (color[1] as f32 / 255.0).powf(2.2);
    let b = (color[2] as f32 / 255.0).powf(2.2);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn calculate_contrast_ratio(color1: [u8; 3], color2: [u8; 3]) -> f32 {
    let l1 = relative_luminance(color1);
    let l2 = relative_luminance(color2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

fn interpolate_gradient(colors: &ThemeColors, t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);

    if t < 0.5 {
        // Interpolate between low and mid
        let local_t = t * 2.0;
        lerp_color(colors.spectrum_low, colors.spectrum_mid, local_t)
    } else {
        // Interpolate between mid and high
        let local_t = (t - 0.5) * 2.0;
        lerp_color(colors.spectrum_mid, colors.spectrum_high, local_t)
    }
}

fn lerp_color(c1: [u8; 3], c2: [u8; 3], t: f32) -> [u8; 3] {
    [
        ((c1[0] as f32) * (1.0 - t) + (c2[0] as f32) * t) as u8,
        ((c1[1] as f32) * (1.0 - t) + (c2[1] as f32) * t) as u8,
        ((c1[2] as f32) * (1.0 - t) + (c2[2] as f32) * t) as u8,
    ]
}

fn is_between_colors(color: [u8; 3], c1: [u8; 3], c2: [u8; 3]) -> bool {
    for i in 0..3 {
        let min = c1[i].min(c2[i]);
        let max = c1[i].max(c2[i]);
        if color[i] < min || color[i] > max {
            return false;
        }
    }
    true
}

fn colors_equal(c1: [u8; 3], c2: [u8; 3]) -> bool {
    c1[0] == c2[0] && c1[1] == c2[1] && c1[2] == c2[2]
}

fn assert_egui_color_matches(egui_color: egui::Color32, theme_color: [u8; 3]) {
    assert_eq!(egui_color.r(), theme_color[0]);
    assert_eq!(egui_color.g(), theme_color[1]);
    assert_eq!(egui_color.b(), theme_color[2]);
}
