// IMU sensor values read and displayed on the OLED

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
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
    };

use arrayvec::ArrayString;
use core::fmt;

//use shared_bus;
//use nb::block;

use lsm9ds1::interface::{I2cInterface,
                        i2c::{AgAddress, MagAddress}};
use lsm9ds1::{accel, gyro, mag, LSM9DS1Init};

const BOOT_DELAY_MS: u16 = 100; //small delay for the I2C to initiate correctly and start on boot without having to reset the board

#[entry]
fn main() -> ! {
    
    let p = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let port1 = hal::gpio::p1::Parts::new(p.P1);

    let mut led = port0.p0_13.into_push_pull_output(Level::Low);

    let _vdd_env = port0.p0_22.into_push_pull_output(Level::High); // powers the LSM9DS1 sensor, as per board schematics
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

    //let manager = shared_bus::CortexMBusManager::new(i2c);

    // set up I2C    
    let mut i2c1 = Twim::new(p.TWIM1, i2c1_pins, hal::twim::Frequency::K400);

    // set up SSD1306 display
    //let interface = I2CDIBuilder::new().init(manager.acquire());
    let interface = I2CDIBuilder::new().init(i2c);
    
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();          
    disp.init().unwrap();
    disp.flush().unwrap();

    // initialize LSM9DS1 sensor    
    let ag_addr = AgAddress::_2; // 0x6B
    let mag_addr = MagAddress::_2; // 0x1E

    let i2c_interface = I2cInterface::init(i2c1, ag_addr, mag_addr);

    let mut lsm9ds1 = LSM9DS1Init {
                    ..Default::default()
                    }.with_interface(i2c_interface);

    lsm9ds1.begin_accel().unwrap();
    lsm9ds1.begin_gyro().unwrap();
    lsm9ds1.begin_mag().unwrap();

    loop {       

        // clean up the screen
        for m in 0..128 {
            for n in 0..32 {
                disp.set_pixel(m, n, 0);
            }
        }
                
        let (x,y,z) = lsm9ds1.read_gyro().unwrap(); // read gyroscope values

        let text_style = TextStyleBuilder::new(Font6x8).text_color(BinaryColor::On).build();

        let mut x_buf = ArrayString::<[u8; 16]>::new(); 
        let mut y_buf = ArrayString::<[u8; 16]>::new(); 
        let mut z_buf = ArrayString::<[u8; 16]>::new(); 

        val_display(&mut x_buf, x, "G_x");
        val_display(&mut y_buf, y, "G_y");
        val_display(&mut z_buf, z, "G_z");

        Text::new(x_buf.as_str(), Point::new(0, 0)).into_styled(text_style).draw(&mut disp).unwrap();
        Text::new(y_buf.as_str(), Point::new(0, 9)).into_styled(text_style).draw(&mut disp).unwrap();
        Text::new(z_buf.as_str(), Point::new(0, 18)).into_styled(text_style).draw(&mut disp).unwrap();

        disp.flush().unwrap();

        delay.delay_ms(100_u32);              

        if led.is_set_high().unwrap() {
            led.set_low().unwrap();
            }
        else {
            led.set_high().unwrap();
            }

    }
    
}

// helper function to display the sensor values

pub fn val_display(buf: &mut ArrayString<[u8; 16]>, val: f32, msg: &str) {   
    
    fmt::write(buf, format_args!("{}: {:.04}", msg, val)).unwrap();    
    
}

