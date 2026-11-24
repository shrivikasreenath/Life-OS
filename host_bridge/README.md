# Life OS — Windows Wi-Fi Host Bridge

## Architecture & Technical Background

In virtualization platforms like **Oracle VirtualBox** and **QEMU**, the hypervisor creates an emulated **PCI Ethernet Network Interface Card** (specifically the **Intel 82540EM / PRO/1000 MT Desktop**) attached to the virtual machine.

The guest operating system (Life OS) communicates directly with this virtual Ethernet NIC via PCI and MMIO, transmitting real Ethernet frames through VirtualBox NAT to the outside world.

Because VirtualBox abstracts the laptop's physical Wi-Fi hardware behind this virtual Ethernet device, the guest kernel cannot directly access 802.11 wireless radio management frames (scanning, beacon frames, BSSID probes) on the host's physical Wi-Fi chip without host assistance.

To solve this hardware boundary genuinely without faking or simulating data, Life OS uses a **Host-Side Wi-Fi Bridge**:

1. **Host Helper (`WiFiBridge.ps1`)**:
   - Runs natively on Windows 11.
   - Accesses real physical Wi-Fi scanning APIs via `netsh wlan show networks mode=bssid` and `netsh wlan show interfaces`.
   - Listens on UDP port `18888` on the VirtualBox NAT host interface (`10.0.2.2`).
2. **Guest Network Stack (`src/net/stack.rs`)**:
   - Life OS Network Manager sends a UDP query packet `SCAN` to `10.0.2.2:18888`.
   - The host helper replies with the actual scanned wireless networks, real SSIDs, real signal strengths (%), and security protocols.
3. **Connection Requests**:
   - When Shrivika selects a network in Life OS and requests connection, Life OS sends `CONNECT <SSID>`.
   - Windows attempts to associate with the network, and the helper confirms whether Windows succeeded before reporting the status back to Life OS.

## Security & Protocol Specification

- **Port**: UDP 18888
- **Payloads**:
  - `SCAN`: Requests nearby physical Wi-Fi networks.
  - `STATUS`: Queries current Wi-Fi connection and signal strength.
  - `CONNECT <SSID>`: Requests host association with the designated network.
- **Safety**: The helper only accepts predefined commands (`SCAN`, `STATUS`, `CONNECT`) and strictly sanitizes input to prevent arbitrary command execution.

## Usage Instructions

1. Double-click `START_WIFI_HELPER.bat` on the Windows host before starting VirtualBox.
2. Inside Life OS, open the **Network Manager** application.
3. Switch to the **Wi-Fi (Host Bridge)** tab and click **Scan Now**.
4. Real nearby networks will appear with signal meters and connection status!
