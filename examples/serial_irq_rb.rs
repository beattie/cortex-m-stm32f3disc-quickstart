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
    serial::{Serial,
        Event:: {
            ReceiveDataRegisterNotEmpty,
            TransmitDataRegisterEmtpy,
        },
    },
    interrupt,
    pac::USART1,
    gpio::{
        gpioc::
        {PC4, PC5}, PushPull, AF7},
    delay::Delay,
};
use fring;
use format_no_std;

type SerialType = Serial<USART1, (PC4<AF7<PushPull>>, PC5<AF7<PushPull>>)>;

static mut SERIAL: Option<SerialType> = None;

static XMIT_BUF: fring::Buffer::<256> = fring::Buffer::new();

unsafe fn get_serial() -> &'static mut SerialType {
    if let Some(ref mut gpioc) = SERIAL { &mut *gpioc } else { panic!() }
}

// put byte into xmit queue
fn put_byte(byte: u8) -> Result<(), ()> {
    let mut xmit = unsafe { XMIT_BUF.producer() };
    let mut w = xmit.write(1);
    if w.len() == 0 {
        Err(())
    } else {
        w[0] = byte;
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Serial Interrupt Demo");

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

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
    serial.enable_interrupt(ReceiveDataRegisterNotEmpty);

    let mut tick: u32 = 0;
    let mut buffer = [0u8; 32];
    loop {
        delay.delay_ms(1000u16);
        rprintln!("Tick {}", tick);
        let b = format_no_std::show(
            &mut buffer,
            format_args!("Tick {}\r\n", tick),
            ).unwrap().as_bytes();
        for i in 0..b.len() {
            match put_byte(b[i]) {
                Ok(()) => {
                    continue;
                }
                Err(()) => {
                    rprintln!("xmit full");
                    break;
                }
            };
        }
        // enable transmit interrupt, even if nothing added to queue becaus it's full
        serial.enable_interrupt(TransmitDataRegisterEmtpy);
        tick += 1;
    }
}

#[interrupt]
fn USART1_EXTI25() {
    let serial = unsafe { get_serial() };
    // check for transmit ready for next byte
    if serial.triggered_events().contains(TransmitDataRegisterEmtpy) {
        let mut xmit = unsafe { XMIT_BUF.consumer() };
        let r = xmit.read(1);
        if r.len() > 0 {
            match r.first() {
                Some(byte) => {
                    serial.write(*byte).unwrap();
                }
                None => {
                    ();
                }
            };
        }
        drop(r);
        // if consumer empty turn off transmit interrupt
        if xmit.data_size() < 1 {
            serial.disable_interrupt(TransmitDataRegisterEmtpy);
        }
    }
    // check for input ready
    if serial.triggered_events().contains(ReceiveDataRegisterNotEmpty) {
        // read byte and add it to ring buffer
        match serial.read() {
            Ok(byte) => {
                match put_byte(byte) {
                    Ok(()) => {
                        serial.enable_interrupt(TransmitDataRegisterEmtpy);
                    }
                    Err(()) => {
                        rprintln!("xmit full");
                    }
                };
            }
            Err(_error) => {
                ();
            }
        };
    }
}
