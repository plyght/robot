use crate::error::Result;
use std::time::{Duration, Instant};

#[cfg(feature = "serial")]
struct MockSerialPort;

#[cfg(feature = "serial")]
impl std::io::Read for MockSerialPort {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

#[cfg(feature = "serial")]
impl std::io::Write for MockSerialPort {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "serial")]
impl serialport::SerialPort for MockSerialPort {
    fn name(&self) -> Option<String> {
        Some("mock".to_string())
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        Ok(9600)
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        Ok(serialport::DataBits::Eight)
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        Ok(serialport::FlowControl::None)
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        Ok(serialport::Parity::None)
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        Ok(serialport::StopBits::One)
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(10)
    }

    fn set_baud_rate(&mut self, _baud_rate: u32) -> serialport::Result<()> {
        Ok(())
    }

    fn set_data_bits(&mut self, _data_bits: serialport::DataBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_flow_control(
        &mut self,
        _flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        Ok(())
    }

    fn set_parity(&mut self, _parity: serialport::Parity) -> serialport::Result<()> {
        Ok(())
    }

    fn set_stop_bits(&mut self, _stop_bits: serialport::StopBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_timeout(&mut self, _timeout: Duration) -> serialport::Result<()> {
        Ok(())
    }

    fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        Ok(true)
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        Ok(true)
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn set_break(&self) -> serialport::Result<()> {
        Ok(())
    }

    fn clear_break(&self) -> serialport::Result<()> {
        Ok(())
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn clear(&self, _buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        Ok(())
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn serialport::SerialPort>> {
        Ok(Box::new(MockSerialPort))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmgState {
    Idle,
    Triggered,
    Executing,
}

pub struct EmgReader {
    #[cfg(feature = "serial")]
    port: Box<dyn serialport::SerialPort>,
    #[cfg(not(feature = "serial"))]
    _phantom: std::marker::PhantomData<()>,
    threshold: u16,
    debounce_duration: Duration,
    last_trigger: Option<Instant>,
    state: EmgState,
    #[cfg(feature = "serial")]
    buffer: String,
}

impl EmgReader {
    #[cfg(feature = "serial")]
    pub fn new(port_name: &str, baud_rate: u32, threshold: u16) -> Result<Self> {
        if port_name == "mock" {
            return Ok(Self {
                port: Box::new(MockSerialPort),
                threshold,
                debounce_duration: Duration::from_millis(200),
                last_trigger: None,
                state: EmgState::Idle,
                buffer: String::with_capacity(32),
            });
        }
        use crate::error::HandError;
        let port = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(10))
            .open()
            .map_err(|e| HandError::Communication(format!("Failed to open EMG port: {}", e)))?;
        Ok(Self {
            port,
            threshold,
            debounce_duration: Duration::from_millis(200),
            last_trigger: None,
            state: EmgState::Idle,
            buffer: String::with_capacity(32),
        })
    }

    #[cfg(not(feature = "serial"))]
    pub fn new(_port_name: &str, _baud_rate: u32, threshold: u16) -> Result<Self> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
            threshold,
            debounce_duration: Duration::from_millis(200),
            last_trigger: None,
            state: EmgState::Idle,
        })
    }

    pub fn set_threshold(&mut self, threshold: u16) {
        self.threshold = threshold;
    }

    pub fn threshold(&self) -> u16 {
        self.threshold
    }

    pub fn set_debounce_duration(&mut self, duration: Duration) {
        self.debounce_duration = duration;
    }

    pub fn get_state(&self) -> EmgState {
        self.state
    }

    pub fn set_state(&mut self, state: EmgState) {
        self.state = state;
    }

    #[cfg(feature = "serial")]
    pub fn read_value(&mut self) -> Result<Option<u16>> {
        use crate::error::HandError;
        use std::io::Read;
        let mut temp_buf = [0u8; 32];
        match self.port.read(&mut temp_buf) {
            Ok(n) if n > 0 => {
                let data = String::from_utf8_lossy(&temp_buf[..n]);
                self.buffer.push_str(&data);

                if let Some(newline_pos) = self.buffer.find('\n') {
                    let line = self.buffer[..newline_pos].trim().to_string();
                    self.buffer = self.buffer[newline_pos + 1..].to_string();

                    if let Ok(value) = line.parse::<u16>() {
                        return Ok(Some(value.min(1023)));
                    }
                }
                Ok(None)
            }
            Ok(_) => Ok(None),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(HandError::Communication(format!("EMG read error: {}", e))),
        }
    }

    #[cfg(not(feature = "serial"))]
    pub fn read_value(&mut self) -> Result<Option<u16>> {
        Ok(None)
    }

    pub fn poll(&mut self) -> Result<bool> {
        if let Some(value) = self.read_value()? {
            let above_threshold = value >= self.threshold;

            if above_threshold && self.state == EmgState::Idle {
                if let Some(last) = self.last_trigger {
                    if last.elapsed() < self.debounce_duration {
                        return Ok(false);
                    }
                }

                self.state = EmgState::Triggered;
                self.last_trigger = Some(Instant::now());
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn inject_value(&mut self, value: u16) -> Result<bool> {
        let above_threshold = value >= self.threshold;

        if above_threshold && self.state == EmgState::Idle {
            if let Some(last) = self.last_trigger {
                if last.elapsed() < self.debounce_duration {
                    return Ok(false);
                }
            }

            self.state = EmgState::Triggered;
            self.last_trigger = Some(Instant::now());
            return Ok(true);
        }

        Ok(false)
    }
}

pub struct MockEmgReader {
    threshold: u16,
    current_value: u16,
}

impl MockEmgReader {
    pub fn new(threshold: u16) -> Self {
        Self {
            threshold,
            current_value: 0,
        }
    }

    pub fn set_value(&mut self, value: u16) {
        self.current_value = value.min(1023);
    }

    pub fn check_threshold(&self) -> bool {
        self.current_value >= self.threshold
    }

    pub fn get_value(&self) -> u16 {
        self.current_value
    }
}
