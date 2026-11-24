# LIFE OS Feature Catalogue & User Manual
**Author:** Shrivika  
**Desktop Environment:** Life Desktop Compositor (1024x768x32bpp)  
**Integrated Applications:** 8 GUI Applications + 23 CLI Commands  

---

## 1. Graphical Desktop Environment

The Life OS desktop is engineered with a modern, high-contrast dark theme optimized for technical clarity and demonstration:

### Desktop Components
1. **Desktop Application Icons**:
   - 8 major applications placed directly on the desktop with 32x32 pixel art icons and clean labels:
     1. **Web Browser** (Globe icon)
     2. **Terminal** (Green terminal prompt icon)
     3. **File Manager** (Golden folder icon)
     4. **Network Manager** (Green signal bars icon)
     5. **Games Suite** (Purple gamepad controller icon)
     6. **System Information** (Cyan microchip icon)
     7. **Settings** (Slate gear/cog icon)
     8. **Trash** (Slate wastebasket icon)
   - Single-click to select (highlights with cyan bounding outline).
   - Double-click to launch the application.
   - Drag-and-drop icon movement across the desktop with session persistence and boundary clamping.
2. **Procedural Wallpaper Engine**:
   - 3 switchable aesthetic wallpapers (cycled via `F5` or Desktop Context Menu):
     1. **Cyber Grid** (Default): Jet black grid with cyan coordinate lines and top branding.
     2. **Deep Cobalt Aurora**: Deep oceanic gradient with horizontal aurora rays.
     3. **Midnight Slate**: Obsidian gradient with subtle starfield dots.
3. **Upgraded Taskbar (38px)**:
   - **Start Button**: Cyan-accented menu trigger with custom vector Life OS logo.
   - **Pinned Quick Launch**: Browser, Terminal, File Manager, Games.
   - **Running Task Tabs**: Window pills showing active/minimized status, application icons, titles, and active cyan underline.
   - **System Tray**:
     - Ethernet Link LED (Green = Active, Red = Disconnected).
     - Wi-Fi indicator + live connected SSID label (e.g. `Vedav`).
     - Volume speaker icon with mute toggle indicator.
     - Real-Time Clock & Date (HH:MM:SS / YYYY-MM-DD) read directly from hardware RTC.
     - Clickable clock that opens the **Calendar Pop-up Panel**.
4. **Calendar Pop-up Panel**:
   - Displays real-time 24h clock, full month date, day-of-week headers (`Su Mo Tu We Th Fr Sa`), and calendar day grid with current day highlighted in vibrant cyan.
   - Opened by clicking the taskbar clock or pressing `F2`.
5. **Start Menu**:
   - User profile section: Avatar `S`, name `Shrivika`, handle `shrivika@life-os`.
   - Application launcher list with pixel icons and number shortcuts `1..8`.
   - Power options: `Shutdown` and `Reboot` action buttons.
   - Opened by clicking Start or pressing `F1`.
6. **Desktop Right-Click Context Menu**:
   - Opened by right-clicking on the desktop or pressing `F3`.
   - Actions:
     - `Refresh Desktop`: Deselects icons and clears stale selection states.
     - `Change Wallpaper`: Cycles between the 3 procedural wallpaper themes.
     - `Arrange Icons`: Re-aligns all desktop icons into an orderly grid column.
     - `Create Folder`: Creates a new directory inside `/home/shrivika`.
     - `Open Terminal`: Launches the terminal shell.
     - `Display Settings`: Opens System Settings.
     - `System Info`: Opens the System Information dashboard.
7. **Window Manager Engine**:
   - Double-buffered compositing preventing screen tearing.
   - Active window glowing cyan border.
   - Titlebar controls: Minimize (Yellow), Maximize / Restore (Cyan), Close (Red).
   - Smooth titlebar dragging and Z-index depth layering.

---

## 2. Integrated GUI Applications

