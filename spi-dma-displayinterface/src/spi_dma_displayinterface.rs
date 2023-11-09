//! DMA SPI interface for display drivers

use core::cell::RefCell;

// use embedded_hal::digital::v2::OutputPin;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use hal::{
    dma::{SpiPeripheral, ChannelTypes},
    prelude::_esp_hal_dma_DmaTransfer,
    spi::{
        DuplexMode, IsFullDuplex,
    },
};
use hal::gpio::OutputPin;
use hal::spi::master::dma::SpiDmaTransfer;
use hal::spi::master::InstanceDma;

const DMA_BUFFER_SIZE: usize = 2048;
type SpiDma<'d, T, C, M> = hal::spi::master::dma::SpiDma<'d, T, C, M>;

fn send_u8<'d, T, C, M>(
    spi: SpiDma<'d, T, C, M>,
    words: DataFormat<'_>,
) -> Result<SpiDma<'d, T, C, M>, DisplayError>
where
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode + IsFullDuplex,
{
    match words {
        DataFormat::U8(slice) => {
            let send_buffer = dma_buffer1();
            send_buffer[..slice.len()].copy_from_slice(slice);

            let transfer = spi.dma_write(&send_buffer[..slice.len()]).unwrap();
            let (_, spi) = transfer.wait().unwrap();
            Ok(spi)
        }
        DataFormat::U16(slice) => {
            use byte_slice_cast::*;

            let send_buffer = dma_buffer1();
            send_buffer[..slice.len() * 2].copy_from_slice(slice.as_byte_slice());

            let transfer = spi.dma_write(&send_buffer[..slice.len() * 2]).unwrap();
            let (_, spi) = transfer.wait().unwrap();
            Ok(spi)
        }
        DataFormat::U16LE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_le();
            }

            let send_buffer = dma_buffer1();
            send_buffer[..slice.len() * 2].copy_from_slice(slice.as_byte_slice());

            let transfer = spi.dma_write(&send_buffer[..slice.len() * 2]).unwrap();
            let (_, spi) = transfer.wait().unwrap();
            Ok(spi)
        }
        DataFormat::U16BE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_be();
            }

            let send_buffer = dma_buffer1();
            send_buffer[..slice.len() * 2].copy_from_slice(slice.as_byte_slice());

            let transfer = spi.dma_write(&send_buffer[..slice.len() * 2]).unwrap();
            let (_, spi) = transfer.wait().unwrap();
            Ok(spi)
        }
        DataFormat::U8Iter(iter) => {
            let send_buffer = dma_buffer1();
            let mut send_buffer = &mut send_buffer[..];
            let mut spi = spi;

            loop {
                let mut idx = 0;
                loop {
                    let b = iter.next();

                    match b {
                        Some(b) => send_buffer[idx] = b,
                        None => break,
                    }

                    idx += 1;

                    if idx >= DMA_BUFFER_SIZE {
                        break;
                    }
                }

                if idx > 0 {
                    let transfer = spi.dma_write(&mut send_buffer[..idx]).unwrap();
                    (send_buffer, spi) = transfer.wait().unwrap();
                } else {
                    break;
                }
            }
            Ok(spi)
        }
        DataFormat::U16LEIter(iter) => {
            let send_buffer = [
                RefCell::new(Some(&mut dma_buffer1()[..])),
                RefCell::new(Some(&mut dma_buffer2()[..])),
            ];
            let mut spi = Some(spi);

            let mut current_buffer = 0;
            let mut transfer: Option<SpiDmaTransfer<'d, T, C, _, M>> = None;
            loop {
                if let Some(transfer) = transfer {
                    let (relaimed_buffer, reclaimed_spi) = transfer.wait().unwrap();
                    spi = Some(reclaimed_spi);
                    send_buffer[current_buffer].replace(Some(relaimed_buffer));

                    current_buffer = (current_buffer + 1) % send_buffer.len();
                }

                let buffer = send_buffer[current_buffer].take().unwrap();
                let mut idx = 0;
                loop {
                    let b = iter.next();

                    match b {
                        Some(b) => {
                            let b = b.to_le_bytes();
                            buffer[idx + 0] = b[0];
                            buffer[idx + 1] = b[1];
                        }
                        None => break,
                    }

                    idx += 2;

                    if idx >= DMA_BUFFER_SIZE {
                        break;
                    }
                }

                if idx > 0 {
                    transfer = Some(spi.take().unwrap().dma_write(&mut buffer[..idx]).unwrap());
                } else {
                    break;
                }
            }
            Ok(spi.unwrap())
        }
        DataFormat::U16BEIter(iter) => {
            let send_buffer = [
                RefCell::new(Some(&mut dma_buffer1()[..])),
                RefCell::new(Some(&mut dma_buffer2()[..])),
            ];
            let mut spi = Some(spi);

            let mut current_buffer = 0;
            let mut transfer: Option<SpiDmaTransfer<'d, T, C, _, M>> = None;
            loop {
                let buffer = send_buffer[current_buffer].take().unwrap();
                let mut idx = 0;
                loop {
                    let b = iter.next();

                    match b {
                        Some(b) => {
                            buffer[idx + 0] = ((b & 0xff00) >> 8) as u8;
                            buffer[idx + 1] = (b & 0xff) as u8;
                        }
                        None => break,
                    }

                    idx += 2;

                    if idx >= DMA_BUFFER_SIZE {
                        break;
                    }
                }

                if let Some(transfer) = transfer {
                    let (relaimed_buffer, reclaimed_spi) = transfer.wait().unwrap();
                    spi = Some(reclaimed_spi);
                    let done_buffer = current_buffer.wrapping_sub(1) % send_buffer.len();
                    send_buffer[done_buffer].replace(Some(relaimed_buffer));
                }

                if idx > 0 {
                    transfer = Some(spi.take().unwrap().dma_write(&mut buffer[..idx]).unwrap());
                    current_buffer = (current_buffer + 1) % send_buffer.len();
                } else {
                    break;
                }
            }
            Ok(spi.unwrap())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command as well as a chip-select pin
pub struct SPIInterface<'d, DC, CS, T, C, M>
where
    DC: OutputPin,
    CS: OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode,
{
    spi: RefCell<Option<SpiDma<'d, T, C, M>>>,
    dc: DC,
    cs: CS,
}

#[allow(unused)]
impl<'d, DC, CS, T, C, M> SPIInterface<'d, DC, CS, T, C, M>
where
    DC: OutputPin,
    CS: OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode,
{
    pub fn new(spi: SpiDma<'d, T, C, M>, dc: DC, cs: CS) -> Self {
        Self {
            spi: RefCell::new(Some(spi)),
            dc,
            cs,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SpiDma<'d, T, C, M>, DC, CS) {
        (self.spi.take().unwrap(), self.dc, self.cs)
    }
}

impl<'d, DC, CS, T, C, M> WriteOnlyDataCommand for SPIInterface<'d, DC, CS, T, C, M>
where
    DC: OutputPin + hal::prelude::_embedded_hal_digital_v2_OutputPin,
    CS: OutputPin + hal::prelude::_embedded_hal_digital_v2_OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode + IsFullDuplex,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let res = send_u8(self.spi.take().unwrap(), cmds);

        // Deassert chip select pin
        self.cs.set_high().ok();

        match res {
            Ok(spi) => {
                self.spi.replace(Some(spi));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let res = send_u8(self.spi.take().unwrap(), buf);

        // Deassert chip select pin
        self.cs.set_high().ok();

        match res {
            Ok(spi) => {
                self.spi.replace(Some(spi));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command pin
pub struct SPIInterfaceNoCS<'d, DC, T, C, M>
where
    DC: OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode,
{
    spi: RefCell<Option<SpiDma<'d, T, C, M>>>,
    dc: DC,
}

#[allow(unused)]
impl<'d, DC, T, C, M> SPIInterfaceNoCS<'d, DC, T, C, M>
where
    DC: OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode,
{
    /// Create new SPI interface for communciation with a display driver
    pub fn new(spi: SpiDma<'d, T, C, M>, dc: DC) -> Self {
        Self {
            spi: RefCell::new(Some(spi)),
            dc,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SpiDma<'d, T, C, M>, DC) {
        (self.spi.take().unwrap(), self.dc)
    }
}

impl<'d, DC, T, C, M> WriteOnlyDataCommand for SPIInterfaceNoCS<'d, DC, T, C, M>
where
    DC: OutputPin + hal::prelude::_embedded_hal_digital_v2_OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode + IsFullDuplex,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let res = send_u8(self.spi.take().unwrap(), cmds);

        match res {
            Ok(spi) => {
                self.spi.replace(Some(spi));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let res = send_u8(self.spi.take().unwrap(), buf);

        match res {
            Ok(spi) => {
                self.spi.replace(Some(spi));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

fn dma_buffer1() -> &'static mut [u8; DMA_BUFFER_SIZE] {
    static mut BUFFER: [u8; DMA_BUFFER_SIZE] = [0u8; DMA_BUFFER_SIZE];
    unsafe { &mut BUFFER }
}

fn dma_buffer2() -> &'static mut [u8; DMA_BUFFER_SIZE] {
    static mut BUFFER: [u8; DMA_BUFFER_SIZE] = [0u8; DMA_BUFFER_SIZE];
    unsafe { &mut BUFFER }
}