# Spooky Maze for ESP32-S3-LCD-EV-Board

The implementation is using ESP-IDF + Rust. This is slightly different approach than rest of
examples in the repo. The reason is that there is no Rust driver for the display, so we rely
on ESP-IDF and LVGL to handle the communication with the display.

## Build

```
idf.py build flash monitor
```
