pub mod serial_text;

pub use serial_text::{MockSerialController, TextSerialController};

use crate::error::Result;

pub trait ServoProtocol {
    fn send_servo_command(&mut self, servo_id: u8, finger_name: &str, angle: f32) -> Result<()>;
    fn send_raw_command(&mut self, command: &str) -> Result<()>;
}

