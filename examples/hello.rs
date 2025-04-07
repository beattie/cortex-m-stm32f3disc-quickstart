//! Prints "Hello, world!" on the host console using semihosting

#![no_main]
#![no_std]
#![allow(unused_imports)]

use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f3xx_hal::{self, interrupt};

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!");

    loop {}
}
