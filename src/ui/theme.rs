use egui::{Color32, Visuals, Style, Rounding, Vec2};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Theme {
    #[default]
    Mocha,
    Macchiato,
    Frappe,
    Latte,
    Light,
    Dark,
    Custom(CustomTheme),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomTheme {
    pub name: String,
    pub colors: ThemeColors,
    pub styling: ThemeStyling,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    pub primary: Color32,
    pub secondary: Color32,
    pub accent: Color32,
    pub background: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub spectrum_colors: Vec<Color32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeStyling {
    pub button_rounding: f32,
    pub panel_rounding: f32,
    pub slider_rounding: f32,
    pub window_shadow: bool,
    pub button_shadow: bool,
    pub panel_margin: f32,
    pub item_spacing: f32,
    pub indent: f32,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: Color32::from_rgb(137, 180, 250),
            secondary: Color32::from_rgb(203, 166, 247),
            accent: Color32::from_rgb(250, 179, 135),
            background: Color32::from_rgb(30, 30, 46),
            surface: Color32::from_rgb(49, 50, 68),
            text: Color32::from_rgb(205, 214, 244),
            text_secondary: Color32::from_rgb(147, 153, 178),
            success: Color32::from_rgb(166, 227, 161),
            warning: Color32::from_rgb(249, 226, 175),
            error: Color32::from_rgb(243, 139, 168),
            spectrum_colors: vec![
                Color32::from_rgb(137, 180, 250), // Blue
                Color32::from_rgb(203, 166, 247), // Purple
                Color32::from_rgb(245, 194, 231), // Pink
                Color32::from_rgb(243, 139, 168), // Red
                Color32::from_rgb(250, 179, 135), // Orange
                Color32::from_rgb(249, 226, 175), // Yellow
                Color32::from_rgb(166, 227, 161), // Green
                Color32::from_rgb(148, 226, 213), // Teal
            ],
        }
    }
}

impl Default for ThemeStyling {
    fn default() -> Self {
        Self {
            button_rounding: 8.0,
            panel_rounding: 12.0,
            slider_rounding: 6.0,
            window_shadow: true,
            button_shadow: false,
            panel_margin: 8.0,
            item_spacing: 8.0,
            indent: 16.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeManager {
    current_theme: Theme,
    custom_themes: HashMap<String, CustomTheme>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self {
            current_theme: Theme::default(),
            custom_themes: HashMap::new(),
        }
    }
}

impl ThemeManager {
    pub fn new(theme: Theme) -> Self {
        Self {
            current_theme: theme,
            custom_themes: HashMap::new(),
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }

    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    pub fn add_custom_theme(&mut self, theme: CustomTheme) {
        self.custom_themes.insert(theme.name.clone(), theme);
    }

    pub fn get_custom_theme(&self, name: &str) -> Option<&CustomTheme> {
        self.custom_themes.get(name)
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        let visuals = self.get_visuals();
        ctx.set_visuals(visuals);

        let style = self.get_style();
        ctx.set_style(style);
    }

    pub fn get_visuals(&self) -> Visuals {
        match &self.current_theme {
            Theme::Mocha => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(catppuccin_egui::MOCHA.text);
                visuals
            },
            Theme::Macchiato => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(catppuccin_egui::MACCHIATO.text);
                visuals
            },
            Theme::Frappe => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(catppuccin_egui::FRAPPE.text);
                visuals
            },
            Theme::Latte => {
                let mut visuals = Visuals::light();
                visuals.override_text_color = Some(catppuccin_egui::LATTE.text);
                visuals
            },
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
            Theme::Custom(custom) => self.create_custom_visuals(custom),
        }
    }

    pub fn get_style(&self) -> Style {
        let mut style = Style::default();
        let styling = self.get_styling();

        style.visuals.button_frame = true;
        style.visuals.widgets.inactive.rounding = Rounding::same(styling.button_rounding);
        style.visuals.widgets.hovered.rounding = Rounding::same(styling.button_rounding);
        style.visuals.widgets.active.rounding = Rounding::same(styling.button_rounding);

        style.spacing.item_spacing = Vec2::splat(styling.item_spacing);
        style.spacing.indent = styling.indent;
        style.spacing.button_padding = Vec2::new(16.0, 8.0);
        style.spacing.slider_width = 200.0;

        if styling.window_shadow {
            style.visuals.window_shadow.color = Color32::from_black_alpha(64);
            style.visuals.window_shadow.offset = Vec2::new(4.0, 4.0);
            style.visuals.window_shadow.blur = 8.0;
        }

        style
    }

    pub fn get_colors(&self) -> ThemeColors {
        match &self.current_theme {
            Theme::Custom(custom) => custom.colors.clone(),
            _ => self.get_default_colors_for_theme(&self.current_theme),
        }
    }

