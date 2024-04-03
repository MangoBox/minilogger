#![deny(unsafe_code)]
#![no_std]
#![no_main]


use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::option::Option::Some;
use core::option::Option::None;
//use adxl345_eh_driver::Driver;
use adxl343::Adxl343;
use defmt::{panic, *};
use embedded_hal::i2c;
use embedded_graphics::mono_font::iso_8859_10::FONT_5X7;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10,ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
//use embedded_hal::i2c::I2c;
use sh1106::mode::displaymode::DisplayMode;
use sh1106::mode::RawMode;
use stm32f1xx_hal::i2c::BlockingI2c;
use adxl343::accelerometer::Accelerometer;
use stm32f1xx_hal::i2c::DutyCycle;
use stm32f1xx_hal::i2c::I2c;
use stm32f1xx_hal::i2c::Mode;
use {defmt_rtt as _, panic_probe as _};
//use accelerometer::Accelerometer;
//use adxl345_driver2::{i2c::Device, Adxl345Reader, Adxl345Writer};

use shared_bus::BusManagerSimple;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};
/// Output scale is 4mg/LSB.
const SCALE_MULTIPLIER: f64 = 0.004;
/// Average Earth gravity in m/s²
const EARTH_GRAVITY_MS2: f64 = 9.80665;

use sh1106::{prelude::*, Builder};

//static I2C_MUTEX: Mutex<ThreadModeRawMutex, Option<I2c<I2C1>>> = Mutex::new(None);

#[entry]
fn main() -> ! {
    info!("hello");
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpiob = dp.GPIOB.split();

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    info!("initialised components...");
    let i2c = I2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400.kHz(),
            duty_cycle: DutyCycle::Ratio16to9,
        },
        clocks
    ).blocking_default(clocks);
    //let i2c1 = I2c::new(p.I2C1, , p.PB6, p.PB7, Irqs, NoDma, hz(100000), p);
    let bus = shared_bus::BusManagerSimple::new(i2c);

    info!("initialised i2c...");
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(bus.acquire_i2c()).into();
    display.init().unwrap();
    display.flush().unwrap();

    info!("started display...");
    //let adxl_proxy = bus.acquire_i2c();
    //let mut adxl345 = Driver::new(bus.acquire_i2c()).unwrap();
    let mut adxl = Adxl343::new(bus.acquire_i2c()).unwrap();

    loop {
        let values = adxl.accel_norm().unwrap();
        let x = values.x as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        let y = values.y as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        let z = values.z as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        info!("X-axis = {}, Y-axis = {}, Z-axis = {}", x, y, z);

        //Write data to the display
        let main_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
        let small_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
        let text = "MINILOGGER";
        Text::with_alignment(
            text,
            display.bounding_box().center() + Point::new(0, 20),
            main_style,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();
        
        Text::with_alignment(
            "L. Davies & B. Caley",
            display.bounding_box().center() + Point::new(0, 0),
            small_style,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();

        display.flush().unwrap();
    }
}