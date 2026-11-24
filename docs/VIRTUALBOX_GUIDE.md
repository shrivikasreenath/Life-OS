# Oracle VirtualBox Configuration & Deployment Guide for LIFE OS
**Author:** Shrivika  
**Target:** Oracle VirtualBox 7.0 / 7.1  
**Tested Firmware:** UEFI 64-bit (`EFI64`) & Legacy BIOS  
**Recommended Virtual Hardware:** Intel PRO/1000 MT Desktop (82540EM) + VBoxSVGA  

---

## 1. Virtual Machine Creation Walkthrough

Follow these exact steps to create and configure a virtual machine optimized for Life OS in Oracle VirtualBox:

### Step 1: Create Virtual Machine
1. Open **Oracle VirtualBox**.
2. Click **Machine -> New** (or press `Ctrl+N`).
3. Fill in the parameters:
   - **Name:** `Life OS`
   - **Folder:** Default VM directory
   - **ISO Image:** Browse and select `C:\Users\USER\Downloads\LifeOS.iso` (or `\\wsl.localhost\Ubuntu-22.04\home\vedav\life-os\build\LifeOS.iso`)
   - **Type:** `Other`
   - **Version:** `Other/Unknown (64-bit)`
4. Click **Next**.

---

### Step 2: Hardware Configuration
1. **Base Memory (RAM):**
   - Minimum: `2048 MB` (2 GB)
   - **Recommended:** `4096 MB` (4 GB)
2. **Processors (CPUs):**
   - Set to `2 CPUs` (or 4 CPUs).
3. **Firmware (Critical Setting):**
   - Check the box: **Enable EFI (special OSes only)**.
   - *Why:* Limine hybrid ISO boots natively and reliably in 64-bit UEFI mode with full GOP framebuffer graphics.
4. Click **Next**.

---

### Step 3: Virtual Hard Disk (Optional)
- Life OS operates directly as a live bootable bare-metal operating system with an in-memory Virtual File System (LifeFS).
- You may allocate a standard virtual hard disk (e.g. 2 GB VDI) or proceed without one.
- Click **Finish**.

---

## 2. Advanced VM Settings Adjustment

Before starting the machine, open **Settings** (`Ctrl+S`) and verify:

### 1. Display Settings
- **Video Memory:** Set to `128 MB` (Maximum slider).
- **Graphics Controller:** Select `VBoxSVGA` (or `VMSVGA`).
- **Acceleration:** 3D Acceleration can remain unchecked.

### 2. Storage Settings
- Ensure the **IDE Controller** or **SATA Controller** has the `LifeOS.iso` attached to the optical drive.
- Set Optical Drive to **Port 0**.

### 3. Network Settings (Crucial for e1000 Driver)
- **Adapter 1:**
  - Check **Enable Network Adapter**.
  - **Attached to:** `NAT` (VirtualBox Network Address Translation).
  - Click **Advanced**:
    - **Adapter Type:** Select **Intel PRO/1000 MT Desktop (82540EM)**.
    - **Promiscuous Mode:** `Deny` (or `Allow VMs`).
    - **Cable Connected:** Checked.
  - *Why:* The Life OS kernel includes a dedicated native driver (`src/drivers/e1000.rs`) written specifically for PCI Device ID `0x100e` (Intel 82540EM).

---

## 3. Host Wi-Fi Bridge Activation (CRITICAL)

To enable real Wi-Fi network scanning from within Life OS:

### Step 1: Enable VirtualBox NAT Host Loopback Routing
By default, VirtualBox NAT security drops guest packets sent to the host loopback `10.0.2.2`. To allow Life OS to reach the Windows Wi-Fi helper on port 18888, run this command in Windows PowerShell:
```powershell
& 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe' modifyvm "Life OS" --nat-localhostreachable1 on
```
*(Verify that `showvminfo "Life OS"` reports `LocalhostReachable: 1`).*

### Step 2: Start the Windows Wi-Fi Bridge Helper
On the Windows host laptop:
1. Navigate to `host_bridge/` inside the project folder.
2. Double-click `START_WIFI_HELPER.bat` (or run `powershell.exe -ExecutionPolicy Bypass -File host_bridge\WiFiBridge.ps1`).
3. The helper will report:
   ```
   ============================================================
            LIFE OS — WINDOWS HOST WI-FI BRIDGE HELPER          
   ============================================================
     Owner: Shrivika
     Listening on UDP 0.0.0.0:18888
     Ready to serve Wi-Fi scan and status requests from Life OS
   ```
4. Now boot Life OS! When you open `Network & Wi-Fi Manager` or run `wifi scan` in the terminal, real physical Wi-Fi networks in the room will be discovered immediately!

---

## 4. Booting and Running Life OS
1. In VirtualBox, select **Life OS** and click **Start**.
2. Limine will display the boot menu and launch the kernel within 2 seconds.
3. The high-resolution desktop (1024x768 @ 32 bpp) will appear with 8 desktop icons, taskbar, terminal shell, and live hardware telemetry!
