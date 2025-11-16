use crate::error::Result;
use crate::protocol::ServoProtocol;

#[cfg(feature = "serial")]
use std::io::Write;

pub struct TextSerialController {
    #[cfg(feature = "serial")]
    port: Box<dyn serialport::SerialPort>,
    #[cfg(not(feature = "serial"))]
    _phantom: std::marker::PhantomData<()>,
}

impl TextSerialController {
    #[cfg(feature = "serial")]
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self> {
        use crate::error::HandError;
        let port = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(100))
            .open()
            .map_err(|e| HandError::Communication(format!("Failed to open serial port: {}", e)))?;
        Ok(Self { port })
    }

    #[cfg(not(feature = "serial"))]
    pub fn new(_port_name: &str, _baud_rate: u32) -> Result<Self> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

impl ServoProtocol for TextSerialController {
    fn send_servo_command(&mut self, servo_id: u8, finger_name: &str, angle: f32) -> Result<()> {
        let command = format!("servo{} {} {}\n", servo_id, finger_name, angle as i32);
        self.send_raw_command(&command)
    }

    #[cfg(feature = "serial")]
    fn send_raw_command(&mut self, command: &str) -> Result<()> {
        self.port
            .write_all(command.as_bytes())
            .map_err(|e| HandError::Communication(format!("Failed to send command: {}", e)))?;
        self.port
            .flush()
            .map_err(|e| HandError::Communication(format!("Failed to flush: {}", e)))?;
        Ok(())
    }

    #[cfg(not(feature = "serial"))]
    fn send_raw_command(&mut self, command: &str) -> Result<()> {
        println!("MOCK: Sending command: {}", command.trim());
        Ok(())
    }
}

pub struct MockSerialController;

impl MockSerialController {
    pub fn new() -> Self {
        Self
    }
}

impl ServoProtocol for MockSerialController {
    fn send_servo_command(&mut self, servo_id: u8, finger_name: &str, angle: f32) -> Result<()> {
        println!(
            "MOCK: servo{} {} {} degrees",
            servo_id, finger_name, angle as i32
        );
        Ok(())
    }

    fn send_raw_command(&mut self, command: &str) -> Result<()> {
        println!("MOCK: {}", command.trim());
        Ok(())
    }
}

