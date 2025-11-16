use robot_hand::{
    CommunicationConfig, FingerConfig, HandConfig, JointConfig, MotorType, Protocol, WristConfig,
};

fn main() -> robot_hand::Result<()> {
    let config = HandConfig {
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
                ],
            },
            FingerConfig {
                name: "Index".to_string(),
                joints: vec![JointConfig {
                    name: "MCP".to_string(),
                    motor_type: MotorType::PwmServo,
                    channel: 2,
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
                channel: 10,
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
    };

    println!("Saving configuration to config/custom.toml...");
    config.to_file("config/custom.toml")?;

    println!("Loading configuration from config/custom.toml...");
    let loaded_config = HandConfig::from_file("config/custom.toml")?;

    println!("Configuration loaded successfully!");
    println!("Number of fingers: {}", loaded_config.fingers.len());
    println!("Has wrist pitch: {}", loaded_config.wrist.pitch.is_some());

    Ok(())
}
