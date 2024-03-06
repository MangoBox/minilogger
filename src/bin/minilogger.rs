#![no_std]
#![no_main]

use core::sync::atomic::AtomicU32;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{adc, bind_interrupts};
use embassy_stm32::peripherals::ADC1;
use {defmt_rtt as _, panic_probe as _};

/*bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});*/

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    
    info!("Hello World!");
    loop {
        info!("hey there!");
    }
}
