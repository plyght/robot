use crate::error::Result;
use std::time::{Duration, Instant};

#[cfg(feature = "serial")]
use std::io::Read;

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
                    let line = self.buffer[..newline_pos].trim();
                    self.buffer = self.buffer[newline_pos + 1..].to_string();

                    if let Ok(value) = line.parse::<u16>() {
                        return Ok(Some(value.min(1023)));
                    }
                }
                Ok(None)
            }
            Ok(_) => Ok(None),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(HandError::Communication(format!(
                "EMG read error: {}",
                e
            ))),
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

