pub mod config;
pub mod control;
pub mod emg;
pub mod error;
pub mod hand;
pub mod hardware;
pub mod platform;
pub mod protocol;
pub mod vision;

pub use config::{
    CommunicationConfig, FingerConfig, HandConfig, JointConfig, MotorType, Protocol, WristConfig,
};
pub use control::{HandController, MotionPlanner, PickupSequence, SequenceStep, Trajectory, TrajectoryPoint, VisionController, VisionControllerConfig, create_default_finger_servo_map};
pub use emg::{EmgReader, EmgState, MockEmgReader};
pub use error::{HandError, Result};
pub use hand::{Finger, Hand, Joint, Wrist};
pub use hardware::{DcMotor, I2cController, Motor, MotorController, PwmServo, StepperMotor};
pub use platform::{I2cPlatformController, LinuxPwmController, MockController};
pub use protocol::{ServoProtocol, TextSerialController, MockSerialController};
pub use vision::{DetectedObject, BoundingBox, GripPattern, GripPatternType, MockObjectDetector, ObjectDetector, classify_object_type, select_best_object};
