# esp32-spooky-maze-game

Rust reimplementation of simple game for ESP32. Work in progress.

## Build and flash

```
cargo espflash --release
```

### Build for ESP32-S2 with ILI9341

```
cargo espflash --release --features esp32s2_ili9341
```

### Build for ESP32-S2-USB-ORG with ST7789

```
cargo espflash --release --features esp32s2_usb_otg
```

### Build for ESP32-S3-USB-ORG with ST7789

```
cargo espflash --release --features esp32s3_usb_otg
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
