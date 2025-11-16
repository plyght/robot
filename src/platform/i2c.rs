use crate::error::Result;
use crate::hardware::MotorController;

pub struct I2cPlatformController {
    address: u8,
}

impl I2cPlatformController {
    pub fn new(address: u8) -> Result<Self> {
        Ok(Self { address })
    }
}

impl MotorController for I2cPlatformController {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()> {
        let data = [channel, (value >> 8) as u8, (value & 0xFF) as u8];
        self.write_data(self.address, &data)
    }

    fn read_pwm(&mut self, _channel: u8) -> Result<u16> {
        Ok(0)
    }

    fn write_data(&mut self, _address: u8, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    fn read_data(&mut self, _address: u8, buffer: &mut [u8]) -> Result<usize> {
        Ok(buffer.len())
    }
}
