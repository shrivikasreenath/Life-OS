# LIFE OS Build & Compilation Manual
**Author:** Shrivika  
**Target:** `x86_64-unknown-none`  
**Output Target:** `build/LifeOS.iso` (<100 MB requirement)

---

## 1. Prerequisites & Toolchains

Building Life OS requires a Linux environment (Ubuntu 22.04 LTS under WSL2 or native Linux) with the following tools installed:

### Required Dependencies
```bash
# Update system package lists
sudo apt-get update

# Install ISO creation and binary utilities
sudo apt-get install -y xorriso mtools git curl build-essential
```

### Rust Toolchain Setup
Life OS requires the Rust `nightly` channel due to experimental bare-metal features (`feature(abi_x86_interrupt)`, `feature(alloc_error_handler)`).

The project includes `rust-toolchain.toml` which automatically configures:
```toml
[toolchain]
channel = "nightly"
components = ["rust-src", "llvm-tools-preview"]
targets = ["x86_64-unknown-none"]
```

To manually verify or configure:
```bash
# Install nightly toolchain
rustup toolchain install nightly

# Add bare-metal x86_64 target
rustup target add x86_64-unknown-none --toolchain nightly

# Add standard source components
rustup component add rust-src --toolchain nightly
```

---

## 2. Automated Build Script (`build.sh`)

Life OS provides a fully automated build script that handles kernel compilation, Limine bootloader staging, hybrid ISO creation, and size validation.

### Running the Build
```bash
chmod +x build.sh
./build.sh
```

### Build Process Steps
1. **Compilation**:
   Executes `cargo build --release --target x86_64-unknown-none`.
   Optimizations: LTO (Link-Time Optimization) enabled, dead-code elimination, release strip.
2. **Bootloader Setup**:
   Clones and builds Limine bootloader v7 binary stages if not already cached.
3. **ISO Root Assembly**:
   Constructs `build/iso_root` directory:
   - `/boot/kernel.elf` — The compiled kernel binary.
   - `/boot/limine/limine.cfg` — Limine boot configuration.
   - `/boot/limine/limine-bios.sys` — BIOS second-stage bootloader.
   - `/EFI/BOOT/BOOTX64.EFI` — 64-bit UEFI bootloader binary.
4. **Hybrid ISO Generation**:
   Invokes `xorriso` to create an ISO 9660 filesystem with El Torito UEFI and BIOS boot records:
   ```bash
   xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
           -no-emul-boot -boot-load-size 4 -boot-info-table \
           --efi-boot EFI/BOOT/BOOTX64.EFI \
           -efi-boot-part --efi-boot-image --protective-msdos-label \
           build/iso_root -o build/LifeOS.iso
   ```
5. **Limine Deployment**:
   Executes `limine bios-install build/LifeOS.iso` to embed stage 1 & 2 sectors into the ISO's GPT partition header.
6. **Size Requirement Verification**:
   Inspects the final ISO file size and asserts that it is strictly below 100 MB.

---

## 3. Strict Size Verification & Monitoring

The final ISO size is continuously monitored on every build.

### Current Build Measurement
```
ISO Path : build/LifeOS.iso
ISO Size : 3,579,904 bytes (3.41 MB)
Status   : PASS (Requirement: strictly < 100 MB)
Margin   : 96.59 MB headroom available
```

### Factors Keeping Size Minimal
- **`no_std` Architecture**: No bloated C standard library or Linux userland binaries included.
- **Micro-Kernel Footprint**: Pure Rust drivers and internal bitmap fonts (8x16 fixed raster table, no multi-megabyte TTF engines).
- **Procedural Graphics**: Algorithmic wallpaper rendering and pixel art icons rather than heavy PNG/JPEG image bundles.

---

## 4. Emulation & Testing (`run_qemu.sh`)

To quickly run and test the compiled ISO in QEMU:
```bash
chmod +x run_qemu.sh
./run_qemu.sh
```

Flags passed to QEMU:
```bash
qemu-system-x86_64 \
    -cdrom build/LifeOS.iso \
    -m 4G \
    -smp 2 \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -serial stdio \
    -vga std
```
