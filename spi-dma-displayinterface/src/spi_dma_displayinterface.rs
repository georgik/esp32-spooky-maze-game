//! DMA SPI interface for display drivers

// Source: https://github.com/bjoernQ/rust-esp32s3-ili9341/blob/main/src/spi_dma_displayinterface.rs

use core::cell::RefCell;

#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;
#[cfg(feature = "esp32c6")]
use esp32c6_hal as hal;
#[cfg(feature = "esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature = "esp32s3")]
use esp32s3_hal as hal;

use byte_slice_cast::AsByteSlice;
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use hal::gpio::{Output, OutputPin, PushPull};
use hal::spi::master::dma::SpiDmaTransfer;
use hal::spi::master::InstanceDma;
use hal::{
    dma::{ChannelTypes, SpiPeripheral},
    prelude::_esp_hal_dma_DmaTransfer,
    spi::{DuplexMode, IsFullDuplex},
};

const DMA_BUFFER_SIZE: usize = 4096;
type SpiDma<'d, T, C, M> = hal::spi::master::dma::SpiDma<'d, T, C, M>;

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
    avg_data_len_hint: usize,
    spi: RefCell<Option<SpiDma<'d, T, C, M>>>,
    transfer: RefCell<Option<SpiDmaTransfer<'d, T, C, &'static mut [u8], M>>>,
    dc: DC,
    cs: Option<CS>,
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
    pub fn new(avg_data_len_hint: usize, spi: SpiDma<'d, T, C, M>, dc: DC, cs: CS) -> Self {
        Self {
            avg_data_len_hint,
            spi: RefCell::new(Some(spi)),
            transfer: RefCell::new(None),
            dc,
            cs: Some(cs),
        }
    }

    /// Consume the display interface and return
    /// the underlying peripheral driver and GPIO pins used by it
    pub fn release(self) -> (SpiDma<'d, T, C, M>, DC, Option<CS>) {
        (self.spi.take().unwrap(), self.dc, self.cs)
    }

    fn send_u8(&mut self, words: DataFormat<'_>) -> Result<(), DisplayError>
    where
        T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
        C: ChannelTypes,
        C::P: SpiPeripheral,
        M: DuplexMode + IsFullDuplex,
    {
        if let Some(transfer) = self.transfer.take() {
            let (_, reclaimed_spi) = transfer.wait().unwrap();
            self.spi.replace(Some(reclaimed_spi));
        }

        match words {
            DataFormat::U8(slice) => {
                use byte_slice_cast::*;

                let send_buffer = dma_buffer1();
                send_buffer[..slice.len()].copy_from_slice(slice.as_byte_slice());

                self.single_transfer(&mut send_buffer[..slice.len()]);
            }
            DataFormat::U16(slice) => {
                use byte_slice_cast::*;

                let send_buffer = dma_buffer1();
                send_buffer[..slice.len() * 2].copy_from_slice(slice.as_byte_slice());

                self.single_transfer(&mut send_buffer[..slice.len() * 2]);
            }
            DataFormat::U16LE(slice) => {
                use byte_slice_cast::*;
                for v in slice.as_mut() {
                    *v = v.to_le();
                }

                let send_buffer = dma_buffer1();
                send_buffer[..slice.len() * 2].copy_from_slice(slice.as_byte_slice());

                self.single_transfer(&mut send_buffer[..slice.len() * 2]);
            }
            DataFormat::U16BE(slice) => {
                use byte_slice_cast::*;
                for v in slice.as_mut() {
                    *v = v.to_be();
                }

                let send_buffer = dma_buffer1();
                send_buffer[..slice.len() * 2].copy_from_slice(slice.as_byte_slice());

                self.single_transfer(&mut send_buffer[..slice.len() * 2]);
            }
            DataFormat::U8Iter(iter) => {
                self.iter_transfer(iter, |v| v.to_be_bytes());
            }
            DataFormat::U16LEIter(iter) => {
                self.iter_transfer(iter, |v| v.to_le_bytes());
            }
            DataFormat::U16BEIter(iter) => {
                self.iter_transfer(iter, |v| v.to_be_bytes());
            }
            _ => {
                return Err(DisplayError::DataFormatNotImplemented);
            }
        }
        Ok(())
    }

    fn single_transfer(&mut self, send_buffer: &'static mut [u8])
    where
        M: DuplexMode + IsFullDuplex,
    {
        let transfer = self.spi.take().unwrap().dma_write(send_buffer).unwrap();
        let (_, reclaimed_spi) = transfer.wait().unwrap();
        self.spi.replace(Some(reclaimed_spi));
    }

    fn iter_transfer<WORD>(
        &mut self,
        iter: &mut dyn Iterator<Item = WORD>,
        convert: fn(WORD) -> <WORD as num_traits::ToBytes>::Bytes,
    ) where
        WORD: num_traits::int::PrimInt + num_traits::ToBytes,
        M: DuplexMode + IsFullDuplex,
    {
        let mut desired_chunk_sized =
            self.avg_data_len_hint - ((self.avg_data_len_hint / DMA_BUFFER_SIZE) * DMA_BUFFER_SIZE);
        let mut spi = Some(self.spi.take().unwrap());
        let mut current_buffer = 0;
        let mut transfer: Option<SpiDmaTransfer<'d, T, C, _, M>> = None;
        loop {
            let buffer = if current_buffer == 0 {
                &mut dma_buffer1()[..]
            } else {
                &mut dma_buffer2()[..]
            };
            let mut idx = 0;
            loop {
                let b = iter.next();

                match b {
                    Some(b) => {
                        let b = convert(b);
                        let b = b.as_byte_slice();
                        buffer[idx + 0] = b[0];
                        if b.len() == 2 {
                            buffer[idx + 1] = b[1];
                        }
                        idx += b.len();
                    }
                    None => break,
                }

                if idx >= usize::min(desired_chunk_sized, DMA_BUFFER_SIZE) {
                    break;
                }
            }
            desired_chunk_sized = DMA_BUFFER_SIZE;

            if let Some(transfer) = transfer {
                if idx > 0 {
                    let (relaimed_buffer, reclaimed_spi) = transfer.wait().unwrap();
                    spi = Some(reclaimed_spi);
                } else {
                    // last transaction inflight
                    self.transfer.replace(Some(transfer));
                }
            }

            if idx > 0 {
                transfer = Some(spi.take().unwrap().dma_write(&mut buffer[..idx]).unwrap());
                current_buffer = (current_buffer + 1) % 2;
            } else {
                break;
            }
        }
        self.spi.replace(spi);
    }
}

pub fn new_no_cs<'d, DC, T, C, M>(
    avg_data_len_hint: usize,
    spi: SpiDma<'d, T, C, M>,
    dc: DC,
) -> SPIInterface<'d, DC, hal::gpio::Gpio0<Output<PushPull>>, T, C, M>
where
    DC: OutputPin,
    T: InstanceDma<C::Tx<'d>, C::Rx<'d>>,
    C: ChannelTypes,
    C::P: SpiPeripheral,
    M: DuplexMode,
{
    SPIInterface {
        avg_data_len_hint,
        spi: RefCell::new(Some(spi)),
        transfer: RefCell::new(None),
        dc,
        cs: None::<hal::gpio::Gpio0<Output<PushPull>>>,
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
        if let Some(cs) = self.cs.as_mut() {
            cs.set_low().map_err(|_| DisplayError::CSError)?;
        }

        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let res = self.send_u8(cmds);

        // Deassert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_high().ok();
        }

        res
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // Assert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_low().map_err(|_| DisplayError::CSError)?;
        }

        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let res = self.send_u8(buf);

        // Deassert chip select pin
        if let Some(cs) = self.cs.as_mut() {
            cs.set_high().ok();
        }

        res
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