    pub fn get_styling(&self) -> ThemeStyling {
        match &self.current_theme {
            Theme::Custom(custom) => custom.styling.clone(),
            _ => ThemeStyling::default(),
        }
    }

    fn create_custom_visuals(&self, custom: &CustomTheme) -> Visuals {
        let mut visuals = Visuals::dark();

        visuals.window_fill = custom.colors.background;
        visuals.panel_fill = custom.colors.surface;
        visuals.extreme_bg_color = custom.colors.background;
        visuals.code_bg_color = custom.colors.surface;

        visuals.widgets.noninteractive.bg_fill = custom.colors.surface;
        visuals.widgets.noninteractive.fg_stroke.color = custom.colors.text;

        visuals.widgets.inactive.bg_fill = custom.colors.surface;
        visuals.widgets.inactive.fg_stroke.color = custom.colors.text_secondary;

        visuals.widgets.hovered.bg_fill = custom.colors.primary;
        visuals.widgets.hovered.fg_stroke.color = custom.colors.text;

        visuals.widgets.active.bg_fill = custom.colors.accent;
        visuals.widgets.active.fg_stroke.color = custom.colors.text;

        visuals.selection.bg_fill = custom.colors.primary;
        visuals.hyperlink_color = custom.colors.accent;

        visuals
    }

    fn get_default_colors_for_theme(&self, theme: &Theme) -> ThemeColors {
        match theme {
            Theme::Mocha => ThemeColors {
                primary: Color32::from_rgb(137, 180, 250),
                secondary: Color32::from_rgb(203, 166, 247),
                accent: Color32::from_rgb(250, 179, 135),
                background: Color32::from_rgb(30, 30, 46),
                surface: Color32::from_rgb(49, 50, 68),
                text: Color32::from_rgb(205, 214, 244),
                text_secondary: Color32::from_rgb(147, 153, 178),
                success: Color32::from_rgb(166, 227, 161),
                warning: Color32::from_rgb(249, 226, 175),
                error: Color32::from_rgb(243, 139, 168),
                spectrum_colors: vec![
                    Color32::from_rgb(137, 180, 250),
                    Color32::from_rgb(203, 166, 247),
                    Color32::from_rgb(245, 194, 231),
                    Color32::from_rgb(243, 139, 168),
                    Color32::from_rgb(250, 179, 135),
                    Color32::from_rgb(249, 226, 175),
                    Color32::from_rgb(166, 227, 161),
                    Color32::from_rgb(148, 226, 213),
                ],
            },
            Theme::Light => ThemeColors {
                primary: Color32::from_rgb(30, 102, 245),
                secondary: Color32::from_rgb(136, 57, 239),
                accent: Color32::from_rgb(254, 127, 45),
                background: Color32::from_rgb(255, 255, 255),
                surface: Color32::from_rgb(248, 250, 252),
                text: Color32::from_rgb(15, 23, 42),
                text_secondary: Color32::from_rgb(100, 116, 139),
                success: Color32::from_rgb(34, 197, 94),
                warning: Color32::from_rgb(251, 191, 36),
                error: Color32::from_rgb(239, 68, 68),
                spectrum_colors: vec![
                    Color32::from_rgb(30, 102, 245),
                    Color32::from_rgb(136, 57, 239),
                    Color32::from_rgb(219, 39, 119),
                    Color32::from_rgb(239, 68, 68),
                    Color32::from_rgb(254, 127, 45),
                    Color32::from_rgb(251, 191, 36),
                    Color32::from_rgb(34, 197, 94),
                    Color32::from_rgb(20, 184, 166),
                ],
            },
            _ => ThemeColors::default(),
        }
    }
}

impl Theme {
    pub fn all() -> Vec<Self> {
        vec![
            Theme::Mocha,
            Theme::Macchiato,
            Theme::Frappe,
            Theme::Latte,
            Theme::Light,
            Theme::Dark,
        ]
    }

    pub fn display_name(&self) -> &str {
        match self {
            Theme::Mocha => "Catppuccin Mocha",
            Theme::Macchiato => "Catppuccin Macchiato",
            Theme::Frappe => "Catppuccin Frappe",
            Theme::Latte => "Catppuccin Latte",
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::Custom(custom) => &custom.name,
        }
    }
}

impl CustomTheme {
    pub fn new(name: String, colors: ThemeColors, styling: ThemeStyling) -> Self {
        Self {
            name,
            colors,
            styling,
        }
    }

    pub fn dark_theme(name: String) -> Self {
        Self {
            name,
            colors: ThemeColors::default(),
            styling: ThemeStyling::default(),
        }
    }

    pub fn light_theme(name: String) -> Self {
        Self {
            name,
            colors: ThemeColors {
                background: Color32::WHITE,
                surface: Color32::LIGHT_GRAY,
                text: Color32::BLACK,
                text_secondary: Color32::DARK_GRAY,
                ..Default::default()
            },
            styling: ThemeStyling::default(),
        }
    }
}