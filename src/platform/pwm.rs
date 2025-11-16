use crate::error::Result;
use crate::hardware::MotorController;

#[cfg(feature = "linux-pwm")]
use linux_embedded_hal::sysfs_pwm::Pwm;

pub struct LinuxPwmController {
    #[cfg(feature = "linux-pwm")]
    pwm_chips: Vec<Pwm>,
    #[cfg(not(feature = "linux-pwm"))]
    _phantom: std::marker::PhantomData<()>,
}

impl LinuxPwmController {
    #[cfg(feature = "linux-pwm")]
    pub fn new(chip_id: u32, channels: Vec<u32>) -> Result<Self> {
        let mut pwm_chips = Vec::new();
        for channel in channels {
            pwm_chips.push(Pwm::new(chip_id, channel)?);
        }
        Ok(Self { pwm_chips })
    }

    #[cfg(not(feature = "linux-pwm"))]
    pub fn new(_chip_id: u32, _channels: Vec<u32>) -> Result<Self> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

impl MotorController for LinuxPwmController {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()> {
        #[cfg(feature = "linux-pwm")]
        {
            if let Some(pwm) = self.pwm_chips.get_mut(channel as usize) {
                pwm.set_duty_cycle_ns(value as u32 * 1000)?;
            }
        }
        let _ = (channel, value);
        Ok(())
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
