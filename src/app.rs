use std::sync::Arc;
use std::time::{Duration, Instant};
use eframe::App;
use egui::{CentralPanel, Context, TopBottomPanel};
use parking_lot::RwLock;

use crate::sys::metrics::{MetricsCollector, SystemMetrics};
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
    image_loaders_installed: bool,
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
            image_loaders_installed: true,
        }
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

        let metrics_read = self.metrics.read();

        // 1. Top Header Bar
        TopBottomPanel::top("header_panel")
            .exact_height(42.0)
            .show(ctx, |ui| {
                HeaderView::show(
                    ui,
                    &metrics_read,
                    &mut self.current_filter,
                    &mut self.refresh_ms,
                );
            });

        // 2. Main Content Area
        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                    match self.current_filter {
                        ViewFilter::All => {
                            // Top Row: CPU & GPU & Memory
                            ui.columns(3, |columns| {
                                CpuPanelView::show(&mut columns[0], &metrics_read.cpu);
                                GpuPanelView::show(&mut columns[1], &metrics_read.gpu);
                                MemPanelView::show(&mut columns[2], &metrics_read.mem);
                            });

                            ui.add_space(6.0);

                            // Bottom Row: Disks/Net & Process Manager
                            ui.columns(2, |columns| {
                                DiskNetPanelView::show(
                                    &mut columns[0],
                                    &metrics_read.disks,
                                    &metrics_read.networks,
                                );

                                let collector_ref = &mut self.collector;
                                self.proc_view.show(
                                    &mut columns[1],
                                    &metrics_read.processes,
                                    &mut |pid| collector_ref.kill_process(pid),
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
                            let collector_ref = &mut self.collector;
                            self.proc_view.show(
                                ui,
                                &metrics_read.processes,
                                &mut |pid| collector_ref.kill_process(pid),
                            );
                        }
                    }
                });
        });
    }
}
