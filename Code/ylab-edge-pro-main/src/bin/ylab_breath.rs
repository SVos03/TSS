#![no_std]
#![no_main]


/// CONFIGURATION
/// 
/// Adc
static DEV: [bool; 3] = [true, true, false];
static HZ: [u64; 3] = [300, 50, 30];
const BAUD: u32 = 2_000_000;

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
use ylab::ysns::adc as yadc;
use ylab::ysns::yco2;
/// + data transport/storage
use ylab::ytfk::bsu as ybsu;


/// ## UI task
/// 
/// The ui task only signals the state 
/// 
//use ylab::yuio::disp as ydsp;
/// 



#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq, )] // testing equality
enum AppState {Send}

/// In a usual multi-threaded app you would write the interaction model
/// in the main task. However, with dual-core the main task is no longer 
/// async. Since all communication channels are static, this really doesn't matter.
/// 
/// The initial state is set and a signal is send to the LED.
/// The event loop waits for button events (long or short press) 
/// and changes the states, accordingly.
/// If an actual state change has occured, the state is signaled to the UI 
/// and initialized if that is necessary. In this case, entering Send 
/// starts the sensor sampling.
/// 
/// From an architectural point of view, this is a nice setup, too. 
/// Basically, we are separating the very different tasks of 
/// peripherals/spawning and ui handling. It would be easy to just plugin a 
/// different ui, by just reqriting this task. For example, another ui
/// could use the RGB led to signal states, or collect commands from a serial console.
///
/// Conclusion so far: If we take the Embassy promise for granted, that async is zero-cost, 
/// the separation of functionality into different tasks reduces dependencies. It introduces 
/// the complexity of signalling.
///
/// ## Init
/// 
/// + Initializing peripherals 
/// + spawning tasks
/// + assigning periphs to tasks

use hal::adc;
use hal::dma::NoDma;
use hal::usart::{Config, Uart};
use hal::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});
use embassy_time::Delay;
use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = hal::init(Default::default());
    let mut config = Config::default();
    config.baudrate = BAUD;
    let usart = Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH6, NoDma, config);
    match usart {
        Ok(usart) => spawner.spawn(ybsu::task(usart)).unwrap(),
        Err(_)  => {},
    }
    spawner.spawn(control_task()).unwrap();


    if DEV[0]{
        let mut delay = Delay;
        let adc1 = adc::Adc::new(p.ADC1, &mut delay);
        spawner.spawn(yadc::adcbank_1(adc1, 
                                    (p.PA0, p.PA1, p.PA4, p.PB0, p.PC1, p.PC0, p.PC3, p.PC2), 
                                    HZ[0], 0)).unwrap();
    };
    
    //#[cfg(feature = "lsm6-grove4")]
    // Activating the second I2C controller on Grove 4
    // and spawning a task for the LSM6 acceleration sensor
    
    if DEV[1]{
        println!("I2C interrupts");
        use hal::i2c;
        bind_interrupts!(struct Irqs {
            I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
            I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
        });
        println!("I2C new");
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
        println!("I2C OK");
        //spawner.spawn(ydsp::task(i2c1)).unwrap();
        //spawner.spawn(ylab::ysns::ads1015::task(i2c1, HZ[1])).unwrap();
        spawner.spawn(ylab::ysns::yco2::task(i2c1, 1)).unwrap();
        println!("I2C task ended");
    }

}

/// ## Control task
/// 
/// bare minimum for Pro


#[embassy_executor::task]
async fn control_task() { 
    let _state = AppState::Send;

    loop {
        Timer::after_millis(5).await;
        if yadc::READY.load(ORD) {
            yadc::SAMPLE.store(true, ORD);
            println!("ADC sampling started");
            break
        }
    }

    loop {
        Timer::after_millis(5).await;
        if yco2::READY.load(ORD) {
            yco2::SAMPLE.store(true, ORD);
            println!("CO2 sampling started");
            break
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
        

