# BTOP GUI - 高效能 Rust & egui 系統監視器

> 一款基於 **Rust** 與 **egui (eframe)** 開發，靈感源自 btop/htop 的美型桌面級系統監視器。支援繁體中文 **思源黑體 (Source Han Sans)**、**全純白 SVG 向量圖標**、多核心 CPU 折線圖、GPU 數據、記憶體/ Swap 分佈、磁碟與網路動態走勢圖、**崩潰防禦增量自動存檔**、**特定進程深度資源監控儀表板** 以及 **一鍵匯出 CSV / JSON 紀錄**。

---

## 🌟 專案特色 (Features)

- 🦀 **Rust 核心與極致效能**：基於 `eframe` / `egui` 即時渲染，低記憶體佔用與高幀率繪製。
- 🛡️ **崩潰防禦與日誌自動持久化 (Crash Protection & Auto-Save)**：
  - 後台即時將系統指標增量寫入 `logs/sysmon_autosave.csv`。
  - 每次寫入均執行強制 `flush()` 清除系統快取。即使發生藍屏崩潰、斷電或強制關閉，崩潰前最後一刻的紀錄完全保留不遺失！
- 📌 **特定進程深度資源監控 (Targeted Process Profiler)**：
  - 進程列表中可點擊 **`📌 釘選`** 任一進程 (PID)。
  - 啟動 **`📌 特效進程監控儀表板 (Focused Process Inspector)`**，專屬獨立繪製該 PID 的 **CPU %**、**記憶體 (MB)** 與 **磁碟 I/O (KB/s)** 動態歷史趨勢圖。
- 💾 **一鍵匯出紀錄檔 (Log Export Engine)**：
  - 頂部工具列支援 **`💾 匯出 CSV`** 與 **`📄 匯出 JSON`**。
  - 自動生成帶有時間戳記的報告檔案至 `logs/sysmon_export_YYYYMMDD_HHMMSS.csv` 與 `.json`。
- 🎨 **btop 霓虹暗黑美學**：高對比 Cyberpunk 色彩計畫（Cyan CPU、Green GPU、Magenta RAM、Orange Storage、Blue Net、Red Process）。
- 🔤 **思源黑體 CJK 支援**：內建嵌入 **Source Han Sans (Noto Sans TC)**，完全無缺字或亂碼。
- 🤍 **高對比純白 SVG 圖標**：所有標題、按鈕與工具列圖示均使用清晰的純白 (`#FFFFFF`) 向量 SVG Icon。
- 📈 **即時動態折線圖表**：使用 `egui_plot` 繪製 CPU 核心、GPU 負載、RAM 佔用與網路上下載速度走勢。
- ⚡ **進程管理器 (Process Manager)**：
  - 支持名稱 / PID / 使用者即時關鍵字過濾。
  - 支持按 CPU %、記憶體、PID、進程名稱、磁碟 I/O 動態排序（升降序切換）。
  - 提供安全終止進程的 **Kill 二次確認彈窗**。

---

## 🚀 快速開始 (Quick Start)

### 前置需求 (Prerequisites)
- [Rust Toolchain](https://www.rust-lang.org/) (建議 v1.75.0 或更新版本)
- Cargo 套件管理器

### 1. 複製專案倉庫 (Clone Repository)
```bash
git clone https://github.com/Pihai0202/summer-project.git
cd summer-project
```

### 2. 運行開發版本 (Run in Debug Mode)
```bash
cargo run
```


---

## 📖 操作指南 (Usage Guide)

| 功能模組 | 操作說明 |
| :--- | :--- |
| **🟢 自動存檔 (Crash Proof)** | 系統背景自動增量存檔至 `logs/sysmon_autosave.csv`，具備崩潰保護。 |
| **💾 匯出 CSV / JSON** | 點擊頂部列右側按鈕，立即將當前系統狀態與進程快照匯出至 `logs/` 目錄。 |
| **📌 釘選特定進程監控** | 在進程列表中點擊 **`📌 釘選`**，開啟專屬折線圖表即時監控該 PID 的 CPU % 與 RAM MB。 |
| **視圖分頁切換** | 自由切換 `All` / `CPU/GPU` / `RAM` / `Disks/Net` / `Processes` 檢視面板。 |
| **結束進程 (Kill Process)** | 點擊進程右側的 **`❌ 結束`** 按鈕，於二次確認視窗核對 PID 後點擊 **`確認結束 (Kill)`**。 |

---

## 📁 專案目錄結構 (Project Structure)

```text
btop-egui/
├── Cargo.toml                  # 專案套件設定與依賴庫
├── README.md                   # 專案說明文件
├── logs/                       # 日誌自動存檔與匯出檔目錄
│   └── sysmon_autosave.csv     # 崩潰防禦增量持久化檔案
├── assets/
│   └── fonts/
│       └── SourceHanSansTC.ttf # 嵌入式思源黑體字體檔
└── src/
    ├── main.rs                 # 原生視窗入口與 eframe 啟動
    ├── app.rs                  # 主應用程式更新循環與 Toast 提示
    ├── sys/
    │   ├── mod.rs
    │   ├── metrics.rs          # 背景系統數據採集與進程釘選追蹤
    │   └── logger.rs           # 崩潰防禦 AutoSaveLogger 與 LogExporter 匯出引擎
    └── ui/
        ├── mod.rs
        ├── theme.rs            # btop 暗黑主題與字體加載
        ├── icons.rs            # 純白 SVG 向量圖示渲染器
        ├── header.rs           # 頂部導覽列與匯出/頻率控制
        ├── cpu_panel.rs        # CPU 核心分佈與折線圖卡片
        ├── gpu_panel.rs        # GPU 顯卡動態卡片
        ├── mem_panel.rs        # RAM / Swap 記憶體卡片
        ├── disk_net_panel.rs   # 儲存裝置與網路 Sparkline 卡片
        └── proc_panel.rs       # 進程管理器與 Focused Process Inspector
```

---

## 📄 授權條款 (License)

本專案基於 MIT 授權條款開放原始碼。詳細內容請參閱 LICENSE 檔案。
