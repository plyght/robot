pub mod config;
pub mod control;
pub mod error;
pub mod hand;
pub mod hardware;
pub mod platform;

pub use config::{
    CommunicationConfig, FingerConfig, HandConfig, JointConfig, MotorType, Protocol, WristConfig,
};
pub use control::{HandController, MotionPlanner, Trajectory, TrajectoryPoint};
pub use error::{HandError, Result};
pub use hand::{Finger, Hand, Joint, Wrist};
pub use hardware::{DcMotor, I2cController, Motor, MotorController, PwmServo, StepperMotor};
pub use platform::{I2cPlatformController, LinuxPwmController, MockController};
