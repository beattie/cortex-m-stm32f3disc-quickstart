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

    let mut xmit = unsafe { XMIT_BUF.producer() };

    let mut tick: u32 = 0;
    let mut buffer = [0u8; 32];
    loop {
        delay.delay_ms(1000u16);
        rprintln!("Tick {}", tick);
        // let output = "Tick\n\r".as_bytes();
        // let mut w = xmit.write(output.len());
        let mut b = format_no_std::show(
            &mut buffer,
            format_args!("Tick {}\r\n", tick),
            ).unwrap().as_bytes();
        let mut w = xmit.write(b.len());
        if w.len() == 0 {
            rprintln!("xmit buffer full");
            continue;
        }
        for i in 0..w.len() {
            w[i] = b[i];
        }
        serial.enable_interrupt(TransmitDataRegisterEmtpy);
        tick += 1;
    }
}

#[interrupt]
fn USART1_EXTI25() {
    // enabling and disabling the interrupt too rapidly seems to cause problems
    // so track the conditions that decide to enable or disable the transmit
    // interrupt
    let mut interrupt_dis = false;
    let mut interrupt_ena = false;
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
            interrupt_dis = true;
        }
    }
    // check for input ready
    if serial.triggered_events().contains(ReceiveDataRegisterNotEmpty) {
        // read byte and add it to ring buffer
        match serial.read() {
            Ok(byte) => {
                let mut xmit = unsafe { XMIT_BUF.producer() };
                let mut w = xmit.write(1);
                // if byte added to ring buffer enable transmit interrupt
                if w.len() > 0 {
                    w[0] = byte;
                    interrupt_ena = true;
                }
            }
            Err(_error) => {
                ();
            }
        };
    }
    if interrupt_ena {
        serial.enable_interrupt(TransmitDataRegisterEmtpy);
    } else if interrupt_dis {
        serial.disable_interrupt(TransmitDataRegisterEmtpy);
    }
}
