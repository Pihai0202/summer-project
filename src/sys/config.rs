use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    Neon,        // 暗黑霓虹
    GreenMatrix, // 綠黑矩陣
    IceBlue,     // 冰藍極光
    WarmOrange,  // 日落暖橙
}

impl Default for ThemePreset {
    fn default() -> Self {
        Self::Neon
    }
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Neon => "暗黑霓虹 (Cyberpunk)",
            Self::GreenMatrix => "綠黑矩陣 (Matrix)",
            Self::IceBlue => "冰藍極光 (Ice Blue)",
            Self::WarmOrange => "日落暖橙 (Warm Orange)",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub enable_autosave: bool,
    pub theme_preset: ThemePreset,
    pub cpu_color: [u8; 4],
    pub gpu_color: [u8; 4],
    pub ram_color: [u8; 4],
    pub disk_color: [u8; 4],
    pub net_color: [u8; 4],
    pub proc_color: [u8; 4],
    pub history_capacity: usize,
    pub show_system_processes: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enable_autosave: true,
            theme_preset: ThemePreset::Neon,
            cpu_color: [0, 240, 255, 255],     // Cyan
            gpu_color: [16, 185, 129, 255],   // Green
            ram_color: [236, 72, 153, 255],   // Pink/Magenta
            disk_color: [245, 158, 11, 255],   // Orange
            net_color: [99, 102, 241, 255],    // Blue
            proc_color: [239, 68, 68, 255],    // Red
            history_capacity: 60,
            show_system_processes: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        PathBuf::from("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(mut file) = File::open(&path) {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(json_str.as_bytes()).map_err(|e| e.to_string())?;
        file.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn apply_preset(&mut self, preset: ThemePreset) {
        self.theme_preset = preset;
        match preset {
            ThemePreset::Neon => {
                self.cpu_color = [0, 240, 255, 255];
                self.gpu_color = [16, 185, 129, 255];
                self.ram_color = [236, 72, 153, 255];
                self.disk_color = [245, 158, 11, 255];
                self.net_color = [99, 102, 241, 255];
                self.proc_color = [239, 68, 68, 255];
            }
            ThemePreset::GreenMatrix => {
                self.cpu_color = [34, 197, 94, 255];   // Bright Green
                self.gpu_color = [16, 185, 129, 255];  // Emerald
                self.ram_color = [74, 222, 128, 255];  // Lime Green
                self.disk_color = [134, 239, 172, 255]; // Soft Green
                self.net_color = [52, 211, 153, 255];  // Mint Green
                self.proc_color = [239, 68, 68, 255];  // Red Accent
            }
            ThemePreset::IceBlue => {
                self.cpu_color = [56, 189, 248, 255];  // Sky Blue
                self.gpu_color = [99, 102, 241, 255];  // Indigo
                self.ram_color = [168, 85, 247, 255];  // Purple
                self.disk_color = [14, 165, 233, 255]; // Cyan
                self.net_color = [129, 140, 248, 255]; // Soft Blue
                self.proc_color = [244, 63, 94, 255];  // Rose
            }
            ThemePreset::WarmOrange => {
                self.cpu_color = [245, 158, 11, 255];  // Amber Orange
                self.gpu_color = [234, 88, 12, 255];   // Dark Orange
                self.ram_color = [239, 68, 68, 255];   // Red
                self.disk_color = [251, 146, 60, 255];  // Light Orange
                self.net_color = [217, 119, 6, 255];   // Golden
                self.proc_color = [225, 29, 72, 255];  // Crimson
            }
        }
    }
}
