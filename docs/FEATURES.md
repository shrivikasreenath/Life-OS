# LIFE OS Feature Catalogue & User Manual
**Author:** Shrivika  
**Desktop Environment:** Life Desktop Compositor (1024x768x32bpp)  
**Integrated Applications:** 8 GUI Applications + 22 CLI Commands  

---

## 1. Graphical Desktop Environment

The Life OS desktop is engineered with a modern, high-contrast dark theme optimized for technical clarity and demonstration:

### Desktop Components
1. **Procedural Wallpaper**: Dark navy/slate geometric background with anti-aliased cross-hatchings rendered in pure software.
2. **Bottom Taskbar (36px)**:
   - **Start Button**: Cyan-accented menu trigger with custom vector Life OS logo.
   - **Running Task Tabs**: 140px window pills showing active/minimized status, application icons, and titles.
   - **System Tray**:
     - Ethernet Link LED (Green = Active, Red = Disconnected).
     - Wi-Fi signal icon.
     - Real-Time Clock & Calendar (HH:MM:SS / YYYY-MM-DD) read directly from hardware RTC.
3. **Start Menu**:
   - User profile section: Avatar `S`, name `Shrivika`, handle `shrivika@life-os`.
   - Application launcher list with pixel icons.
   - Quick shutdown and reboot action buttons.
4. **Window Manager**:
   - Double-buffered compositing preventing screen tearing.
   - Glowing cyan active window outline.
   - Titlebar controls: Minimize (Yellow), Maximize (Cyan), Close (Red).
   - Dynamic window dragging and Z-index depth layering.

---

## 2. Integrated GUI Applications

### 1. Terminal (`shrivika@life-os`)
A full-featured terminal shell supporting 22 bare-metal commands, colored command prompts, unknown command handling, and output scrolling.
- Run `help` in terminal for the complete command list.
- Run `whoami` to verify ownership (`shrivika`).

### 2. Life Web Browser
A graphical lightweight web browser designed for local documentation and intranet browsing.
- **Navigation Controls**: Back (`<`), Forward (`>`), Reload (`R`), Home (`H`).
- **Address Bar**: Type URLs or internal schemes (`life://welcome`).
- **Welcome Page**: Displays detailed technical breakdown of Life OS kernel capabilities.

### 3. File Manager (LifeFS Explorer)
A tree-view graphical file browser for the in-memory Virtual File System.
- Browse directories: `/home/shrivika`, `/home/shrivika/Documents`, `/home/shrivika/Projects`, `/etc`, `/system`.
- View file metadata (name, size, type).
- Create new text files and view file contents in real-time.

### 4. Network & Wi-Fi Manager
A dual-mode network diagnostic and scanning utility.
- **Ethernet Tab**: Real-time packet RX/TX counters, MAC address, IP address, and gateway.
- **Wi-Fi Scanner Tab**: Interacts with the Windows host bridge to display physical wireless access points, BSSIDs, signal bars, and encryption types.

### 5. Games Suite (4-in-1 Arcade)
Four classic playable games running bare-metal on the kernel timer:
1. **Snake Classic**:
   - Arrow keys: Steer snake.
   - `R` key: Restart game.
   - Eat red food dots to grow and increase score.
2. **Retro Pong**:
   - `W` / `Up Arrow`: Move left paddle up.
   - `S` / `Down Arrow`: Move left paddle down.
   - AI controls the right paddle; first to 7 points wins.
3. **Tic-Tac-Toe**:
   - Arrow keys / numbers `1..9`: Select square.
   - `Space` / `Enter`: Place marker (X).
   - Smart CPU opponent places O.
4. **2048 Puzzle**:
   - Arrow keys: Slide tiles in 4 directions.
   - Tiles with identical numbers merge (2+2=4, 4+4=8, ...).

### 6. System Information
Provides live telemetry read directly from CPU registers and memory managers:
- Processor model, family, and hardware feature flags (TSC, MSR, SSE, etc.).
- Total physical memory, usable RAM, and free frame counts.
- Dynamic heap pool allocation statistics.
- System uptime in seconds and timer ticks.

### 7. System Settings
Configuration panel for user preferences:
- Display resolution toggle (1024x768 / 800x600).
- Color theme options (Cyber Slate Dark, Cobalt Blue, Midnight Violet).
- Sound and alert effects toggle.

### 8. About Life OS
A college presentation card showcasing:
- Project Name: **LIFE OS**
- Author: **Shrivika**
- Department: Computer Science & Engineering
- Core architectural achievements and ISO size benchmarks.

---

## 3. Terminal Commands Reference

| Command | Usage | Description |
| :--- | :--- | :--- |
| `help` | `help` | Prints the complete catalogue of 22 commands. |
| `whoami` | `whoami` | Displays current active user identity (`shrivika`). |
| `uname` | `uname` | Prints operating system and kernel version info. |
| `clear` | `clear` | Clears all current lines from the terminal screen. |
| `pwd` | `pwd` | Prints the present working directory. |
| `ls` | `ls [path]` | Lists files and folders in current or specified path. |
| `cd` | `cd <path>` | Changes working directory (`..` for parent, `~` for home). |
| `cat` | `cat <file>` | Prints the text contents of a file to terminal. |
| `echo` | `echo <text>` | Echoes back arbitrary text to the screen. |
| `date` | `date` | Displays current calendar date from CMOS RTC. |
| `time` | `time` | Displays current 24-hour time from CMOS RTC. |
| `memory` | `memory` | Displays physical RAM, usable RAM, heap pool, and paging. |
| `cpu` | `cpu` | Displays processor architecture, model, and CPUID features. |
| `network` | `network` | Displays Intel 82540EM hardware MAC, link state, and packet stats. |
| `ifconfig`| `ifconfig` | Displays IP address, netmask, gateway, and DHCP status. |
| `ping` | `ping <ip>` | Tests network reachability with ICMP echo requests. |
| `dns` | `dns <host>` | Queries DNS resolver for IP address of given domain. |
| `browser` | `browser` | Launches the Life Web Browser GUI window. |
| `games` | `games` | Launches the 4-in-1 Arcade Games Suite GUI window. |
| `about` | `about` | Opens the About Life OS college presentation window. |
| `version` | `version` | Displays the kernel version, compiler build, and target. |
| `reboot` | `reboot` | Safely resets the CPU and restarts the computer. |
| `shutdown`| `shutdown` | Powers down the system hardware. |
