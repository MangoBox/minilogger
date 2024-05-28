#![no_std]
#![no_main]

use defmt::*;
use embedded_hal_bus::i2c::RefCellDevice;
use {defmt_rtt as _, panic_probe as _};

use core::{cell::RefCell, option::Option::None, sync::atomic::{self, AtomicBool}};

use embassy_sync::mutex::Mutex;
use embassy_executor::Spawner;
use embassy_stm32::{init, peripherals::*};
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
use adxl345_driver2::{i2c::Device, Adxl345Reader, Adxl345Writer};
use core::sync::atomic::AtomicU32; 

/// Output scale is 4mg/LSB.
const SCALE_OFFSET: i16 = 1000;

//global atomic u32 data
static X: AtomicU32 = AtomicU32::new(0);
static Y: AtomicU32 = AtomicU32::new(0);
static Z: AtomicU32 = AtomicU32::new(0);
static INIT_CHECK: AtomicBool = AtomicBool::new(false);


use sh1106::{prelude::*, Builder};
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

type SharedI2c = Mutex<ThreadModeRawMutex, Option<RefCell<I2c<'static, I2C1>>>>;
static SHARED_I2C: SharedI2c = Mutex::new(None);

#[embassy_executor::task]
async fn log_acceleration() {
    loop {
        {
            let guard = SHARED_I2C.lock().await;
            let i2c = guard.as_ref().unwrap();

            let mut adxl345 = adxl345_driver2::i2c::Device::new(RefCellDevice::new(i2c)).unwrap();

            //measurment mode
            adxl345.set_power_control(8).unwrap();

            let (x, y, z) = adxl345.acceleration().unwrap();
            
            let x: u32 = (x + SCALE_OFFSET).try_into().unwrap();
            X.store(x, core::sync::atomic::Ordering::Relaxed);
            let y = (y + SCALE_OFFSET).try_into().unwrap();
            Y.store(y, core::sync::atomic::Ordering::Relaxed);
            let z = (z + SCALE_OFFSET).try_into().unwrap();
            Z.store(z, core::sync::atomic::Ordering::Relaxed);
            info!("x: {}, y: {}, z: {}", x, y, z);
        }
        Timer::after_millis(10).await;
    }
}

#[embassy_executor::task]
async fn display_text() {

    let main_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let small_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);

    loop {
        {
            let guard = SHARED_I2C.lock().await;
            let i2c = guard.as_ref().unwrap();

            let mut display: GraphicsMode<_> = Builder::new().connect_i2c(RefCellDevice::new(i2c)).into();

            if !INIT_CHECK.load(atomic::Ordering::Relaxed){
                display.init().unwrap();
                INIT_CHECK.store(true, atomic::Ordering::Relaxed);
            }

            let text = "Minilogger";

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

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    
    let config: Config = Config::default();
    let p = embassy_stm32::init(config);
    
    let mut i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );

    {
        SHARED_I2C.lock().await.replace(RefCell::new(i2c));
    }

    spawner.spawn(log_acceleration()).unwrap();
    spawner.spawn(display_text()).unwrap();
}
