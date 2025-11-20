use robot_hand::{
    CommunicationConfig, FingerConfig, HandConfig, HandController, JointConfig, MotorType,
    Protocol, WristConfig,
};

#[test]
fn test_hand_initialization() {
    let config = create_test_config();
    let mut hand = HandController::new(config).unwrap();

    assert!(hand.initialize().is_ok());
    assert!(hand.shutdown().is_ok());
}

#[test]
fn test_finger_movement() {
    let config = create_test_config();
    let mut hand = HandController::new(config).unwrap();

    hand.initialize().unwrap();

    assert!(hand.move_finger(0, &[45.0, 30.0, 20.0]).is_ok());

    hand.shutdown().unwrap();
}

#[test]
fn test_invalid_finger_id() {
    let config = create_test_config();
    let mut hand = HandController::new(config).unwrap();

    hand.initialize().unwrap();

    assert!(hand.move_finger(99, &[45.0]).is_err());
}

#[test]
fn test_open_close_hand() {
    let config = create_test_config();
    let mut hand = HandController::new(config).unwrap();

    hand.initialize().unwrap();

    assert!(hand.open_hand().is_ok());
    assert!(hand.close_hand().is_ok());

    hand.shutdown().unwrap();
}

#[test]
fn test_grasp() {
    let config = create_test_config();
    let mut hand = HandController::new(config).unwrap();

    hand.initialize().unwrap();

    assert!(hand.grasp(50.0).is_ok());

    hand.shutdown().unwrap();
}

#[test]
fn test_wrist_movement() {
    let config = create_test_config();
    let mut hand = HandController::new(config).unwrap();

    hand.initialize().unwrap();

    assert!(hand.move_wrist([10.0, 0.0, -5.0]).is_ok());

    hand.shutdown().unwrap();
}

#[test]
fn test_config_from_string() {
    let toml_str = r#"
        [communication]
        protocol = "mock"
        
        [[fingers]]
        name = "TestFinger"
        
        [[fingers.joints]]
        name = "Joint1"
        motor_type = "pwmservo"
        channel = 0
        min_angle = 0.0
        max_angle = 90.0
        offset = 0.0
        min_pulse = 500
        max_pulse = 2500
        
        [wrist]
    "#;

    assert!(HandConfig::from_string(toml_str).is_ok());
}

fn create_test_config() -> HandConfig {
    HandConfig {
        fingers: vec![
            FingerConfig {
                name: "Thumb".to_string(),
                joints: vec![
                    JointConfig {
                        name: "CMC".to_string(),
                        motor_type: MotorType::PwmServo,
                        channel: 0,
                        min_angle: 0.0,
                        max_angle: 90.0,
                        offset: 0.0,
                        min_pulse: 500,
                        max_pulse: 2500,
                    },
                    JointConfig {
                        name: "MCP".to_string(),
                        motor_type: MotorType::PwmServo,
                        channel: 1,
                        min_angle: 0.0,
                        max_angle: 90.0,
                        offset: 0.0,
                        min_pulse: 500,
                        max_pulse: 2500,
                    },
                    JointConfig {
                        name: "IP".to_string(),
                        motor_type: MotorType::PwmServo,
                        channel: 2,
                        min_angle: 0.0,
                        max_angle: 90.0,
                        offset: 0.0,
                        min_pulse: 500,
                        max_pulse: 2500,
                    },
                ],
            },
            FingerConfig {
                name: "Index".to_string(),
                joints: vec![JointConfig {
                    name: "MCP".to_string(),
                    motor_type: MotorType::PwmServo,
                    channel: 3,
                    min_angle: 0.0,
                    max_angle: 90.0,
                    offset: 0.0,
                    min_pulse: 500,
                    max_pulse: 2500,
                }],
            },
        ],
        wrist: WristConfig {
            pitch: Some(JointConfig {
                name: "Pitch".to_string(),
                motor_type: MotorType::PwmServo,
                channel: 15,
                min_angle: -45.0,
                max_angle: 45.0,
                offset: 0.0,
                min_pulse: 500,
                max_pulse: 2500,
            }),
            roll: None,
            yaw: None,
        },
        communication: CommunicationConfig {
            protocol: Protocol::Mock,
            serial_port: String::new(),
            baud_rate: 115200,
            i2c_address: 0x40,
        },
    }
}
