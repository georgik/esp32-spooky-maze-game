#!/usr/bin/env bash

set -e

# Check whether cargo-espflash is installed, if not install it from GitHub
if ! command -v cargo-espflash &> /dev/null
then
    cargo install cargo-espflash --git
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
build_firmware esp32-s2-kaluga esp32s2
build_firmware esp32-s2-usb-otg esp32s2
build_firmware esp32-s3-usb-otg esp32s3
build_firmware esp32-s3-box esp32s3
build_firmware m5stack-core2 esp32
build_firmware m5stack-fire esp32

# Following builds are blocked by https://github.com/esp-rs/rust-build/issues/202
#build_firmware esp32-c3-devkit-rust esp32c3
#build_firmware esp32-c6-devkit esp32c6

