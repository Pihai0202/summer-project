use egui::{Align, Frame, Layout, ScrollArea, Ui};
use crate::sys::metrics::ProcItem;
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
        on_kill: &mut impl FnMut(u32) -> Result<(), String>,
    ) {
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
                            .color(BtopTheme::PROC_RED)
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
                    .num_columns(7)
                    .spacing([12.0, 4.0])
                    .min_col_width(50.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("PID").color(BtopTheme::TEXT_MUTED).strong());
                        ui.label(egui::RichText::new("進程名稱 (Name)").color(BtopTheme::TEXT_MUTED).strong());
                        ui.label(egui::RichText::new("CPU %").color(BtopTheme::CPU_CYAN).strong());
                        ui.label(egui::RichText::new("Memory").color(BtopTheme::RAM_MAGENTA).strong());
                        ui.label(egui::RichText::new("Disk I/O").color(BtopTheme::DISK_ORANGE).strong());
                        ui.label(egui::RichText::new("使用者").color(BtopTheme::TEXT_MUTED).strong());
                        ui.label(egui::RichText::new("操作").color(BtopTheme::PROC_RED).strong());
                        ui.end_row();
                    });

                ui.separator();

                // Process Rows List
                ScrollArea::vertical()
                    .max_height(280.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("proc_table_body")
                            .num_columns(7)
                            .spacing([12.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for proc_ in filtered {
                                    let is_selected = self.selected_pid == Some(proc_.pid);

                                    // PID
                                    let pid_text = egui::RichText::new(format!("{}", proc_.pid))
                                        .color(if is_selected { BtopTheme::CPU_CYAN } else { BtopTheme::TEXT_MUTED })
                                        .monospace();
                                    if ui.selectable_label(is_selected, pid_text).clicked() {
                                        self.selected_pid = Some(proc_.pid);
                                    }

                                    // Name
                                    ui.label(
                                        egui::RichText::new(&proc_.name)
                                            .color(BtopTheme::TEXT_PRIMARY)
                                            .strong(),
                                    );

                                    // CPU %
                                    let cpu_color = if proc_.cpu_usage > 50.0 {
                                        BtopTheme::PROC_RED
                                    } else if proc_.cpu_usage > 10.0 {
                                        BtopTheme::DISK_ORANGE
                                    } else {
                                        BtopTheme::CPU_CYAN
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("{:.1}%", proc_.cpu_usage))
                                            .color(cpu_color)
                                            .strong(),
                                    );

                                    // Memory MB / %
                                    let mem_mb = proc_.mem_bytes as f64 / (1024.0 * 1024.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:.1} MB ({:.1}%)",
                                            mem_mb, proc_.mem_pct
                                        ))
                                        .color(BtopTheme::RAM_MAGENTA),
                                    );

                                    // Disk I/O
                                    let total_io_kb = (proc_.read_bytes_sec + proc_.write_bytes_sec) / 1024.0;
                                    ui.label(
                                        egui::RichText::new(format!("{:.1} KB/s", total_io_kb))
                                            .color(BtopTheme::DISK_ORANGE),
                                    );

                                    // User
                                    ui.label(
                                        egui::RichText::new(&proc_.user)
                                            .color(BtopTheme::TEXT_MUTED)
                                            .size(11.0),
                                    );

                                    // Kill button
                                    ui.horizontal(|ui| {
                                        if ui
                                            .button(egui::RichText::new(" ❌ 結束 ").color(BtopTheme::PROC_RED).size(11.0))
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
                                    .color(BtopTheme::PROC_RED)
                                    .strong(),
                            )
                            .clicked()
                        {
                            let _ = on_kill(pid);
                            self.kill_confirm_pid = None;
                        }

                        if ui.button("取消").clicked() {
                            self.kill_confirm_pid = None;
                        }
                    });
                });
        }
    }
}
