#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};
use adxl345_driver2::{i2c::Device, Adxl345Reader, Adxl345Writer};
use embassy_stm32::i2c;
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals;
use embassy_stm32::dma::NoDma;
use embassy_stm32::time::hz;
use embassy_time::{Delay, Timer};
use embassy_stm32::{adc};
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::peripherals::ADC;


/// Output scale is 4mg/LSB.
const SCALE_MULTIPLIER: f64 = 0.004;
/// Average Earth gravity in m/s²
const EARTH_GRAVITY_MS2: f64 = 9.80665;


// bind_interrupts!(struct Irqs { 
//     //ADC1_2 => adc::InterruptHandler<ADC1>;
    
// });

#[embassy_executor::task]
async fn adxl() {
    let p = embassy_stm32::init(Default::default());

    let mybus = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );

    bind_interrupts!(struct Irqs {
        I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
        ADC1_COMP => adc::InterruptHandler<ADC>;
    });
    
    let mut adxl345 = Device::new(mybus).unwrap();

    adxl345
        .set_data_format(8)
        .unwrap();
    // Set measurement mode on.
    adxl345
        .set_power_control(8)
        .unwrap();

    let mut adc = Adc::new(p.ADC, Irqs, &mut Delay {});
    adc.set_sample_time(SampleTime::Cycles71_5);
    let mut pin_0 = p.PA0;
    let mut pin_1 = p.PA1;

    let mut vrefint = adc.enable_vref(&mut Delay {});
    loop{
        // Set full scale output and range to 2G.


        let (x, y, z) = adxl345
            .acceleration()
            .unwrap();
        let x = x as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        let y = y as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;
        let z = z as f64 * SCALE_MULTIPLIER * EARTH_GRAVITY_MS2;

        let v0 = adc.read(&mut pin_0).await;
        let v1 = adc.read(&mut pin_1).await;
        info!("PA0: {}\tPA1: {} \tX-axis = {}\tY-axis = {}\tZ-axis = {}", v0, v1, x, y, z);
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    _spawner.spawn(adxl()).unwrap();
}
