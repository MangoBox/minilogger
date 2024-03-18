#![no_std]
#![no_main]

use core::option::Option::Some;
use defmt::{panic, *};
use embassy_executor::Spawner; 
//use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::{bind_interrupts, i2c, peripherals, dma::NoDma, time::hz, Config};
use embassy_stm32::i2c::I2c;
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

use sh1106::{prelude::*, Builder};

/*bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});*/
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
use embassy_stm32::peripherals::*;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    
    let mut config = Config::default();
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

    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

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

    info!("Hello World!");
    loop {
        info!("hey there!");
    }
}
