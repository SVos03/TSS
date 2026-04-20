#![no_std]
#![no_main]

/// CONFIGURATION
///
/// Adc
static DEV: [bool; 3] = [true, true, false];
static HZ: [u64; 3] = [300, 50, 30];
const BAUD: u32 = 2_000_000;

use ylab::ysns::adc as yadc;
use ylab::ysns::yco2;
/// + data transport/storage
use ylab::ytfk::bsu as ybsu;
/// # YLab Edge
///
/// __YLab Edge Pro__ is a sensor recording firmware for STM32 Nucleo boards.

//use defmt::*;

// use heapless::String;
/// +  multi-threading with async
/// + timing using Embassy time
/// + peripherals
/// use embassy_stm32 as hal;
/// + thread-safe data transfer and control
///
/// + built-in ADC sensors
use ylab::*;

/// ## UI task
///
/// The ui task only signals the state
///
//use ylab::yuio::disp as ydsp;
///

#[derive(
    Debug, // used as fmt
    Clone,
    Copy, // because next_state
    PartialEq,
    Eq,
)] // testing equality
enum AppState {
    Send,
}

/// ## Init
///
/// + Initializing peripherals
/// + spawning tasks
/// + assigning periphs to tasks
use hal::adc;
use hal::dma::NoDma;
use hal::i2c;
use hal::usart::{Config, Uart};
use hal::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3 => usart::InterruptHandler<peripherals::USART3>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
use embassy_executor::Spawner;
use embassy_time::Delay;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = hal::init(Default::default());
    let mut config = Config::default();
    config.baudrate = BAUD;
    let usart = p.USART2;
    let tx = p.PA3;
    let rx = p.PA2;
    let usart_dma = p.DMA1_CH6;
    let usart = Uart::new(usart, tx, rx, Irqs, usart_dma, NoDma, config);
    match usart {
        Ok(usart) => spawner.spawn(ybsu::task(usart)).unwrap(),
        Err(_) => {
            println!("USART connection failed")
        }
    }

    println!("I2C new");
    spawner.spawn(control_task()).unwrap();
    let i2c1 = i2c::I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        NoDma,
        NoDma,
        hal::time::Hertz(100_000),
        Default::default(),
    );

    spawner
        .spawn(ysns::SenFive::task(i2c1, Duration::from_secs(60), 2))
        .unwrap();
    //spawner.spawn(ydsp::task(i2c1)).unwrap();                 // optional display
    //spawner.spawn(ylab::ysns::yco2::task(i2c1, 1)).unwrap();  // CO2 sensor
    println!("I2C task ended");
}

/// ## Control task
///
/// This does nothiong at the moment
/// READY is a shared Mutex, true when the sensor was correctly initialized.
/// with SAMPLE the task is commanded to send data to the ybsu::SINK

#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;

    loop {
        Timer::after_millis(5).await;
        if yadc::READY.load(ORD) {
            yadc::SAMPLE.store(true, ORD);
            println!("ADC sampling started");
            break;
        }
    }

    loop {
        Timer::after_millis(5).await;
        if yco2::READY.load(ORD) {
            yco2::SAMPLE.store(true, ORD);
            println!("CO2 sampling started");
            break;
        }
    }

    yco2::SAMPLE.store(true, ORD);
}

/*pub use core::sync::atomic::Ordering;
//use ydsp::{FourLines, OneLine};
#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;
    yadc::SAMPLE.store(true, Ordering::Relaxed);
    let title: OneLine = "YLab".try_into().unwrap();
    let disp_text: FourLines = [ Some(title) ,None, None, None];
    ydsp::TEXT.signal(disp_text);
}*/
