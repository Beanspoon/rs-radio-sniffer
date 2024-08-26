#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::{CriticalSection, Mutex};
use nrf52832_hal::{self as hal, pac::Peripherals};
use rtt_target::{debug_rprintln, rprint, rprintln};

use hal::pac::interrupt;

extern crate panic_halt;

struct Packet {
    length: u8,
    payload: [u8; 255],
}

static mut PACKET: Packet = Packet {
    length: 0,
    payload: [0; 255],
};

static P: Mutex<RefCell<Option<Peripherals>>> = Mutex::new(RefCell::new(None));

#[interrupt]
unsafe fn RADIO() {
    static mut COUNTER: u32 = 0;

    rprintln!(
        "[{}] l: {}; {:02x?}",
        COUNTER,
        PACKET.length,
        &PACKET.payload[..PACKET.length as usize]
    );
    *COUNTER += 1;

    let cs = unsafe { CriticalSection::new() };
    if let Some(peripherals) = P.borrow(&cs).take() {
        peripherals.RADIO.events_end.write(|w| unsafe { w.bits(0) });
        peripherals
            .RADIO
            .tasks_start
            .write(|w| unsafe { w.bits(1) });
        P.borrow(&cs).replace(Some(peripherals));
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();
    rprintln!("*** Rust-powered radio sniffer ***");

    let peripherals = hal::pac::Peripherals::take().unwrap();

    rprint!("Starting HF clock...");
    peripherals
        .CLOCK
        .tasks_hfclkstart
        .write(|w| unsafe { w.bits(1) });

    while peripherals.CLOCK.hfclkstat.read().state()
        == hal::pac::clock::hfclkstat::STATE_A::NOT_RUNNING
    {}
    rprintln!("started!");

    rprintln!("Initialising radio...");
    radio_init(&peripherals);

    // Configure radio events
    hal::pac::NVIC::mask(hal::pac::Interrupt::RADIO);
    peripherals.RADIO.intenset.write(|w| w.end().set_bit());
    unsafe { hal::pac::NVIC::unmask(hal::pac::Interrupt::RADIO) };

    let packet_location = unsafe { core::ptr::addr_of!(PACKET) };
    rprintln!("Packet address: {:?}", packet_location as u32);

    peripherals
        .RADIO
        .packetptr
        .write(|w| w.packetptr().variant(packet_location as u32));

    let x = peripherals.RADIO.packetptr.read();
    rprintln!("Read: {:#?}", x.bits());

    peripherals.RADIO.tasks_rxen.write(|w| unsafe { w.bits(1) });

    {
        let cs = unsafe { CriticalSection::new() };
        P.borrow(&cs).replace(Some(peripherals));
    }

    loop {}
}

fn radio_init(peripherals: &hal::pac::Peripherals) {
    let radio = &peripherals.RADIO;
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
            .variant(255)
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
    let factor_mask = factors.iter().fold(0, |mask, factor| mask | (1 << factor));
    debug_rprintln!("CRC polynomial factor mask: {:#034b}", factor_mask);
    radio.crcpoly.write(|w| w.crcpoly().variant(factor_mask));

    // Enable shortcuts
    radio.shorts.write(|w| w.ready_start().set_bit());
}
