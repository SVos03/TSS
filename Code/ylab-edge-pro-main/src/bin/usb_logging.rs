//! USB Logging
//!
//! This creates the possibility to send log::info/warn/error/debug! to USB serial port.
//! Can be used to improvise data streaming via USB

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_time::{Duration, Instant, Ticker};

use embassy_stm32 as hal;

use hal::dma::NoDma;
use hal::usart::{Config, Uart};
use hal::{bind_interrupts, peripherals, usart};
//use heapless::String;
use {defmt_rtt as _, panic_probe as _};

mod bsu {
    /* #[derive(Serialize, Deserialize, Debug)]
    pub enum Sensor {None, Adc1_1}*/

    // data handling
    use heapless::Vec;
    use postcard::to_vec;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Obs {
        pub dev: i8,
        pub time: i32,
        pub read: [u32; 1],
    }

    // Channel
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::channel::Channel;
    pub static SINK: Channel<CriticalSectionRawMutex, Obs, 2> = Channel::new();

    // USB
    use embassy_stm32 as hal;
    use hal::peripherals;
    use hal::usart::Uart;

    #[embassy_executor::task]
    pub async fn task(mut usart: Uart<'static, peripherals::USART3, peripherals::DMA1_CH3>) {
        loop {
            let obs = SINK.recv().await;
            let ser: Vec<u8, 9> = to_vec(&obs).unwrap();
            usart.write(&ser).await.unwrap();
            usart.write("\n".as_bytes()).await.unwrap();
        }
    }
}

/* MAIN */
#[embassy_executor::main]
async fn init(spawner: Spawner) {
    let p = hal::init(Default::default());

    bind_interrupts!(struct Irqs {
        USART3 => usart::InterruptHandler<peripherals::USART3>;
    });
    let usart = Uart::new(
        p.USART3,
        p.PD9,
        p.PD8,
        Irqs,
        p.DMA1_CH3,
        NoDma,
        Config::default(),
    );
    spawner.spawn(bsu::task(usart)).unwrap();

    let mut counter = 0;
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        ticker.next().await;
        counter += 1;
        let obs = bsu::Obs {
            dev: 0,
            time: Instant::now().as_micros() as i32,
            read: [counter],
        };
        bsu::SINK.send(obs).await;
    }
}
