// doesn't work, hangs up on initializing sensor

#![no_main]
#![no_std]

use panic_halt as _;

use nrf52840_hal as hal;

use hal::{pac::{CorePeripherals, Peripherals},
        prelude::*,
        gpio::Level,
        delay::Delay,        
        Twim,       
        };

use cortex_m_rt::entry;

use ssd1306::{prelude::*, Builder, I2CDIBuilder};

use embedded_graphics::{
    fonts::{Font6x12, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
    };

use arrayvec::ArrayString;
use core::fmt;

//use shared_bus;
//use nb::block;

use hts221;

const BOOT_DELAY_MS: u16 = 100; //small delay for the I2C to initiate correctly and start on boot without having to reset the board

#[entry]
fn main() -> ! {
    
    let p = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    let port0 = hal::gpio::p0::Parts::new(p.P0);
    
    let mut led = port0.p0_13.into_push_pull_output(Level::Low);

    let _vdd_env = port0.p0_22.into_push_pull_output(Level::High);

    /*

    NOT SURE HOW TO CHANGE DRIVE LEVEL OF A SINGLE PIN, THIS WON'T WORK:

    let mut vdd_env = port0.p0_22.conf().write(|w| {
        w.dir().output();
        w.input().connect();I
        w.pull().disabled();
        w.drive().h0h1();
        w.sense().disabled();
        w
    });

    */

    // define I2C0 pins 
    let scl = port0.p0_02.into_floating_input().degrade(); // clock
    let sda = port0.p0_31.into_floating_input().degrade(); // data

    let i2c_pins = hal::twim::Pins{
        scl: scl,
        sda: sda
    };    

    // define I2C1 pins
    let scl1 = port0.p0_15.into_floating_input().degrade(); // clock
    let sda1 = port0.p0_14.into_floating_input().degrade(); // data

    let i2c1_pins = hal::twim::Pins{
        scl: scl1,
        sda: sda1
    };    

    // set up delay provider
    let mut delay = Delay::new(core.SYST);
    
    led.set_high().unwrap();

    // wait for just a moment
    delay.delay_ms(BOOT_DELAY_MS);
    
    // set up I2C0    
    let mut i2c = Twim::new(p.TWIM0, i2c_pins, hal::twim::Frequency::K400);

    // set up I2C1    
    let mut i2c1 = Twim::new(p.TWIM1, i2c1_pins, hal::twim::Frequency::K400);

    // set up SSD1306 display
    let interface = I2CDIBuilder::new().init(i2c);
    
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();          
    disp.init().unwrap();
    disp.flush().unwrap();

    // initialize sensor
    let mut hts221 = hts221::Builder::new()                
    .with_default_7bit_address()
    .with_avg_t(hts221::AvgT::Avg256)
    .with_avg_h(hts221::AvgH::Avg512)    
    .build(&mut i2c1).unwrap();

    led.set_low().unwrap(); // if LED goes off, sensor got correctly initialized

    loop {       

        // clean up the digits
        for m in 0..128 {
            for n in 0..12 {
                disp.set_pixel(m, n, 0);
            }
        }

        let temperature_x8 = hts221.temperature_x8(&mut i2c1).unwrap();
        let temp = temperature_x8 / 8;

        let text_style = TextStyleBuilder::new(Font6x12).text_color(BinaryColor::On).build();

        let mut buf = ArrayString::<[u8; 16]>::new();

        val_display(&mut buf, temp, "T");

        Text::new(buf.as_str(), Point::new(0, 0)).into_styled(text_style).draw(&mut disp).unwrap();

        disp.flush().unwrap();

        delay.delay_ms(250_u32);              

        if led.is_set_high().unwrap() {
            led.set_low().unwrap();
            }
        else {
            led.set_high().unwrap();
            }

    }
    
}

pub fn val_display(buf: &mut ArrayString<[u8; 16]>, val: i16, msg: &str) {   
    
    fmt::write(buf, format_args!("{}: {:04}", msg, val)).unwrap();    
    
}
