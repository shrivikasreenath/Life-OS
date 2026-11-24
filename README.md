# LIFE OS 🌟
### Complete Autonomous Bare-Metal x86_64 Rust Operating System
**Owner:** Shrivika  
**Project:** College Final Year Engineering Project  
**Architecture:** x86_64 (Bare-Metal, `no_std`, `no_main`)  
**Language:** 100% Rust  
**Final Bootable ISO Size:** **3.52 MB** (3,690,496 bytes) — *Strictly below the 100 MB requirement!*

---

## 🚀 Overview

**LIFE OS** is a modern, bare-metal operating system engineered completely from scratch in Rust without any runtime dependency on Linux, Windows, or standard C runtime libraries. Developed autonomously for **Shrivika's** college computer science & engineering demonstration, Life OS combines high-performance low-level systems programming with a polished graphical desktop user experience and genuine Wi-Fi connectivity through an integrated host bridge.

### Key Identity Requirements
- **OS Name:** Life OS
- **System Owner:** Shrivika
- **Terminal User:** `shrivika`
- **Command `whoami` Output:** `shrivika` (Verified and tested live)
- **Target Platform:** Oracle VirtualBox (UEFI & BIOS) & Bare-Metal x86_64 PC

---

## 📸 Screenshots from Live Oracle VirtualBox Execution

| Feature | Screenshot | Description |
| :--- | :--- | :--- |
| **Complete Graphical Desktop** | ![Desktop](docs/screenshots/life_os_desktop_upgraded.png) | 8 desktop icons with 32x32 pixel art, taskbar with pinned quick-launch apps, system tray with ETH LED & live Wi-Fi SSID |
| **Terminal & Identity** | ![Terminal](docs/screenshots/life_os_whoami_upgraded.png) | Interactive shell with cyan border, `whoami` returning `shrivika` |
| **Start Menu Launcher** | ![Start Menu](docs/screenshots/life_os_start_menu.png) | Start menu with Shrivika user avatar, 8 app shortcuts (hotkeys 1-8), and power controls |
| **Calendar Pop-up Panel** | ![Calendar](docs/screenshots/life_os_calendar.png) | Full calendar grid with current date highlighted, opened via tray clock or `F2` |
| **Desktop Context Menu** | ![Context Menu](docs/screenshots/life_os_context_menu.png) | Right-click context menu (Refresh, Change Wallpaper, Arrange Icons, Create Folder, Open Terminal, Settings, SysInfo) |
| **Cycled Desktop Wallpaper** | ![Wallpaper](docs/screenshots/life_os_wallpaper_theme.png) | Deep Cobalt Aurora theme (3 selectable wallpapers switchable on the fly via `F5`) |
| **Network & Wi-Fi Manager** | ![Wi-Fi Manager](docs/screenshots/life_os_wifi_manager.png) | Discovered real physical Wi-Fi network `Vedav's Iphone` (94% signal, WPA2, 2.4 GHz ch6, Connected) |
| **Wi-Fi Network Selection** | ![Wi-Fi Selected](docs/screenshots/life_os_wifi_selected.png) | Wi-Fi network selection row, masked password input box, Connect and Disconnect buttons |
| **Terminal Wi-Fi Scanner** | ![Wi-Fi Scan](docs/screenshots/life_os_wifi_scan_cli.png) | Terminal command `wifi scan` discovering real laptop physical Wi-Fi networks |
| **Terminal Wi-Fi Status** | ![Wi-Fi Status](docs/screenshots/life_os_wifi_status_cli.png) | Terminal command `wifi status` querying live connection state from Windows WLAN API |
| **Terminal Wi-Fi Connect** | ![Wi-Fi Connect](docs/screenshots/life_os_wifi_connect_cli.png) | Terminal command `wifi connect` verifying live connection attempt over UDP bridge |
| **File Manager (LifeFS)** | ![File Manager](docs/screenshots/life_os_file_manager.png) | Graphical explorer with toolbar (+File, +Folder, Delete, Up ..), breadcrumb path, and file list |
| **File Manager File Creation**| ![File Created](docs/screenshots/life_os_file_created.png) | Real-time creation of files and folders inside in-memory virtual filesystem |
| **File Manager Text Viewer** | ![File Preview](docs/screenshots/life_os_file_preview.png) | Built-in text file preview overlay reading `Kernel_Architecture.txt` |
| **Life Web Browser** | ![Browser](docs/screenshots/life_os_browser.png) | Web browser with navigation buttons (`<`, `>`, `R`, `H`), URL input, and `life://welcome` page |
| **System Information** | ![SysInfo](docs/screenshots/life_os_sysinfo.png) | Hardware telemetry: 4102 MB RAM, 32 MB heap pool, x86_64 CPU features, uptime ticks |
| **System Settings** | ![Settings](docs/screenshots/life_os_settings.png) | System settings panel with theme controls, RTC timezone, and power buttons |
| **Maximized Window** | ![Maximized](docs/screenshots/life_os_settings_maximized.png) | Window management: full maximize/restore above taskbar (`F12`) |
| **Games: Snake Classic** | ![Snake](docs/screenshots/life_os_games_snake.png) | Real-time Snake arcade game with live scoring and food collection |
| **Games: Retro Pong** | ![Pong](docs/screenshots/life_os_games_pong.png) | 2-player/AI arcade Pong with collision physics and score counter |
| **Games: Tic-Tac-Toe** | ![TicTacToe](docs/screenshots/life_os_games_tictactoe.png) | Interactive 3x3 strategy game with win/draw tracking |
| **Games: 2048 Puzzle** | ![2048](docs/screenshots/life_os_games_2048.png) | Full 4x4 sliding puzzle game with tile-combining mechanics |

