use esp_println::println;

// Based on https://github.com/tuupola/axp192

const AXP192_ADDRESS: u8 = 0x34;
/* Power control registers */
const AXP192_POWER_STATUS: u8 = 0x00;
const AXP192_CHARGE_STATUS: u8 = 0x01;
const AXP192_OTG_VBUS_STATUS: u8 = 0x04;
const AXP192_DATA_BUFFER0: u8 = 0x06;
const AXP192_DATA_BUFFER1: u8 = 0x07;
const AXP192_DATA_BUFFER2: u8 = 0x08;
const AXP192_DATA_BUFFER3: u8 = 0x09;
const AXP192_DATA_BUFFER4: u8 = 0x0a;
const AXP192_DATA_BUFFER5: u8 = 0x0b;
/* Output control: 2 EXTEN, 0 DCDC2 */
const AXP192_EXTEN_DCDC2_CONTROL: u8 = 0x10;
/* Power output control: 6 EXTEN, 4 DCDC2, 3 LDO3, 2 LDO2, 1 DCDC3, 0 DCDC1 */
const AXP192_DCDC13_LDO23_CONTROL: u8 = 0x12;
const AXP192_DCDC2_VOLTAGE: u8 = 0x23;
const AXP192_DCDC2_SLOPE: u8 = 0x25;
const AXP192_DCDC1_VOLTAGE: u8 = 0x26;
const AXP192_DCDC3_VOLTAGE: u8 = 0x27;
/* Output voltage control: 7-4 LDO2, 3-0 LDO3 */
const AXP192_LDO23_VOLTAGE: u8 = 0x28;
const AXP192_VBUS_IPSOUT_CHANNEL: u8 = 0x30;
const AXP192_SHUTDOWN_VOLTAGE: u8 = 0x31;
const AXP192_SHUTDOWN_BATTERY_CHGLED_CONTROL: u8 = 0x32;
const AXP192_CHARGE_CONTROL_1: u8 = 0x33;
const AXP192_CHARGE_CONTROL_2: u8 = 0x34;
const AXP192_BATTERY_CHARGE_CONTROL: u8 = 0x35;
const AXP192_PEK: u8 = 0x36;
const AXP192_DCDC_FREQUENCY: u8 = 0x37;
const AXP192_BATTERY_CHARGE_LOW_TEMP: u8 = 0x38;
const AXP192_BATTERY_CHARGE_HIGH_TEMP: u8 = 0x39;
const AXP192_APS_LOW_POWER1: u8 = 0x3A;
const AXP192_APS_LOW_POWER2: u8 = 0x3B;
const AXP192_BATTERY_DISCHARGE_LOW_TEMP: u8 = 0x3c;
const AXP192_BATTERY_DISCHARGE_HIGH_TEMP: u8 = 0x3d;
const AXP192_DCDC_MODE: u8 = 0x80;
const AXP192_ADC_ENABLE_1: u8 = 0x82;
const AXP192_ADC_ENABLE_2: u8 = 0x83;
const AXP192_ADC_RATE_TS_PIN: u8 = 0x84;
const AXP192_GPIO30_INPUT_RANGE: u8 = 0x85;
const AXP192_GPIO0_ADC_IRQ_RISING: u8 = 0x86;
const AXP192_GPIO0_ADC_IRQ_FALLING: u8 = 0x87;
const AXP192_TIMER_CONTROL: u8 = 0x8a;
const AXP192_VBUS_MONITOR: u8 = 0x8b;
const AXP192_TEMP_SHUTDOWN_CONTROL: u8 = 0x8f;

/* GPIO control registers */
const AXP192_GPIO0_CONTROL: u8 = 0x90;
const AXP192_GPIO0_LDOIO0_VOLTAGE: u8 = 0x91;
const AXP192_GPIO1_CONTROL: u8 = 0x92;
const AXP192_GPIO2_CONTROL: u8 = 0x93;
const AXP192_GPIO20_SIGNAL_STATUS: u8 = 0x94;
const AXP192_GPIO43_FUNCTION_CONTROL: u8 = 0x95;
const AXP192_GPIO43_SIGNAL_STATUS: u8 = 0x96;
const AXP192_GPIO20_PULLDOWN_CONTROL: u8 = 0x97;
const AXP192_PWM1_FREQUENCY: u8 = 0x98;
const AXP192_PWM1_DUTY_CYCLE_1: u8 = 0x99;
const AXP192_PWM1_DUTY_CYCLE_2: u8 = 0x9a;
const AXP192_PWM2_FREQUENCY: u8 = 0x9b;
const AXP192_PWM2_DUTY_CYCLE_1: u8 = 0x9c;
const AXP192_PWM2_DUTY_CYCLE_2: u8 = 0x9d;
const AXP192_N_RSTO_GPIO5_CONTROL: u8 = 0x9e;

