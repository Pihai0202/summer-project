# BTOP GUI - 高效能 Rust & egui 系統監視器

> 一款基於 **Rust** 與 **egui (eframe)** 開發，靈感源自 btop/htop 的美型桌面級系統監視器。支援繁體中文 **思源黑體 (Source Han Sans)**、**全純白 SVG 向量圖標**、多核心 CPU 折線圖、GPU 數據、記憶體/ Swap 分佈、磁碟與網路動態走勢圖，以及強大的進程管理器。

---

## 🌟 專案特色 (Features)

- 🦀 **Rust 核心與極致效能**：基於 `eframe` / `egui` 即時渲染，低記憶體佔用與高幀率繪製。
- 🎨 **btop 霓虹暗黑美學**：高對比 Cyberpunk 色彩計畫（Cyan CPU、Green GPU、Magenta RAM、Orange Storage、Blue Net、Red Process）。
- 🔤 **思源黑體 CJK 支援**：內建嵌入 **Source Han Sans (Noto Sans TC)**，完全無缺字或亂碼。
- 🤍 **高對比純白 SVG 圖標**：所有標題、按鈕與工具列圖示均使用清晰的純白 (`#FFFFFF`) 向量 SVG Icon。
- 📈 **即時動態折線圖表**：使用 `egui_plot` 繪製 CPU 核心、GPU 負載、RAM 佔用與網路上下載速度走勢。
- ⚡ **進程管理器 (Process Manager)**：
  - 支持名稱 / PID / 使用者即時關鍵字過濾。
  - 支持按 CPU %、記憶體、PID、進程名稱、磁碟 I/O 動態排序（升降序切換）。
  - 提供安全終止進程的 **Kill 二次確認彈窗**。
- 🎛️ **靈活視圖切換與刷新控制**：可自訂 `250ms` / `500ms` / `1000ms` / `2000ms` 更新頻率，並可自由切換 `All` / `CPU/GPU` / `RAM` / `Disks/Net` / `Processes` 檢視分頁。

---

## 🚀 快速開始 (Quick Start)

### 前置需求 (Prerequisites)
- [Rust Toolchain](https://www.rust-lang.org/) (建議 v1.75.0 或更新版本)
- Cargo 套件管理器

### 1. 複製專案倉庫 (Clone Repository)
```bash
git clone https://github.com/Pihai0202/btop-egui.git
cd btop-egui
```

### 2. 運行開發版本 (Run in Debug Mode)
```bash
cargo run
```


---

## 📖 操作指南 (Usage Guide)

| 視圖分頁 | 說明與操作功能 |
| :--- | :--- |
| **全部 (All)** | 一覽所有硬體面板（CPU、GPU、RAM、Disks、Network）與進程列表。 |
| **CPU / GPU** | 專注檢視 CPU 總使用率、多核心分佈 (`Core 0..N`)、時脈 (`GHz/MHz`) 與 GPU VRAM/溫度。 |
| **記憶體 (RAM)** | 查看 RAM 與 Swap 置換空間已用量/剩餘量 (GB) 與即時佔用曲線圖。 |
| **硬碟 / 網路** | 檢視各磁碟區空間狀態、網卡 IP 以及即時下載 (⬇ KB/s) 與上傳 (⬆ KB/s) 走勢。 |
| **進程 (Procs)** | 全螢幕進程搜尋、多欄位排序與 `❌ 結束` 進程操作。 |

### 結束進程 (Kill Process)
1. 在進程管理器中找到目標進程，點擊右側的 **`❌ 結束`** 按鈕。
2. 畫面跳出確認視窗，核對 PID 與進程名稱無誤後點擊 **`確認結束 (Kill)`** 即可終止該進程。

---

## 📁 專案目錄結構 (Project Structure)

```text
btop-egui/
├── Cargo.toml                  # 專案套件設定與依賴庫
├── README.md                   # 專案說明文件
├── assets/
│   └── fonts/
│       └── SourceHanSansTC.ttf # 嵌入式思源黑體字體檔
└── src/
    ├── main.rs                 # 原生視窗入口與 eframe 啟動
    ├── app.rs                  # 主應用程式更新循環與 Layout 佈局
    ├── sys/
    │   ├── mod.rs
    │   └── metrics.rs          # 背景系統數據採集與 sysinfo / nvml 封裝
    └── ui/
        ├── mod.rs
        ├── theme.rs            # btop 暗黑主題與字體加載
        ├── icons.rs            # 純白 SVG 向量圖示渲染器
        ├── header.rs           # 頂部導覽列與頻率/視圖控制
        ├── cpu_panel.rs        # CPU 核心分佈與折線圖卡片
        ├── gpu_panel.rs        # GPU 顯卡動態卡片 (含 NVML 與 Fallback)
        ├── mem_panel.rs        # RAM / Swap 記憶體卡片
        ├── disk_net_panel.rs   # 儲存裝置與網路 Sparkline 卡片
        └── proc_panel.rs       # 進程管理器 (搜尋/排序/Kill 對話框)
```

---

## 🛠️ 技術棧 (Tech Stack)

- **GUI 框架**: [`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe)
- **圖表繪製**: [`egui_plot`](https://github.com/emilk/egui/tree/master/crates/egui_plot)
- **SVG 向量圖形**: [`egui_extras`](https://github.com/emilk/egui/tree/master/crates/egui_extras) (features: `svg`, `image`)
- **系統指標採集**: [`sysinfo`](https://github.com/GuillaumeGomez/sysinfo)
- **NVIDIA GPU Telemetry**: [`nvml-wrapper`](https://github.com/CensoredMethod/nvml-wrapper)

---

## 📄 授權條款 (License)

本專案基於 MIT 授權條款開放原始碼。詳細內容請參閱 LICENSE 檔案。