---

## ⚡ Core Technical Highlights

### 1. Bare-Metal Rust Kernel
- **`no_std` and `no_main`**: Genuine microkernel architecture written completely in pure Rust.
- **Linker & Memory Map**: Custom 64-bit linker script (`linker.ld`) mapping the kernel higher-half at `0xffffffff80000000`.
- **Limine Bootloader v7**: Modern bootloader protocol supporting both UEFI (`EFI64`) and Legacy BIOS hybrid ISO loading.
- **Memory Management Subsystem**:
  - Higher-Half Direct Map (HHDM) physical memory access.
  - Custom Physical Frame Allocator parsing Limine memory map entries.
  - Dynamic 32 MB Kernel Heap Allocator (`linked_list_allocator`).
- **Interrupt Handling & CPU Protection**:
  - Global Descriptor Table (GDT) with 64-bit Kernel Code, Kernel Data, and Task State Segment (TSS).
  - Interrupt Stack Table (IST) protecting against double faults.
  - Complete Interrupt Descriptor Table (IDT) handling CPU exceptions (Divide-by-zero, Page Fault, GPF, Breakpoint).
  - Dual 8259 PIC remapped to vectors 32..47 with timer, keyboard, and mouse IRQ routing.
  - Programmable Interval Timer (PIT) calibrated at 100 Hz.

### 2. Device Drivers
- **PS/2 Keyboard Driver**: Event decoder with ring buffer supporting full US 104-key typing and command execution.
- **PS/2 Mouse Driver**: 3-button packet decoder with screen boundary clamping and graphical cursor compositing.
- **CMOS RTC Clock**: Direct I/O port reading providing real calendar date and 24-hour wall-clock time in system tray.
- **PCI Bus Enumerator**: Comprehensive configuration space scanner detecting vendor, device, class, and MMIO BARs.
- **Intel 82540EM (e1000) Gigabit NIC Driver**: Complete hardware driver featuring 32-entry circular RX/TX DMA descriptor rings, status polling, and automatic link detection.

### 3. Real Wi-Fi Discovery & Connection (Windows Host Bridge)
- **Virtual Hypervisor Challenge**: VirtualBox and QEMU do not expose host 802.11 physical Wi-Fi radios directly to virtual machines; they expose virtual Ethernet adapters (Intel e1000).
- **The Bridge Architecture**: A background Windows host helper (`host_bridge/WiFiBridge.ps1` / `START_WIFI_HELPER.bat`) queries the laptop's physical Wi-Fi adapter via Windows Native WLAN APIs (`netsh wlan show networks mode=bssid` and `netsh wlan show interfaces`).
- **UDP Inter-Process Protocol**:
  - Life OS sends scan/connect queries from guest `10.0.2.15` to host loopback `10.0.2.2:18888`.
  - Windows host replies with actual physical SSIDs, signal percentage, authentication type, band, and channel.
  - Real Wi-Fi connection handling: Life OS passes selected SSID and WPA2 key to host bridge, which creates a temporary profile and establishes connection via `netsh wlan connect`.
