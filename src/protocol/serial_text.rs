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
        let mut port = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(500))
            .open()
            .map_err(|e| HandError::Communication(format!("Failed to open serial port: {}", e)))?;

        // Wait for Arduino to reset and be ready (opening serial port triggers reset)
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Clear any startup messages
        let mut buffer = [0u8; 256];
        let _ = port.read(&mut buffer);

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
    fn send_servo_command(&mut self, servo_id: u8, _finger_name: &str, angle: f32) -> Result<()> {
        let command = format!("S{}:{}\n", servo_id, angle as i32);
        self.send_raw_command(&command)
    }

    #[cfg(feature = "serial")]
    fn send_raw_command(&mut self, command: &str) -> Result<()> {
        use crate::error::HandError;
        use std::io::Read;
        eprintln!(
            "DEBUG: Sending: {} (bytes: {:?})",
            command.trim(),
            command.as_bytes()
        );
        self.port
            .write_all(command.as_bytes())
            .map_err(|e| HandError::Communication(format!("Failed to send command: {}", e)))?;
        self.port
            .flush()
            .map_err(|e| HandError::Communication(format!("Failed to flush: {}", e)))?;

        std::thread::sleep(std::time::Duration::from_millis(200));
        let mut buffer = [0u8; 256];
        let mut total_read = 0;
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(300) {
            match self.port.read(&mut buffer[total_read..]) {
                Ok(n) if n > 0 => {
                    total_read += n;
                    if total_read >= buffer.len() {
                        break;
                    }
                }
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    break;
                }
                Err(e) => {
                    eprintln!("DEBUG: Error reading: {}", e);
                    break;
                }
            }
        }
        if total_read > 0 {
            let response = String::from_utf8_lossy(&buffer[..total_read]);
            eprintln!(
                "DEBUG: Arduino response ({} bytes): {}",
                total_read,
                response.trim()
            );
        } else {
            eprintln!("DEBUG: No response from Arduino (timeout)");
        }
        Ok(())
    }

    #[cfg(not(feature = "serial"))]
    fn send_raw_command(&mut self, command: &str) -> Result<()> {
        println!("MOCK: Sending command: {}", command.trim());
        Ok(())
    }
}

pub struct MockSerialController;

impl Default for MockSerialController {
    fn default() -> Self {
        Self
    }
}

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
