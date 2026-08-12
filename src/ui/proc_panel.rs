use egui::{Align, Frame, Layout, ScrollArea, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::sys::config::AppConfig;
use crate::sys::metrics::{FocusedProcessTracker, ProcItem};
use crate::ui::icons::SvgIcons;
use crate::ui::theme::BtopTheme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortColumn {
    Cpu,
    Memory,
    Pid,
    Name,
    DiskIo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcAction {
    Focus(u32, String, String),
    Unfocus,
    Kill(u32),
}

pub struct ProcPanelView {
    pub search_query: String,
    pub sort_col: SortColumn,
    pub sort_desc: bool,
    pub selected_pid: Option<u32>,
    pub kill_confirm_pid: Option<(u32, String)>,
}

impl Default for ProcPanelView {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            sort_col: SortColumn::Cpu,
            sort_desc: true,
            selected_pid: None,
            kill_confirm_pid: None,
        }
    }
}

impl ProcPanelView {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        processes: &[ProcItem],
        focused_process: Option<&FocusedProcessTracker>,
        config: &AppConfig,
    ) -> Vec<ProcAction> {
        let mut actions = Vec::new();
        let cpu_color = BtopTheme::cpu_color(config);
        let ram_color = BtopTheme::ram_color(config);
        let disk_color = BtopTheme::disk_color(config);
        let proc_color = BtopTheme::proc_color(config);

        // 1. Render Dedicated Focused Process Profiler Inspector if active
        if let Some(focused) = focused_process {
            Frame::canvas(ui.style())
                .fill(BtopTheme::BG_CARD)
                .stroke(egui::Stroke::new(1.5_f32, cpu_color))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(SvgIcons::render_white("focus_icon", SvgIcons::PROCESS, 20.0));
                        ui.heading(
                            egui::RichText::new(format!(
                                "📌 特效進程監控儀表板 (PID: {} - {})",
                                focused.pid, focused.name
                            ))
                            .color(cpu_color)
                            .size(15.0)
                            .strong(),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .button(egui::RichText::new("❌ 取消釘選").color(proc_color))
                                .clicked()
                            {
                                actions.push(ProcAction::Unfocus);
                            }
                        });
                    });

                    ui.add_space(6.0);

                    let last_cpu = focused.cpu_history.last();
                    let last_mem = focused.mem_history.last();
                    let last_disk = focused.disk_history.last();

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("使用者: {}", focused.user))
                                .color(BtopTheme::TEXT_MUTED)
                                .size(12.0),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("當前 CPU: {:.1}%", last_cpu))
                                .color(cpu_color)
                                .strong(),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("當前 RAM: {:.1} MB", last_mem))
                                .color(ram_color)
                                .strong(),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("當前 Disk I/O: {:.1} KB/s", last_disk))
                                .color(disk_color)
                                .strong(),
                        );
                    });

                    ui.add_space(8.0);

                    // Dual Line Charts: Process CPU & Memory Trends
                    ui.columns(2, |cols| {
                        let cpu_pts: PlotPoints = focused
                            .cpu_history
                            .values()
                            .enumerate()
                            .map(|(i, &v)| [i as f64, v])
                            .collect();
                        let cpu_line = Line::new(cpu_pts)
                            .color(cpu_color)
                            .width(2.0_f32)
                            .name("Proc CPU %");

                        Plot::new("focused_cpu_plot")
                            .height(75.0)
                            .show_axes([false, true])
                            .show_grid(true)
                            .include_y(0.0)
                            .allow_zoom(false)
                            .allow_drag(false)
                            .show(&mut cols[0], |plot_ui| plot_ui.line(cpu_line));

                        let mem_pts: PlotPoints = focused
                            .mem_history
                            .values()
                            .enumerate()
                            .map(|(i, &v)| [i as f64, v])
                            .collect();
                        let mem_line = Line::new(mem_pts)
                            .color(ram_color)
                            .width(2.0_f32)
                            .name("Proc RAM MB");

                        Plot::new("focused_mem_plot")
                            .height(75.0)
                            .show_axes([false, true])
                            .show_grid(true)
                            .include_y(0.0)
                            .allow_zoom(false)
                            .allow_drag(false)
                            .show(&mut cols[1], |plot_ui| plot_ui.line(mem_line));
                    });
                });

            ui.add_space(8.0);
        }

        // 2. Main Process Table Card
        Frame::canvas(ui.style())
            .fill(BtopTheme::BG_CARD)
            .stroke(egui::Stroke::new(1.0_f32, BtopTheme::BORDER))
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header & Controls in Pure White Icon
                ui.horizontal(|ui| {
                    ui.add(SvgIcons::render_white("proc_icon", SvgIcons::PROCESS, 18.0));
                    ui.heading(
                        egui::RichText::new("進程管理器 (Process Manager)")
                            .color(proc_color)
                            .size(15.0)
                            .strong(),
                    );

                    ui.separator();

                    // Search input with White Search Icon
                    ui.add(SvgIcons::render_white("search_icon", SvgIcons::SEARCH, 14.0));
                    ui.add_sized(
                        [180.0, 22.0],
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("搜尋 PID 或名稱..."),
                    );

                    // Sort Column Dropdown
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(if self.sort_desc { "⬇ 降序" } else { "⬆ 升序" })
                            .clicked()
                        {
                            self.sort_desc = !self.sort_desc;
                        }

                        egui::ComboBox::from_id_source("proc_sort_combo")
                            .selected_text(match self.sort_col {
                                SortColumn::Cpu => "排序: CPU %",
                                SortColumn::Memory => "排序: 記憶體",
                                SortColumn::Pid => "排序: PID",
                                SortColumn::Name => "排序: 進程名稱",
                                SortColumn::DiskIo => "排序: 磁碟 I/O",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.sort_col, SortColumn::Cpu, "CPU %");
                                ui.selectable_value(&mut self.sort_col, SortColumn::Memory, "記憶體 (Memory)");
                                ui.selectable_value(&mut self.sort_col, SortColumn::Pid, "PID");
                                ui.selectable_value(&mut self.sort_col, SortColumn::Name, "進程名稱");
                                ui.selectable_value(&mut self.sort_col, SortColumn::DiskIo, "磁碟 I/O");
                            });
                    });
                });

                ui.add_space(8.0);

                // Filter & Sort processes
                let mut filtered: Vec<&ProcItem> = processes
                    .iter()
                    .filter(|p| {
                        if !config.show_system_processes && (p.user == "SYSTEM" || p.pid < 100) {
                            return false;
                        }
                        if self.search_query.is_empty() {
                            true
                        } else {
                            let query = self.search_query.to_lowercase();
                            p.name.to_lowercase().contains(&query)
                                || p.pid.to_string().contains(&query)
                                || p.user.to_lowercase().contains(&query)
                        }
                    })
                    .collect();

                filtered.sort_by(|a, b| {
                    let cmp = match self.sort_col {
                        SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                        SortColumn::Memory => a.mem_bytes.cmp(&b.mem_bytes),
                        SortColumn::Pid => a.pid.cmp(&b.pid),
                        SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        SortColumn::DiskIo => (a.read_bytes_sec + a.write_bytes_sec)
                            .partial_cmp(&(b.read_bytes_sec + b.write_bytes_sec))
                            .unwrap_or(std::cmp::Ordering::Equal),
                    };
                    if self.sort_desc {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });

                ui.label(
                    egui::RichText::new(format!("顯示 {} 個進程", filtered.len()))
                        .color(BtopTheme::TEXT_MUTED)
                        .size(11.0),
                );

                ui.add_space(4.0);

                // Table Header
                egui::Grid::new("proc_table_header")
                    .num_columns(8)
                    .spacing([10.0, 4.0])
                    .min_col_width(45.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("PID").color(BtopTheme::TEXT_MUTED).strong());
                        ui.label(egui::RichText::new("進程名稱 (Name)").color(BtopTheme::TEXT_MUTED).strong());
                        ui.label(egui::RichText::new("CPU %").color(cpu_color).strong());
                        ui.label(egui::RichText::new("Memory").color(ram_color).strong());
                        ui.label(egui::RichText::new("Disk I/O").color(disk_color).strong());
                        ui.label(egui::RichText::new("使用者").color(BtopTheme::TEXT_MUTED).strong());
                        ui.label(egui::RichText::new("監控").color(cpu_color).strong());
                        ui.label(egui::RichText::new("操作").color(proc_color).strong());
                        ui.end_row();
                    });

                ui.separator();

                // Process Rows List
                ScrollArea::vertical()
                    .max_height(260.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("proc_table_body")
                            .num_columns(8)
                            .spacing([10.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for proc_ in filtered {
                                    let is_selected = self.selected_pid == Some(proc_.pid);
                                    let is_focused = focused_process.map(|f| f.pid) == Some(proc_.pid);

                                    // PID
                                    let pid_text = egui::RichText::new(format!("{}", proc_.pid))
                                        .color(if is_focused {
                                            cpu_color
                                        } else if is_selected {
                                            ram_color
                                        } else {
                                            BtopTheme::TEXT_MUTED
                                        })
                                        .monospace();
                                    if ui.selectable_label(is_selected, pid_text).clicked() {
                                        self.selected_pid = Some(proc_.pid);
                                    }

                                    // Name
                                    ui.label(
                                        egui::RichText::new(&proc_.name)
                                            .color(if is_focused { cpu_color } else { BtopTheme::TEXT_PRIMARY })
                                            .strong(),
                                    );

                                    // CPU %
                                    let cpu_item_color = if proc_.cpu_usage > 50.0 {
                                        proc_color
                                    } else if proc_.cpu_usage > 10.0 {
                                        disk_color
                                    } else {
                                        cpu_color
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("{:.1}%", proc_.cpu_usage))
                                            .color(cpu_item_color)
                                            .strong(),
                                    );

                                    // Memory MB / %
                                    let mem_mb = proc_.mem_bytes as f64 / (1024.0 * 1024.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:.1} MB ({:.1}%)",
                                            mem_mb, proc_.mem_pct
                                        ))
                                        .color(ram_color),
                                    );

                                    // Disk I/O
                                    let total_io_kb = (proc_.read_bytes_sec + proc_.write_bytes_sec) / 1024.0;
                                    ui.label(
                                        egui::RichText::new(format!("{:.1} KB/s", total_io_kb))
                                            .color(disk_color),
                                    );

                                    // User
                                    ui.label(
                                        egui::RichText::new(&proc_.user)
                                            .color(BtopTheme::TEXT_MUTED)
                                            .size(11.0),
                                    );

                                    // Pin / Focus Button
                                    ui.horizontal(|ui| {
                                        if is_focused {
                                            if ui
                                                .button(egui::RichText::new("📌 已釘選").color(cpu_color).size(11.0))
                                                .clicked()
                                            {
                                                actions.push(ProcAction::Unfocus);
                                            }
                                        } else {
                                            if ui
                                                .button(egui::RichText::new("📌 釘選").color(BtopTheme::TEXT_MUTED).size(11.0))
                                                .clicked()
                                            {
                                                actions.push(ProcAction::Focus(proc_.pid, proc_.name.clone(), proc_.user.clone()));
                                            }
                                        }
                                    });

                                    // Kill button
                                    ui.horizontal(|ui| {
                                        if ui
                                            .button(egui::RichText::new("❌ 結束").color(proc_color).size(11.0))
                                            .clicked()
                                        {
                                            self.kill_confirm_pid = Some((proc_.pid, proc_.name.clone()));
                                        }
                                    });

                                    ui.end_row();
                                }
                            });
                    });
            });

        // Kill Confirmation Modal Dialog
        if let Some((pid, name)) = self.kill_confirm_pid.clone() {
            egui::Window::new("確認結束進程")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.add(SvgIcons::render_white("kill_warn", SvgIcons::KILL, 24.0));
                        ui.label(
                            egui::RichText::new(format!(
                                "您確定要強制結束進程嗎？\nPID: {} - {}",
                                pid, name
                            ))
                            .color(BtopTheme::TEXT_PRIMARY)
                            .strong(),
                        );
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("確認結束 (Kill)")
                                    .color(proc_color)
                                    .strong(),
                            )
                            .clicked()
                        {
                            actions.push(ProcAction::Kill(pid));
                            self.kill_confirm_pid = None;
                        }

                        if ui.button("取消").clicked() {
                            self.kill_confirm_pid = None;
                        }
                    });
                });
        }

        actions
    }
}
