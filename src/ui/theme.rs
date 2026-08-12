use egui::{Color32, Context, FontFamily, Visuals};
use crate::sys::config::AppConfig;

pub struct BtopTheme;

impl BtopTheme {
    // btop Base Background & Surface Palette
    pub const BG_MAIN: Color32 = Color32::from_rgb(15, 19, 26);        // #0f131a
    pub const BG_CARD: Color32 = Color32::from_rgb(24, 29, 39);        // #181d27
    pub const BG_HEADER: Color32 = Color32::from_rgb(20, 24, 33);      // #141821
    pub const BORDER: Color32 = Color32::from_rgb(45, 55, 72);         // #2d3748

    // Default Accents
    pub const CPU_CYAN: Color32 = Color32::from_rgb(0, 240, 255);
    pub const GPU_GREEN: Color32 = Color32::from_rgb(16, 185, 129);
    pub const RAM_MAGENTA: Color32 = Color32::from_rgb(236, 72, 153);
    pub const DISK_ORANGE: Color32 = Color32::from_rgb(245, 158, 11);
    pub const NET_BLUE: Color32 = Color32::from_rgb(99, 102, 241);
    pub const PROC_RED: Color32 = Color32::from_rgb(239, 68, 68);
    pub const PURPLE: Color32 = Color32::from_rgb(168, 85, 247);

    // Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(241, 245, 249); // #f1f5f9
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);   // #94a3b8

    pub fn to_color32(arr: [u8; 4]) -> Color32 {
        Color32::from_rgba_premultiplied(arr[0], arr[1], arr[2], arr[3])
    }

    pub fn cpu_color(config: &AppConfig) -> Color32 {
        Self::to_color32(config.cpu_color)
    }

    pub fn gpu_color(config: &AppConfig) -> Color32 {
        Self::to_color32(config.gpu_color)
    }

    pub fn ram_color(config: &AppConfig) -> Color32 {
        Self::to_color32(config.ram_color)
    }

    pub fn disk_color(config: &AppConfig) -> Color32 {
        Self::to_color32(config.disk_color)
    }

    pub fn net_color(config: &AppConfig) -> Color32 {
        Self::to_color32(config.net_color)
    }

    pub fn proc_color(config: &AppConfig) -> Color32 {
        Self::to_color32(config.proc_color)
    }

    pub fn apply(ctx: &Context) {
        // 1. Setup Source Han Sans Font (思源黑體)
        let mut fonts = egui::FontDefinitions::default();
        
        fonts.font_data.insert(
            "SourceHanSans".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/SourceHanSansTC.ttf")),
        );

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "SourceHanSans".to_owned());

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("SourceHanSans".to_owned());

        ctx.set_fonts(fonts);

        // 2. Custom Visual Styling
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Self::BG_MAIN;
        visuals.window_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke.color = Self::BORDER;
        visuals.widgets.noninteractive.bg_stroke.width = 1.0;

        visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 36, 48);
        visuals.widgets.inactive.bg_stroke.color = Self::BORDER;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 50, 68);
        visuals.widgets.hovered.bg_stroke.color = Self::CPU_CYAN;
        visuals.widgets.active.bg_fill = Color32::from_rgb(52, 62, 84);

        ctx.set_visuals(visuals);
    }
}
