use egui::{Align, Context, Layout, Window};
use crate::sys::config::{AppConfig, ThemePreset};
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

pub struct SettingsModalView;

impl SettingsModalView {
    pub fn show(ctx: &Context, is_open: &mut bool, config: &mut AppConfig) -> bool {
        let mut config_changed = false;

        if !*is_open {
            return false;
        }

        Window::new("⚙️ 系統監視器設定 (Settings)")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([540.0, 480.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                // Title Bar with White Settings SVG Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("settings_icon", SvgIcons::SETTINGS, 22.0));
                    ui.heading(
                        egui::RichText::new("偏好設定與主題自訂")
                            .color(BtopTheme::CPU_CYAN)
                            .size(16.0)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("❌ 關閉 (Close)").clicked() {
                            *is_open = false;
                        }
                    });
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        // Section 1: 日誌與存檔設定 (Auto-Save Persistence Settings)
                        ui.horizontal(|ui| {
                            ui.add(SvgIcons::render_white("disk_sec", SvgIcons::DISK, 16.0));
                            ui.label(
                                egui::RichText::new("1. 崩潰防禦與日誌存檔 (Auto-Save & Logs)")
                                    .color(BtopTheme::TEXT_PRIMARY)
                                    .strong()
                                    .size(13.0),
                            );
                        });

                        ui.indent("sec1_indent", |ui| {
                            if ui
                                .checkbox(
                                    &mut config.enable_autosave,
                                    egui::RichText::new("啟用崩潰防禦增量自動存檔 (Crash-Proof Auto-Save)")
                                        .color(BtopTheme::TEXT_PRIMARY),
                                )
                                .changed()
                            {
                                config_changed = true;
                            }

                            ui.label(
                                egui::RichText::new(
                                    "ℹ️ 開啟後背景每隔固定時間自動將指標寫入 logs/sysmon_autosave.csv 並強制 flush 清除快取，即使系統藍屏或斷電數據也不遺失。",
                                )
                                .color(BtopTheme::TEXT_MUTED)
                                .size(11.0),
                            );
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Section 2: 顏色與配色方案 (Theme Presets & Custom Accents)
                        ui.horizontal(|ui| {
                            ui.add(SvgIcons::render_white("theme_sec", SvgIcons::PROCESS, 16.0));
                            ui.label(
                                egui::RichText::new("2. 主題配色與調色盤 (Theme Presets & Colors)")
                                    .color(BtopTheme::TEXT_PRIMARY)
                                    .strong()
                                    .size(13.0),
                            );
                        });

                        ui.indent("sec2_indent", |ui| {
                            ui.label(
                                egui::RichText::new("配色預設組 (Presets):")
                                    .color(BtopTheme::TEXT_MUTED)
                                    .size(12.0),
                            );

                            ui.horizontal(|ui| {
                                let presets = [
                                    (ThemePreset::Neon, "🌌 暗黑霓虹"),
                                    (ThemePreset::GreenMatrix, "🟢 綠黑矩陣"),
                                    (ThemePreset::IceBlue, "❄️ 冰藍極光"),
                                    (ThemePreset::WarmOrange, "🌅 日落暖橙"),
                                ];

                                for (preset, label) in presets {
                                    let is_active = config.theme_preset == preset;
                                    if ui.selectable_label(is_active, label).clicked() {
                                        config.apply_preset(preset);
                                        config_changed = true;
                                    }
                                }
                            });

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new("自訂硬體調色盤 (Custom Color Pickers):")
                                    .color(BtopTheme::TEXT_MUTED)
                                    .size(12.0),
                            );

                            egui::Grid::new("color_picker_grid")
                                .num_columns(4)
                                .spacing([16.0, 8.0])
                                .show(ui, |ui| {
                                    // CPU Color
                                    ui.add(SvgIcons::render_white("cpu_c_ico", SvgIcons::CPU, 14.0));
                                    ui.label("CPU 顏色:");
                                    if ui.color_edit_button_srgba_unmultiplied(&mut config.cpu_color).changed() {
                                        config_changed = true;
                                    }
                                    ui.end_row();

                                    // GPU Color
                                    ui.add(SvgIcons::render_white("gpu_c_ico", SvgIcons::GPU, 14.0));
                                    ui.label("GPU 顏色:");
                                    if ui.color_edit_button_srgba_unmultiplied(&mut config.gpu_color).changed() {
                                        config_changed = true;
                                    }
                                    ui.end_row();

                                    // RAM Color
                                    ui.add(SvgIcons::render_white("ram_c_ico", SvgIcons::RAM, 14.0));
                                    ui.label("記憶體 (RAM) 顏色:");
                                    if ui.color_edit_button_srgba_unmultiplied(&mut config.ram_color).changed() {
                                        config_changed = true;
                                    }
                                    ui.end_row();

                                    // Storage Color
                                    ui.add(SvgIcons::render_white("disk_c_ico", SvgIcons::DISK, 14.0));
                                    ui.label("儲存 (Disk) 顏色:");
                                    if ui.color_edit_button_srgba_unmultiplied(&mut config.disk_color).changed() {
                                        config_changed = true;
                                    }
                                    ui.end_row();

                                    // Network Color
                                    ui.add(SvgIcons::render_white("net_c_ico", SvgIcons::NETWORK, 14.0));
                                    ui.label("網路 (Network) 顏色:");
                                    if ui.color_edit_button_srgba_unmultiplied(&mut config.net_color).changed() {
                                        config_changed = true;
                                    }
                                    ui.end_row();
                                });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Section 3: 圖表與效能設定 (Performance & Monitoring)
                        ui.horizontal(|ui| {
                            ui.add(SvgIcons::render_white("perf_sec", SvgIcons::SETTINGS, 16.0));
                            ui.label(
                                egui::RichText::new("3. 圖表紀錄與監控設定 (Charts & Performance)")
                                    .color(BtopTheme::TEXT_PRIMARY)
                                    .strong()
                                    .size(13.0),
                            );
                        });

                        ui.indent("sec3_indent", |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("折線圖歷史紀錄長度:")
                                        .color(BtopTheme::TEXT_PRIMARY),
                                );
                                if ui
                                    .selectable_value(&mut config.history_capacity, 30, "30 秒")
                                    .changed()
                                {
                                    config_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut config.history_capacity, 60, "60 秒")
                                    .changed()
                                {
                                    config_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut config.history_capacity, 120, "120 秒")
                                    .changed()
                                {
                                    config_changed = true;
                                }
                            });

                            ui.add_space(4.0);

                            if ui
                                .checkbox(
                                    &mut config.show_system_processes,
                                    egui::RichText::new("在進程列表中顯示系統核心服務 (Show System Processes)")
                                        .color(BtopTheme::TEXT_PRIMARY),
                                )
                                .changed()
                            {
                                config_changed = true;
                            }
                        });
                    });
            });

        if config_changed {
            let _ = config.save();
        }

        config_changed
    }
}