/* Interrupt control registers */
const AXP192_ENABLE_CONTROL_1: u8 = 0x40;
const AXP192_ENABLE_CONTROL_2: u8 = 0x41;
const AXP192_ENABLE_CONTROL_3: u8 = 0x42;
const AXP192_ENABLE_CONTROL_4: u8 = 0x43;
const AXP192_ENABLE_CONTROL_5: u8 = 0x4a;
const AXP192_IRQ_STATUS_1: u8 = 0x44;
const AXP192_IRQ_STATUS_2: u8 = 0x45;
const AXP192_IRQ_STATUS_3: u8 = 0x46;
const AXP192_IRQ_STATUS_4: u8 = 0x47;
const AXP192_IRQ_STATUS_5: u8 = 0x4d;

/* ADC data registers */
const AXP192_ACIN_VOLTAGE: u8 = 0x56;
const AXP192_ACIN_CURRENT: u8 = 0x58;
const AXP192_VBUS_VOLTAGE: u8 = 0x5a;
const AXP192_VBUS_CURRENT: u8 = 0x5c;
const AXP192_TEMP: u8 = 0x5e;
const AXP192_TS_INPUT: u8 = 0x62;
const AXP192_GPIO0_VOLTAGE: u8 = 0x64;
const AXP192_GPIO1_VOLTAGE: u8 = 0x66;
const AXP192_GPIO2_VOLTAGE: u8 = 0x68;
const AXP192_GPIO3_VOLTAGE: u8 = 0x6a;
const AXP192_BATTERY_POWER: u8 = 0x70;
const AXP192_BATTERY_VOLTAGE: u8 = 0x78;
const AXP192_CHARGE_CURRENT: u8 = 0x7a;
const AXP192_DISCHARGE_CURRENT: u8 = 0x7c;
const AXP192_APS_VOLTAGE: u8 = 0x7e;
const AXP192_CHARGE_COULOMB: u8 = 0xb0;
const AXP192_DISCHARGE_COULOMB: u8 = 0xb4;
const AXP192_COULOMB_COUNTER_CONTROL: u8 = 0xb8;

/* Computed ADC */
const AXP192_COULOMB_COUNTER: u8 = 0xff;

//12, 25-28, 92-93
pub enum Command {
    Dcdc13Ldo23Control(bool),
    Dcdc2Slope(bool),
    Dcdc1Voltage(bool),
    Dcdc3Voltage(bool),
    Ldo23Voltage(bool),
    Gpio1Control(bool),
    Gpio2Control(bool)
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
            //Command::Dcdc3Voltage(on) => ([AXP192_ADDRESS, AXP192_LDO2 , 0x0], 3),
            Command::Dcdc13Ldo23Control(on) => ([AXP192_DCDC13_LDO23_CONTROL, 119], 2),
            Command::Dcdc2Slope(on) => ([AXP192_DCDC2_SLOPE, 0x0], 2),
            Command::Dcdc1Voltage(on) => ([AXP192_DCDC1_VOLTAGE, 106], 2),
            Command::Dcdc3Voltage(on) => ([AXP192_DCDC3_VOLTAGE, 104], 2),
            Command::Ldo23Voltage(on) => ([AXP192_LDO23_VOLTAGE, 242], 2),
            Command::Gpio1Control(on) => ([AXP192_GPIO1_CONTROL, 0x0], 2),
            Command::Gpio2Control(on) => ([AXP192_GPIO2_CONTROL, 104], 2),
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

        match cmd {
            DataFormat::U8(data) => {

            let result = self.i2c
                .write_read(self.addr, &[data[0]], &mut data_buf)
                .map_err(|_| Axp192Error::WriteError);
            println!("read value for command {:?}: {:?}", data[0], data_buf[0]);

            println!("write value for command {:?}: {:?}", data[0], data[1]);
            self.i2c
                .write(self.addr, &data)
                .map_err(|_| Axp192Error::WriteError)
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
        Command::Dcdc13Ldo23Control(true).send(&mut self.interface)?;
        Command::Dcdc2Slope(true).send(&mut self.interface)?;
        Command::Dcdc1Voltage(true).send(&mut self.interface)?;
        Command::Dcdc3Voltage(true).send(&mut self.interface)?;
        Command::Ldo23Voltage(true).send(&mut self.interface)?;
        Command::Gpio1Control(true).send(&mut self.interface)?;
        Command::Gpio2Control(true).send(&mut self.interface)?;

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



