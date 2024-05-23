#![no_std]
#![no_main]

use defmt::*;
use core::cell::RefCell;
use core::option::Option::Some;
use core::option::Option::None;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, i2c, peripherals, dma::NoDma, time::hz, Config};
use embassy_stm32::i2c::I2c;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;
use embedded_graphics::mono_font::iso_8859_10::FONT_5X7;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use {defmt_rtt as _, panic_probe as _};
use panic_probe::*;
use adxl345_driver2::{i2c::Device, Adxl345Reader, Adxl345Writer};
use embassy_sync::blocking_mutex::Mutex;


static I2C1_MUTEX: Mutex<ThreadModeRawMutex, RefCell<Option<I2c<'static, I2C1>>>> = Mutex::new(RefCell::new(None));
static DEVICE: Mutex<ThreadModeRawMutex, RefCell<Option<Device<I2c<'static, I2C1>>>>> = Mutex::new(RefCell::new(None));

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
async fn adxl() {
    I2C1_MUTEX.lock(|bus_rc| {
        DEVICE.lock(|device| {
            let mut adxl345 = Device::new(bus_rc.take().unwrap()).unwrap();
            adxl345.init().unwrap();
            adxl345
                .set_data_format(8)
                .unwrap();
            // Set measurement mode on.
            adxl345
                .set_power_control(8)
                .unwrap();

            device.replace(Some(adxl345));
        });    
    });



    loop {
        DEVICE.lock(|device| {
            let mut binding = device.borrow_mut();
            let mut adxl345 = binding.as_mut().unwrap();
            // Set full scale output and range to 2G.
            let (x, y, z) = adxl345.acceleration().unwrap();
            let x = x as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
            let y = y as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
            info!("x: {}, y: {}, z: {}", x, y, z);
        });
        Timer::after_millis(10).await;
    }
}

#[embassy_executor::task]
async fn sh1106() {
    I2C1_MUTEX.lock(|bus_rc| {
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(bus_rc.take().unwrap()).into();

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
    });
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    
    let config = Config::default();
    let p = embassy_stm32::init(config);
    
    let i2c = i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );
    /*let i2c2 = i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );*/
    I2C1_MUTEX.lock(|i| {
        i.replace(Some(i2c));
    });

    _spawner.spawn(adxl()).unwrap();
    _spawner.spawn(sh1106()).unwrap();
}
