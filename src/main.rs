#![no_std]
#![no_main]

// Our Modules
mod device_constants;
mod mpq8880a;

// RTIC Tasks
pub mod startup;
pub mod tasks;

use tasks::*;

// HAL Access
use rp235x_hal as hal;

// global logger
use defmt_rtt as _; 

// Monotonics
use rtic_monotonics::rp235x::prelude::*;
rp235x_timer_monotonic!(Mono);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("Panic: {}", info);
    // Halt the CPU
    hal::halt();
}

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: rp235x_hal::block::ImageDef = rp235x_hal::block::ImageDef::secure_exe();

#[rtic::app(
    device = hal::pac,
    dispatchers = [PIO2_IRQ_0, PIO2_IRQ_1, DMA_IRQ_0],
)]
mod app {

    use crate::device_constants::{
        OnboardLED, SAMPLE_COUNT,
    };

    use super::*;

    use bin_packets::packets::ApplicationPacket;
    use bin_packets::time::Timestamp;

    use hal::gpio::{self};

    use heapless::Deque;
    use rp235x_hal::uart::UartPeripheral;
    pub const XTAL_FREQ_HZ: u32 = 12_000_000u32;

    pub type UART0Bus = UartPeripheral<
        rp235x_hal::uart::Enabled,
        rp235x_hal::pac::UART0,
        (
            gpio::Pin<gpio::bank0::Gpio0, gpio::FunctionUart, gpio::PullDown>,
            gpio::Pin<gpio::bank0::Gpio1, gpio::FunctionUart, gpio::PullDown>,
        ),
    >;

    #[shared]
    pub struct Shared {
        pub downlink_packets: Deque<ApplicationPacket, 128>,
        pub samples_buffer: [u16; SAMPLE_COUNT],
    }

    #[local]
    pub struct Local {
        pub onboard_led: OnboardLED,
    }

    #[init(local = [adc: Option<hal::Adc> = None])]
    fn init(ctx: init::Context) -> (Shared, Local) {
        startup::startup(ctx)
    }

    extern "Rust" {

        // Heartbeats the main led (and sends packets after arming)
        #[task(shared = [downlink_packets], local = [onboard_led], priority = 1)]
        async fn heartbeat(mut ctx: heartbeat::Context);

    }

    /// Returns the current time in nanoseconds since power-on
    pub fn epoch_ns() -> u64 {
        Mono::now().duration_since_epoch().to_nanos()
    }

    /// Returns the current time as a timestamp
    pub fn now_timestamp() -> Timestamp {
        Timestamp::new(epoch_ns())
    }
}
