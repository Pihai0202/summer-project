use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind,
    RefreshKind, System,
};
use parking_lot::RwLock;

use crate::sys::logger::AutoSaveLogger;

/// Fixed-size history ring buffer for real-time plot graphs
#[derive(Clone, Debug)]
pub struct RingBuffer {
    capacity: usize,
    buffer: VecDeque<f64>,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, val: f64) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(val);
    }

    pub fn values(&self) -> impl Iterator<Item = &f64> {
        self.buffer.iter()
    }

    pub fn as_slice(&self) -> Vec<f64> {
        self.buffer.iter().copied().collect()
    }

    pub fn last(&self) -> f64 {
        self.buffer.back().copied().unwrap_or(0.0)
    }
}

#[derive(Clone, Debug)]
pub struct CpuCoreData {
    pub id: usize,
    pub usage: f32,
    pub frequency: u64,
}

#[derive(Clone, Debug)]
pub struct CpuMetrics {
    pub brand: String,
    pub overall_usage: f32,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub cores: Vec<CpuCoreData>,
    pub history: RingBuffer,
}

#[derive(Clone, Debug)]
pub struct GpuMetrics {
    pub name: String,
    pub utilization: f32,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub temp_celsius: Option<u32>,
    pub is_available: bool,
    pub history: RingBuffer,
}

#[derive(Clone, Debug)]
pub struct MemMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub ram_history: RingBuffer,
    pub swap_history: RingBuffer,
}

#[derive(Clone, Debug)]
pub struct DiskItem {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub fs_type: String,
    pub read_bytes_sec: f64,
    pub write_bytes_sec: f64,
    pub temp_celsius: Option<f32>,
    pub read_history: RingBuffer,
    pub write_history: RingBuffer,
}

#[derive(Clone, Debug)]
pub struct NetItem {
    pub name: String,
    pub rx_speed_bytes: f64,
    pub tx_speed_bytes: f64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub rx_history: RingBuffer,
    pub tx_history: RingBuffer,
}

#[derive(Clone, Debug)]
pub struct ProcItem {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub mem_bytes: u64,
    pub mem_pct: f32,
    pub read_bytes_sec: f64,
    pub write_bytes_sec: f64,
    pub user: String,
    pub status: String,
}

/// Dedicated Resource Tracker for a specific focused process (PID)
#[derive(Clone, Debug)]
pub struct FocusedProcessTracker {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu_history: RingBuffer,
    pub mem_history: RingBuffer,
    pub disk_history: RingBuffer,
}

