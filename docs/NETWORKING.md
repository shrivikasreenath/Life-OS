# LIFE OS Networking & Wi-Fi Architecture Guide
**Author:** Shrivika  
**NIC Driver:** Intel 82540EM (e1000) Gigabit Controller  
**TCP/IP Engine:** `smoltcp` Bare-Metal Network Stack  
**Host Integration:** Windows Host Wi-Fi Bridge (`WiFiBridge.ps1`)  

---

## 1. Network Architecture Overview

Life OS solves one of the hardest problems in bare-metal hypervisor operating systems: **genuine dual-mode internet and Wi-Fi connectivity**.

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
|  | - DHCP Client (Auto-Config) |               | - Broadcasts queries to Host:18888   | |
|  | - ARP Cache & Address Resol |               | - Receives live Wi-Fi SSID lists     | |
|  | - IPv4 / ICMP Ping / DNS    |               | - Parses RSSI dBm & Auth protocols   | |
|  | - TCP Sockets (HTTP Client) |               +------------------+-------------------+ |
|  +--------------+--------------+                                  |                     |
|                 |                                                 |                     |
|                 v                                                 |                     |
|  +----------------------------------------------------------------+------------------+  |
|  |             Intel 82540EM (e1000) Gigabit Controller Driver                       |  |
|  |             - 32-entry circular RX/TX DMA rings (2KB buffers)                     |  |
|  |             - MMIO BAR0 register operations (CTRL, RCTL, TCTL)                    |  |
|  +-----------------------------------------------------------------------------------+  |
+--------------------------------------------|--------------------------------------------+
                                             | Virtual Ethernet Frames
                                             v
+-----------------------------------------------------------------------------------------+
|                                ORACLE VIRTUALBOX NAT                                    |
|                       Virtual Switch Gateway: 10.0.2.2 / 10.0.2.3                       |
+--------------------------------------------|--------------------------------------------+
                                             | NAT Traffic / UDP Port 18888
                                             v
+-----------------------------------------------------------------------------------------+
|                                  WINDOWS HOST SYSTEM                                    |
|  +---------------------------------------+   +---------------------------------------+  |
|  | Physical Laptop Internet Uplink       |   | Windows Wi-Fi Bridge Helper           |  |
|  | (NAT forwarding guest HTTP/DNS)       |   | (`START_WIFI_HELPER.bat`)             |  |
|  |                                       |   | - Queries Windows Native WLAN APIs    |  |
|  |                                       |   | - Answers guest UDP queries on 18888  |  |
|  +---------------------------------------+   +---------------------------------------+  |
+-----------------------------------------------------------------------------------------+
```

---

## 2. The Hypervisor Limitation & Engineering Solution

### The Technical Challenge
In Oracle VirtualBox (and VMware/QEMU), hypervisors expose a **virtual Ethernet network adapter** (such as Intel PRO/1000 MT) to guest operating systems. They **do not** expose the physical Wi-Fi radio or IEEE 802.11 management frames directly to the guest unless an external dedicated USB Wi-Fi dongle is passed through.

### The Honest Engineering Solution
Rather than simulating fake or hardcoded Wi-Fi networks:
1. **Virtual Ethernet Mode**: Life OS provides a genuine Intel 82540EM hardware driver and `smoltcp` TCP/IP stack to communicate across the VirtualBox NAT virtual switch.
2. **Windows Host Wi-Fi Bridge**: Life OS connects via UDP to a companion host utility (`WiFiBridge.ps1`) running on the Windows host. This utility scans real physical Wi-Fi access points in the room using Windows `netsh wlan` / Native Wifi APIs and transmits the live SSIDs, signal strengths, and security ciphers to Life OS.

---

## 3. Intel 82540EM (e1000) Hardware Driver

Located in `src/drivers/e1000.rs`:

### 1. PCI Discovery & BAR Mapping
- Scans PCI bus for Vendor ID `0x8086` and Device ID `0x100E`.
- Reads `BAR0` to obtain the physical MMIO base address (typically `0xe8600000`).
- Translates the physical address to virtual address space using Higher-Half Direct Map (HHDM).

### 2. MAC Address Acquisition
The hardware MAC address is read from the Receive Address registers:
- Low 32 bits: Offset `0x5400` (`RAL0`)
- High 16 bits: Offset `0x5404` (`RAH0`)

### 3. Circular DMA Descriptor Rings
- **Receive (RX) Ring**:
  - Allocated in physical frames.
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
2. **ARP (Address Resolution Protocol)**: Dynamically resolves IPv4 addresses to Ethernet MAC addresses.
3. **IPv4**: Packet routing, fragmentation handling, and TTL decrements.
4. **ICMP**: Echo Request and Echo Reply handling for the `ping` terminal command.
5. **UDP**: Datagram delivery for DNS queries and Wi-Fi host bridge communication.
6. **TCP**: Three-way handshake (`SYN`, `SYN-ACK`, `ACK`), sequence number tracking, flow control, and HTTP client.
7. **DHCP**: Automatic network parameter negotiation on VirtualBox NAT (assigns `10.0.2.15`, gateway `10.0.2.2`, DNS `10.0.2.3`).

---

## 5. Windows Host Wi-Fi Bridge Setup

The host bridge files are located in the `host_bridge/` directory:
- `host_bridge/WiFiBridge.ps1` — PowerShell UDP server querying `netsh wlan show networks mode=bssid`.
- `host_bridge/START_WIFI_HELPER.bat` — 1-click batch launcher for Windows.

### How to Run the Host Helper
1. On the Windows host laptop, open file explorer and navigate to:
   `C:\Users\USER\life-os\host_bridge\` (or project folder).
2. Double-click `START_WIFI_HELPER.bat`.
3. The helper starts listening on UDP port `18888`.
4. In Life OS, open **Network Manager** or run `network` in the terminal:
   - Life OS queries `10.0.2.2:18888` via UDP.
   - The host responds with live nearby Wi-Fi SSIDs and signal strengths.
   - Life OS displays the real physical Wi-Fi networks in the GUI!
