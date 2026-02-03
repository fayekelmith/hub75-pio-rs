//! Displays an animated Nyan cat
#![no_std]
#![no_main]
#![feature(generic_const_exprs)]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use panic_probe as _;

use bsp::hal::pio::PIOExt;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use hub75_pio;
use hub75_pio::dma::DMAExt;
use hub75_pio::lut::GammaLut;

use rp_pico as bsp;

/// Display buffer for 64x64 RGB LED matrix with 12-bit color depth (shades)
static mut DISPLAY_BUFFER: hub75_pio::DisplayMemory<64, 64, 12> = hub75_pio::DisplayMemory::new();


#[entry]
fn main() -> ! {
    info!("Program start");

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Split PIO0 SM
    let (mut pio, sm0, sm1, sm2, _) = pac.PIO0.split(&mut pac.RESETS);

    // Reset DMA
    let resets = pac.RESETS;
    resets.reset.modify(|_, w| w.dma().set_bit());
    resets.reset.modify(|_, w| w.dma().clear_bit());
    while resets.reset_done.read().dma().bit_is_clear() {}

    // Split DMA
    let dma = pac.DMA.split();

    let lut = {
        let lut: GammaLut<12, _, _> = GammaLut::new();
        lut.init((1.0, 1.0, 1.0))
    };
    let mut display = unsafe {
        hub75_pio::Display::new(
            &mut DISPLAY_BUFFER,
            hub75_pio::DisplayPins {
                r1: pins.gpio0.into_function().into_pull_type().into_dyn_pin(),
                g1: pins.gpio1.into_function().into_pull_type().into_dyn_pin(),
                b1: pins.gpio2.into_function().into_pull_type().into_dyn_pin(),
                r2: pins.gpio3.into_function().into_pull_type().into_dyn_pin(),
                g2: pins.gpio4.into_function().into_pull_type().into_dyn_pin(),
                b2: pins.gpio5.into_function().into_pull_type().into_dyn_pin(),
                addr: [
                    pins.gpio6.into_function().into_pull_type().into_dyn_pin(),
                    pins.gpio7.into_function().into_pull_type().into_dyn_pin(),
                    pins.gpio8.into_function().into_pull_type().into_dyn_pin(),
                    pins.gpio9.into_function().into_pull_type().into_dyn_pin(),
                    pins.gpio10.into_function().into_pull_type().into_dyn_pin(),
                ],
                clk: pins.gpio11.into_function().into_pull_type().into_dyn_pin(),
                lat: pins.gpio12.into_function().into_pull_type().into_dyn_pin(),
                oe: pins.gpio13.into_function().into_pull_type().into_dyn_pin(),
            },
            &mut pio,
            (sm0, sm1, sm2),
            (dma.ch0, dma.ch1, dma.ch2, dma.ch3),
            false,
            &lut,
        )
    };

    // Row scanner
   loop {
    for row in 0..64 {
        for x in 0..64 {
            display.set_pixel(x, row, Rgb888::WHITE);
        }
        display.commit();
        delay.delay_ms(100); 
    }
}
}
