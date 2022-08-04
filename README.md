# esp32-spooky-maze-game

Rust reimplementation of simple game for ESP32. Work in progress.

## Build and flash

Check `target` configurad in the file `.cargo/config.toml`.
It should be one of following values:
```
target = "xtensa-esp32-none-elf"
target = "xtensa-esp32s2-none-elf"
target = "xtensa-esp32s3-none-elf"
target = "riscv32imc-unknown-none-elf"
```

If no value is selected, make sure to specify target on command line.

Check default `features` in `Cargo.toml`. Make sure that default set contains your board and display combinations.

If no value is selected, make sure to specify features on command line.

### Build for ESP32 Wrover Kit

```
cargo espflash --release --target xtensa-esp32-none-elf --features esp32_wrover_kit
```

### Build for ESP32-S2 with ILI9341

```
cargo espflash --release --target xtensa-esp32s2-none-elf --features esp32s2_ili9341
```

### Build for ESP32-S2-USB-OTG with ST7789

```
cargo espflash --release --target xtensa-esp32s2-none-elf --features esp32s2_usb_otg
```

### Build for ESP32-S3-USB-OTG with ST7789

```
cargo espflash --release --target xtensa-esp32s3-none-elf --features esp32s3_usb_otg
```

### Build for ESP32-C3 with ILI9341

```
cargo espflash --release --target riscv32imc-unknown-none-elf --features esp32c3_ili9341
```


## Plans

- [ ] add support for ESP32-S3-USB-OTG
- [ ] add Wokwi simulation
- [ ] add GitPod, CodeSpaces and VS Code Dev Container integration
- [ ] add support for sprite
- [ ] add support for interactivng with the character

## Notes

Rendering for ESP32-S2

SPI freq 80kHz - 9.8s
SPI freq 1000kHz - 1.0s
SPI freq 10000kHz - 0.32s
SPI freq 100000kHz - 0.25s
