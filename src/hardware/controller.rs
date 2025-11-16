use crate::error::Result;
use crate::hardware::motor::MotorController;

pub struct I2cController {
    address: u8,
}

impl I2cController {
    pub fn new(address: u8) -> Self {
        Self { address }
    }
}

impl MotorController for I2cController {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()> {
        let data = [channel, (value >> 8) as u8, (value & 0xFF) as u8];
        self.write_data(self.address, &data)
    }

    fn read_pwm(&mut self, channel: u8) -> Result<u16> {
        let mut buffer = [0u8; 2];
        self.read_data(channel, &mut buffer)?;
        Ok(((buffer[0] as u16) << 8) | buffer[1] as u16)
    }

    fn write_data(&mut self, _address: u8, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    fn read_data(&mut self, _address: u8, buffer: &mut [u8]) -> Result<usize> {
        Ok(buffer.len())
    }
}

pub struct SerialController {
    #[cfg(feature = "serial")]
    port: Box<dyn serialport::SerialPort>,
    #[cfg(not(feature = "serial"))]
    _phantom: std::marker::PhantomData<()>,
}

impl SerialController {
    #[cfg(feature = "serial")]
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(100))
            .open()?;
        Ok(Self { port })
    }

    #[cfg(not(feature = "serial"))]
    pub fn new(_port_name: &str, _baud_rate: u32) -> Result<Self> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

impl MotorController for SerialController {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()> {
        let data = [channel, (value >> 8) as u8, (value & 0xFF) as u8];
        self.write_data(0, &data)
    }

    fn read_pwm(&mut self, channel: u8) -> Result<u16> {
        let mut buffer = [0u8; 2];
        self.read_data(channel, &mut buffer)?;
        Ok(((buffer[0] as u16) << 8) | buffer[1] as u16)
    }

    #[cfg(feature = "serial")]
    fn write_data(&mut self, _address: u8, data: &[u8]) -> Result<()> {
        use std::io::Write;
        self.port.write_all(data)?;
        Ok(())
    }

    #[cfg(not(feature = "serial"))]
    fn write_data(&mut self, _address: u8, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    #[cfg(feature = "serial")]
    fn read_data(&mut self, _address: u8, buffer: &mut [u8]) -> Result<usize> {
        use std::io::Read;
        Ok(self.port.read(buffer)?)
    }

    #[cfg(not(feature = "serial"))]
    fn read_data(&mut self, _address: u8, buffer: &mut [u8]) -> Result<usize> {
        Ok(buffer.len())
    }
}
