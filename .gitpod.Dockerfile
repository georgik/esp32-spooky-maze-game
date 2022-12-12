# Note: gitpod/workspace-base image references older version of CMake, it's necessary to install newer one
FROM gitpod/workspace-full
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# ARGS
ARG CONTAINER_USER=gitpod
ARG CONTAINER_GROUP=gitpod
ARG TOOLCHAIN_VERSION=1.65.0.1
ARG ESP_BOARD="esp32,esp32s2,esp32s3,esp32c3"
ARG INSTALL_RUST_TOOLCHAIN=espup

# Install dependencies for building wokwi-server
RUN sudo install-packages libudev-dev
# RUN sudo install-packages git curl gcc ninja-build libudev-dev \
#     libusb-1.0-0 libssl-dev pkg-config libtinfo5 clang \
#     libsdl2-dev npm
# Set User
USER ${CONTAINER_USER}
WORKDIR /home/${CONTAINER_USER}

# Install Rust toolchain, extra crates and esp-idf
ENV PATH=${PATH}:/home/${CONTAINER_USER}/.cargo/bin:/home/${CONTAINER_USER}/opt/bin
ADD --chown=${CONTAINER_USER}:${CONTAINER_GROUP} \
    https://github.com/esp-rs/espup/releases/download/v0.2.3/espup-x86_64-unknown-linux-gnu \
    /home/${CONTAINER_USER}/${INSTALL_RUST_TOOLCHAIN}
RUN chmod a+x ${INSTALL_RUST_TOOLCHAIN} \
    && ./${INSTALL_RUST_TOOLCHAIN} install \
    --extra-crates "ldproxy,cargo-espflash,wokwi-server,web-flash" \
    --export-file /home/${CONTAINER_USER}/export-esp.sh \
    --targets "${ESP_BOARD}"
#    && rustup component add clippy rustfmt
