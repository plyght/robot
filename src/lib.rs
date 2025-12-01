pub mod config;
pub mod control;
pub mod emg;
pub mod error;
pub mod hand;
pub mod hardware;
pub mod kinematics;
pub mod platform;
pub mod protocol;
pub mod vision;

pub use config::{
    CommunicationConfig, FingerConfig, HandConfig, JointConfig, MotorType, Protocol, WristConfig,
};
pub use control::{
    create_default_finger_servo_map, HandController, MotionPlanner, PickupSequence, SequenceStep,
    Trajectory, TrajectoryPoint, VisionController, VisionControllerConfig,
};
pub use emg::{EmgReader, EmgState, MockEmgReader};
pub use error::{HandError, Result};
pub use hand::{Finger, Hand, Joint, Wrist};
pub use hardware::{DcMotor, Finger as HardwareFinger, I2cController, Motor, MotorController, PwmServo, ServoConfig, ServoMap, StepperMotor};
pub use kinematics::{ForwardKinematics, HandGeometry, InverseKinematics, JointAngles, Position3D};
pub use platform::{I2cPlatformController, LinuxPwmController, MockController};
pub use protocol::{MockSerialController, ServoProtocol, TextSerialController};
pub use vision::{
    classify_object_type, cleanup_temp_files, create_tracking_data, ensure_temp_dir,
    select_best_object, BoundingBox, DepthProService, DetectedObject, GripPattern, GripPatternType,
    MockObjectDetector, ObjectDepth, ObjectDetector, ObjectTrackingData,
};

#[cfg(feature = "opencv")]
pub use vision::{create_tracking_with_image, ObjectTrackingWithImage};

#[cfg(feature = "opencv")]
pub use vision::OpenCVDetector;
