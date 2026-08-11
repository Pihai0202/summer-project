use egui::{Frame, ProgressBar, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::sys::metrics::{DiskItem, NetItem};
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

pub struct DiskNetPanelView;

impl DiskNetPanelView {
    pub fn show(ui: &mut Ui, disks: &[DiskItem], networks: &[NetItem]) {
        Frame::canvas(ui.style())
            .fill(BtopTheme::BG_CARD)
            .stroke(egui::Stroke::new(1.0_f32, BtopTheme::BORDER))
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // 1. Disks Section Header in Pure White Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("disk_icon", SvgIcons::DISK, 18.0));
                    ui.heading(
                        egui::RichText::new("儲存裝置 (Disks)")
                            .color(BtopTheme::DISK_ORANGE)
                            .size(15.0)
                            .strong(),
                    );
                });

                ui.add_space(6.0);

                if disks.is_empty() {
                    ui.label(egui::RichText::new("未檢測到磁碟").color(BtopTheme::TEXT_MUTED));
                } else {
                    for disk in disks {
                        let total_gb = disk.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                        let avail_gb = disk.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                        let used_gb = total_gb - avail_gb;
                        let used_pct = if total_gb > 0.0 {
                            (used_gb / total_gb) * 100.0
                        } else {
                            0.0
                        };

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("💾 {}", disk.mount_point))
                                    .color(BtopTheme::TEXT_PRIMARY)
                                    .strong()
                                    .size(12.0),
                            );
                            ui.label(
                                egui::RichText::new(format!("({})", disk.fs_type))
                                    .color(BtopTheme::TEXT_MUTED)
                                    .size(10.0),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:.1} GB / {:.1} GB ({:.0}%)",
                                        used_gb, total_gb, used_pct
                                    ))
                                    .color(BtopTheme::TEXT_MUTED)
                                    .size(11.0),
                                );
                            });
                        });

                        ui.add_space(2.0);

                        let bar = ProgressBar::new(used_pct as f32 / 100.0)
                            .fill(BtopTheme::DISK_ORANGE);
                        ui.add_sized([ui.available_width(), 10.0], bar);

                        ui.add_space(6.0);
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                // 2. Network Section Header in Pure White Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("net_icon", SvgIcons::NETWORK, 18.0));
                    ui.heading(
                        egui::RichText::new("網路介面 (Network)")
                            .color(BtopTheme::NET_BLUE)
                            .size(15.0)
                            .strong(),
                    );
                });

                ui.add_space(6.0);

                for net in networks.iter().take(3) {
                    let rx_kb = net.rx_speed_bytes / 1024.0;
                    let tx_kb = net.tx_speed_bytes / 1024.0;

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("🌐 {}", net.name))
                                .color(BtopTheme::TEXT_PRIMARY)
                                .strong()
                                .size(12.0),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "⬇ {:.1} KB/s  ⬆ {:.1} KB/s",
                                    rx_kb, tx_kb
                                ))
                                .color(BtopTheme::NET_BLUE)
                                .strong()
                                .size(11.0),
                            );
                        });
                    });

                    ui.add_space(4.0);

                    // Sparkline history graph for network Rx/Tx
                    let rx_points: PlotPoints = net
                        .rx_history
                        .values()
                        .enumerate()
                        .map(|(i, &val)| [i as f64, val])
                        .collect();

                    let tx_points: PlotPoints = net
                        .tx_history
                        .values()
                        .enumerate()
                        .map(|(i, &val)| [i as f64, val])
                        .collect();

                    let rx_line = Line::new(rx_points)
                        .color(BtopTheme::CPU_CYAN)
                        .width(1.5_f32)
                        .name("Download KB/s");

                    let tx_line = Line::new(tx_points)
                        .color(BtopTheme::RAM_MAGENTA)
                        .width(1.5_f32)
                        .name("Upload KB/s");

                    Plot::new(format!("net_plot_{}", net.name))
                        .height(55.0)
                        .show_axes([false, false])
                        .show_grid(false)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .show(ui, |plot_ui| {
                            plot_ui.line(rx_line);
                            plot_ui.line(tx_line);
                        });

                    ui.add_space(6.0);
                }
            });
    }
}
