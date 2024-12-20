fn main() {
    let esp32_c3_devkit_rust = cfg!(feature = "esp32-c3-devkit-rust");
    let esp32_c3_lcdkit = cfg!(feature = "esp32-c3-lcdkit");

    let selected_features = [esp32_c3_devkit_rust, esp32_c3_lcdkit];

    // Count selected features
    let enabled_features = selected_features.iter().filter(|&&f| f).count();

    // Enforce exactly one feature
    if enabled_features != 1 {
        panic!(
            "You must enable exactly one feature. Available features are: \
            `esp32-c3-lcdkit` and `esp32-c3-devkit-rust`. \
            For example: `cargo build --features esp32-c3-lcdkit`."
        );
    }
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
}
