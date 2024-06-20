#ifndef ESPRESSIF_RUST_LIB_H
#define ESPRESSIF_RUST_LIB_H

#include <stdint.h>

// Function to load assets
extern void load_assets();

// Function to get the ghost1 image. It returns a pointer to the image data.
extern const uint8_t* get_ghost1_image();

#endif // ESPRESSIF_RUST_LIB_H
