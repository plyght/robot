use crate::error::{HandError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandConfig {
    pub fingers: Vec<FingerConfig>,
    pub wrist: WristConfig,
    pub communication: CommunicationConfig,
}

impl HandConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: HandConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_string(content: &str) -> Result<Self> {
        let config: HandConfig = toml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.fingers.is_empty() {
            return Err(HandError::Config(
                "At least one finger must be configured".to_string(),
            ));
        }

        for (i, finger) in self.fingers.iter().enumerate() {
            if finger.joints.is_empty() {
                return Err(HandError::Config(format!(
                    "Finger {} must have at least one joint",
                    i
                )));
            }
        }

        Ok(())
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| HandError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerConfig {
    pub name: String,
    pub joints: Vec<JointConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointConfig {
    pub name: String,
    pub motor_type: MotorType,
    pub channel: u8,
    pub min_angle: f32,
    pub max_angle: f32,
    pub offset: f32,
    #[serde(default)]
    pub min_pulse: u16,
    #[serde(default)]
    pub max_pulse: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MotorType {
    PwmServo,
    Stepper,
    Dc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WristConfig {
    pub pitch: Option<JointConfig>,
    pub roll: Option<JointConfig>,
    pub yaw: Option<JointConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    pub protocol: Protocol,
    #[serde(default)]
    pub serial_port: String,
    #[serde(default)]
    pub baud_rate: u32,
    #[serde(default)]
    pub i2c_address: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Serial,
    I2c,
    Mock,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            protocol: Protocol::Mock,
            serial_port: String::new(),
            baud_rate: 115200,
            i2c_address: 0x40,
        }
    }
}
