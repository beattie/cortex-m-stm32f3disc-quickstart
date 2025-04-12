#![allow(unused_variables)]
#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_rtt_target as _; // logs messages to the host stderr; requires a debugger

use core::cell::RefCell;

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprintln};
use critical_section::Mutex;
 
use stm32f3xx_hal::{
    interrupt,
    gpio::{self, Edge, Input},
    pac::{Peripherals, NVIC},
    delay::Delay,
    prelude::*
};
use stm32f3xx_hal::pac::Interrupt;

type ButtonPin = gpio::PA0<Input>;

static mut  RUN:bool = true;

static BUTTON: Mutex<RefCell<Option<ButtonPin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello from stm32f3discovery quickstart");
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut delay = Delay::new(cp.SYST, rcc.cfgr.freeze(&mut flash.acr));
    let mut syscfg = dp.SYSCFG.constrain(&mut rcc.apb2);

    let mut exti = dp.EXTI;
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // Configuring the user button to trigger an interrupt when the button is pressed.
    let mut user_button = gpioa.pa0.into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
    syscfg.select_exti_interrupt_source(&user_button);
    user_button.trigger_on_edge(&mut exti, Edge::Rising);
    user_button.enable_interrupt(&mut exti);
    let interrupt_num = user_button.interrupt(); // hal::pac::Interrupt::EXTI0

    // Moving ownership to the global BUTTON so we can clear the interrupt pending bit.
    critical_section::with(|cs| *BUTTON.borrow(cs).borrow_mut() = Some(user_button));


    unsafe {
        NVIC::unmask(Interrupt::EXTI0);
    }


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
            if unsafe {RUN == true} {
                led.set_high().unwrap();
                delay.delay_ms(450u16);
                led.set_low().unwrap();
                delay.delay_ms(50u16);
            }
        }
    }
}

#[interrupt]
fn EXTI0() {
    rprintln!("User Button");
    critical_section::with(|cs| {
        // Clear the interrupt pending bit so we don't infinitely call this routine
        BUTTON
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt();
        unsafe {if RUN { RUN = false } else { RUN = true} };
    })
}
