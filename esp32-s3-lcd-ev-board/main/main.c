/*
 * SPDX-FileCopyrightText: 2023 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: CC0-1.0
 */

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"

#include "lv_demos.h"
#include "bsp/esp-bsp.h"
#include "spooky_idf.h"

static char *TAG = "app_main";

/* Print log about SRAM and PSRAM memory */
#define LOG_MEM_INFO    (0)

#define CANVAS_WIDTH  16
#define CANVAS_HEIGHT 16

void rust_task(void* param) {

    bsp_display_lock(0);

    // Create a label and set its text to "hello world"
    lv_obj_t *label = lv_label_create(lv_scr_act());   // Create a label on the current screen
    lv_label_set_text(label, "Spooky says: Hello from Rust!");            // Set the label's text
    lv_obj_center(label);
    load_assets();
    // Get the ghost1 image data from Rust
    const uint8_t* ghost1_data = get_ghost1_image();
    static lv_color_t canvas_buf[CANVAS_WIDTH * CANVAS_HEIGHT];
    if(ghost1_data != NULL) {  // Ensure the image data is not NULL
        for (int i = 0; i < CANVAS_WIDTH * CANVAS_HEIGHT; i++) {
            // Extract RGB565 color components from the image data
            lv_color_t color;
            color.ch.red = ghost1_data[i * 3];
            color.ch.green = ghost1_data[i * 3 + 1];
            color.ch.blue = ghost1_data[i * 3 + 2];

            canvas_buf[i] = color;
        }
    }


    // Create a canvas object and set its buffer
    lv_obj_t *canvas = lv_canvas_create(lv_scr_act());
    lv_canvas_set_buffer(canvas, canvas_buf, CANVAS_WIDTH, CANVAS_HEIGHT, LV_IMG_CF_TRUE_COLOR);
    lv_obj_align_to(canvas, label, LV_ALIGN_OUT_BOTTOM_MID, 0, 10);  // Position the canvas 10 pixels below the label

    bsp_display_unlock();

    while (true) {
        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }
}


void app_main(void)
{
#if CONFIG_BSP_LCD_SUB_BOARD_480_480
    // For the newest version sub board, we need to set `BSP_LCD_VSYNC` to high before initialize LCD
    // It's a requirement of the LCD module and will be added into BSP in the future
    gpio_config_t io_conf = {};
    io_conf.pin_bit_mask = BIT64(BSP_LCD_VSYNC);
    io_conf.mode = GPIO_MODE_OUTPUT;
    io_conf.pull_up_en = true;
    gpio_config(&io_conf);
    gpio_set_level(BSP_LCD_VSYNC, 1);
#endif

    bsp_i2c_init();
    lv_disp_t *disp = bsp_display_start();

#if CONFIG_BSP_DISPLAY_LVGL_AVOID_TEAR
    ESP_LOGI(TAG, "Avoid lcd tearing effect");
#if CONFIG_BSP_DISPLAY_LVGL_FULL_REFRESH
    ESP_LOGI(TAG, "LVGL full-refresh");
#elif CONFIG_BSP_DISPLAY_LVGL_DIRECT_MODE
    ESP_LOGI(TAG, "LVGL direct-mode");
#endif
#endif

    ESP_LOGI(TAG, "Display LVGL demo");
    xTaskCreate(rust_task, "rust_task", 36240, NULL, 5, NULL);

}
