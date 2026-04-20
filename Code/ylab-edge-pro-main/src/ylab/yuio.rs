
pub mod led {
    // LED control
    use embassy_time::{Duration, Timer};
    use embassy_time::Instant;
    use super::hal as hal;
    use hal::gpio::{AnyPin, Output, Level};
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    pub enum State {Vibrate, Blink, Steady, Interrupt, Off}
    pub static LED: Signal<CriticalSectionRawMutex, State> = Signal::new();
    
    #[embassy_executor::task]
    pub async fn task(led_pin: AnyPin) {
        let mut led 
                = Output::new(led_pin, Level::Low);
        loop {  
                let next_signal = LED.wait().await;
                match  next_signal {
                    State::Vibrate      => {
                        for _ in 1..10 {
                            led.set_high();
                            Timer::after(Duration::from_millis(25))
                            .await;
                            led.set_low();
                            Timer::after(Duration::from_millis(25))
                            .await;
                        };
                    },
                    State::Blink  => {
                        led.set_low();
                        Timer::after(Duration::from_millis(25))
                        .await;
                        led.set_high();
                        Timer::after(Duration::from_millis(50))
                        .await;
                        led.set_low()},
                    State::Steady => {led.set_high()},
                    State::Off    => {led.set_low()},
                    State::Interrupt  => {
                        led.toggle();
                        Timer::after(Duration::from_millis(5))
                        .await;
                        led.toggle();}
                }   
            };
        }
}
