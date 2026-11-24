#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

source ~/.cargo/env 2>/dev/null || true

echo "=========================================================="
echo "          BUILDING LIFE OS - BARE-METAL RUST OS           "
echo "=========================================================="

echo "[1/5] Compiling Life OS Kernel in Release Mode..."
cargo build --release --target x86_64-unknown-none

KERNEL_BIN="target/x86_64-unknown-none/release/life-os"
if [ ! -f "$KERNEL_BIN" ]; then
    echo "ERROR: Kernel binary $KERNEL_BIN not found!"
    exit 1
fi

echo "[2/5] Ensuring Limine bootloader is ready..."
if [ ! -f "limine/limine" ]; then
    make -C limine
fi

echo "[3/5] Assembling ISO root filesystem..."
rm -rf build/iso_root
mkdir -p build/iso_root/boot/limine build/iso_root/EFI/BOOT build/iso_root/limine

cp "$KERNEL_BIN" build/iso_root/kernel.elf
cp "$KERNEL_BIN" build/iso_root/boot/kernel.elf

cp limine.cfg build/iso_root/
cp limine.cfg build/iso_root/boot/limine.cfg
cp limine.cfg build/iso_root/boot/limine/limine.cfg
cp limine.cfg build/iso_root/EFI/BOOT/limine.cfg

cp limine/limine-bios.sys build/iso_root/
cp limine/limine-bios.sys build/iso_root/boot/
cp limine/limine-bios.sys build/iso_root/boot/limine/
cp limine/limine-bios-cd.bin build/iso_root/boot/
cp limine/limine-uefi-cd.bin build/iso_root/boot/
cp limine/BOOTX64.EFI build/iso_root/EFI/BOOT/

echo "[4/5] Creating Hybrid ISO with xorriso..."
mkdir -p build
xorriso -as mkisofs -b boot/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    build/iso_root -o build/LifeOS.iso 2>&1 | tail -n 5

echo "[5/5] Installing Limine BIOS boot sector into ISO..."
./limine/limine bios-install build/LifeOS.iso

ISO_SIZE_BYTES=$(stat -c%s "build/LifeOS.iso")
ISO_SIZE_MB=$(awk "BEGIN {printf \"%.2f\", $ISO_SIZE_BYTES / 1048576}")

echo "=========================================================="
echo "          LIFE OS BUILD COMPLETE!                         "
echo "  ISO Path: build/LifeOS.iso                              "
echo "  ISO Size: $ISO_SIZE_MB MB ($ISO_SIZE_BYTES bytes)       "
echo "=========================================================="

# Strictly check ISO size requirement (< 100 MB)
if [ "$ISO_SIZE_BYTES" -gt 104857600 ]; then
    echo "ERROR: ISO size ($ISO_SIZE_MB MB) exceeds 100 MB limit!"
    exit 1
else
    echo "SUCCESS: ISO size is strictly below 100 MB requirement!"
fi
