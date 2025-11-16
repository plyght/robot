use crate::error::Result;

pub trait Motor: Send {
    fn set_position(&mut self, angle: f32) -> Result<()>;

    fn get_position(&self) -> Result<f32>;

    fn enable(&mut self) -> Result<()>;

    fn disable(&mut self) -> Result<()>;

    fn is_enabled(&self) -> bool;

    fn set_speed(&mut self, speed: f32) -> Result<()> {
        let _ = speed;
        Ok(())
    }

    fn get_limits(&self) -> (f32, f32);
}

pub trait MotorController: Send {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()>;

    fn read_pwm(&mut self, channel: u8) -> Result<u16>;

    fn write_data(&mut self, address: u8, data: &[u8]) -> Result<()>;

    fn read_data(&mut self, address: u8, buffer: &mut [u8]) -> Result<usize>;
}
