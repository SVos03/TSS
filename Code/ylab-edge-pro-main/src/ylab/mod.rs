#![no_std]

pub use core::fmt::Write;
pub use core::sync::atomic::AtomicBool;
pub use core::sync::atomic::Ordering;
pub use defmt::println;
pub use defmt::Format;
pub use embassy_stm32 as hal;
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as RawMutex;
pub use embassy_sync::channel::Channel;
pub use embassy_sync::mutex::Mutex;
pub use embassy_sync::signal::Signal;
pub use embassy_time as time;
pub use hal::exti::ExtiInput;
pub use heapless::{String, Vec};
pub use time::{Delay, Duration, Instant, Ticker, Timer};

/// Standard ordering for Arcs
///
/// Because STM32 chips don't do parallel computing
/// we go with relaxed.
pub static ORD: Ordering = Ordering::Relaxed;

/// Sub modules
///
/// + YLab sensors
pub mod ysns;
/// + YLab transfer formats & kodices
pub mod ytfk;

pub use ytfk::data::Ytf;
