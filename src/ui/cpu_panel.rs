use egui::{Color32, Frame, ProgressBar, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::sys::config::AppConfig;
use crate::sys::metrics::CpuMetrics;
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

pub struct CpuPanelView;

impl CpuPanelView {
    pub fn show(ui: &mut Ui, cpu: &CpuMetrics, config: &AppConfig) {
        let cpu_color = BtopTheme::cpu_color(config);

        Frame::canvas(ui.style())
            .fill(BtopTheme::BG_CARD)
            .stroke(egui::Stroke::new(1.0_f32, BtopTheme::BORDER))
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header in Pure White Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("cpu_icon", SvgIcons::CPU, 18.0));
                    ui.heading(
                        egui::RichText::new("CPU 處理器")
                            .color(cpu_color)
                            .size(15.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} 實體 / {} 邏輯核心",
                                cpu.physical_cores, cpu.logical_cores
                            ))
                            .color(BtopTheme::TEXT_MUTED)
                            .size(12.0),
                        );
                    });
                });

                ui.add_space(6.0);

                // CPU Model & Usage Bar
                ui.label(
                    egui::RichText::new(&cpu.brand)
                        .color(BtopTheme::TEXT_PRIMARY)
                        .size(12.0)
                        .strong(),
                );

                ui.add_space(4.0);

                let usage = cpu.overall_usage;
                let bar_color = Self::usage_color(usage, cpu_color);

                ui.horizontal(|ui| {
                    let progress_bar = ProgressBar::new(usage / 100.0)
                        .text(format!("{:.1}%", usage))
                        .fill(bar_color)
                        .animate(true);
                    ui.add_sized([ui.available_width() - 80.0, 18.0], progress_bar);

                    let avg_freq = cpu
                        .cores
                        .iter()
                        .map(|c| c.frequency)
                        .sum::<u64>()
                        .checked_div(cpu.cores.len().max(1) as u64)
                        .unwrap_or(0);

                    ui.label(
                        egui::RichText::new(format!("{:.2} GHz", avg_freq as f64 / 1000.0))
                            .color(cpu_color)
                            .strong(),
                    );
                });

                ui.add_space(8.0);

                // Per-Core Usage Grid
                ui.collapsing(
                    egui::RichText::new("核心使用率分佈 (Per-Core Breakdown)")
                        .color(BtopTheme::TEXT_MUTED)
                        .size(12.0),
                    |ui| {
                        egui::Grid::new("cpu_cores_grid")
                            .num_columns(4)
                            .spacing([12.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (i, core) in cpu.cores.iter().enumerate() {
                                    let col_color = Self::usage_color(core.usage, cpu_color);
                                    ui.label(
                                        egui::RichText::new(format!("C{:02}", core.id))
                                            .color(BtopTheme::TEXT_MUTED)
                                            .size(11.0),
                                    );
                                    ui.add_sized(
                                        [80.0, 12.0],
                                        ProgressBar::new(core.usage / 100.0)
                                            .fill(col_color)
                                            .show_percentage(),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}%", core.usage))
                                            .color(col_color)
                                            .size(11.0)
                                            .strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{} MHz", core.frequency))
                                            .color(BtopTheme::TEXT_MUTED)
                                            .size(10.0),
                                    );

                                    if (i + 1) % 2 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    },
                );

                ui.add_space(8.0);

                // Real-time History Graph using egui_plot
                let history_points: PlotPoints = cpu
                    .history
                    .values()
                    .enumerate()
                    .map(|(i, &val)| [i as f64, val])
                    .collect();

                let line = Line::new(history_points)
                    .color(cpu_color)
                    .width(2.0_f32)
                    .name("CPU %");

                Plot::new("cpu_history_plot")
                    .height(90.0)
                    .show_axes([false, true])
                    .show_grid(true)
                    .include_y(0.0)
                    .include_y(100.0)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .show(ui, |plot_ui| plot_ui.line(line));
            });
    }

    fn usage_color(usage: f32, default_color: Color32) -> Color32 {
        if usage > 85.0 {
            BtopTheme::PROC_RED
        } else if usage > 60.0 {
            BtopTheme::DISK_ORANGE
        } else {
            default_color
        }
    }
}
