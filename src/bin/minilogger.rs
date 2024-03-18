#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    //ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    //adxl
    
    info!("Hello World!");
    loop {
        info!("hey there!");
    }
}