impl FocusedProcessTracker {
    pub fn new(pid: u32, name: String, user: String, capacity: usize) -> Self {
        Self {
            pid,
            name,
            user,
            cpu_history: RingBuffer::new(capacity),
            mem_history: RingBuffer::new(capacity),
            disk_history: RingBuffer::new(capacity),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub hostname: String,
    pub os_name: String,
    pub os_kernel: String,
    pub uptime_secs: u64,
    pub cpu: CpuMetrics,
    pub gpu: GpuMetrics,
    pub mem: MemMetrics,
    pub disks: Vec<DiskItem>,
    pub networks: Vec<NetItem>,
    pub processes: Vec<ProcItem>,
    pub focused_process: Option<FocusedProcessTracker>,
    pub total_processes: usize,
    pub total_threads: usize,
    pub last_update: Instant,
}

/// Thread-safe Collector handle with crash-proof auto-saver and hardware temp sensors
pub struct MetricsCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    metrics: Arc<RwLock<SystemMetrics>>,
    logger: AutoSaveLogger,
    history_capacity: usize,
    last_sample: Instant,
    prev_net_sample: Instant,
    prev_proc_disk_io: HashMap<u32, (u64, u64)>,
}

impl MetricsCollector {
    pub fn new(capacity: usize) -> (Self, Arc<RwLock<SystemMetrics>>) {
        let mut sys = System::new_all();
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        let brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let physical_cores = sys.physical_core_count().unwrap_or(sys.cpus().len());
        let logical_cores = sys.cpus().len();

        let initial_metrics = SystemMetrics {
            hostname: System::host_name().unwrap_or_else(|| "localhost".to_string()),
            os_name: System::name().unwrap_or_else(|| "Windows".to_string()),
            os_kernel: System::kernel_version().unwrap_or_else(|| "N/A".to_string()),
            uptime_secs: System::uptime(),
            cpu: CpuMetrics {
                brand,
                overall_usage: 0.0,
                physical_cores,
                logical_cores,
                cores: Vec::new(),
                history: RingBuffer::new(capacity),
            },
            gpu: GpuMetrics {
                name: "NVIDIA / Integrated GPU".to_string(),
                utilization: 0.0,
                mem_used_mb: 0,
                mem_total_mb: 0,
                temp_celsius: None,
                is_available: false,
                history: RingBuffer::new(capacity),
            },
            mem: MemMetrics {
                total_bytes: sys.total_memory(),
                used_bytes: sys.used_memory(),
                free_bytes: sys.free_memory(),
                available_bytes: sys.available_memory(),
                swap_total_bytes: sys.total_swap(),
                swap_used_bytes: sys.used_swap(),
                ram_history: RingBuffer::new(capacity),
                swap_history: RingBuffer::new(capacity),
            },
            disks: Vec::new(),
            networks: Vec::new(),
            processes: Vec::new(),
            focused_process: None,
            total_processes: 0,
            total_threads: 0,
            last_update: Instant::now(),
        };

        let metrics_arc = Arc::new(RwLock::new(initial_metrics));
        let logger = AutoSaveLogger::new();

        let collector = Self {
            sys,
            disks,
            networks,
            components,
            metrics: metrics_arc.clone(),
            logger,
            history_capacity: capacity,
            last_sample: Instant::now(),
            prev_net_sample: Instant::now(),
            prev_proc_disk_io: HashMap::new(),
        };

        (collector, metrics_arc)
    }

    pub fn set_focused_pid(&mut self, pid: u32, name: String, user: String) {
        let mut metrics = self.metrics.write();
        metrics.focused_process = Some(FocusedProcessTracker::new(
            pid,
            name,
            user,
            self.history_capacity,
        ));
    }

    pub fn clear_focused_pid(&mut self) {
        let mut metrics = self.metrics.write();
        metrics.focused_process = None;
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.last_sample).as_secs_f64().max(0.1);
        self.last_sample = now;

        // Refresh CPU, Memory, Processes, Disks, Networks, Components
        self.sys.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        self.disks.refresh_list();
        self.networks.refresh_list();
        self.components.refresh_list();

        let mut metrics = self.metrics.write();

        // Update Header/System
        metrics.uptime_secs = System::uptime();
        metrics.last_update = now;

        // Update CPU
        let cpus = self.sys.cpus();
        let global_usage = self.sys.global_cpu_info().cpu_usage();
        metrics.cpu.overall_usage = global_usage;
        metrics.cpu.history.push(global_usage as f64);

