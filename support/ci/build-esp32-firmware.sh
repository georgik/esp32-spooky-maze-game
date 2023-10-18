#!/usr/bin/env bash

set -e

source ~/export-esp.sh

# Check whether cargo-espflash is installed, if not install it from GitHub
if ! command -v cargo-espflash &> /dev/null
then
    # Compiling cargo-espflash from source takes too long, so we download the binary from GitHub
    #cargo install cargo-espflash --git https://github.com/esp-rs/espflash.git
    curl -L https://github.com/esp-rs/espflash/releases/download/v2.0.0-rc.3/cargo-espflash-x86_64-unknown-linux-gnu.zip -o cargo-espflash.zip
    unzip cargo-espflash.zip
    mv cargo-espflash ~/.cargo/bin/
fi

# Function to build the firmware by entering directory and running cargo-espflash
function build_firmware {
    cd $FIRMWARE_DIR
    VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f2)
    CHIP=$(grep 'hal = { package =' Cargo.toml | cut -d '"' -f2 | cut -d '-' -f1)

    cargo espflash save-image --chip ${CHIP} --release --merge --skip-padding spooky-maze-${FIRMWARE_DIR}.bin
    cd -
}

for FIRMWARE_DIR in `ls -d esp* m5stack*`; do
    build_firmware "$FIRMWARE_DIR"
done

#build_firmware esp-wrover-kit
#build_firmware esp32-c3-devkit-rust
