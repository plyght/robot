use crate::config::{HandConfig, JointConfig, MotorType, Protocol};
use crate::error::{HandError, Result};
use crate::hand::{Finger, Hand, Joint, Wrist};
use crate::hardware::{DcMotor, I2cController, Motor, MotorController, PwmServo, StepperMotor};
use crate::platform::MockController;

pub struct HandController {
    hand: Hand,
    config: HandConfig,
}

impl HandController {
    pub fn new(config: HandConfig) -> Result<Self> {
        let controller = Self::create_controller(&config)?;
        let fingers = Self::create_fingers(&config, controller.as_ref())?;
        let wrist = Self::create_wrist(&config, controller.as_ref())?;
        let hand = Hand::new(fingers, wrist);

        Ok(Self { hand, config })
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.hand.initialize()
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.hand.shutdown()
    }

    pub fn move_finger(&mut self, finger_id: usize, angles: &[f32]) -> Result<()> {
        self.hand.set_finger_pose(finger_id, angles)
    }

    pub fn move_wrist(&mut self, orientation: [f32; 3]) -> Result<()> {
        self.hand
            .set_wrist_orientation(orientation[0], orientation[1], orientation[2])
    }

    pub fn open_hand(&mut self) -> Result<()> {
        for i in 0..self.hand.finger_count() {
            let finger = self
                .hand
                .get_finger(i)
                .ok_or(HandError::InvalidFingerId(i))?;
            let joint_count = finger.joint_count();
            let open_pose: Vec<f32> = vec![0.0; joint_count];
            self.hand.set_finger_pose(i, &open_pose)?;
        }
        Ok(())
    }

    pub fn close_hand(&mut self) -> Result<()> {
        for i in 0..self.hand.finger_count() {
            let finger = self
                .hand
                .get_finger(i)
                .ok_or(HandError::InvalidFingerId(i))?;
            let joint_count = finger.joint_count();
            let close_pose: Vec<f32> = vec![90.0; joint_count];
            self.hand.set_finger_pose(i, &close_pose)?;
        }
        Ok(())
    }

    pub fn grasp(&mut self, object_size: f32) -> Result<()> {
        let close_amount = (100.0 - object_size).clamp(0.0, 90.0);

        for i in 0..self.hand.finger_count() {
            let finger = self
                .hand
                .get_finger(i)
                .ok_or(HandError::InvalidFingerId(i))?;
            let joint_count = finger.joint_count();
            let grasp_pose: Vec<f32> = vec![close_amount; joint_count];
            self.hand.set_finger_pose(i, &grasp_pose)?;
        }
        Ok(())
    }

    pub fn hand(&self) -> &Hand {
        &self.hand
    }

    pub fn hand_mut(&mut self) -> &mut Hand {
        &mut self.hand
    }

    pub fn config(&self) -> &HandConfig {
        &self.config
    }

    fn create_controller(config: &HandConfig) -> Result<Box<dyn MotorController>> {
        match config.communication.protocol {
            Protocol::Serial => {
                #[cfg(feature = "serial")]
                {
                    use crate::hardware::SerialController;
                    Ok(Box::new(SerialController::new(
                        &config.communication.serial_port,
                        config.communication.baud_rate,
                    )?))
                }
                #[cfg(not(feature = "serial"))]
                {
                    Err(HandError::NotSupported(
                        "Serial support not enabled. Enable 'serial' feature".to_string(),
                    ))
                }
            }
            Protocol::I2c => Ok(Box::new(I2cController::new(
                config.communication.i2c_address,
            ))),
            Protocol::Mock => Ok(Box::new(MockController::new())),
        }
    }

    fn create_motor(
        joint_config: &JointConfig,
        _controller: &dyn MotorController,
    ) -> Result<Box<dyn Motor>> {
        match joint_config.motor_type {
            MotorType::PwmServo => {
                let controller_clone = MockController::new();
                Ok(Box::new(PwmServo::new(
                    joint_config.channel,
                    joint_config.min_angle,
                    joint_config.max_angle,
                    joint_config.min_pulse,
                    joint_config.max_pulse,
                    Box::new(controller_clone),
                )))
            }
            MotorType::Stepper => Ok(Box::new(StepperMotor::new(
                joint_config.channel as usize,
                joint_config.min_angle,
                joint_config.max_angle,
                200,
            ))),
            MotorType::Dc => Ok(Box::new(DcMotor::new(
                joint_config.channel as usize,
                joint_config.min_angle,
                joint_config.max_angle,
            ))),
        }
    }

    fn create_fingers(
        config: &HandConfig,
        controller: &dyn MotorController,
    ) -> Result<Vec<Finger>> {
        let mut fingers = Vec::new();

        for (i, finger_config) in config.fingers.iter().enumerate() {
            let mut joints = Vec::new();

            for joint_config in &finger_config.joints {
                let motor = Self::create_motor(joint_config, controller)?;
                let joint = Joint::new(motor, joint_config.name.clone(), joint_config.offset);
                joints.push(joint);
            }

            fingers.push(Finger::new(i, finger_config.name.clone(), joints));
        }

        Ok(fingers)
    }

    fn create_wrist(config: &HandConfig, controller: &dyn MotorController) -> Result<Wrist> {
        let pitch_motor = config
            .wrist
            .pitch
            .as_ref()
            .map(|c| Self::create_motor(c, controller))
            .transpose()?;

        let roll_motor = config
            .wrist
            .roll
            .as_ref()
            .map(|c| Self::create_motor(c, controller))
            .transpose()?;

        let yaw_motor = config
            .wrist
            .yaw
            .as_ref()
            .map(|c| Self::create_motor(c, controller))
            .transpose()?;

        Ok(Wrist::new(pitch_motor, roll_motor, yaw_motor))
    }
}
