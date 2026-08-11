use std::sync::Arc;
use std::time::{Duration, Instant};
use eframe::App;
use egui::{CentralPanel, Context, TopBottomPanel};
use parking_lot::RwLock;

use crate::sys::logger::LogExporter;
use crate::sys::metrics::{MetricsCollector, SystemMetrics};
use crate::ui::proc_panel::ProcAction;
use crate::ui::{
    BtopTheme, CpuPanelView, DiskNetPanelView, GpuPanelView, HeaderView, MemPanelView,
    ProcPanelView, ViewFilter,
};

pub struct SystemMonitorApp {
    collector: MetricsCollector,
    metrics: Arc<RwLock<SystemMetrics>>,
    current_filter: ViewFilter,
    refresh_ms: u64,
    last_update: Instant,
    proc_view: ProcPanelView,
    notification_msg: Option<(String, Instant)>,
}

impl SystemMonitorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply Source Han Sans font & btop theme
        BtopTheme::apply(&cc.egui_ctx);

        // Install SVG loaders
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let (collector, metrics) = MetricsCollector::new(60);

        Self {
            collector,
            metrics,
            current_filter: ViewFilter::All,
            refresh_ms: 1000,
            last_update: Instant::now(),
            proc_view: ProcPanelView::default(),
            notification_msg: None,
        }
    }

    fn show_toast(&mut self, msg: String) {
        self.notification_msg = Some((msg, Instant::now()));
    }
}

impl App for SystemMonitorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Ensure continuous repainting based on refresh rate
        ctx.request_repaint_after(Duration::from_millis(self.refresh_ms));

        // Periodic metrics sampling
        let now = Instant::now();
        if now.duration_since(self.last_update).as_millis() as u64 >= self.refresh_ms {
            self.collector.update();
            self.last_update = now;
        }

        let mut export_csv_requested = false;
        let mut export_json_requested = false;
        let mut proc_actions = Vec::new();

        // 1. Top Header Bar & Central Panel (Read Scope)
        {
            let metrics_read = self.metrics.read();

            TopBottomPanel::top("header_panel")
                .exact_height(42.0)
                .show(ctx, |ui| {
                    HeaderView::show(
                        ui,
                        &metrics_read,
                        &mut self.current_filter,
                        &mut self.refresh_ms,
                        &mut || export_csv_requested = true,
                        &mut || export_json_requested = true,
                    );
                });

            CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                        match self.current_filter {
                            ViewFilter::All => {
                                ui.columns(3, |columns| {
                                    CpuPanelView::show(&mut columns[0], &metrics_read.cpu);
                                    GpuPanelView::show(&mut columns[1], &metrics_read.gpu);
                                    MemPanelView::show(&mut columns[2], &metrics_read.mem);
                                });

                                ui.add_space(6.0);

                                ui.columns(2, |columns| {
                                    DiskNetPanelView::show(
                                        &mut columns[0],
                                        &metrics_read.disks,
                                        &metrics_read.networks,
                                    );

                                    proc_actions = self.proc_view.show(
                                        &mut columns[1],
                                        &metrics_read.processes,
                                        metrics_read.focused_process.as_ref(),
                                    );
                                });
                            }
                            ViewFilter::CpuGpu => {
                                ui.columns(2, |columns| {
                                    CpuPanelView::show(&mut columns[0], &metrics_read.cpu);
                                    GpuPanelView::show(&mut columns[1], &metrics_read.gpu);
                                });
                            }
                            ViewFilter::Memory => {
                                MemPanelView::show(ui, &metrics_read.mem);
                            }
                            ViewFilter::DisksNet => {
                                DiskNetPanelView::show(ui, &metrics_read.disks, &metrics_read.networks);
                            }
                            ViewFilter::Processes => {
                                proc_actions = self.proc_view.show(
                                    ui,
                                    &metrics_read.processes,
                                    metrics_read.focused_process.as_ref(),
                                );
                            }
                        }
                    });
            });
        }

        // 2. Process Actions Execution
        for action in proc_actions {
            match action {
                ProcAction::Focus(pid, name, user) => {
                    self.collector.set_focused_pid(pid, name.clone(), user);
                    self.show_toast(format!("📌 已開始深層追蹤進程 PID {} ({})", pid, name));
                }
                ProcAction::Unfocus => {
                    self.collector.clear_focused_pid();
                    self.show_toast("📌 已取消進程釘選監控".to_string());
                }
                ProcAction::Kill(pid) => match self.collector.kill_process(pid) {
                    Ok(_) => self.show_toast(format!("✅ 已成功終止進程 PID {}", pid)),
                    Err(err) => self.show_toast(format!("❌ 終止進程失敗: {}", err)),
                },
            }
        }

        // 3. Handle Log Exports
        if export_csv_requested {
            let metrics_snap = self.metrics.read().clone();
            match LogExporter::export_csv(&metrics_snap) {
                Ok(path) => self.show_toast(format!("✅ CSV 記錄檔已成功匯出至: {}", path.display())),
                Err(err) => self.show_toast(format!("❌ CSV 匯出失敗: {}", err)),
            }
        }

        if export_json_requested {
            let metrics_snap = self.metrics.read().clone();
            match LogExporter::export_json(&metrics_snap) {
                Ok(path) => self.show_toast(format!("✅ JSON 記錄檔已成功匯出至: {}", path.display())),
                Err(err) => self.show_toast(format!("❌ JSON 匯出失敗: {}", err)),
            }
        }

        // 4. Render Toast Notification Banner
        if let Some((ref msg, time)) = self.notification_msg.clone() {
            if time.elapsed() < Duration::from_secs(4) {
                TopBottomPanel::top("toast_panel").show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(msg)
                                .color(BtopTheme::CPU_CYAN)
                                .size(13.0)
                                .strong(),
                        );
                    });
                });
            } else {
                self.notification_msg = None;
            }
        }
    }
}
