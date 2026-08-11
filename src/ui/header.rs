use egui::{Align, Layout, Ui};
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
        current_filter: &mut ViewFilter,
        refresh_ms: &mut u64,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            // Brand Logo / Icon in Pure White
            ui.add(SvgIcons::render_white("app_logo", SvgIcons::CPU, 22.0));
            
            ui.heading(
                egui::RichText::new("BTOP GUI")
                    .color(BtopTheme::CPU_CYAN)
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
            ui.label(
                egui::RichText::new(format!("🐧 {}", metrics.os_kernel))
                    .color(BtopTheme::TEXT_MUTED)
                    .size(12.0),
            );

            ui.separator();

            // Uptime
            let uptime = metrics.uptime_secs;
            let days = uptime / 86400;
            let hours = (uptime % 86400) / 3600;
            let mins = (uptime % 3600) / 60;
            let secs = uptime % 60;
            let uptime_str = if days > 0 {
                format!("⏱ {}d {}h {}m", days, hours, mins)
            } else {
                format!("⏱ {}h {}m {}s", hours, mins, secs)
            };
            ui.label(
                egui::RichText::new(uptime_str)
                    .color(BtopTheme::TEXT_MUTED)
                    .size(12.0),
            );

            // Right-aligned toolbar
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                    (ViewFilter::CpuGpu, "CPU / GPU", BtopTheme::CPU_CYAN),
                    (ViewFilter::Memory, "記憶體 (RAM)", BtopTheme::RAM_MAGENTA),
                    (ViewFilter::DisksNet, "硬碟/網路", BtopTheme::DISK_ORANGE),
                    (ViewFilter::Processes, "進程 (Procs)", BtopTheme::PROC_RED),
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
