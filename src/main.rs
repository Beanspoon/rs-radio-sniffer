#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;
use nrf52832_hal;

#[entry]
fn main() -> ! {
    loop {}
}
