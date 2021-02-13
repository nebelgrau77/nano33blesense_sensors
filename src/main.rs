// this works! needs a better function that will accept a value name (R, G, B, C) etc.

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
    fonts::{Font8x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
    };

use arrayvec::ArrayString;
use core::fmt;

use shared_bus;

use nb::block;

use apds9960::Apds9960;

const BOOT_DELAY_MS: u16 = 100; //small delay for the I2C to initiate correctly and start on boot without having to reset the board

#[entry]
fn main() -> ! {
    
    let p = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let port1 = hal::gpio::p1::Parts::new(p.P1);

    let mut led = port0.p0_13.into_push_pull_output(Level::Low);

    let mut _apds_pwr = port0.p0_20.into_push_pull_output(Level::High);
    let _vdd_env = port0.p0_22.into_push_pull_output(Level::High); // powers the HTS221 sensor, as per board schematics
    let _r_pullup = port1.p1_00.into_push_pull_output(Level::High); // necessary for SDA1 and SCL1 to work, as per board schematics


    // set up delay provider
    let mut delay = Delay::new(core.SYST);


    // define I2C pins
    let scl = port0.p0_02.into_floating_input().degrade(); // clock
    let sda = port0.p0_31.into_floating_input().degrade(); // data

    let i2c_pins = hal::twim::Pins{
        scl: scl,
        sda: sda
    };    

    // define I2C pins
    let scl1 = port0.p0_15.into_floating_input().degrade(); // clock
    let sda1 = port0.p0_14.into_floating_input().degrade(); // data

    let i2c1_pins = hal::twim::Pins{
        scl: scl1,
        sda: sda1
    };    
    
    // wait for just a moment
    delay.delay_ms(BOOT_DELAY_MS);
    
    // set up I2C    
    let mut i2c = Twim::new(p.TWIM0, i2c_pins, hal::twim::Frequency::K400);

    let manager = shared_bus::CortexMBusManager::new(i2c);

    // set up I2C    
    let mut i2c1 = Twim::new(p.TWIM1, i2c1_pins, hal::twim::Frequency::K400);

    // set up SSD1306 display
    let interface = I2CDIBuilder::new().init(manager.acquire());
    
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();          
    disp.init().unwrap();
    disp.flush().unwrap();

    //led.set_high().unwrap(); // LED on after I2C got set up correctly

    // initialize sensor    
    let mut sensor = Apds9960::new(i2c1);
    sensor.enable().unwrap();
    sensor.enable_light().unwrap();

    //led.set_low().unwrap(); // if LED goes off, the sensor got initialized


    loop {       

        for m in 0..64 {
            for n in 0..64 {
                disp.set_pixel(m, n, 0);
            }
        }
        
        let light = block!(sensor.read_light()).unwrap();

        let text_style = TextStyleBuilder::new(Font8x16).text_color(BinaryColor::On).build();

        let mut c_buf = ArrayString::<[u8; 16]>::new(); 
        let mut r_buf = ArrayString::<[u8; 16]>::new();
        let mut g_buf = ArrayString::<[u8; 16]>::new();
        let mut b_buf = ArrayString::<[u8; 16]>::new();

        val_display(&mut c_buf, light.clear, "C");
        Text::new(c_buf.as_str(), Point::new(0, 0)).into_styled(text_style).draw(&mut disp).unwrap();

        val_display(&mut r_buf, light.red, "R");
        Text::new(r_buf.as_str(), Point::new(0, 16)).into_styled(text_style).draw(&mut disp).unwrap();

        val_display(&mut g_buf, light.green, "G");
        Text::new(g_buf.as_str(), Point::new(0, 32)).into_styled(text_style).draw(&mut disp).unwrap();

        val_display(&mut b_buf, light.blue, "B");
        Text::new(b_buf.as_str(), Point::new(0, 48)).into_styled(text_style).draw(&mut disp).unwrap();


        disp.flush().unwrap();

        delay.delay_ms(500_u32);              

        if led.is_set_high().unwrap() {
            led.set_low().unwrap();
            }
        else {
            led.set_high().unwrap();
            }

    }
    
}


pub fn val_display(buf: &mut ArrayString<[u8; 16]>, val: u16, msg: &str) {   
    
    fmt::write(buf, format_args!("{}: {:04}", msg, val)).unwrap();    
    
}
