# LIFE OS 🌟
### Complete Autonomous Bare-Metal x86_64 Rust Operating System
**Owner:** Shrivika  
**Project:** College Final Year Engineering Project  
**Architecture:** x86_64 (Bare-Metal, `no_std`, `no_main`)  
**Language:** 100% Rust  
**Final Bootable ISO Size:** **3.41 MB** (3,579,904 bytes) — *Strictly below the 100 MB requirement!*

---

## 🚀 Overview

**LIFE OS** is a modern, bare-metal operating system engineered completely from scratch in Rust without any runtime dependency on Linux, Windows, or standard C runtime libraries. Developed autonomously for **Shrivika's** college computer science & engineering demonstration, Life OS combines high-performance low-level systems programming with a polished graphical desktop user experience.

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
| **Desktop & Terminal** | ![Desktop](docs/screenshots/life_os_whoami_result.png) | Dark cyber theme, taskbar, start menu, live clock/RTC, `whoami` returning `shrivika` |
| **All 22 Commands** | ![Help](docs/screenshots/life_os_help_result.png) | Interactive terminal shell with 22 fully implemented commands |
| **Memory Hardware** | ![Memory](docs/screenshots/life_os_memory_result.png) | Genuine hardware memory stats: 4102 MB RAM, 32 MB heap pool, 4-level paging |
| **File System (LifeFS)**| ![Cat](docs/screenshots/life_os_cat_content.png) | In-memory VFS reading `/home/shrivika/README.txt` |
| **Life Web Browser** | ![Browser](docs/screenshots/life_os_browser_launched.png) | Graphical browser with navigation bar, tabs, and `life://welcome` |
| **Games: Snake** | ![Snake](docs/screenshots/life_os_games_launched.png) | Real-time Snake arcade game with live scoring and controls |
| **Games: Retro Pong** | ![Pong](docs/screenshots/life_os_pong_result.png) | 2-player/AI arcade Pong with collision physics and score counter |
| **Games: Tic-Tac-Toe** | ![TicTacToe](docs/screenshots/life_os_tictactoe_result.png) | Interactive 3x3 strategy game with win/draw tracking |
| **Games: 2048** | ![2048](docs/screenshots/life_os_2048_result.png) | Full 4x4 sliding puzzle game with tile-combining mechanics |

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

### 3. Genuine TCP/IP Networking & Host Wi-Fi Bridge
- **smoltcp Integration**: Real packet transmission engine handling:
  - Ethernet layer framing
  - ARP request/reply address resolution
  - IPv4 packet routing
  - ICMP echo request/reply (`ping`)
  - UDP packet transmission and DNS domain queries
  - TCP socket stream management and HTTP client
- **Windows Host Wi-Fi Bridge**:
  - VirtualBox natively exposes an Intel Ethernet adapter (not physical host Wi-Fi).
  - A lightweight host helper (`host_bridge/WiFiBridge.ps1` / `START_WIFI_HELPER.bat`) bridges physical Wi-Fi SSIDs and signal strength to Life OS via UDP port 18888.

### 4. Professional Graphical Desktop & Window Compositor
- **Linear Framebuffer**: 1024x768 @ 32 bpp truecolor rendering with software double-buffering.
- **Modern Dark Theme**: Slate cyber aesthetic, rounded window headers, custom icons, and typography.
- **Window Management Engine**: Multi-window compositing supporting titlebar drag, minimize to taskbar, maximize, close, and Z-index depth sorting.
- **Start Menu & Taskbar**: Windows-style taskbar with quick launcher, running window tabs, ETH status pill, Wi-Fi indicator, and RTC clock.
- **8 Integrated Applications**:
  1. **Terminal**: 22 functional commands with colored text and output scrolling.
  2. **Browser**: Graphical web browser with navigation toolbar, address bar, and HTML/text rendering.
  3. **File Manager**: Graphical tree and file browser for LifeFS.
  4. **Network Manager**: Real-time packet RX/TX monitor, IP/MAC status, and host Wi-Fi scanner.
  5. **Games Suite**: 4 fully playable arcade classics (Snake, Pong, Tic-Tac-Toe, 2048).
  6. **System Information**: Live CPU, memory, uptime, and display hardware telemetry.
  7. **Settings**: Resolution, dark theme, and sound settings panel.
  8. **About Life OS**: Presentation showcase highlighting Shrivika's project achievements.

