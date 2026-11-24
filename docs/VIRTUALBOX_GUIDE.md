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

### 4. Serial Port Settings (Recommended for Live Kernel Telemetry)
- To capture live kernel debug output:
  - Check **Enable Serial Port**.
  - **Port Number:** `COM1` (IRQ 4, I/O Port `0x3F8`).
  - **Port Mode:** `Raw File` (or `Disconnected`).
  - **Path/Address:** `C:\Users\USER\VirtualBox VMs\Life OS\serial.log`.

---

## 3. CLI Configuration via VBoxManage

If you prefer configuring via PowerShell or Windows Command Prompt:
```powershell
# Create VM
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" createvm --name "Life OS" --ostype "Other_64" --register

# Configure CPU, Memory, Firmware, and Display
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" modifyvm "Life OS" --memory 4096 --cpus 2 --firmware efi64 --vram 128 --graphicscontroller vboxsvga

# Configure Storage Controller and Attach ISO
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" storagectl "Life OS" --name "IDE" --add ide
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" storageattach "Life OS" --storagectl "IDE" --port 0 --device 0 --type dvddrive --medium "C:\Users\USER\Downloads\LifeOS.iso"

# Configure Intel 82540EM NIC with NAT
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" modifyvm "Life OS" --nic1 nat --nictype1 82540EM --cableconnected1 on

# Enable COM1 Serial Logging to file
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" modifyvm "Life OS" --uart1 0x3f8 4 --uartmode1 file "C:\Users\USER\VirtualBox VMs\Life OS\serial.log"

# Start the VM
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" startvm "Life OS"
```

---

## 4. Troubleshooting & FAQ

### Issue: Screen remains black on early boot
- **Cause:** Virtual machine configured with legacy BIOS having disk geometry translation mismatch.
- **Solution:** Change firmware to `EFI64` under **System -> Motherboard -> Enable EFI**.

### Issue: Network status shows "Link Down" or "No NIC"
- **Cause:** Network adapter type is set to PCnet-FAST III or VirtIO instead of Intel PRO/1000 MT.
- **Solution:** In VM Settings -> Network -> Advanced -> Adapter Type, select **Intel PRO/1000 MT Desktop (82540EM)**.

### Issue: Keyboard input feels unresponsive
- **Cause:** VirtualBox window has lost mouse/keyboard focus.
- **Solution:** Click inside the VM display window to capture PS/2 mouse and keyboard input.
