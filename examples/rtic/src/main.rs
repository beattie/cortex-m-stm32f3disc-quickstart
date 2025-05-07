#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::gpio::{Output, PushPull, PEx, PC4, PC5, AF7};
use stm32f3xx_hal::{serial,serial::{Serial,
        Event::{
            ReceiveDataRegisterNotEmpty,
            TransmitDataRegisterEmtpy,
        },
    },
};
use stm32f3xx_hal::pac::USART1;
use stm32f3xx_hal::prelude::*;

systick_monotonic!(Mono, 1000);

pub struct RingBuffer {
    iptr: usize,
    optr: usize,
    buffer: [ u8; 256 ]
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            iptr: 0,
            optr: 0,
            buffer: [0; 256],
        }
    }

    pub fn put(&mut self, byte: u8) -> Result<(), ()> {
        let next = (self.iptr + 1) % 256;
        if next != self.optr {
            self.buffer[self.iptr] = byte;
            self.iptr = next;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get(&mut self) -> Result<u8, ()> {
        let byte = self.buffer[self.optr];
        if self.iptr != self.optr {
            self.optr = (self.optr + 1) % 256;
            Ok(byte)
        } else {
            Err(())
        }
    }

    pub fn empty(&self) -> bool {
        self.iptr == self.optr
    }

    pub fn full(&self) -> bool {
        ((self.iptr + 1) % 256) == self.optr
    }
}

pub struct SerialPort {
    xmit: RingBuffer,
    recv: RingBuffer,
    serial: Serial<USART1, (PC4<AF7<PushPull>>, PC5<AF7<PushPull>>)>,
}

impl SerialPort {
    fn new(xmit: RingBuffer, recv: RingBuffer,
           serial: Serial<USART1, (PC4<AF7<PushPull>>, PC5<AF7<PushPull>>)>) ->
                SerialPort {
        SerialPort { xmit: xmit, recv: recv, serial: serial }
    }

    fn output_byte(&mut self) -> Result<(), ()> {
        let res = self.xmit.get();
        match res {
            Ok(byte) => {   // get a byte from the transmit queue
                let _error = self.serial.write(byte);
                Ok(())
            }
            Err(()) => {    // xmit queue empty return error
                Err(())
            }
        }
    }

    fn input_byte(&mut self) -> Result<u8, ()> {
        let res = self.serial.read();
        match res {
            Ok(byte) => {   // get byte for serial port
                let _ = self.recv.put(byte);    // add to recv queue
                let _ = self.xmit.put(byte);    // echo
                Ok(byte)
            }
            Err(_error) => {    //
                Err(())
            }
        }
    }
}
            

#[app(device = stm32f3xx_hal::pac, peripherals = true, dispatchers = [SPI1, SPI2])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        serial_port: SerialPort,
    }

    #[local]
    struct Local {
        leds: [ PEx<Output<PushPull>>; 8 ],
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        let xmit = RingBuffer::new();
        let recv = RingBuffer::new();
        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        // Initialize the systick interrupt
        Mono::start(cx.core.SYST, 36_000_000); // default STM32F303 clock-rate
                                               // is 36MHz

        rprintln!("init");

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(36.MHz())
            .pclk1(36.MHz())
            .freeze(&mut flash.acr);

        // Setup LED
        let mut gpioe = cx.device.GPIOE.split(&mut rcc.ahb);
        let mut leds = [
            gpioe.pe8.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe9.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe10.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe11.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe12.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe13.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe14.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
            gpioe.pe15.
                    into_push_pull_output(&mut gpioe.moder,
                                  &mut gpioe.otyper).downgrade(),
        ];
        for led in leds.iter_mut() {
            led.set_low().unwrap();
        }

        let mut gpioc = cx.device.GPIOC.split(&mut rcc.ahb);
        let rx = gpioc.pc5.into_af_push_pull::<7>(
                &mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);
        let tx = gpioc.pc4.into_af_push_pull::<7>(
                &mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);
        let serial = serial::Serial::new(
            cx.device.USART1,
            (tx, rx),
            115200.Bd(), // Set your desired baud rate
            clocks,
            &mut rcc.apb2,
        );

        let mut serial_port = SerialPort::new(xmit, recv, serial);

        // enable serial interrupts
        serial_port.serial.enable_interrupt(ReceiveDataRegisterNotEmpty);
        
        // enqueue "Hello World"
        enqueue::spawn("Hello World\r\n".as_bytes()).ok();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared { serial_port }, Local { leds })
    }

    #[task(shared = [serial_port])]
    async fn enqueue(mut cx: enqueue::Context, str: &[u8]) {
        cx.shared.serial_port.lock(|serial_port| {
            for i in str {
                serial_port.xmit.put(*i).unwrap();
            }
            serial_port.serial.enable_interrupt(TransmitDataRegisterEmtpy);
        });
    }

    #[task(local = [leds])]
    async fn blink(cx: blink::Context) {
        loop {
            for led in cx.local.leds.iter_mut() {
                rprintln!("blink");
                led.set_high().unwrap();
                Mono::delay(1000.millis()).await;
                led.set_low().unwrap();
            }

        }
    }

    #[task(binds = USART1_EXTI25, shared = [serial_port])]
    fn usart1(mut cx: usart1::Context) {
        cx.shared.serial_port.lock(|serial_port| {
            if serial_port.serial.triggered_events().contains(
                    TransmitDataRegisterEmtpy) {
                match serial_port.output_byte() {
                    Ok(()) => {
                    }
                    Err(_error) => {    // transmit queue empty set flag 
                        serial_port.serial.disable_interrupt(
                                TransmitDataRegisterEmtpy);
                    }
                }
            }
            if serial_port.serial.triggered_events().contains(
                    ReceiveDataRegisterNotEmpty) {
                match serial_port.input_byte() {
                    Ok(_byte) => {
                        serial_port.serial.enable_interrupt(
                                TransmitDataRegisterEmtpy);
                    }
                    Err(_error) => {
                    }
                }
            }
        });
    }
}
