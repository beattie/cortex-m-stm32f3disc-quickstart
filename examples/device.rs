//! Using a device crate
//!
//! Crates generated using [`svd2rust`] are referred to as device crates. These crates provide an
//! API to access the peripherals of a device.
//!
//! [`svd2rust`]: https://crates.io/crates/svd2rust
//!
//! This example depends on the [`stm32f3xx_hal`] crate so you'll have to
//! uncomment it in your Cargo.toml.
//!
//! [`stm32f3xx_hal`]: https://crates.io/crates/stm32f3xx_hal
//!
//! ---

#![no_main]
#![no_std]

#[allow(unused_extern_crates)]
// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_rtt_target as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprint};
//use critical_section::Mutex;
 
use cortex_m::peripheral::syst::SystClkSource;
use stm32f3xx_hal::{
    interrupt,
    gpio::Edge,
    pac::{Peripherals, NVIC, Interrupt},
    prelude::*
};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut syscfg = dp.SYSCFG.constrain(&mut rcc.apb2);

    let mut syst = cp.SYST;
    // configure the system timer to wrap around every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(8_000_000); // 1s
    syst.enable_counter();

    let mut exti = dp.EXTI;
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // Configuring the user button to trigger an interrupt when the button is pressed.
    let mut user_button = gpioa.pa0.into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
    syscfg.select_exti_interrupt_source(&user_button);
    user_button.trigger_on_edge(&mut exti, Edge::Rising);
    user_button.enable_interrupt(&mut exti);
    // let interrupt_num = user_button.interrupt(); // hal::pac::Interrupt::EXTI0

    unsafe {
        NVIC::unmask(Interrupt::EXTI0);
    }

    loop {
        // busy wait until the timer wraps around
        while !syst.has_wrapped() {}

        // trigger the `EXTI0` interrupt
        NVIC::pend(Interrupt::EXTI0);
    }
}

#[interrupt]
fn EXTI0() {
    rprint!(".");
}
