use egui::{Align, Layout, Ui};
use crate::sys::config::AppConfig;
use crate::sys::metrics::SystemMetrics;
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewFilter {
    All,
    CpuGpu,
    Memory,
    DisksNet,
    Processes,
}

pub struct HeaderView;

impl HeaderView {
    pub fn show(
        ui: &mut Ui,
        metrics: &SystemMetrics,
        config: &AppConfig,
        current_filter: &mut ViewFilter,
        refresh_ms: &mut u64,
        on_export_csv: &mut impl FnMut(),
        on_export_json: &mut impl FnMut(),
        on_open_settings: &mut impl FnMut(),
    ) {
        let cpu_color = BtopTheme::cpu_color(config);
        let gpu_color = BtopTheme::gpu_color(config);
        let ram_color = BtopTheme::ram_color(config);
        let disk_color = BtopTheme::disk_color(config);
        let proc_color = BtopTheme::proc_color(config);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            // Brand Logo / Icon in Pure White
            ui.add(SvgIcons::render_white("app_logo", SvgIcons::CPU, 22.0));
            
            ui.heading(
                egui::RichText::new("BTOP GUI")
                    .color(cpu_color)
                    .strong()
                    .size(19.0),
            );

            ui.separator();

            // Hostname & Kernel Info
            ui.label(
                egui::RichText::new(format!("💻 {}", metrics.hostname))
                    .color(BtopTheme::TEXT_PRIMARY)
                    .strong(),
            );

            ui.separator();

            // Auto-Save Crash Protection Indicator
            if config.enable_autosave {
                ui.label(
                    egui::RichText::new("🟢 自動存檔中 (Crash Proof)")
                        .color(gpu_color)
                        .size(11.0),
                );
            } else {
                ui.label(
                    egui::RichText::new("⚪ 自動存檔已關閉")
                        .color(BtopTheme::TEXT_MUTED)
                        .size(11.0),
                );
            }

            // Right-aligned toolbar
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Settings Button with White Gear SVG Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("settings_btn_icon", SvgIcons::SETTINGS, 16.0));
                    if ui.button("設定").clicked() {
                        on_open_settings();
                    }
                });

                ui.separator();

                // Export Buttons
                if ui
                    .button(egui::RichText::new("📄 匯出 JSON").color(ram_color).size(12.0))
                    .clicked()
                {
                    on_export_json();
                }

                if ui
                    .button(egui::RichText::new("💾 匯出 CSV").color(disk_color).size(12.0))
                    .clicked()
                {
                    on_export_csv();
                }

                ui.separator();

                // Refresh Rate Combo
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Refresh:")
                            .color(BtopTheme::TEXT_MUTED)
                            .size(12.0),
                    );
                    egui::ComboBox::from_id_source("refresh_interval")
                        .selected_text(format!("{}ms", *refresh_ms))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(refresh_ms, 250, "250ms");
                            ui.selectable_value(refresh_ms, 500, "500ms");
                            ui.selectable_value(refresh_ms, 1000, "1000ms");
                            ui.selectable_value(refresh_ms, 2000, "2000ms");
                        });
                });

                ui.separator();

                // Filter Tabs Buttons
                let filters = [
                    (ViewFilter::All, "全部 (All)", BtopTheme::TEXT_PRIMARY),
                    (ViewFilter::CpuGpu, "CPU / GPU", cpu_color),
                    (ViewFilter::Memory, "記憶體 (RAM)", ram_color),
                    (ViewFilter::DisksNet, "硬碟/網路", disk_color),
                    (ViewFilter::Processes, "進程 (Procs)", proc_color),
                ];

                for (filter, label, color) in filters.iter().rev() {
                    let is_selected = *current_filter == *filter;
                    let text = if is_selected {
                        egui::RichText::new(*label).color(*color).strong()
                    } else {
                        egui::RichText::new(*label).color(BtopTheme::TEXT_MUTED)
                    };

                    if ui.selectable_label(is_selected, text).clicked() {
                        *current_filter = *filter;
                    }
                }
            });
        });
    }
}
