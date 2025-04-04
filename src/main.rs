#![allow(unused_variables)]
#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32f3xx_hal::{pac::Peripherals, delay::Delay, prelude::*};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut delay = Delay::new(cp.SYST, rcc.cfgr.freeze(&mut flash.acr));

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);


    let mut leds = [
        gpioe.pe9.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe10.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe11.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe12.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe13.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe14.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe15.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        gpioe.pe8.into_push_pull_output(
                        &mut gpioe.moder, &mut gpioe.otyper).downgrade(),
    ];

    for led in leds.iter_mut() {
        led.set_low().unwrap();
        delay.delay_ms(50u32);
    }
    loop {
        for led in leds.iter_mut() {
            led.set_high().unwrap();
            delay.delay_ms(450u16);
            led.set_low().unwrap();
            delay.delay_ms(50u16);
        }
        /*
        leds[0].toggle().unwrap();
        delay.delay_ms(1000u32);
        */
    }
}
