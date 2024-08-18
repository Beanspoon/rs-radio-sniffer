#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use nrf52832_hal as hal;

#[entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(peripherals.P0);
    let mut led0 = port0.p0_17.into_push_pull_output(hal::gpio::Level::High);
    let mut led1 = port0.p0_18.into_push_pull_output(hal::gpio::Level::High);

    let core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let mut systick_delay = hal::Delay::new(core_peripherals.SYST);

    loop {
        led0.set_low().unwrap();
        systick_delay.delay_ms(1000);
        led1.set_low().unwrap();
        systick_delay.delay_ms(1000);
        led0.set_high().unwrap();
        systick_delay.delay_ms(1000);
        led1.set_high().unwrap();
        systick_delay.delay_ms(1000);
    }
}
