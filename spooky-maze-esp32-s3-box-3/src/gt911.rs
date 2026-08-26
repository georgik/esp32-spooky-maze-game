// Source: https://github.com/opsnull/rust-slint-printdemo/blob/main/mcu-board-support/gt911.rs

// 参考：
// GT911: https://github.com/enelson1001/rust-esp32s3-lvgl-clickme/blob/master/src/gt911.rs
// TT21100 的驱动实现：https://github.com/jessebraham/tt21100/blob/main/tt21100/src/lib.rs
// TT21100 的例子：https://github.com/sambenko/esp-box-tt21100-example/blob/main/src/main.rs
// GT911 寄存器列表：https://github.com/STMicroelectronics/stm32-gt911/blob/main/gt911_reg.h
// esp-idf 的 C GT911 驱动：https://github.com/espressif/esp-bsp/blob/master/components/lcd_touch/esp_lcd_touch_gt911/esp_lcd_touch_gt911.c

// 说明：
// 1. ESP32-S3-BOX-3 的 GT911 I2C 地址是 0x14，而非默认的 0x5d（测试发现）；参考：
//   https://github.com/espressif/esp-bsp/blob/master/components/lcd_touch/esp_lcd_touch_gt911/include/esp_lcd_touch_gt911.h#L34C1-L40C4
// 2. 使用较老版本的 embedded_hal 库（ 0.2.5 版本，而非新的 1.0.0 版本），从而与其他库兼容；
// 3. 由于 Touch 和 LCD 共享 reset 引脚，而 LCD 初始化时已经设置了 reset，故 Touch 不需要 reset，去掉了相关逻辑，否则会导致 LCD 显示白屏；
// 4. 自定义 Error，封装其他类型 Error，便于报错；
// 5. 新增 IRQ 引脚和基于 irq 引脚的 data_available() 方法，后续 slint 调用该方法来判断是否有触摸事件；

// 已知问题：
// 1. 一次触摸后产生多次触摸 event（参考串口日志），所以需要在 Slint 等应用层做过滤。

use core::{array::TryFromSliceError, fmt::Debug};

/// A minimal implementation of the GT911 to work with Lvgl since Lvgl only uses a single touch point
/// The default orientation and size are based on the aliexpress ESP 7 inch capactive touch development
/// board model ESP-8048S070C
use embedded_hal::{
    blocking::i2c::{Write, WriteRead},
    digital::v2::InputPin,
};

// 可能是两个地址：0x5d、0x14，ESP32-S3-BOX-3 开发版是 0x14，如果使用出错则会 panic。

// !! A panic occured in 'examples/mcu-board-support/esp32_s3_box.rs', at line 205, column 33

// PanicInfo {
//     payload: Any { .. },
//     message: Some(
//         called `Result::unwrap()` on an `Err` value: BusError(AckCheckFailed),
//     ),
//     location: Location {
//         file: "examples/mcu-board-support/esp32_s3_box.rs",
//         line: 205,
//         col: 33,
//     },
//     can_unwind: true,
//     force_no_backtrace: false,
// }

// Backtrace:

const DEFAULT_GT911_ADDRESS: u8 = 0x14;

/// Any type of error which may occur while interacting with the device
#[derive(Debug)]
pub enum Error<E> {
    /// Some error originating from the communication bus
    BusError(E),
    /// The message length did not match the expected value
    InvalidMessageLen(usize),
    /// Reading a GPIO pin resulted in an error
    IOError,
    /// Tried to read a touch point, but no data was available
    NoDataAvailable,
    /// Error converting a slice to an array
    TryFromSliceError,
}

impl<E> From<TryFromSliceError> for Error<E> {
    fn from(_: TryFromSliceError) -> Self {
        Self::TryFromSliceError
    }
}

/// Documented registers of the device
#[allow(dead_code)]
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum Reg {
    ProductId = 0x8140,
    // 返回 info 数据：触摸点数量、是否有按键、buffer status 是否有效。在 buffer status 有效的情况下，读取触摸点数量和是否有按键
    PointInfo = 0x814E,
    // 第一个触摸点数据地址
    Point1 = 0x814F,
    // Touch Key：0：key 被按下，其它：key 被释放，需要 0x814E 中的 Have Key 有效时才读取
    Key1 = 0x8093,
}

