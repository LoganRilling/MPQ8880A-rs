use embedded_hal::i2c::{Error, SevenBitAddress, I2c};
use micromath::F32Ext;

use crate::mpq8880a::MPQCommand::{CLEAR_FAULTS, MFR_VOUT_CTRL, OPERATION, VOUT_COMMAND};

#[repr(u8)]
enum MPQCommand {
    OPERATION = 0x01,
    CLEAR_FAULTS = 0x03,
    WRITE_PROTECT = 0x10,
    STORE_USER_ALL = 0x15,
    RESTORE_USER_ALL = 0x16,

    VOUT_COMMAND = 0x21,
    VOUT_MAX = 0x24,
    VOUT_MARGIN_HIGH = 0x25,
    VOUT_MARGIN_LOW = 0x26,

    VIN_ON = 0x35,
    VIN_OFF = 0x36,
    VIN_OV_FAULT_LIMIT = 0x55,

    TON_DELAY = 0x60,
    TON_RISE = 0x61,
    TOFF_DELAY = 0x64,
    TOFF_FALL = 0x65,

    STATUS_BYTE = 0x78,
    STATUS_WORD = 0x79,
    STATUS_VOUT = 0x7A,
    STATUS_INPUT = 0x7C,
    STATUS_CML = 0x7E,

    READ_VIN = 0x88,
    READ_VOUT = 0x8B,
    READ_IOUT = 0x8C,

    MFR_ADDRESS = 0xD0,
    MFR_DITHER_CTRL = 0xD1,
    MFR_VOUT_CTRL = 0xD2,
    MFR_VOUT_TRANSITION_RATE = 0xD3,
    MFR_FREQ_SWITCH = 0xD4,
    MFR_COMP1_CTRL = 0xD6,
    MFR_COMP2_CTRL = 0xD7,
    MFR_SLOPE_CTRL = 0xD8,
    MFR_HYS_CTRL = 0xD9,
    MFR_PWM_CTRL = 0xDA,
    MFR_FAULT_CTRL = 0xDB,
    MFR_PRODUCT_ID = 0xDD,
    MFR_PRODUCT_VERSION = 0xDE,
    MFR_USER_VERSION = 0xDF,
    MFR_OC_FAULT_LIMIT = 0xE0,
    MFR_OC_RESPONSE = 0xE1,
    MFR_VIN_OV_FAULT_RESPONSE = 0xE2,
    MFR_VOUT_OV_FAULT_LIMIT = 0xE3,
    MFR_VOUT_OV_FAULT_RESPONSE = 0xE4,
    MFR_VOUT_UV_FAULT_LIMIT = 0xE5,
    MFR_VOUT_UV_FAULT_RESPONSE = 0xE6,
    MFR_RAIL_ADDRESS = 0xE8,
    MFR_LIGHT_CTRL = 0xE9,
    MFR_PWR_STAGE_SET = 0xEA,
}

#[derive(Clone, Copy)]
pub enum MPQFeedBackDividerRatio
{
    RATIO1_1 = 0b000,
    RATIO1_2 = 0b001,
    RATIO1_4 = 0b010,
    RATIO1_8 = 0b011,
    RATIO1_16 = 0b100,
}

pub struct MPQ8880A<I2C>
{
    i2c: I2C,
    address: u8,
    feedback_ratio: MPQFeedBackDividerRatio
}

impl<I2C: I2c<SevenBitAddress>> MPQ8880A<I2C>
{
    fn write_byte_data(&mut self, comm: MPQCommand, data: u8)
    {   
        self.i2c.write(self.address, &[
            comm as u8,
            data
        ]).expect("Send I2C command with a byte of data");
    }
    fn write_read_byte_data(&mut self, comm: MPQCommand, data: u8, read_slice: &mut [u8])
    {
        self.i2c.write_read(self.address, &[
            comm as u8,
            data
        ], read_slice).expect("Send read/write I2C command with a byte of data");
    }

    fn write_2byte_data(&mut self, comm: MPQCommand, data: u16)
    {   
        let data1 = (data & 0x00FF) as u8;
        let data2 = ((data & 0xFF00) >> 8) as u8;

        self.i2c.write(self.address, &[
            comm as u8,
            data1,
            data2
        ]).expect("Send I2C command with 2 bytes of data");
    }
    fn write_read_2byte_data(&mut self, comm: MPQCommand, data: u16, read_slice: &mut [u8])
    {
        let data1 = (data & 0x00FF) as u8;
        let data2 = ((data & 0xFF00) >> 8) as u8;

        self.i2c.write_read(self.address, &[
            comm as u8,
            data1,
            data2
        ], read_slice).expect("Send read/write I2C command with 2 bytes of data");
    }

    fn write_byte(&mut self, comm: MPQCommand)
    {   
        self.i2c.write(self.address, &[(comm as u8)]).expect("Send I2C command");
    }
    fn write_read_byte(&mut self, comm: MPQCommand, read_slice: &mut [u8])
    {
        self.i2c.write_read(self.address, &[(comm as u8)], read_slice).expect("Send read/write I2C command");
    }

    pub fn new(i2c: I2C, address: u8) -> MPQ8880A<I2C>
    {
        let mut mpq = MPQ8880A
        {
            i2c,
            address,
            feedback_ratio: MPQFeedBackDividerRatio::RATIO1_1
        };

        mpq.write_byte_data(OPERATION, 0b1000); // Power on

        return mpq;
    }

    pub fn clear_faults(&mut self)
    {
        self.write_byte(CLEAR_FAULTS);
    }

    pub fn set_feedback_divider_ratio(&mut self, ratio: MPQFeedBackDividerRatio)
    {
        self.write_byte_data(MFR_VOUT_CTRL, ratio as u8);
        self.feedback_ratio = ratio;
    }

    pub fn set_reference_voltage(&mut self, voltage: u16)
    {
        self.write_2byte_data(VOUT_COMMAND, voltage);
    }

    pub fn estimate_reference_voltage(&self, output_voltage: f32) -> f32
    {
        return output_voltage * (1.0/((2.0_f32).powi(self.feedback_ratio as i32)) / 0.0015);
    }

    pub fn turn_on_off(&mut self, on_off: bool)
    {
        match on_off
        {
            true => self.write_byte_data(MPQCommand::OPERATION, 0b1000), // Power on
            false => self.write_byte_data(MPQCommand::OPERATION, 0b0000) // Power off
        }
    }

    pub fn read_output_voltage(&mut self) -> f32
    {
        let vout_slice = &mut [0u8; 2];
        self.write_read_byte(MPQCommand::READ_VOUT, vout_slice);

        let mut vout: u16 = (vout_slice[0] as u16) << 8;
        vout = vout | (vout_slice[1] as u16);

        return (vout as f32 ) * 0.003 / (2.0.powi(self.feedback_ratio as i32));
    }

    pub fn read_input_voltage(&mut self) -> f32
    {
        let vin_slice = &mut [0u8; 2];
        self.write_read_byte(MPQCommand::READ_VIN, vin_slice);

        let mut vin: u16 = (vin_slice[0] as u16) << 8;
        vin = vin | (vin_slice[1] as u16);

        return (vin as f32 ) * 0.003 / (2.0.powi(self.feedback_ratio as i32));
    }
}