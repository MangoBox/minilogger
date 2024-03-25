#![no_std]
#![no_main]

use core::option::Option::Some;
use defmt::{panic, *};
use embassy_executor::Spawner; 
use embassy_stm32::pac::I2C1;
//use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::{bind_interrupts, i2c, peripherals, dma::NoDma, time::hz, Config};
use embassy_stm32::i2c::I2c;
use embassy_time::Timer;
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
use {defmt_rtt as _, panic_probe as _};
use adxl345_driver2::{i2c::Device, Adxl345Reader, Adxl345Writer};


/// Output scale is 4mg/LSB.
const SCALE_MULTIPLIER: f64 = 0.004;
/// Average Earth gravity in m/s²
const EARTH_GRAVITY_MS2: f64 = 9.80665;


use sh1106::{prelude::*, Builder};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
use embassy_stm32::peripherals::*;


#[embassy_executor::task]
async fn adxl(bus: I2c<'_, I2C1>) {
    
    let mut adxl345 = Device::new(bus).unwrap();

    adxl345
        .set_data_format(8)
        .unwrap();
    // Set measurement mode on.
    adxl345
        .set_power_control(8)
        .unwrap();

    loop{
        // Set full scale output and range to 2G.


        let (x, y, z) = adxl345
            .acceleration()
            .unwrap();
        let x = x as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        let y = y as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        let z = z as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        info!("X-axis = {}, Y-axis = {}, Z-axis = {}", x, y, z);
        Timer::after_millis(150).await;
    }
}

#[embassy_executor::task]
async fn sh1106(bus: I2c<'_,I2C1>) {
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(bus).into();

    display.init().unwrap();
    display.flush().unwrap();

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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    
    let mut config = Config::default();
    let p = embassy_stm32::init(config);
    
    let i2c1 = i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );
    let i2c2 = i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );


    _spawner.spawn(adxl(i2c1)).unwrap();
    _spawner.spawn(sh1106(i2c2)).unwrap();
}
