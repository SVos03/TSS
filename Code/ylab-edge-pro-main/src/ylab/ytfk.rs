pub use super::*;

pub mod data {
pub use super::*;

/// A Sample is an array of values  
/// with a timestamp and a sensory identifier.
/// 
/// This is supposed to collect readings, where the length depends on the sensor 
/// (e.g. 6 for acceleration sensors):
/// 
/// + Size and data type of the array depend on the sensory
/// + The basic reading type must implement Into<YtfType>.

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Sample<M, const N: usize>
    where M: Into<YtfType>
    {
    pub sensory: u8,
    pub time: Instant,
    pub read: [M; N],
}

/// Ylab reads sensor data into fixed arrays [Option<f64>; 8]
/// 
/// TODO: Check if this can be reduced to f32 to increase performance
pub const YTF_LEN: usize = 8;
pub type YtfType = f64;
pub type YtfRead = [Option<YtfType>; YTF_LEN];

/// A YTF record is an array of N measures 
/// 
/// with a time-stamp and the id of the sensor array (sensory).
/// Other types than M can be used, if they implement Into<M>.
/// 

pub struct Ytf {
    pub sensory: u8,
    pub time: Instant,
    pub read: YtfRead,
}

/// Converting Sample to YTF
/// 
/// Arrays of arbitrary size are converted to fixed size arrays 
/// with optional types.

impl<M: Into<YtfType>, const N: usize> Into<Ytf> for Sample<M, N> {
    fn into(self) -> Ytf {
        let mut out: YtfRead = [None; YTF_LEN];
        for (i, r) in self.read.into_iter().enumerate() {
            out[i] = Some(r.into());
        }
        Ytf {
            sensory: self.sensory,
            time: self.time,
            read: out
        }
    }
}

/// Display YTF as CSV
/// 
/// the default formatting of YTF is CSV
impl core::fmt::Display for Ytf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{},{}", self.time.as_micros(), self.sensory).unwrap();
        for r in self.read {
            match r {
                Some(v) => {
                    write!(f, ",{:.3}", v).unwrap();
                    //println!("{}", v);
                    },
                None => {write!(f, ",").unwrap();}
            }
        }
        write!(f, "")
    }
}
}


pub mod bsu {
    pub use super::*;

    // Channel
    pub static SINK: Channel<RawMutex, Ytf, 2> = Channel::new();
    
    // USB
    use hal::usart::Uart;
    use hal::peripherals;
    
    #[embassy_executor::task]
    pub async fn task(mut usart: Uart<'static, 
                      peripherals::USART2, 
                      peripherals::DMA1_CH6>) {
        loop {
            let sample: Ytf = SINK.receive().await;
            let mut msg: Vec<u8, 256> = Vec::new();
            match core::write!(&mut msg, "{}", sample) {
                Ok(_) => usart.write(&msg).await.unwrap(),
                Err(_) => {},
            }
            usart.write(&msg).await.unwrap()
        }
    }
}

 