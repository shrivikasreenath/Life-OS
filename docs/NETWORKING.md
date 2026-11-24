# LIFE OS Networking & Wi-Fi Architecture Guide
**Author:** Shrivika  
**NIC Driver:** Intel 82540EM (e1000) Gigabit Controller  
**TCP/IP Engine:** `smoltcp` Bare-Metal Network Stack  
**Host Integration:** Windows Host Wi-Fi Bridge (`host_bridge/WiFiBridge.ps1`)  

---

## 1. Network Architecture Overview

Life OS solves one of the hardest problems in bare-metal hypervisor operating systems: **genuine dual-mode internet and physical Wi-Fi connectivity**.

```
+-----------------------------------------------------------------------------------------+
|                                    LIFE OS GUEST                                        |
|  +-----------------------------+               +--------------------------------------+ |
|  | Applications (Browser, Ping)|               | Network Manager (GUI & CLI)          | |
|  +--------------+--------------+               +------------------+-------------------+ |
|                 |                                                 |                     |
|                 v                                                 v                     |
|  +-----------------------------+               +--------------------------------------+ |
|  | smoltcp TCP/IP Stack        |               | Host Bridge UDP Client               | |
|  | - DHCP Client (Auto-Config) |               | - Queries Host Loopback: 10.0.2.2    | |
|  | - ARP Cache & Address Resol |               | - Commands: SCAN, CONNECT, STATUS    | |
|  | - IPv4 / ICMP Ping / DNS    |               | - UDP Socket Port 18888              | |
|  | - TCP Sockets (HTTP Client) |               +------------------+-------------------+ |
|  +--------------+--------------+                                  |                     |
|                 |                                                 |                     |
|                 v                                                 |                     |
|  +----------------------------------------------------------------+------------------+  |
|  |             Intel 82540EM (e1000) Gigabit Controller Driver                       |  |
|  |             - 32-entry circular RX/TX DMA rings (2KB physical buffers)            |  |
|  |             - MMIO BAR0 register operations (CTRL, RCTL, TCTL, STATUS)            |  |
|  +-----------------------------------------------------------------------------------+  |
+--------------------------------------------|--------------------------------------------+
                                             | Virtual Ethernet Frames
                                             v
+-----------------------------------------------------------------------------------------+
|                                ORACLE VIRTUALBOX NAT                                    |
|              Virtual Switch Gateway: 10.0.2.2 / DNS: 10.0.2.3                           |
|              Setting: --nat-localhostreachable1 on (Allows guest -> host UDP)           |
+--------------------------------------------|--------------------------------------------+
                                             | UDP Datagrams to 127.0.0.1:18888
                                             v
+-----------------------------------------------------------------------------------------+
|                                  WINDOWS HOST SYSTEM                                    |
|  +---------------------------------------+   +---------------------------------------+  |
|  | Physical Laptop Internet Uplink       |   | Windows Wi-Fi Bridge Helper           |  |
|  | (NAT forwarding guest HTTP/DNS)       |   | (`START_WIFI_HELPER.bat`)             |  |
|  |                                       |   | - Queries Windows Native WLAN APIs    |  |
|  |                                       |   | - Answers guest UDP queries on 18888  |  |
|  |                                       |   | - Executes netsh wlan connect/profile |  |
|  +---------------------------------------+   +---------------------------------------+  |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Hypervisor Constraints & Honest Root-Cause Engineering

### The Core Problem
VirtualBox and QEMU do not expose physical 802.11 Wi-Fi radios to guest virtual machines. Guests only see a virtual Ethernet controller (`82540EM`). In earlier builds of Life OS, the Network Manager reported `HostBridge_Waiting` instead of live networks.

### Root-Cause Investigation & Fixes
1. **VirtualBox NAT Host Isolation**:
   - *Issue:* By default, VirtualBox NAT security drops any guest packets sent to the host loopback `10.0.2.2`.
   - *Fix:* Executed `VBoxManage modifyvm "Life OS" --nat-localhostreachable1 on` to allow the NAT engine to route guest UDP datagrams directly to `127.0.0.1:18888`.
2. **e1000 PHY Auto-Negotiation Delay**:
   - *Issue:* After software reset (`CTRL.RST`), the e1000 PHY takes ~20 ms to establish link. Checking `is_link_up()` immediately after reset returned `false`, causing `src/net/stack.rs` to abort network stack initialization.
   - *Fix:* Added PHY stabilization delay and made `is_link_up()` read the live hardware `STATUS` register (`0x0008`, bit 1). Initialized `NET_STACK` whenever the NIC hardware driver is present.
3. **e1000 RX Ring Index Desynchronization**:
   - *Issue:* `receive()` used `let next = (self.rx_cur + 1) % NUM_DESC;` which checked descriptor 1 on the first packet, skipping descriptor 0 and desynchronizing the ring.
   - *Fix:* Aligned descriptor processing to inspect `self.rx_cur`, update tail pointer `RDT = cur`, and advance `self.rx_cur = (cur + 1) % NUM_DESC`.
4. **WLAN API Command Timeout**:
   - *Issue:* `netsh wlan connect` requires up to 4.5 seconds to configure an XML profile, signal the physical Wi-Fi adapter, and verify interface state. The 3.5s timeout expired before the response arrived.
   - *Fix:* Extended `send_host_wifi_command` timeout to `6500ms` in `src/net/stack.rs`.
5. **Whitespace-Tolerant SSID Parsing**:
   - *Issue:* `WiFiBridge.ps1` regex expected SSIDs anchored at the start of the line, dropping indented networks.
   - *Fix:* Updated regex to `^\s*SSID\s+\d+\s*:\s*(.+)$` and added full extraction for Signal %, Authentication, Band (2.4 GHz / 5 GHz), Channel, and Connected state.

---

## 3. Intel 82540EM (e1000) Hardware Driver

Located in `src/drivers/e1000.rs`:

### 1. PCI Discovery & BAR Mapping
- Scans PCI configuration space for Vendor ID `0x8086` and Device ID `0x100E`.
- Reads `BAR0` to obtain the physical MMIO base address (`0xe8600000`).
- Translates the physical address to virtual address space using Higher-Half Direct Map (HHDM).

### 2. MAC Address Acquisition
The hardware MAC address is read from the Receive Address registers:
- Low 32 bits: Offset `0x5400` (`RAL0`)
- High 16 bits: Offset `0x5404` (`RAH0`)

### 3. Circular DMA Descriptor Rings
- **Receive (RX) Ring**:
  - 32 descriptors (`RxDesc`), each pointing to a dedicated 2048-byte physical receive buffer.
  - Registers: `RDBAL`/`RDBAH` (Base), `RDLEN` (Length = `32 * 16`), `RDH` (Head), `RDT` (Tail).
  - Packet reception verified via Descriptor Done (`DD`, bit 0) in the status byte.
- **Transmit (TX) Ring**:
  - 32 descriptors (`TxDesc`), each pointing to a dedicated 2048-byte physical transmit buffer.
  - Registers: `TDBAL`/`TDBAH` (Base), `TDLEN` (Length = `32 * 16`), `TDH` (Head), `TDT` (Tail).
  - Packets transmitted with End-Of-Packet (`EOP`) and Report-Status (`RS`) flags.

---

## 4. `smoltcp` TCP/IP Protocol Stack

Located in `src/net/smoltcp_adapter.rs` and `src/net/stack.rs`:

### Supported Protocols
1. **Ethernet II**: Framing and hardware MAC header encapsulation.
2. **ARP**: Address Resolution Protocol dynamically mapping IPv4 to MAC addresses.
3. **IPv4**: Packet routing and TTL tracking.
4. **ICMP**: Echo Request/Reply handling for `ping`.
5. **UDP**: Datagram transmission for DNS and host bridge communication.
6. **TCP**: Three-way handshake (`SYN`, `SYN-ACK`, `ACK`), sequence tracking, flow control, and HTTP client.
7. **DHCP**: Automatic network parameter negotiation on VirtualBox NAT (assigns `10.0.2.15`, gateway `10.0.2.2`, DNS `10.0.2.3`).

---

## 5. Host Bridge Protocol & Commands

The Windows helper (`host_bridge/WiFiBridge.ps1`) listens on UDP port `18888` on the host:

| Command | Direction | Description | Response Format |
| :--- | :--- | :--- | :--- |
| `SCAN` | Guest -> Host | Query physical Wi-Fi networks | `SSID\|SIGNAL%\|SECURITY\|BAND\|CHANNEL\|CONNECTED\n...` |
| `STATUS` | Guest -> Host | Query current Wi-Fi interface state | `CONNECTED\|SSID\|SIGNAL\|SECURITY\|BAND\|CHANNEL` |
| `CONNECT <ssid>\|<pwd>` | Guest -> Host | Connect to Wi-Fi with WPA2 key | `SUCCESS: Connected to <ssid>` or `FAILED: ...` |
| `DISCONNECT` | Guest -> Host | Disconnect physical Wi-Fi | `SUCCESS: Disconnected` |

### CLI Terminal Commands
Users can manage Wi-Fi directly from the terminal shell:
- `wifi scan` — Discovers physical Wi-Fi networks in the room.
- `wifi status` — Prints live interface connection telemetry.
- `wifi connect <SSID> [password]` — Initiates connection through host bridge.
- `wifi disconnect` — Disconnects from current Wi-Fi network.

### GUI Integration
- Opening **Network & Wi-Fi Manager** displays the `Wi-Fi (Host Bridge)` tab.
- Pressing `Scan Now` (or `s`/`r`) polls the host and populates the real-time table.
- Selecting a network row highlights it in cyan and populates the connection box.
- Passwords are typed into the masked input field with `Connect` / `Disconnect` buttons.
- The taskbar system tray automatically updates with the real SSID name (e.g. `Vedav`) and signal bars!

---

## 6. VirtualBox Freeze Elimination & Non-Blocking State Machine

### 1. Root Cause Analysis
During initial tests in Oracle VirtualBox, clicking **Scan Now** or **Connect** caused the entire guest operating system to freeze:
1. **Re-entrant Mutex Deadlock**:
   - In `src/gui/mod.rs`, `DESKTOP.lock()` was acquired during `update_and_render()` and held across `desktop.handle_mouse(mouse)`.
   - When the user clicked on a window, `desktop.rs` called `crate::apps::dispatch_click(...)` which called `NETWORK_MANAGER.lock().handle_click(...)`.
   - Inside `scan_wifi()`, `connect_selected()`, and `disconnect_selected()`, the code called `let mut desktop = crate::gui::desktop::DESKTOP.lock();`.
   - Because `spin::Mutex` is not re-entrant, this single-threaded spinlock immediately deadlocked the CPU forever.
2. **Synchronous GUI Starvation**:
   - The network send function ran a blocking `while pit::uptime_ms() - start < 6500` loop calling `pit::sleep_ms(15)`.
   - Because this loop executed directly inside the mouse click handler within the GUI frame, the entire GUI event loop was halted, completely freezing the cursor and screen repaints.

### 2. Architectural Solution
1. **Complete Decoupling of `DESKTOP.lock()`**:
   - Removed all `DESKTOP.lock()` calls from `network_manager.rs`.
   - In `desktop.rs`, `handle_mouse` now returns `Option<(AppId, i32, i32, u32, u32)>` for pending window clicks, releasing `DESKTOP.lock()` before dispatching the click to the application.
   - The system tray Wi-Fi status indicator queries a lock-free helper `get_wifi_tray_status()` via `NETWORK_MANAGER.try_lock()`, ensuring zero lock contention or re-entrancy.
2. **Non-Blocking Asynchronous State Machine**:
   - Replaced synchronous waits with `enum WifiOpState`:
     - `Idle`
     - `Scanning { handle, start_ms, anim_frame }`
     - `Connecting { handle, start_ms, anim_frame }`
     - `Disconnecting { handle, start_ms }`
   - Non-blocking UDP primitives (`start_host_wifi_command`, `check_host_wifi_response`, `cancel_host_wifi_command`) initiate requests and poll without sleeping.
   - The state machine is driven frame-by-frame by `update_apps()` called in `src/main.rs`.
   - The GUI desktop continues rendering at full 60 FPS throughout any scan, connect, or disconnect operation.
   - Live animated status spinners (`[|]`, `[/]`, `[-]`, `[\]`) indicate in-flight operations.
3. **Strict Timeouts & Graceful Error Recovery**:
   - Strict timeouts: 4.0s for scans, 7.5s for connections, 3.5s for disconnections.
   - Incomplete or missing responses clean up UDP sockets automatically and output human-readable messages (e.g. `Scan timeout: Windows Wi-Fi Bridge did not respond. Check START_WIFI_HELPER.`).
   - Failed scans are immediately retryable without rebooting the OS.
4. **Enhanced Host Bridge (`WiFiBridge.ps1`)**:
   - Deduplicates multiple BSSIDs per SSID (keeping highest signal).
   - Generates escaped XML profiles for both Open and WPA2-Personal networks.
   - Accurately reports `[Hidden Network]` for blank SSIDs.
   - Detects offline or disabled host adapters gracefully.
