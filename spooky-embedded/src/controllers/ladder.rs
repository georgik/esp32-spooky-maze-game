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

#[cfg(feature = "esp32c6")]
pub const ADC_LADDER_PIN_NUMBER: u8 = 2;

#[cfg(feature = "esp32s2")]
pub const ADC_LADDER_PIN_NUMBER: u8 = 6;

use embedded_hal::adc::OneShot;
use hal::adc::{AdcPin, ADC, ADC1};
use hal::gpio::{Analog, GpioPin};
use hal::prelude::nb;
use log::debug;
use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

pub struct LadderMovementController<'a> {
    last_action: Action,
    adc1: ADC<'a, ADC1>,
    adc_ladder_pin: AdcPin<GpioPin<Analog, ADC_LADDER_PIN_NUMBER>, ADC1>,
}

impl<'a> LadderMovementController<'a> {
    pub fn new(
        adc1: ADC<'a, ADC1>,
        adc_ladder_pin: AdcPin<GpioPin<Analog, ADC_LADDER_PIN_NUMBER>, ADC1>,
    ) -> Self {
        Self {
            last_action: Action::None,
            adc1,
            adc_ladder_pin,
        }
    }

    fn update_last_action(&mut self) {
        let resistor_value: u16 = nb::block!(self.adc1.read(&mut self.adc_ladder_pin)).unwrap();

        debug!("Resistor value: {}", resistor_value);
        if resistor_value > 4000 && resistor_value < 5000 {
            self.last_action = Action::Right;
        } else if resistor_value >= 5000 && resistor_value < 6000 {
            self.last_action = Action::Left;
        } else if resistor_value >= 6000 && resistor_value < 7000 {
            self.last_action = Action::Down;
        } else if resistor_value >= 7000 && resistor_value < 8180 {
            self.last_action = Action::Up;
        } else {
            self.last_action = Action::None;
        }
    }
}

impl MovementController for LadderMovementController<'_> {
    fn set_active(&mut self, _index: usize) {
        // Implementation for set_active, if required
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        // Update the last_action based on the latest resistor value
        self.update_last_action();
    }
}
