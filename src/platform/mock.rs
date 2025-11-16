use crate::error::Result;
use crate::hardware::MotorController;
use std::collections::HashMap;

pub struct MockController {
    pwm_values: HashMap<u8, u16>,
    data_store: HashMap<u8, Vec<u8>>,
}

impl MockController {
    pub fn new() -> Self {
        Self {
            pwm_values: HashMap::new(),
            data_store: HashMap::new(),
        }
    }
}

impl Default for MockController {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorController for MockController {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()> {
        self.pwm_values.insert(channel, value);
        Ok(())
    }

    fn read_pwm(&mut self, channel: u8) -> Result<u16> {
        Ok(*self.pwm_values.get(&channel).unwrap_or(&0))
    }

    fn write_data(&mut self, address: u8, data: &[u8]) -> Result<()> {
        self.data_store.insert(address, data.to_vec());
        Ok(())
    }

    fn read_data(&mut self, address: u8, buffer: &mut [u8]) -> Result<usize> {
        if let Some(data) = self.data_store.get(&address) {
            let len = data.len().min(buffer.len());
            buffer[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Ok(0)
        }
    }
}
