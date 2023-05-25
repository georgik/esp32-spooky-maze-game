use esp_println::println;


pub const AXP192_ACIN_VOLTAGE: u8 = 0x56;
pub const AXP192_VBUS_VOLTAGE: u8 = 0x57;
pub const AXP192_ACIN_CURRENT: u8 = 0x58;
pub const AXP192_VBUS_CURRENT: u8 = 0x59;
pub const AXP192_TEMP: u8 = 0x5E;
pub const AXP192_TS_INPUT: u8 = 0x61;
pub const AXP192_BATTERY_POWER: u8 = 0x70;
pub const AXP192_BATTERY_VOLTAGE: u8 = 0x78;
pub const AXP192_CHARGE_CURRENT: u8 = 0x7A;
pub const AXP192_DISCHARGE_CURRENT: u8 = 0x7C;
pub const AXP192_APS_VOLTAGE: u8 = 0x7E;
pub const AXP192_COULOMB_COUNTER: u8 = 0xB0;
pub const AXP192_ADDRESS: u8 = 0x34;
pub const AXP192_LDO23_VOLTAGE: u8 = 0x28;
pub const AXP192_LDO2: u8 = 0x12;
pub const AXP192_DCDC3_VOLTAGE: u8 = 0x27;

pub enum Command {
    Dcdc3Voltage(bool),
    Ldo2(bool),
    Ldo23Voltage(bool),
    EndOfCommands
}

pub enum DataFormat<'a> {
    /// Slice of unsigned bytes
    U8(&'a [u8]),
}

impl Command {
    // Send command to AXP192
    pub fn send<I>(self, iface: &mut I) -> Result<(), Axp192Error>
    where
        I: Axp192ReadWrite,
    {
        let (data, len) = match self {
            // Command structure: address, command, data, count & 0xf1
            Command::Dcdc3Voltage(on) => ([AXP192_ADDRESS, AXP192_LDO2 , 0x0], 3),
            Command::Ldo23Voltage(on) => ([AXP192_ADDRESS, 0xcc, 0x0], 2),
            Command::Ldo2(on) => ([AXP192_ADDRESS, AXP192_LDO2 , 0xff], 3),
            Command::EndOfCommands => ([AXP192_ADDRESS, 0xff, 0x0], 3),
        };
        iface.send_commands(DataFormat::U8(&data[0..len]))
    }
}

#[derive(Debug)]
pub enum Axp192Error {
    NotSupported,
    InvalidArgument,
    ReadError,
    WriteError,
}

pub trait Axp192ReadWrite {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), Axp192Error>;
    // fn read(&self, addr: u8, reg: u8, buffer: &mut [u8]) -> Result<(), Axp192Error>;
    // fn write(&self, addr: u8, reg: u8, buffer: &[u8]) -> Result<(), Axp192Error>;
}

pub struct Axp192<I> {
    interface: I,
}

// Implement Axp192ReadWrite for I2CInterface
impl<I> Axp192ReadWrite for I2CInterface<I>
where
    I: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
{
    // Send commands over I2C to AXP192
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), Axp192Error> {
        let mut data_buf = [0];
        println!("send_commands");
        match cmd {
            DataFormat::U8(data) => {
                let result = self.i2c
                    .write_read(self.addr, &[0x12], &mut data_buf)
                    .map_err(|_| Axp192Error::WriteError);
                println!("data_buf: {:?}", data_buf);

                // Turn on all default values
                self.i2c
                    .write(self.addr, &[0x12, 119])
                    .map_err(|_| Axp192Error::WriteError).unwrap();
                result

                // self.i2c
                //     .read(self.addr, data)
                //     .map_err(|_| Axp192Error::ReadError);
                // self.i2c
                //     .write(self.addr, data)
                //     .map_err(|_| Axp192Error::WriteError)
            }
        }
    }

    // fn read(&self, addr: u8, reg: u8, buffer: &mut [u8]) -> Result<(), Axp192Error> {
    //     // Implement read logic here
    //     unimplemented!()
    // }

    // fn write(&self, addr: u8, reg: u8, buffer: &[u8]) -> Result<(), Axp192Error> {
    //     // Implement write logic here
    //     unimplemented!()
    // }
}

impl<I> Axp192<I>
    where
        I: Axp192ReadWrite,
{
    // Create a new AXP192 interface
    pub fn new(interface: I) -> Self {
        Self { interface }
    }

    // Initialize AXP192
    pub fn init(&mut self) -> Result<(), Axp192Error> {
        // Command::Ldo23Voltage(true).send(&mut self.interface)?;
        Command::Ldo2(true).send(&mut self.interface)?;
        Command::EndOfCommands.send(&mut self.interface)?;
        Ok(())
    }

}

pub struct I2CInterface<I2C> {
    i2c: I2C,
    addr: u8,
    data_byte: u8,
}

impl<I2C> I2CInterface<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write/*+ embedded_hal::blocking::i2c::WriteRead*/,
{
    /// Create new I2C interface for communication with a display driver
    pub fn new(i2c: I2C, addr: u8, data_byte: u8) -> Self {
        Self {
            i2c,
            addr,
            data_byte,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver
    pub fn release(self) -> I2C {
        self.i2c
    }
}

#[derive(Debug, Copy, Clone)]
pub struct I2CPowerManagementInterface(());


impl I2CPowerManagementInterface {
    pub fn new<I>(i2c: I) -> I2CInterface<I>
    where
        I: embedded_hal::blocking::i2c::Write,
    {
        Self::new_custom_address(i2c, AXP192_ADDRESS)
    }

    /// Create a new I2C interface with a custom address.
    pub fn new_custom_address<I>(i2c: I, address: u8) -> I2CInterface<I>
    where
        I: embedded_hal::blocking::i2c::Write,
    {
        I2CInterface::new(i2c, address, 0x34)
    }
}



