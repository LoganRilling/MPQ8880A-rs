use defmt::{info, warn};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use fugit::RateExtU32;
use hc12_rs::configuration::baudrates::B9600;
use hc12_rs::configuration::{Channel, HC12Configuration, Power};
use hc12_rs::device::IntoATMode;
use hc12_rs::IntoFU3Mode;
use heapless::Deque;
use rp235x_hal::adc::AdcPin;
use rp235x_hal::clocks::init_clocks_and_plls;
use rp235x_hal::gpio::{PinState, PullNone};
use rp235x_hal::pwm::Slices;
use rp235x_hal::uart::{DataBits, StopBits, UartConfig, UartPeripheral};
use rp235x_hal::{Clock, Sio, Watchdog};
use rtic_monotonics::Monotonic;

use crate::device_constants::pins::{CamMosfetPin, RadioProgrammingPin};
use crate::device_constants::{
    EjectionDetectionPin, EjectorHC12, GreenLed, JupiterUart, RadioUart, RedLed, SAMPLE_COUNT,
};
use crate::hal;
use crate::{app::*, Mono};

// Timestamp for logging
defmt::timestamp!("{=u64:us}", {
    Mono::now().duration_since_epoch().to_nanos()
});

pub fn startup(mut ctx: init::Context<'_>) -> (Shared, Local) {
    // Reset the spinlocks - this is skipped by soft-reset
    unsafe {
        hal::sio::spinlock_reset();
    }

    info!("Picolite startup");

    // Set up clocks
    let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        ctx.device.XOSC,
        ctx.device.CLOCKS,
        ctx.device.PLL_SYS,
        ctx.device.PLL_USB,
        &mut ctx.device.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    Mono::start(ctx.device.TIMER0, &ctx.device.RESETS);

    // The single-cycle I/O block controls our GPIO pins
    let sio = Sio::new(ctx.device.SIO);

    // Set the pins to their default state
    let bank0_pins = hal::gpio::Pins::new(
        ctx.device.IO_BANK0,
        ctx.device.PADS_BANK0,
        sio.gpio_bank0,
        &mut ctx.device.RESETS,
    );

    // Debugging on-board LED pin
    let mut led_pin = bank0_pins
        .gpio25
        .into_pull_type::<PullNone>()
        .into_push_pull_output();
    led_pin.set_low().unwrap();


    // Tasks
    heartbeat::spawn().ok();

    (
        Shared {
            downlink_packets: Deque::new(),
            samples_buffer: [0u16; SAMPLE_COUNT],
        },
        Local {
            onboard_led: led_pin,
        },
    )
}
