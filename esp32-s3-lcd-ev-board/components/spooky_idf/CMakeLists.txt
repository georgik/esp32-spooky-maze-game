idf_component_register(
    SRCS "spooky_idf.c"
    INCLUDE_DIRS "include"
)

# Define the Rust target for the Xtensa and RISC-V architecture
if (CONFIG_IDF_TARGET_ARCH_XTENSA)
    set(RUST_CARGO_TOOLCHAIN "+esp")
    set(RUST_CARGO_TARGET "xtensa-${IDF_TARGET}-none-elf")
elseif (CONFIG_IDF_TARGET_ARCH_RISCV)
    set(RUST_CARGO_TOOLCHAIN "+nightly")
    set(RUST_CARGO_TARGET "riscv32imac-unknown-none-elf")
else()
    message(FATAL_ERROR "Architecture currently not supported")
endif()

# Set the flags for cargo build
set(CARGO_BUILD_FLAGS "-Zbuild-std=core,panic_abort")

# Set directories and target
set(RUST_PROJECT_DIR "${CMAKE_CURRENT_LIST_DIR}/spooky_idf_rs")
set(RUST_BUILD_DIR "${CMAKE_CURRENT_BINARY_DIR}")
set(RUST_TARGET_DIR "${RUST_BUILD_DIR}/target")
set(RUST_STATIC_LIBRARY "${RUST_TARGET_DIR}/${RUST_CARGO_TARGET}/debug/libspooky_idf_rs.a")
# set(RUST_STATIC_LIBRARY "${RUST_TARGET_DIR}/${RUST_CARGO_TARGET}/release/libspooky_idf_rs.a")

# ExternalProject_Add for building the Rust project
ExternalProject_Add(
    rust_crate_target
    PREFIX "${RUST_PROJECT_DIR}"
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND ${CMAKE_COMMAND} -E env
        CARGO_BUILD_TARGET=${RUST_CARGO_TARGET}
        CARGO_BUILD_TARGET_DIR=${RUST_TARGET_DIR}
        cargo ${RUST_CARGO_TOOLCHAIN} build ${CARGO_BUILD_FLAGS} -Zbuild-std-features=compiler-builtins-weak-intrinsics,panic_immediate_abort
        # cargo ${RUST_CARGO_TOOLCHAIN} build --release ${CARGO_BUILD_FLAGS} -Zbuild-std-features=compiler-builtins-weak-intrinsics,panic_immediate_abort
    BUILD_ALWAYS TRUE
    INSTALL_COMMAND ""
    WORKING_DIRECTORY ${RUST_PROJECT_DIR}
    TMP_DIR "${RUST_BUILD_DIR}/tmp"
    STAMP_DIR "${RUST_BUILD_DIR}/stamp"
    DOWNLOAD_DIR "${RUST_BUILD_DIR}"
    SOURCE_DIR "${RUST_PROJECT_DIR}"
    BINARY_DIR "${RUST_PROJECT_DIR}"
    INSTALL_DIR "${RUST_BUILD_DIR}"
    BUILD_BYPRODUCTS "${RUST_STATIC_LIBRARY}"
)

# Add prebuilt Rust library
add_prebuilt_library(rust_crate_lib "${RUST_STATIC_LIBRARY}" REQUIRES "")

# Add dependencies and link Rust library
add_dependencies(${COMPONENT_LIB} rust_crate_target)
target_link_libraries(${COMPONENT_LIB} PUBLIC rust_crate_lib)
