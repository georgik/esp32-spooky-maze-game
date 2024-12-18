fn main() {
    let esp32_c6_devkitc_1 = cfg!(feature = "esp32-c6-devkitc-1");
    let waveshare_esp32_c6_lcd_1_47 = cfg!(feature = "waveshare-esp32-c6-lcd-1-47");

    let selected_features = [
        esp32_c6_devkitc_1,
        waveshare_esp32_c6_lcd_1_47,
    ];

    // Count selected features
    let enabled_features = selected_features.iter().filter(|&&f| f).count();

    // Enforce exactly one feature
    if enabled_features != 1 {
        panic!(
            "You must enable exactly one feature. Available features are: \
            `esp32-c6-devkitc-1` and `waveshare-esp32-c6-lcd-1-47`. \
            For example: `cargo build --features esp32-c6-devkitc-1`."
        );
    }
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
}