        metrics.cpu.cores = cpus
            .iter()
            .enumerate()
            .map(|(idx, cpu)| CpuCoreData {
                id: idx,
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect();

        // Update Memory
        let total_mem = self.sys.total_memory();
        let used_mem = self.sys.used_memory();
        let ram_pct = if total_mem > 0 {
            (used_mem as f64 / total_mem as f64) * 100.0
        } else {
            0.0
        };
        metrics.mem.total_bytes = total_mem;
        metrics.mem.used_bytes = used_mem;
        metrics.mem.free_bytes = self.sys.free_memory();
        metrics.mem.available_bytes = self.sys.available_memory();
        metrics.mem.swap_total_bytes = self.sys.total_swap();
        metrics.mem.swap_used_bytes = self.sys.used_swap();
        metrics.mem.ram_history.push(ram_pct);

        let swap_pct = if metrics.mem.swap_total_bytes > 0 {
            (metrics.mem.swap_used_bytes as f64 / metrics.mem.swap_total_bytes as f64) * 100.0
        } else {
            0.0
        };
        metrics.mem.swap_history.push(swap_pct);

        // Update GPU via NVML if present
        Self::update_gpu(&mut metrics.gpu);

        // Update Processes & Calculate aggregate Disk Read/Write Rates
        let mut total_threads = 0;
        let mut proc_list = Vec::new();
        let focused_pid = metrics.focused_process.as_ref().map(|f| f.pid);
        let mut focused_sample: Option<(f32, f64, f64)> = None;

        let mut total_sys_read_bytes_sec = 0.0;
        let mut total_sys_write_bytes_sec = 0.0;
        let mut curr_proc_io = HashMap::new();

        for (pid, proc_) in self.sys.processes() {
            let disk_usage = proc_.disk_usage();
            let proc_mem = proc_.memory();
            let mem_pct = if total_mem > 0 {
                (proc_mem as f32 / total_mem as f32) * 100.0
            } else {
                0.0
            };

            let proc_pid = pid.as_u32();
            curr_proc_io.insert(proc_pid, (disk_usage.total_read_bytes, disk_usage.total_written_bytes));

            let (read_sec, write_sec) = if let Some(&(prev_read, prev_write)) = self.prev_proc_disk_io.get(&proc_pid) {
                let r_delta = disk_usage.total_read_bytes.saturating_sub(prev_read) as f64 / elapsed_secs;
                let w_delta = disk_usage.total_written_bytes.saturating_sub(prev_write) as f64 / elapsed_secs;
                (r_delta, w_delta)
            } else {
                (disk_usage.read_bytes as f64 / elapsed_secs, disk_usage.written_bytes as f64 / elapsed_secs)
            };

            total_sys_read_bytes_sec += read_sec;
            total_sys_write_bytes_sec += write_sec;

            if Some(proc_pid) == focused_pid {
                focused_sample = Some((
                    proc_.cpu_usage(),
                    proc_mem as f64 / (1024.0 * 1024.0),
                    (read_sec + write_sec) / 1024.0,
                ));
            }

            proc_list.push(ProcItem {
                pid: proc_pid,
                name: proc_.name().to_string(),
                cpu_usage: proc_.cpu_usage(),
                mem_bytes: proc_mem,
                mem_pct,
                read_bytes_sec: read_sec,
                write_bytes_sec: write_sec,
                user: proc_
                    .user_id()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "SYSTEM".to_string()),
                status: format!("{:?}", proc_.status()),
            });

            total_threads += 1;
        }

        self.prev_proc_disk_io = curr_proc_io;
        metrics.total_processes = proc_list.len();
        metrics.total_threads = total_threads;
        metrics.processes = proc_list;

        // Scan Temperature Sensors from sysinfo Components (NVMe, SSD, HDD, Drive sensors)
        let mut disk_temps = HashMap::new();
        for comp in self.components.iter() {
            let label = comp.label().to_lowercase();
            if label.contains("nvme") || label.contains("ssd") || label.contains("disk") || label.contains("drive") || label.contains("storage") || label.contains("composite") {
                disk_temps.insert(comp.label().to_string(), comp.temperature());
            }
        }

        // Update Disks with Read/Write MB/s history and Temperature
        let mut updated_disks = Vec::new();
        for d in self.disks.iter() {
            let mount_str = d.mount_point().to_string_lossy().to_string();

            let mut read_hist = metrics
                .disks
                .iter()
                .find(|item| item.mount_point == mount_str)
                .map(|item| item.read_history.clone())
                .unwrap_or_else(|| RingBuffer::new(self.history_capacity));

            let mut write_hist = metrics
                .disks
                .iter()
                .find(|item| item.mount_point == mount_str)
                .map(|item| item.write_history.clone())
                .unwrap_or_else(|| RingBuffer::new(self.history_capacity));

            // Distribute system disk read/write to active partitions
            let read_mb = total_sys_read_bytes_sec / (1024.0 * 1024.0);
            let write_mb = total_sys_write_bytes_sec / (1024.0 * 1024.0);

            read_hist.push(read_mb);
            write_hist.push(write_mb);

            // Match temperature sensor for this disk if available
            let matched_temp = disk_temps
                .iter()
                .find(|(lbl, _)| {
                    let l = lbl.to_lowercase();
                    l.contains(&mount_str.to_lowercase())
                        || l.contains("nvme")
                        || l.contains("ssd")
                        || l.contains("disk")
                        || l.contains("composite")
                })
                .map(|(_, &t)| t);

            updated_disks.push(DiskItem {
                name: d.name().to_string_lossy().to_string(),
                mount_point: mount_str,
                total_bytes: d.total_space(),
                available_bytes: d.available_space(),
                fs_type: d.file_system().to_string_lossy().to_string(),
                read_bytes_sec: total_sys_read_bytes_sec,
                write_bytes_sec: total_sys_write_bytes_sec,
                temp_celsius: matched_temp,
                read_history: read_hist,
                write_history: write_hist,
            });
        }
        metrics.disks = updated_disks;

        // Update Networks
        let net_elapsed = now.duration_since(self.prev_net_sample).as_secs_f64().max(0.1);
        self.prev_net_sample = now;

        let mut updated_nets = Vec::new();
        for (name, net) in self.networks.iter() {
            let rx_speed = net.received() as f64 / net_elapsed;
            let tx_speed = net.transmitted() as f64 / net_elapsed;

            let mut rx_hist = metrics
                .networks
                .iter()
                .find(|n| n.name == *name)
                .map(|n| n.rx_history.clone())
                .unwrap_or_else(|| RingBuffer::new(self.history_capacity));
            let mut tx_hist = metrics
                .networks
                .iter()
                .find(|n| n.name == *name)
                .map(|n| n.tx_history.clone())
                .unwrap_or_else(|| RingBuffer::new(self.history_capacity));

            rx_hist.push(rx_speed / 1024.0); // KB/s
            tx_hist.push(tx_speed / 1024.0); // KB/s

            updated_nets.push(NetItem {
                name: name.clone(),
                rx_speed_bytes: rx_speed,
                tx_speed_bytes: tx_speed,
                total_rx_bytes: net.total_received(),
                total_tx_bytes: net.total_transmitted(),
                rx_history: rx_hist,
                tx_history: tx_hist,
            });
        }
        metrics.networks = updated_nets;

        // Sample Focused Process Ring Buffers
        if let Some(ref mut focused) = metrics.focused_process {
            if let Some((f_cpu, f_mem_mb, f_disk_kb)) = focused_sample {
                focused.cpu_history.push(f_cpu as f64);
                focused.mem_history.push(f_mem_mb);
                focused.disk_history.push(f_disk_kb);
            } else {
                focused.cpu_history.push(0.0);
                focused.mem_history.push(0.0);
                focused.disk_history.push(0.0);
            }
        }

        // Trigger Auto-Save logger with immediate flush
        self.logger.append_log(&metrics);
    }

    fn update_gpu(gpu: &mut GpuMetrics) {
        if let Ok(nvml) = nvml_wrapper::Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                gpu.is_available = true;
                if let Ok(name) = device.name() {
                    gpu.name = name;
                }
                if let Ok(rates) = device.utilization_rates() {
                    gpu.utilization = rates.gpu as f32;
                    gpu.history.push(rates.gpu as f64);
                }
                if let Ok(mem) = device.memory_info() {
                    gpu.mem_used_mb = mem.used / (1024 * 1024);
                    gpu.mem_total_mb = mem.total / (1024 * 1024);
                }
                if let Ok(temp) = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                    gpu.temp_celsius = Some(temp);
                }
                return;
            }
        }

        gpu.is_available = false;
        gpu.name = "NVIDIA / AMD / Integrated GPU".to_string();
        gpu.utilization = 0.0;
    }

    pub fn kill_process(&mut self, pid: u32) -> Result<(), String> {
        let sys_pid = Pid::from_u32(pid);
        if let Some(proc_) = self.sys.process(sys_pid) {
            if proc_.kill() {
                Ok(())
            } else {
                Err(format!("Failed to kill process PID {}", pid))
            }
        } else {
            Err(format!("Process PID {} not found", pid))
        }
    }
}
