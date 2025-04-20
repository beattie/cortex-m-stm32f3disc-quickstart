#![no_std]
#![no_main]
use panic_rtt_target as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use stm32f3xx_hal::{pac, prelude::*, serial};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Serial Demo");

    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze(&mut flash.acr);

    // Configure GPIO pins PC4 and PC5 for UART alternate function
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);
    let tx = gpioc.pc4.into_af_push_pull::<7>(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);
    let rx = gpioc.pc5.into_af_push_pull::<7>(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);

    // Configure UART1 (or your desired UART)
    let mut serial = serial::Serial::new(
        dp.USART1,
        (tx, rx),
        115200.Bd(), // Set your desired baud rate
        clocks,
        &mut rcc.apb2,
    );

    loop {
        match serial.read() {
            Ok(byte) => {
                serial.write(byte).unwrap();
            }
            Err(_error) => {
                continue;
            }
        };
    }
}
