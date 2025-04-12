//! Prints "Hello, world!" on the host console using semihosting

#![no_main]
#![no_std]
#![allow(unused_imports)]

// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger
use panic_rtt_target as _;

use cortex_m_rt::entry;
// use cortex_m_semihosting::hprintln;
use stm32f3xx_hal::{self, interrupt};
use rtt_target::{rtt_init_print, rprintln};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");

    loop {}
}
