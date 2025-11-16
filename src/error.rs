use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandError {
    #[error("Hardware communication error: {0}")]
    Communication(String),

    #[error("Invalid joint angle: {angle} (joint {joint_id}, limits: {min}..{max})")]
    InvalidJointAngle {
        joint_id: usize,
        angle: f32,
        min: f32,
        max: f32,
    },

    #[error("Invalid finger ID: {0} (expected 0-4)")]
    InvalidFingerId(usize),

    #[error("Invalid joint count: expected {expected}, got {actual}")]
    InvalidJointCount { expected: usize, actual: usize },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Motor failure on joint {joint_id}: {reason}")]
    MotorFailure { joint_id: usize, reason: String },

    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Feature not supported: {0}")]
    NotSupported(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    #[cfg(feature = "serial")]
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),
}

pub type Result<T> = std::result::Result<T, HandError>;
