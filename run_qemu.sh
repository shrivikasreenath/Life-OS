#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f "build/LifeOS.iso" ]; then
    ./build.sh
fi

qemu-system-x86_64 \
    -M q35 \
    -m 2G \
    -cdrom build/LifeOS.iso \
    -boot d \
    -serial stdio \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    "$@"
