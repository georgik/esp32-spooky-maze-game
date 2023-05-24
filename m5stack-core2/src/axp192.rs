
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

pub enum AXP192Error {
    NotSupported,
    InvalidArgument,
    ReadError,
    WriteError,
}

pub trait AXP192ReadWrite {
    fn read(&self, addr: u8, reg: u8, buffer: &mut [u8]) -> Result<(), AXP192Error>;
    fn write(&self, addr: u8, reg: u8, buffer: &[u8]) -> Result<(), AXP192Error>;
}

pub struct AXP192<'a, T: AXP192ReadWrite> {
    handle: &'a T,
}

impl<'a, T: AXP192ReadWrite> AXP192<'a, T> {
    pub fn new(handle: &'a T) -> Self {
        Self { handle }
    }

    fn read_coloumb_counter(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        // Implement read_coloumb_counter logic here
        unimplemented!()
    }

    fn read_battery_power(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        // Implement read_battery_power logic here
        unimplemented!()
    }

    fn read_adc(&self, reg: u8, buffer: &mut f32) -> Result<(), AXP192Error> {
        let mut tmp = [0u8; 4];
        let mut sensitivity = 1.0;
        let mut offset = 0.0;

        match reg {
            AXP192_ACIN_VOLTAGE | AXP192_VBUS_VOLTAGE => {
                // 1.7mV per LSB
                sensitivity = 1.7 / 1000.0;
            }
            AXP192_ACIN_CURRENT => {
                // 
                sensitivity = 0.625;
            }
            AXP192_VBUS_CURRENT => {
                // 
                sensitivity = 0.375;
            }
            AXP192_TEMP => {
                // 0.1C per LSB
                sensitivity = 0.1;
                offset = 0.0;
            }
            AXP192_TS_INPUT => {
                // 0.8mV per LSB
                sensitivity = 0.8 / 1000.0;
            }
            AXP192_BATTERY_POWER => {
                // 1.1mV per LSB
                sensitivity = 1.1 / 1000.0;
            }
            AXP192_BATTERY_VOLTAGE => {
                // 1.1mV per LSB
                sensitivity = 1.1 / 1000.0;
            }
            AXP192_CHARGE_CURRENT => {
                // 0.5mV per LSB
                sensitivity = 0.5 / 1000.0;
            }
            AXP192_DISCHARGE_CURRENT => {
                // 0.5mV per LSB
                sensitivity = 0.5 / 1000.0;
            }
            AXP192_APS_VOLTAGE => {
                // 1.4mV per LSB
                sensitivity = 1.4 / 1000.0;
            }
            _ => {
                return Err(AXP192Error::NotSupported);
            }
        }

        self.handle.read(AXP192_ADDRESS, reg, &mut tmp)?;

        *buffer = ((tmp[0] as u32) << 24
            | (tmp[1] as u32) << 16
            | (tmp[2] as u32) << 8
            | (tmp[3] as u32) << 0) as f32
            * sensitivity
            + offset;

        Ok(())

        }
    }

    fn read(&self, reg: u8, buffer: &mut f32) -> Result<(), AXP192Error> {
        match reg {
            AXP192_COULOMB_COUNTER => {
                self.read_coloumb_counter(buffer)?;
            }
            AXP192_BATTERY_POWER => {
                self.read_battery_power(buffer)?;
            }
            _ => {
                self.read_adc(reg, buffer)?;
            }
        }

        Ok(())
    }

    fn write(&self, reg: u8, buffer: &mut f32) -> Result<(), AXP192Error> {
        // Implement write logic here
        unimplemented!()
    }

    pub fn get_acin_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_VOLTAGE, buffer)
    }

    pub fn get_vbus_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_VOLTAGE, buffer)
    }

    pub fn get_acin_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_CURRENT, buffer)
    }

    pub fn get_vbus_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_CURRENT, buffer)
    }

    pub fn get_temp(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_TEMP, buffer)
    }

    pub fn get_ts_input(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_TS_INPUT, buffer)
    }

    pub fn get_battery_power(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_POWER, buffer)
    }

    pub fn get_battery_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_VOLTAGE, buffer)
    }

    pub fn get_charge_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_CHARGE_CURRENT, buffer)
    }

    pub fn get_discharge_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_DISCHARGE_CURRENT, buffer)
    }

    pub fn get_aps_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_APS_VOLTAGE, buffer)
    }

    pub fn get_coulomb_counter(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_COULOMB_COUNTER, buffer)
    }

    pub fn get_battery_charging_status(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CHARGING_STATUS, buffer)
    }

    pub fn get_acin_present(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_PRESENT, buffer)
    }

    pub fn get_vbus_present(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_PRESENT, buffer)
    }

    pub fn get_acin_overvoltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_OVERVOLTAGE, buffer)
    }

    pub fn get_acin_undervoltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_UNDERVOLTAGE, buffer)
    }

    pub fn get_vbus_overvoltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_OVERVOLTAGE, buffer)
    }

    pub fn get_vbus_undervoltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_UNDERVOLTAGE, buffer)
    }

    pub fn get_battery_temp_low(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_TEMP_LOW, buffer)
    }

    pub fn get_battery_temp_high(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_TEMP_HIGH, buffer)
    }

    pub fn get_chip_temp_low(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_CHIP_TEMP_LOW, buffer)
    }

    pub fn get_chip_temp_high(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_CHIP_TEMP_HIGH, buffer)
    }

    pub fn get_pek_short_press(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_PEK_SHORT_PRESS, buffer)
    }

    pub fn get_pek_long_press(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_PEK_LONG_PRESS, buffer)
    }

    pub fn get_timer_timeout(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_TIMER_TIMEOUT, buffer)
    }

    pub fn get_vbus_invalid(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_INVALID, buffer)
    }

    pub fn get_acin_invalid(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_INVALID, buffer)
    }

    pub fn get_battery_mode(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_MODE, buffer)
    }

    pub fn get_battery_charging(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CHARGING, buffer)
    }

    pub fn get_battery_connected(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CONNECTED, buffer)
    }

    pub fn get_battery_full(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_FULL, buffer)
    }

    pub fn get_battery_voltage_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_VOLTAGE_WARNING, buffer)
    }

    pub fn get_battery_temp_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_TEMP_WARNING, buffer)
    }

    pub fn get_acin_present_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_PRESENT_WARNING, buffer)
    }

    pub fn get_acin_overvoltage_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_OVERVOLTAGE_WARNING, buffer)
    }

    pub fn get_acin_undervoltage_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_UNDERVOLTAGE_WARNING, buffer)
    }

    pub fn get_vbus_present_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_PRESENT_WARNING, buffer)
    }

    pub fn get_vbus_overvoltage_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_OVERVOLTAGE_WARNING, buffer)
    }

    pub fn get_vbus_undervoltage_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_UNDERVOLTAGE_WARNING, buffer)
    }

    pub fn get_pek_short_press_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_PEK_SHORT_PRESS_WARNING, buffer)
    }

    pub fn get_pek_long_press_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_PEK_LONG_PRESS_WARNING, buffer)
    }

    pub fn get_timer_timeout_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_TIMER_TIMEOUT_WARNING, buffer)
    }

    pub fn get_vbus_invalid_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_INVALID_WARNING, buffer)
    }

    pub fn get_acin_invalid_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_INVALID_WARNING, buffer)
    }

    pub fn get_battery_charging_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CHARGING_WARNING, buffer)
    }

    pub fn get_battery_connected_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CONNECTED_WARNING, buffer)
    }

    pub fn get_battery_full_warning(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_FULL_WARNING, buffer)
    }

    pub fn get_battery_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_VOLTAGE, buffer)
    }

    pub fn get_battery_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CURRENT, buffer)
    }

    pub fn get_acin_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_VOLTAGE, buffer)
    }

    pub fn get_acin_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ACIN_CURRENT, buffer)
    }

    pub fn get_vbus_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_VOLTAGE, buffer)
    }

    pub fn get_vbus_current(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_VBUS_CURRENT, buffer)
    }

    pub fn get_aps_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_APS_VOLTAGE, buffer)
    }

    pub fn get_ts_pin_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_TS_PIN_VOLTAGE, buffer)
    }

    pub fn get_timer_control(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_TIMER_CONTROL, buffer)
    }

    pub fn get_battery_charging_control(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_CHARGING_CONTROL, buffer)
    }

    pub fn get_pek_control(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_PEK_CONTROL, buffer)
    }

    pub fn get_adc_enable_1(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ADC_ENABLE_1, buffer)
    }

    pub fn get_adc_enable_2(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ADC_ENABLE_2, buffer)
    }

    pub fn get_adc_rate_token(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ADC_RATE_TOKEN, buffer)
    }

    pub fn get_adc_sampling_rate(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_ADC_SAMPLING_RATE, buffer)
    }

    pub fn get_battery_temperature(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_BATTERY_TEMPERATURE, buffer)
    }

    pub fn get_gpi_status(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_GPIO_STATUS, buffer)
    }

    pub fn get_gpi_io_0_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_GPIO0_VOLTAGE, buffer)
    }

    pub fn get_gpi_io_1_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_GPIO1_VOLTAGE, buffer)
    }

    pub fn get_gpi_io_2_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_GPIO2_VOLTAGE, buffer)
    }

    pub fn get_gpi_io_3_voltage(&self, buffer: &mut f32) -> Result<(), AXP192Error> {
        self.read(AXP192_GPIO3_VOLTAGE, buffer)
    }

}