- **VirtualBox NAT Routing Requirement**:
  - VirtualBox NAT drops guest packets to host loopback by default. Life OS requires:
    ```powershell
    VBoxManage modifyvm "Life OS" --nat-localhostreachable1 on
    ```
- **Live Terminal & GUI Control**:
  - GUI: `Network & Wi-Fi Manager` with tabs, real network table, signal bars, and password box.
  - CLI: `wifi scan`, `wifi status`, `wifi connect <SSID> [password]`, `wifi disconnect`.
  - System Tray: Dynamic tray indicator displaying real connected SSID (e.g. `Vedav`) and signal bars.

### 4. Professional Graphical Desktop & Window Compositor
- **Linear Framebuffer**: 1024x768 @ 32 bpp truecolor rendering with software double-buffering.
- **8 Desktop Application Icons**: Web Browser, Terminal, File Manager, Network Manager, Games, System Info, Settings, Trash with 32x32 pixel art, single-click selection, double-click launch, and dragging.
- **Upgraded Taskbar**:
  - Life OS Start button.
  - Pinned quick-launch icons (Browser, Terminal, File Manager, Games).
  - Running window tabs with cyan active underline.
  - System tray with Ethernet LED, Wi-Fi SSID label, volume toggle, and real-time clock/date.
  - Clickable clock opening the calendar pop-up panel.
- **Desktop Right-Click Context Menu**: Refresh Desktop, Change Wallpaper, Arrange Icons, Create Folder, Open Terminal, Display Settings, System Info.
- **3 Selectable Wallpapers**: Cyber Grid (Default), Deep Cobalt Aurora, Midnight Slate (cycled via `F5` or context menu).
- **File Manager (LifeFS Explorer)**:
  - In-memory Virtual File System (LifeFS) rooted at `/home/shrivika`.
  - Folder navigation, file creation (`+ File`), folder creation (`+ Folder`), deletion, and built-in text preview overlay.
- **Comprehensive Hotkey Suite**:
  - `F1`: Toggle Start Menu
  - `F2`: Toggle Calendar Pop-up Panel
  - `F3`: Toggle Desktop Context Menu
  - `F4`: Close Active Window
  - `F5`: Cycle Desktop Wallpaper
  - `F6`: Cycle Active Window Focus
  - `F7`: Launch Games Suite
  - `F8`: Launch Web Browser
  - `F9`: Launch System Information
  - `F10`: Launch System Settings
  - `F12`: Maximize / Restore Active Window
  - `Esc`: Close Open Menus
  - `1..8`: Instant App Launch from Start Menu

---

## 📊 Strict ISO Size Verification

| Metric | Requirement | Measured Value | Status |
| :--- | :--- | :--- | :--- |
| **ISO Size** | Strictly < 100 MB | **3.52 MB** (3,690,496 bytes) | ✅ **PASS (96.5% below limit)** |
| **Kernel Binary** | Standalone ELF | **1.83 MB** (release mode) | ✅ **PASS** |
| **Boot Time** | Fast boot | **< 2 seconds** to desktop GUI | ✅ **PASS** |

---

## 🛠️ Quick Start & Build Instructions

### 1. Build the ISO (under WSL2 Ubuntu)
```bash
cd /home/vedav/life-os
./build.sh
```

### 2. Configure VirtualBox VM
```powershell
# Create VM and configure NAT host access
& 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe' modifyvm "Life OS" --nat-localhostreachable1 on
```

### 3. Launch Windows Wi-Fi Bridge Helper
Double click `host_bridge/START_WIFI_HELPER.bat` or run:
```powershell
powershell.exe -ExecutionPolicy Bypass -File "host_bridge\WiFiBridge.ps1"
```

### 4. Boot in VirtualBox
Start the VM `"Life OS"`. The operating system boots directly to the graphical desktop in under two seconds!

---

## 👤 Project Ownership
**Student Developer:** Shrivika  
**Institution:** College Final Year Engineering Project  
**Repository Branch:** `main`  
**License:** MIT License  
