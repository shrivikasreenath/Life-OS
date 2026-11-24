# LIFE OS Architecture & Subsystem Specification
**Author:** Shrivika  
**Target:** x86_64 Bare-Metal Architecture  
**Compiler:** Rust `nightly-x86_64-unknown-none`  
**Bootloader Protocol:** Limine v7 Protocol  

---

## 1. High-Level Architecture Overview

Life OS is designed around an autonomous microkernel model built purely in Rust (`no_std`, `no_main`), utilizing direct hardware manipulation, MMIO, and interrupt-driven I/O.

```
+-----------------------------------------------------------------------------------+
|                              USER ENVIRONMENT / GUI                               |
|  +--------------------+ +--------------------+ +--------------------+ +---------+ |
|  | Terminal (23 cmds) | | Life Web Browser   | | Games (4-in-1)     | | FileMgr | |
|  +--------------------+ +--------------------+ +--------------------+ +---------+ |
|  | SysInfo            | | System Settings    | | About Life OS      | | NetMgr  | |
|  +--------------------+ +--------------------+ +--------------------+ +---------+ |
|    Desktop Icons (8)  |  Context Menu (F3)   |  Calendar Pop-up(F2) |  Taskbar  | |
|              Window Manager & Desktop Compositor (Double-Buffered)                |
+-----------------------------------------------------------------------------------+
|                             CORE SERVICES & STACKS                                |
|  +---------------------------------+ +------------------------------------------+ |
|  | LifeFS (In-Memory VFS Tree)     | | smoltcp Network Stack (IPv4/TCP/UDP/DNS) | |
|  +---------------------------------+ +------------------------------------------+ |
|                                      | Host Wi-Fi Bridge (UDP Client 10.0.2.2)  | |
+-----------------------------------------------------------------------------------+
|                                 DEVICE DRIVERS                                    |
|  +-----------------+ +-----------------+ +-----------------+ +------------------+ |
|  | Intel 82540EM   | | PS/2 Keyboard   | | PS/2 Mouse      | | CMOS RTC Clock   | |
|  | Gigabit NIC     | | (US 104-Key)    | | (3-Button)      | | (Date & Time)    | |
|  +-----------------+ +-----------------+ +-----------------+ +------------------+ |
|  | PCI Bus Scanner | | Limine Linear Framebuffer (1024x768x32bpp)                  | |
|  +-----------------+ +----------------------------------------------------------+ |
+-----------------------------------------------------------------------------------+
|                             KERNEL ARCHITECTURE (x86_64)                          |
|  +---------------------------------+ +------------------------------------------+ |
|  | GDT & TSS (IST Stacks)          | | IDT (Exceptions, Dual 8259 PIC IRQs)     | |
|  +---------------------------------+ +------------------------------------------+ |
|  | Physical Frame Allocator        | | Dynamic Heap Allocator (32 MB Pool)      | |
|  +---------------------------------+ +------------------------------------------+ |
|  | 4-Level Paging (HHDM at Offset) | | PIT Timer (100 Hz calibrated)            | |
|  +---------------------------------+ +------------------------------------------+ |
+-----------------------------------------------------------------------------------+
|                                 BARE-METAL HARDWARE                               |
|        AMD Ryzen / Intel x86_64 CPU  |  VirtualBox VM  |  Real Physical RAM       |
+-----------------------------------------------------------------------------------+
```

---

## 2. Boot Flow & Linker Layout

### Boot Flow Sequence
1. **Firmware Initialization**: System firmware (UEFI `EFI64` or Legacy BIOS) loads the Limine bootloader from the boot medium.
2. **Protocol Negotiation**: Limine locates `limine.cfg`, sets video resolution (1024x768x32bpp), creates Higher-Half Direct Map (HHDM), builds memory map, and parses kernel ELF headers.
3. **Control Transfer**: Limine jumps to `_start` in `src/main.rs`.
4. **Subsystem Boot**:
   ```
   arch::init()        -> Serial COM1, GDT, TSS, IDT, PIC (unmasked), PIT 100 Hz
   mm::init()          -> HHDM offset, Frame Allocator, 32 MB Heap
   drivers::init()     -> PS/2 Keyboard, PS/2 Mouse, Intel 82540EM NIC
   fs::init()          -> LifeFS in-memory filesystem tree
   net::init()         -> smoltcp device adapter & network stack
   graphics::init()    -> Framebuffer backbuffer allocation
   apps::init()        -> Games suite & terminal state setup
   gui::init()         -> Desktop window manager & initial app launch
   ```
5. **Main Event Loop**:
   ```rust
   loop {
       net::poll();
       gui::update_and_render();
       x86_64::instructions::hlt();
   }
   ```

### Linker Script Layout (`linker.ld`)
The kernel is linked to reside in the higher-half virtual address space starting at `0xffffffff80000000` (Negative 2GB Long Mode mapping):
```
0xffffffff80000000 +------------------------+
                   | .text (Executable Code) |
                   +------------------------+
                   | .rodata (Read-Only)    |
                   +------------------------+
                   | .data (Initialized)    |
                   +------------------------+
                   | .bss (Zero-Filled)     |
                   +------------------------+
```

---

## 3. Memory Management Subsystem

### 1. Higher-Half Direct Map (HHDM)
Limine maps all physical memory directly to a higher-half virtual offset (e.g., `0xffff800000000000`).
Translation helpers in `src/mm/paging.rs`:
```rust
pub fn phys_to_virt(phys: u64) -> u64 {
    phys + get_hhdm_offset()
}

pub fn virt_to_phys(virt: u64) -> u64 {
    virt - get_hhdm_offset()
}
```

