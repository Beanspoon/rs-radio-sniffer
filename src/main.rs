#![no_std]
#![no_main]

use nrf52832_hal as hal;
use rtt_target::{debug_rprintln, rprintln};

extern crate panic_halt;

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_target::rtt_init_default!();
    rprintln!("*** Rust-powered radio sniffer ***");

    let peripherals = hal::pac::Peripherals::take().unwrap();

    rprintln!("Initialising radio...");
    radio_init(peripherals);

    loop {}
}

fn radio_init(peripherals: hal::pac::Peripherals) {
    let radio = peripherals.RADIO;
    // Enable power
    radio.power.write(|w| unsafe { w.bits(1) });

    // Set frequency
    radio.frequency.write(|w| {
        w.frequency()
            .variant(0)
            .map()
            .variant(nrf52832_hal::pac::radio::frequency::MAP_A::LOW)
    });

    // Configure data whitening
    radio.datawhiteiv.write(|w| w.datawhiteiv().variant(37));

    // Configure addresses
    radio.base0.write(|w| w.base0().variant(0xadbeef00));
    radio.prefix0.write(|w| w.ap0().variant(0xde));
    radio.rxaddresses.write(|w| {
        w.addr0()
            .variant(nrf52832_hal::pac::radio::rxaddresses::ADDR0_A::ENABLED)
    });

    // Configure packets
    radio
        .pcnf0
        .write(|w| w.s0len().set_bit().s1len().variant(2).lflen().variant(6));
    radio.pcnf1.write(|w| {
        w.maxlen()
            .variant(10)
            .balen()
            .variant(3)
            .endian()
            .variant(nrf52832_hal::pac::radio::pcnf1::ENDIAN_A::LITTLE)
            .whiteen()
            .set_bit()
    });

    // Configure CRC
    radio.crccnf.write(|w| {
        w.len()
            .variant(nrf52832_hal::pac::radio::crccnf::LEN_A::THREE)
            .skipaddr()
            .set_bit()
    });
    radio.crcinit.write(|w| w.crcinit().variant(0x555555));

    let factors = [10, 9, 6, 4, 3, 1, 0];
    let factor_mask = 0;
    factors
        .iter()
        .fold(factor_mask, |mask, factor| mask | (1 << factor));
    debug_rprintln!("CRC polynomial factor mask: {:#034b}", factor_mask);
    radio.crcpoly.write(|w| w.crcpoly().variant(factor_mask));
}
