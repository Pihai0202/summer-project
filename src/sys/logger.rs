use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Local;
use crate::sys::metrics::{FocusedProcessTracker, SystemMetrics};

pub struct AutoSaveLogger {
    log_dir: PathBuf,
    autosave_path: PathBuf,
    is_header_written: bool,
}

impl AutoSaveLogger {
    pub fn new() -> Self {
        let log_dir = PathBuf::from("logs");
        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }

        let autosave_path = log_dir.join("sysmon_autosave.csv");
        let is_header_written = autosave_path.exists();

        Self {
            log_dir,
            autosave_path,
            is_header_written,
        }
    }

    /// Appends a new metrics entry with immediate flush to guarantee zero data loss on crash
    pub fn append_log(&mut self, metrics: &SystemMetrics) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.autosave_path)
        {
            if !self.is_header_written {
                let header = "Timestamp,Overall_CPU_Pct,RAM_Used_MB,RAM_Pct,Swap_Used_MB,GPU_Pct,Net_Rx_KBps,Net_Tx_KBps,Focused_PID,Focused_Proc_Name,Focused_CPU_Pct,Focused_Mem_MB\n";
                let _ = file.write_all(header.as_bytes());
                self.is_header_written = true;
            }

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let ram_used_mb = metrics.mem.used_bytes as f64 / (1024.0 * 1024.0);
            let ram_pct = if metrics.mem.total_bytes > 0 {
                (metrics.mem.used_bytes as f64 / metrics.mem.total_bytes as f64) * 100.0
            } else {
                0.0
            };
            let swap_used_mb = metrics.mem.swap_used_bytes as f64 / (1024.0 * 1024.0);
            let net_rx_kb = metrics.networks.iter().map(|n| n.rx_speed_bytes / 1024.0).sum::<f64>();
            let net_tx_kb = metrics.networks.iter().map(|n| n.tx_speed_bytes / 1024.0).sum::<f64>();

            let (f_pid, f_name, f_cpu, f_mem) = if let Some(ref f) = metrics.focused_process {
                (
                    f.pid.to_string(),
                    f.name.clone(),
                    format!("{:.1}", f.cpu_history.last()),
                    format!("{:.1}", f.mem_history.last()),
                )
            } else {
                ("N/A".to_string(), "None".to_string(), "0.0".to_string(), "0.0".to_string())
            };

            let line = format!(
                "{},{:.2},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{},{},{},{}\n",
                timestamp,
                metrics.cpu.overall_usage,
                ram_used_mb,
                ram_pct,
                swap_used_mb,
                metrics.gpu.utilization,
                net_rx_kb,
                net_tx_kb,
                f_pid,
                f_name,
                f_cpu,
                f_mem
            );

            let _ = file.write_all(line.as_bytes());
            // Immediate flush ensures crash resistance!
            let _ = file.flush();
        }
    }
}

pub struct LogExporter;

impl LogExporter {
    /// Exports current system metrics & focused process data to CSV file
    pub fn export_csv(metrics: &SystemMetrics) -> Result<PathBuf, String> {
        let log_dir = PathBuf::from("logs");
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
        }

        let filename = format!("sysmon_export_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
        let filepath = log_dir.join(filename);

        let mut file = File::create(&filepath).map_err(|e| e.to_string())?;

        let header = "Timestamp,Hostname,OS_Kernel,Uptime_Secs,CPU_Usage_Pct,RAM_Used_MB,RAM_Total_MB,GPU_Usage_Pct,Total_Processes,Total_Threads\n";
        file.write_all(header.as_bytes()).map_err(|e| e.to_string())?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let ram_used_mb = metrics.mem.used_bytes as f64 / (1024.0 * 1024.0);
        let ram_total_mb = metrics.mem.total_bytes as f64 / (1024.0 * 1024.0);

        let line = format!(
            "{},{},{},{},{:.2},{:.1},{:.1},{:.1},{},{}\n\n",
            timestamp,
            metrics.hostname,
            metrics.os_kernel,
            metrics.uptime_secs,
            metrics.cpu.overall_usage,
            ram_used_mb,
            ram_total_mb,
            metrics.gpu.utilization,
            metrics.total_processes,
            metrics.total_threads
        );
        file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;

        // Write Top Processes Snapshot
        let proc_header = "--- TOP PROCESSES SNAPSHOT ---\nPID,Process_Name,CPU_Pct,Mem_MB,Mem_Pct,User\n";
        file.write_all(proc_header.as_bytes()).map_err(|e| e.to_string())?;

        for proc_ in metrics.processes.iter().take(50) {
            let proc_mem_mb = proc_.mem_bytes as f64 / (1024.0 * 1024.0);
            let proc_line = format!(
                "{},{},{:.1},{:.1},{:.1},{}\n",
                proc_.pid, proc_.name, proc_.cpu_usage, proc_mem_mb, proc_.mem_pct, proc_.user
            );
            file.write_all(proc_line.as_bytes()).map_err(|e| e.to_string())?;
        }

        file.flush().map_err(|e| e.to_string())?;

        Ok(filepath)
    }

    /// Exports current system metrics & focused process data to JSON file
    pub fn export_json(metrics: &SystemMetrics) -> Result<PathBuf, String> {
        let log_dir = PathBuf::from("logs");
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
        }

        let filename = format!("sysmon_export_{}.json", Local::now().format("%Y%m%d_%H%M%S"));
        let filepath = log_dir.join(filename);

        let mut file = File::create(&filepath).map_err(|e| e.to_string())?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let ram_used_mb = metrics.mem.used_bytes as f64 / (1024.0 * 1024.0);
        let ram_total_mb = metrics.mem.total_bytes as f64 / (1024.0 * 1024.0);

        let top_procs_json: Vec<String> = metrics
            .processes
            .iter()
            .take(30)
            .map(|p| {
                format!(
                    r#"{{"pid": {}, "name": "{}", "cpu_pct": {:.1}, "mem_mb": {:.1}, "user": "{}"}}"#,
                    p.pid, p.name, p.cpu_usage, p.mem_bytes as f64 / (1024.0 * 1024.0), p.user
                )
            })
            .collect();

        let json_content = format!(
            r#"{{
  "timestamp": "{}",
  "hostname": "{}",
  "kernel": "{}",
  "uptime_secs": {},
  "metrics": {{
    "cpu_pct": {:.2},
    "ram_used_mb": {:.1},
    "ram_total_mb": {:.1},
    "gpu_pct": {:.1},
    "total_processes": {},
    "total_threads": {}
  }},
  "top_processes": [
    {}
  ]
}}"#,
            timestamp,
            metrics.hostname,
            metrics.os_kernel,
            metrics.uptime_secs,
            metrics.cpu.overall_usage,
            ram_used_mb,
            ram_total_mb,
            metrics.gpu.utilization,
            metrics.total_processes,
            metrics.total_threads,
            top_procs_json.join(",\n    ")
        );

        file.write_all(json_content.as_bytes()).map_err(|e| e.to_string())?;
        file.flush().map_err(|e| e.to_string())?;

        Ok(filepath)
    }
}