### 2. Physical Frame Allocator
- Located in `src/mm/frame.rs`.
- Traverses Limine's `MemoryMapRequest` response entries.
- Identifies entries marked `EntryType::USABLE`.
- Tracks `total_bytes`, `usable_bytes`, and `allocated_bytes`.
- Allocates individual or contiguous 4096-byte frames for DMA buffers and page tables.

### 3. Global Heap Allocator
- Located in `src/mm/heap.rs`.
- Backed by `linked_list_allocator::LockedHeap`.
- Allocates an initial contiguous pool of **32 MB** (`33,554,432 bytes`) in physical RAM, mapped into virtual memory through HHDM.
- Enables standard Rust heap containers: `Vec`, `String`, `BTreeMap`, `Box`.

---

## 4. Interrupt Architecture & CPU Protection

### GDT & Segment Selectors (`src/arch/gdt.rs`)
To prevent GPFs during interrupt handling (`iretq`), the GDT defines:
1. **Null Descriptor** (`0x00`)
2. **Kernel Code Segment** (`0x08`)
3. **Kernel Data Segment** (`0x10`)
4. **Task State Segment** (`0x18`)

The segment registers (`CS`, `DS`, `ES`, `SS`) are loaded with their respective selectors, and the TSS is loaded via `ltr`.
Double faults utilize a dedicated Interrupt Stack Table (IST Index 0) to ensure reliable panic reporting even on kernel stack exhaustion.

### Interrupt Descriptor Table (`src/arch/idt.rs`)
- Exception gates: Divide Error (0), Breakpoint (3), Invalid Opcode (6), GPF (13), Page Fault (14).
- Hardware IRQs remapped through dual 8259 PICs:
  - Vector 32 (IRQ 0): Programmable Interval Timer (100 Hz)
  - Vector 33 (IRQ 1): PS/2 Keyboard input
  - Vector 44 (IRQ 12): PS/2 Mouse input

### Dual 8259 PIC Remapping (`src/arch/pic.rs`)
- Remaps Master PIC to vectors `32..39` and Slave PIC to `40..47` to avoid conflicts with x86 CPU exceptions.
- UEFI firmware leaves PIC masks set to `0xFF` (all masked). Life OS explicitly writes mask `0x00, 0x00` during initialization to unmask legacy hardware interrupts.

---

## 5. Intel 82540EM (e1000) Network Driver & Host Bridge

Located in `src/drivers/e1000.rs` and `src/net/stack.rs`:
- **PCI Enumeration**: Scans PCI bus for Vendor ID `0x8086` and Device ID `0x100e` (Oracle VirtualBox e1000).
- **Bus Mastering**: Enables PCI Bus Master and Memory Space access bits.
- **MMIO Access**: Reads BAR0 physical base address and maps it to virtual memory through HHDM.
- **Hardware MAC Address**: Reads registers `RAL` (0x5400) and `RAH` (0x5404).
- **Circular DMA Descriptor Rings**:
  - 32 Receive (RX) descriptors (`E1000_NUM_RX_DESC`) with 2048-byte physical packet buffers.
  - 32 Transmit (TX) descriptors (`E1000_NUM_TX_DESC`) with 2048-byte physical packet buffers.
- **Windows Wi-Fi Host Bridge Integration**:
  - Inter-process UDP communication over `10.0.2.2:18888`.
  - Commands supported: `SCAN`, `STATUS`, `CONNECT <SSID>|<pwd>`, `DISCONNECT`.
  - Automatically updates the taskbar system tray with the connected SSID.

---

## 6. Graphical Compositor & Window Manager

Located in `src/graphics/` and `src/gui/`:
- **Double-Buffering**: Framebuffer backbuffer allocated as `Vec<u32>` of size `width * height`.
  - All rendering (wallpaper, icons, window frames, text, cursor) occurs in memory.
  - `present()` performs high-speed `core::ptr::copy_nonoverlapping` to the physical VRAM.
- **Desktop Application Icons**:
  - 8 desktop icons with 32x32 pixel art (Browser, Terminal, File Manager, Network Manager, Games, SysInfo, Settings, Trash).
  - Single-click selection, double-click launch, and dragging support.
- **Desktop Right-Click Context Menu (`F3`)**:
  - Actions: Refresh Desktop, Change Wallpaper, Arrange Icons, Create Folder, Open Terminal, Display Settings, System Info.
- **3 Procedural Wallpapers (`F5`)**:
  - Cyber Grid (Default), Deep Cobalt Aurora, Midnight Slate.
- **Calendar Pop-up Panel (`F2`)**:
  - Opened via clock click or `F2`, displaying full month grid and highlighted current day.
- **Window Management Engine**:
  - Z-ordering: Active window sits at the end of the window list and renders last (on top).
  - Titlebar drag: Moves window `(x, y)` dynamically based on mouse drag delta.
  - Minimize/Maximize/Close: Standard window controls (`F12` to maximize, `F4` to close).
- **Global Hotkey Architecture**:
  - `F1` (Start Menu), `F2` (Calendar), `F3` (Context Menu), `F4` (Close), `F5` (Cycle Wallpaper), `F6` (Cycle Focus), `F7` (Games), `F8` (Browser), `F9` (SysInfo), `F10` (Settings), `F12` (Maximize), `1..8` (Start Menu Launchers).
