use egui::{Frame, ProgressBar, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::sys::config::AppConfig;
use crate::sys::metrics::GpuMetrics;
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

pub struct GpuPanelView;

impl GpuPanelView {
    pub fn show(ui: &mut Ui, gpu: &GpuMetrics, config: &AppConfig) {
        let gpu_color = BtopTheme::gpu_color(config);

        Frame::canvas(ui.style())
            .fill(BtopTheme::BG_CARD)
            .stroke(egui::Stroke::new(1.0_f32, BtopTheme::BORDER))
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header in Pure White Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("gpu_icon", SvgIcons::GPU, 18.0));
                    ui.heading(
                        egui::RichText::new("GPU 顯示卡")
                            .color(gpu_color)
                            .size(15.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(temp) = gpu.temp_celsius {
                            let temp_color = if temp > 80 {
                                BtopTheme::PROC_RED
                            } else if temp > 65 {
                                BtopTheme::DISK_ORANGE
                            } else {
                                gpu_color
                            };
                            ui.horizontal(|ui| {
                                ui.add(SvgIcons::render_white("temp_icon", SvgIcons::TEMP, 14.0));
                                ui.label(
                                    egui::RichText::new(format!("{} °C", temp))
                                        .color(temp_color)
                                        .strong(),
                                );
                            });
                        }
                    });
                });

                ui.add_space(6.0);

                // GPU Name
                ui.label(
                    egui::RichText::new(&gpu.name)
                        .color(BtopTheme::TEXT_PRIMARY)
                        .size(12.0)
                        .strong(),
                );

                ui.add_space(4.0);

                if gpu.is_available {
                    // Usage bar
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("核心 Load:")
                                .color(BtopTheme::TEXT_MUTED)
                                .size(11.0),
                        );
                        let bar = ProgressBar::new(gpu.utilization / 100.0)
                            .text(format!("{:.1}%", gpu.utilization))
                            .fill(gpu_color);
                        ui.add_sized([ui.available_width() - 10.0, 16.0], bar);
                    });

                    ui.add_space(4.0);

                    // VRAM
                    if gpu.mem_total_mb > 0 {
                        let vram_pct = (gpu.mem_used_mb as f32 / gpu.mem_total_mb as f32) * 100.0;
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("VRAM 已用:")
                                    .color(BtopTheme::TEXT_MUTED)
                                    .size(11.0),
                            );
                            let bar = ProgressBar::new(vram_pct / 100.0)
                                .text(format!(
                                    "{:.1} GB / {:.1} GB ({:.0}%)",
                                    gpu.mem_used_mb as f32 / 1024.0,
                                    gpu.mem_total_mb as f32 / 1024.0,
                                    vram_pct
                                ))
                                .fill(BtopTheme::PURPLE);
                            ui.add_sized([ui.available_width() - 10.0, 16.0], bar);
                        });
                    }

                    ui.add_space(6.0);

                    // Real-time History Graph
                    let history_points: PlotPoints = gpu
                        .history
                        .values()
                        .enumerate()
                        .map(|(i, &val)| [i as f64, val])
                        .collect();

                    let line = Line::new(history_points)
                        .color(gpu_color)
                        .width(2.0_f32)
                        .name("GPU %");

                    Plot::new("gpu_history_plot")
                        .height(70.0)
                        .show_axes([false, true])
                        .show_grid(true)
                        .include_y(0.0)
                        .include_y(100.0)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .show(ui, |plot_ui| plot_ui.line(line));
                } else {
                    ui.label(
                        egui::RichText::new("ℹ️ 未檢測到獨立 NVIDIA 顯示卡，正在使用內建 Graphics 模式。")
                            .color(BtopTheme::TEXT_MUTED)
                            .size(11.0),
                    );
                }
            });
    }
}
