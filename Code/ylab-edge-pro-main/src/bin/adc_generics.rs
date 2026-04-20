#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]


use ylab::hal;
//use embassy_stm32::adc::AdcPin;
use hal::adc::{Adc, AdcPin, Instance, SampleTime};
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Delay, Ticker, Duration};
use embassy_executor::Spawner;

#[embassy_executor::task]
pub async fn adc_task<P>(mut adc: Adc<'static, _>,
                mut pin: AdcPin<P>,
                hz: u64){
    //let state: Atomic<super::State> = Atomic::new(State::Offline);
    let mut ticker = Ticker::every(Duration::from_hz(hz));
    let mut _vrefint = adc.enable_vrefint();
    adc.set_sample_time(SampleTime::Cycles3);
    adc.set_resolution(hal::adc::Resolution::TwelveBit);
    loop {
        ticker.next().await;
        let _reading =  adc.read(&mut pin);
        }                
    }



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = hal::init(Default::default());
    let mut delay = Delay;
    let adc0 = Adc::new(p.ADC1, &mut delay);
    spawner.spawn(adc_task(adc0, p.PC0, 2)).unwrap();
}

        