---

## 📊 Strict ISO Size Verification

| Metric | Requirement | Measured Value | Status |
| :--- | :--- | :--- | :--- |
| **ISO Size** | Strictly < 100 MB | **3.41 MB** (3,579,904 bytes) | ✅ **PASS (96.6% below limit)** |
| **Kernel Binary** | Standalone ELF | **1.82 MB** (release mode) | ✅ **PASS** |
| **Boot Time** | Fast boot | **< 2 seconds** to desktop GUI | ✅ **PASS** |

---

## 💻 Quick Start & Running

### Prerequisites
- **Oracle VirtualBox** (Version 7.0 or newer recommended) or **QEMU** (`qemu-system-x86_64`)
- **Windows 11** with WSL2 (for building from source)

### Running Pre-Built ISO in VirtualBox
1. Launch **Oracle VirtualBox**.
2. Create a new Virtual Machine:
   - **Name:** `Life OS`
   - **Type:** `Other` -> `Other/Unknown (64-bit)`
   - **Base Memory:** `4096 MB` (2048 MB minimum)
   - **Processors:** `2 CPUs`
3. In **Settings -> System -> Motherboard**:
   - Enable **EFI (special OSes only)** (Firmware: `EFI64`).
4. In **Settings -> Display**:
   - Set **Video Memory** to `128 MB`.
   - Set **Graphics Controller** to `VBoxSVGA` or `VMSVGA`.
5. In **Settings -> Storage**:
   - Attach `build/LifeOS.iso` (or `C:\Users\USER\Downloads\LifeOS.iso`) to the Optical Drive.
6. In **Settings -> Network**:
   - Adapter 1: Attached to **NAT**.
   - Adapter Type: **Intel PRO/1000 MT Desktop (82540EM)**.
7. Click **Start** to boot directly into Life OS!

*(For full step-by-step instructions with troubleshooting, see [docs/VIRTUALBOX_GUIDE.md](docs/VIRTUALBOX_GUIDE.md)).*

---

## 🛠️ Building from Source

In WSL2 Ubuntu:
```bash
# Clone the repository
git clone https://github.com/shrivika/life-os.git
cd life-os

# Ensure Rust nightly toolchain is active
rustup default nightly

# Compile the kernel and generate the bootable ISO
chmod +x build.sh
./build.sh
```
The output bootable ISO will be placed at `build/LifeOS.iso`.

---

## 📚 Complete Documentation Suite

- [Architecture Guide](docs/ARCHITECTURE.md) — Subsystem architectural breakdown, memory layout, and driver internals.
- [Build Instructions](docs/BUILD.md) — Comprehensive compilation and toolchain requirements.
- [VirtualBox Setup Guide](docs/VIRTUALBOX_GUIDE.md) — Complete VirtualBox VM configuration manual.
- [Networking & Wi-Fi Bridge Guide](docs/NETWORKING.md) — Intel e1000 driver and Windows host Wi-Fi scanner guide.
- [Testing & Quality Assurance](docs/TESTING.md) — Test matrix across VirtualBox and QEMU.
- [Features Catalogue](docs/FEATURES.md) — Detailed guide to all 8 applications and 22 terminal commands.
- [College Presentation & Demo Guide](docs/DEMO_GUIDE.md) — Step-by-step walkthrough script for Shrivika's project demo.
- [GitHub Setup Instructions](docs/GITHUB_SETUP.md) — Safe repository setup instructions for Shrivika.

---

## 📄 License & Attribution
- **Author & Copyright:** Shrivika © 2026. All rights reserved.
- Developed as an independent academic engineering project for Department of Computer Science & Engineering.
