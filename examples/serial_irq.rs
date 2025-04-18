#![no_std]
#![no_main]
use panic_rtt_target as _; // logs messages to the host stderr; requires a debugger

#[allow(unused_imports)]
use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial,
    serial::Serial,
    interrupt,
    pac::USART1,
    gpio::{
        gpioc::
        {PC4, PC5}, PushPull, AF7},
};

type SerialType = Serial<USART1, (PC4<AF7<PushPull>>, PC5<AF7<PushPull>>)>;

static mut SERIAL: Option<SerialType> = None;

unsafe fn get_serial() -> &'static mut SerialType {
    if let Some(ref mut gpioc) = SERIAL { &mut *gpioc } else { panic!() }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Serial Interrupt Demo");

    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze(&mut flash.acr);

    // Configure GPIO pins PC4 and PC5 for UART alternate function
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);
    let tx = gpioc.pc4.into_af_push_pull::<7>(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);
    let rx = gpioc.pc5.into_af_push_pull::<7>(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);

    // Configure UART1 (or your desired UART)
    let serial = serial::Serial::new(
        dp.USART1,
        (tx, rx),
        115200.Bd(), // Set your desired baud rate
        clocks,
        &mut rcc.apb2,
    );

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::USART1_EXTI25);
    }

    unsafe {
        SERIAL = Some(serial);
    }

    let serial = unsafe { get_serial() };
    serial.enable_interrupt(serial::Event::ReceiveDataRegisterNotEmpty);

    loop {
    }
}

#[interrupt]
fn USART1_EXTI25() {
    rprintln!("USART1");
    let serial = unsafe { get_serial() };
    match serial.read() {
        Ok(byte) => {
            serial.write(byte).unwrap();
        }
        Err(_error) => {
            ();
        }
    };
}