### 1. Terminal (`shrivika@life-os`)
A full-featured terminal shell supporting 23 bare-metal commands, colored command prompts, unknown command handling, and output scrolling.
- Run `help` in terminal for the complete command list.
- Run `whoami` to verify ownership (`shrivika`).
- Run `wifi scan` to scan physical Wi-Fi networks via Windows Host Bridge.

### 2. Life Web Browser
A graphical lightweight web browser designed for local documentation and intranet browsing.
- **Navigation Controls**: Back (`<`), Forward (`>`), Reload (`R`), Home (`H`).
- **Address Bar**: Type URLs or internal schemes (`life://welcome`).
- **Welcome Page**: Displays detailed technical breakdown of Life OS kernel capabilities.

### 3. File Manager (LifeFS Explorer)
A tree-view graphical file browser for the in-memory Virtual File System (LifeFS).
- Browse directories: `/home/shrivika`, `/home/shrivika/Documents`, `/home/shrivika/Projects`, `/etc`, `/system`.
- Toolbar buttons:
  - `+ File` (or `n`/`f` key): Create new text file.
  - `+ Folder` (or `d`/`m` key): Create new directory.
  - `Delete` (or `x`/`Del` key): Remove selected file or directory.
  - `Up ..` (or `u`/`Backspace` key): Navigate to parent folder.
- Built-in text viewer: Opening any text file (e.g. `README.txt`, `Kernel_Architecture.txt`) presents an in-window text preview overlay with a close button (`Esc` or `q`).
- *Note:* LifeFS is an in-memory virtual filesystem; changes persist throughout the boot session.

### 4. Network & Wi-Fi Manager
A dual-mode network diagnostic and real Wi-Fi management utility.
- **Ethernet Tab**: Real-time packet RX/TX counters, MAC address, IP address, and gateway.
- **Wi-Fi (Host Bridge) Tab**:
  - Connects to the Windows host bridge over UDP port 18888.
  - Displays real physical Wi-Fi networks in the room with SSID, signal strength bar + %, band (2.4/5GHz), channel, security type, and connection state.
  - Interactive selection row, masked password input box, `Connect` and `Disconnect` buttons.

### 5. Games Suite (4-in-1 Arcade)
Four classic playable games running bare-metal on the kernel timer:
1. **Snake Classic** (`1`): Steer snake with arrow keys, eat food dots, avoid walls.
2. **Retro Pong** (`2`): Move paddle with `W`/`S` or `Up`/`Down`, play against CPU.
3. **Tic-Tac-Toe** (`3`): Select cell with arrow keys or `1..9`, place marker with `Space`/`Enter`.
4. **2048 Puzzle** (`4`): Slide numbered tiles with arrow keys to combine like numbers and reach 2048.

### 6. System Information Dashboard
Real-time hardware telemetry:
- Operating System & Owner identity (`Shrivika`).
- CPU architecture and enabled features.
- Physical RAM (4102 MB total, 4074 MB usable, 4042 MB free).
- Dynamic Kernel Heap Pool (32 MB).
- Display framebuffer specifications (1024x768 @ 32bpp ARGB).
- Network interface state (IPv4, MAC, Uptime ticks).

### 7. System Settings Panel
System configuration controls:
- Display resolution info.
- Desktop theme options.
- CMOS RTC clock status.
- Filesystem subsystem stats.
- System power actions (`Shutdown` and `Reboot`).

### 8. About Life OS
College presentation showcase highlighting Shrivika's architecture, microkernel design, and technical achievements.

---

## 3. Global Hotkey Reference

| Key | Action |
| :--- | :--- |
| `F1` | Toggle Start Menu |
| `F2` | Toggle Calendar Pop-up Panel |
| `F3` | Toggle Desktop Context Menu |
| `F4` | Close Active Window |
| `F5` | Cycle Desktop Wallpaper Theme |
| `F6` | Cycle Active Window Focus |
| `F7` | Launch Games Suite |
| `F8` | Launch Web Browser |
| `F9` | Launch System Information |
| `F10` | Launch System Settings |
| `F12` | Maximize / Restore Active Window |
| `Esc` | Close Open Menus |
| `1..8` | Launch Corresponding App when Start Menu is open |
