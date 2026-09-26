use crate::{app::*, device_constants::SAMPLE_COUNT, Mono};
use bin_packets::{
    devices::DeviceIdentifier,
    packets::{status::Status, ApplicationPacket},
};
use bincode::{config::standard, decode_from_slice, encode_into_slice, error::DecodeError};
use defmt::{debug, info, warn};
use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
use embedded_io::{Read, ReadReady, Write};
use fugit::ExtU64;
use heapless::{Deque, Vec};
use rtic::Mutex;
use rtic_monotonics::Monotonic;

struct DebugInfoExample<'a> {
    some_sensor_value: &'a str,
    some_status_value: &'a str,
}



pub async fn heartbeat(mut ctx: heartbeat::Context<'_>) {
    let onboard_led = ctx.local.onboard_led;

    let rock = "rock";
    let sat = "sat";

    let object = DebugInfoExample {
        some_sensor_value: rock,
        some_status_value: sat,
    };
    // Still blink, but toggle as it is done
    loop {
        onboard_led.toggle().unwrap();

        Mono::delay(300_u64.millis()).await;
        info!("Heartbeat!");
    }
}