/// Represents the orientation of the device
#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Portrait, // Do Not use
    Landscape,
    InvertedPortrait, // Do Not use
    InvertedLandscape,
}

/// Represents the dimensions of the device
#[derive(Copy, Clone, Debug)]
pub struct Dimension {
    pub height: u16,
    pub width: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TouchEvent {
    Point(TouchPoint),
    Key(TouchKey),
    None,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TouchPoint {
    pub id: u8,
    pub x: u16,
    pub y: u16,
    pub size: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TouchKey {
    pub id: u8,
    pub pressed: bool,
}

/// Driver representation holding:
///
/// - The I2C Slave address of the GT911
/// - The I2C Bus used to communicate with the GT911
/// - The screen/panel orientation
/// - The scree/panel dimesions
#[derive(Clone, Debug)]
pub struct GT911<I2C, IRQ> {
    address: u8,
    i2c: I2C,
    irq_pin: IRQ,
    orientation: Orientation,
    size: Dimension,
}

impl<I2C, IRQ, E> GT911<I2C, IRQ>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
    IRQ: InputPin,
    E: Debug,
{
    pub fn new(i2c: I2C, irq_pin: IRQ) -> Self {
        Self {
            address: DEFAULT_GT911_ADDRESS,
            i2c,
            irq_pin,
            orientation: Orientation::Landscape,
            size: Dimension { height: 240, width: 320 }, // 320*240
        }
    }

    pub fn data_available(&self) -> Result<bool, Error<E>> {
        self.irq_pin.is_low().map_err(|_| Error::IOError)
    }

    // pub fn reset(&mut self) -> Result<(), Error<E>> {
    //     //println!("======= Resetting GT911 =======");
    //     self.reset_pin.set_low().map_err(|_| Error::IOError)?;
    //     self.delay.delay_us(100);
    //     self.reset_pin.set_high().map_err(|_| Error::IOError)?;
    //     self.delay.delay_ms(100);

    //     Ok(())
    // }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    pub fn set_size(&mut self, height: u16, width: u16) {
        self.size = Dimension { height, width };
    }

    // no_std 不支持 String 和 std 库。
    pub fn read_product_id(&mut self) -> Result<(), Error<E>> {
        let mut rx_buf: [u8; 4] = [0; 4];

        let product_id_reg: u16 = Reg::ProductId as u16;

        let hi_byte: u8 = (product_id_reg >> 8).try_into().unwrap();
        let lo_byte: u8 = (product_id_reg & 0xFF).try_into().unwrap();
        let tx_buf: [u8; 2] = [hi_byte, lo_byte];

        self.i2c.write_read(self.address, &tx_buf, &mut rx_buf).map_err(|e| Error::BusError(e))?;
        esp_println::println!("driver: read_product_id: {:?}", &rx_buf); // rx_buf 中的字符串应该为 "911"
        Ok(())
    }

    pub fn read_touch(&mut self) -> Result<TouchEvent, Error<E>> {
        let mut rx_buf: [u8; 1] = [0xFF];

        let point_info_reg: u16 = Reg::PointInfo as u16;
        let hi_byte: u8 = (point_info_reg >> 8).try_into().unwrap();
        let lo_byte: u8 = (point_info_reg & 0xFF).try_into().unwrap();
        let tx_buf: [u8; 2] = [hi_byte, lo_byte];

        self.i2c.write_read(self.address, &tx_buf, &mut rx_buf).map_err(|e| Error::BusError(e))?;

        let point_info = rx_buf[0];
        let buffer_status = point_info & 0x80 == 0x80; // buffer status 表示触摸点数量、按键事件是否有效
        let have_key = point_info & 0x10 == 0x10; // key 数据有效
        let touches = point_info & 0xF; // 触摸点数量

        esp_println::println!(
            "driver: pointInfo: {:x?}, bufferStatus: {:?}, haveKey: {:?} touches: {:?}",
            point_info,
            point_info >> 7 & 1u8,
            point_info >> 4 & 1u8,
            point_info & 0xF
        );

        if !buffer_status {
            // 没有有效的 touch key 或 touch point 数据，清理状态寄存器后返回。
            let tx_buf: [u8; 3] = [hi_byte, lo_byte, 0u8];
            self.i2c.write(self.address, &tx_buf).map_err(|e| Error::BusError(e))?;
            return Err(Error::NoDataAvailable);
        }

        let te: TouchEvent;
        if touches > 0 {
            // 有触摸
            let tp = self.read_touch_point(Reg::Point1 as u16).map_err(|_| Error::IOError)?;
            te = TouchEvent::Point(tp);
        } else if have_key {
            // 有按键
            let tk = self.read_touch_key(Reg::Key1 as u16).map_err(|_| Error::IOError)?;
            te = TouchEvent::Key(tk);
        } else {
            // 按键或触摸释放（用 TouchEvent::None 表示释放）
            // Reset point_info register after reading it
            let tx_buf: [u8; 3] = [hi_byte, lo_byte, 0u8];
            self.i2c.write(self.address, &tx_buf).map_err(|e| Error::BusError(e))?;
            return Ok(TouchEvent::None);
        }

        // Reset point_info register after reading it
        let tx_buf: [u8; 3] = [hi_byte, lo_byte, 0u8];
        self.i2c.write(self.address, &tx_buf).map_err(|e| Error::BusError(e))?;

        Ok(te)
    }

    pub fn read_touch_key(&mut self, key_register: u16) -> Result<TouchKey, Error<E>> {
        let hi_byte: u8 = (key_register >> 8).try_into().unwrap();
        let lo_byte: u8 = (key_register & 0xFF).try_into().unwrap();
        let tx_buf: [u8; 2] = [hi_byte, lo_byte];

        // GT911 有 4 个 key
        let mut rx_buf: [u8; 4] = [0; 4];
        self.i2c.write_read(self.address, &tx_buf, &mut rx_buf).map_err(|e| Error::BusError(e))?;

        esp_println::println!("  driver: read_touch_key: {:?}", &rx_buf);

        // ESP32-S3-Box-3 只使用了一个 touch key，故只读取 rx_buf[0] 内容
        let key0: u8 = rx_buf[0];
        Ok(TouchKey { id: 0, pressed: if key0 == 0 { false } else { true } })
    }

    pub fn read_touch_point(&mut self, point_register: u16) -> Result<TouchPoint, Error<E>> {
        let hi_byte: u8 = (point_register >> 8).try_into().unwrap();
        let lo_byte: u8 = (point_register & 0xFF).try_into().unwrap();
        let tx_buf: [u8; 2] = [hi_byte, lo_byte];

        let mut rx_buf: [u8; 7] = [0; 7];
        self.i2c.write_read(self.address, &tx_buf, &mut rx_buf).map_err(|e| Error::BusError(e))?;

        let id: u8 = rx_buf[0];
        let mut x: u16 = rx_buf[1] as u16 + ((rx_buf[2] as u16) << 8);
        let mut y: u16 = rx_buf[3] as u16 + ((rx_buf[4] as u16) << 8);
        let size: u16 = rx_buf[5] as u16 + ((rx_buf[6] as u16) << 8);

        match self.orientation {
            Orientation::Landscape => {
                // Don't need to do anything because x = x and y = y
            }
            Orientation::Portrait => {
                let temp: u16 = x;
                x = y;
                y = self.size.height - temp;
            }
            Orientation::InvertedLandscape => {
                x = self.size.width - x;
                y = self.size.height - y;
            }
            Orientation::InvertedPortrait => {
                let temp: u16 = x;
                x = self.size.width - y;
                y = temp;
            }
        }

        esp_println::println!("  driver: read_touch_point: x/y/id: {}/{}/{}", x, y, id);

        Ok(TouchPoint { id, x, y, size })
    }
}