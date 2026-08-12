use egui::{Frame, ProgressBar, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::sys::config::AppConfig;
use crate::sys::metrics::MemMetrics;
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

pub struct MemPanelView;

impl MemPanelView {
    pub fn show(ui: &mut Ui, mem: &MemMetrics, config: &AppConfig) {
        let ram_color = BtopTheme::ram_color(config);
        let cpu_color = BtopTheme::cpu_color(config);

        Frame::canvas(ui.style())
            .fill(BtopTheme::BG_CARD)
            .stroke(egui::Stroke::new(1.0_f32, BtopTheme::BORDER))
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header in Pure White Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("ram_icon", SvgIcons::RAM, 18.0));
                    ui.heading(
                        egui::RichText::new("記憶體 (RAM & Swap)")
                            .color(ram_color)
                            .size(15.0)
                            .strong(),
                    );
                });

                ui.add_space(8.0);

                let total_gb = mem.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let used_gb = mem.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let free_gb = mem.free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let avail_gb = mem.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

                let ram_pct = if mem.total_bytes > 0 {
                    (mem.used_bytes as f32 / mem.total_bytes as f32) * 100.0
                } else {
                    0.0
                };

                // RAM Usage Bar
                ui.label(
                    egui::RichText::new(format!(
                        "主記憶體 (RAM): {:.2} GB / {:.2} GB 已使用",
                        used_gb, total_gb
                    ))
                    .color(BtopTheme::TEXT_PRIMARY)
                    .size(12.0)
                    .strong(),
                );

                ui.add_space(3.0);

                let ram_bar = ProgressBar::new(ram_pct / 100.0)
                    .text(format!("{:.1}% ({:.2} GB 可用)", ram_pct, avail_gb))
                    .fill(ram_color)
                    .animate(true);
                ui.add_sized([ui.available_width(), 18.0], ram_bar);

                ui.add_space(6.0);

                // Quick stats breakdown
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("可用: {:.2} GB", avail_gb))
                            .color(cpu_color)
                            .size(11.0),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("空閒: {:.2} GB", free_gb))
                            .color(BtopTheme::TEXT_MUTED)
                            .size(11.0),
                    );
                });

                ui.add_space(8.0);

                // Swap Space
                let swap_total_gb = mem.swap_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let swap_used_gb = mem.swap_used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let swap_pct = if mem.swap_total_bytes > 0 {
                    (mem.swap_used_bytes as f32 / mem.swap_total_bytes as f32) * 100.0
                } else {
                    0.0
                };

                ui.label(
                    egui::RichText::new(format!(
                        "置換空間 (Swap): {:.2} GB / {:.2} GB",
                        swap_used_gb, swap_total_gb
                    ))
                    .color(BtopTheme::TEXT_PRIMARY)
                    .size(11.0),
                );

                ui.add_space(3.0);

                let swap_bar = ProgressBar::new(swap_pct / 100.0)
                    .text(format!("{:.1}%", swap_pct))
                    .fill(BtopTheme::PURPLE);
                ui.add_sized([ui.available_width(), 14.0], swap_bar);

                ui.add_space(8.0);

                // Real-time History Graph
                let history_points: PlotPoints = mem
                    .ram_history
                    .values()
                    .enumerate()
                    .map(|(i, &val)| [i as f64, val])
                    .collect();

                let line = Line::new(history_points)
                    .color(ram_color)
                    .width(2.0_f32)
                    .name("RAM %");

                Plot::new("mem_history_plot")
                    .height(80.0)
                    .show_axes([false, true])
                    .show_grid(true)
                    .include_y(0.0)
                    .include_y(100.0)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .show(ui, |plot_ui| plot_ui.line(line));
            });
    }
}
