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
    FIRMWARE_DIR=$1
    CHIP=$2
    cd ${FIRMWARE_DIR}
    cargo espflash save-image --chip ${CHIP} --release --merge spooky-maze-${FIRMWARE_DIR}.bin
    cd ..
}

build_firmware esp-wrover-kit esp32
build_firmware esp32-c3-devkit-rust esp32c3
build_firmware esp32-c6-devkit esp32c6
build_firmware esp32-s2-kaluga esp32s2
build_firmware esp32-s3-usb-otg esp32s3
build_firmware esp32-s3-box esp32s3
build_firmware m5stack-core2 esp32
build_firmware m5stack-fire esp32
