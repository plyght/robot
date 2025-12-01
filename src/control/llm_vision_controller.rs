use crate::control::llm_planner::{LlmPlanner, MovementCommand, SceneState};
use crate::emg::{EmgReader, EmgState};
use crate::error::Result;
use crate::hardware::{Finger, ServoMap};
use crate::kinematics::{ForwardKinematics, InverseKinematics, JointAngles, Position3D};
use crate::protocol::ServoProtocol;
use crate::vision::{select_best_object, DetectedObject, ObjectDetector, DepthProService, ensure_temp_dir};
use std::thread;
use std::time::Duration;

#[cfg(feature = "opencv")]
use crate::vision::HandTracker;

pub struct LlmVisionControllerConfig {
    pub camera_poll_interval: Duration,
    pub emg_poll_interval: Duration,
    pub servo_map: ServoMap,
    pub enable_hand_tracking: bool,
    pub enable_llm_planning: bool,
    pub auto_trigger: bool,
    pub auto_trigger_delay_secs: u64,
    pub hand_base_position: Position3D,
}

impl Default for LlmVisionControllerConfig {
    fn default() -> Self {
        Self {
            camera_poll_interval: Duration::from_millis(100),
            emg_poll_interval: Duration::from_millis(10),
            servo_map: ServoMap::hardware_default(),
            enable_hand_tracking: true,
            enable_llm_planning: true,
            auto_trigger: false,
            auto_trigger_delay_secs: 2,
            hand_base_position: Position3D::new(0.0, 0.0, 0.0),
        }
    }
}

pub struct LlmVisionController<D: ObjectDetector, P: ServoProtocol> {
    detector: D,
    emg_reader: EmgReader,
    protocol: P,
    config: LlmVisionControllerConfig,
    llm_planner: Option<LlmPlanner>,
    #[cfg(feature = "opencv")]
    hand_tracker: Option<HandTracker>,
    depth_service: Option<DepthProService>,
    current_commands: Vec<MovementCommand>,
    command_index: usize,
    pub running: bool,
    fk: ForwardKinematics,
    ik: InverseKinematics,
    current_joint_angles: JointAngles,
}

impl<D: ObjectDetector, P: ServoProtocol> LlmVisionController<D, P> {
    pub fn new(
        detector: D,
        emg_reader: EmgReader,
        protocol: P,
        config: LlmVisionControllerConfig,
    ) -> Result<Self> {
        let llm_planner = if config.enable_llm_planning {
            match LlmPlanner::new() {
                Ok(planner) => {
                    println!("LLM planner initialized successfully");
                    Some(planner)
                }
                Err(e) => {
                    println!("Warning: LLM planner initialization failed: {}", e);
                    println!("Continuing without LLM planning (will use fallback logic)");
                    None
                }
            }
        } else {
            None
        };

        #[cfg(feature = "opencv")]
        let hand_tracker = if config.enable_hand_tracking {
            let (frame_width, frame_height) = detector.get_frame_size();
            Some(HandTracker::new(frame_width, frame_height))
        } else {
            None
        };

        let depth_service = match DepthProService::new(None) {
            Ok(service) => {
                println!("Depth detection service initialized");
                Some(service)
            }
            Err(e) => {
                println!("Warning: Depth detection service failed to initialize: {}", e);
                println!("Continuing without depth detection (will use estimated depths)");
                None
            }
        };

        ensure_temp_dir().ok();

        let base_position = config.hand_base_position;
        let fk = ForwardKinematics::with_default_geometry(base_position);
        let ik = InverseKinematics::with_default_geometry(base_position);

        Ok(Self {
            detector,
            emg_reader,
            protocol,
            config,
            llm_planner,
            #[cfg(feature = "opencv")]
            hand_tracker,
            depth_service,
            current_commands: Vec::new(),
            command_index: 0,
            running: false,
            fk,
            ik,
            current_joint_angles: JointAngles::open(),
        })
    }

    #[cfg(feature = "opencv")]
    pub fn load_hand_tracking_model(&mut self, model_path: &str) -> Result<()> {
        if let Some(ref mut tracker) = self.hand_tracker {
            tracker.load_model(model_path)?;
            println!("Hand tracking model loaded");
        }
        Ok(())
    }

    pub async fn run_async(&mut self) -> Result<()> {
        self.running = true;

        if self.config.auto_trigger {
            println!("   Auto-trigger mode: will detect objects automatically\n");
        }

        while self.running {
            if !self.current_commands.is_empty() && self.emg_reader.get_state() == EmgState::Executing {
                if self.command_index < self.current_commands.len() {
                    let cmd = self.current_commands[self.command_index].clone();
                    print!("   Step {}/{}: ", self.command_index + 1, self.current_commands.len());
                    self.execute_movement_command(&cmd)?;
                    self.command_index += 1;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                } else {
                    println!("   ✓ Done!\n");
                    self.current_commands.clear();
                    self.command_index = 0;
                    self.emg_reader.set_state(EmgState::Idle);
                    if self.config.auto_trigger {
                        tokio::time::sleep(Duration::from_secs(self.config.auto_trigger_delay_secs)).await;
                    }
                }
            }

            let should_trigger = if self.config.auto_trigger {
                self.check_auto_trigger().await?
            } else {
                self.emg_reader.poll()?
            };

            if should_trigger {
                println!("\n🔔 Triggered!");
                self.emg_reader.set_state(EmgState::Executing);

                #[cfg(feature = "opencv")]
                let temp_path = format!("temp/frame_{}.jpg", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                
                #[cfg(feature = "opencv")]
                if self.depth_service.is_some() {
                    let _ = self.detector.save_current_frame(&temp_path);
                }

                let mut objects = self.detector.detect_objects()?;

                if objects.is_empty() {
                    println!("   No objects found\n");
                    self.emg_reader.set_state(EmgState::Idle);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }

                #[cfg(feature = "opencv")]
                if let Some(ref mut depth_service) = self.depth_service {
                    if std::path::Path::new(&temp_path).exists() {
                        if let Ok(depths) = depth_service.process_image_with_cleanup(&temp_path, &objects, true) {
                            for (obj, depth) in objects.iter_mut().zip(depths.iter()) {
                                obj.distance = depth.depth_cm / 100.0;
                            }
                        }
                    }
                }

                let (frame_width, frame_height) = self.detector.get_frame_size();
                let frame_center = (frame_width / 2, frame_height / 2);

                if let Some(selected_obj) = select_best_object(&objects, frame_center) {
                    println!("   Target: {} ({:.0}cm away)", selected_obj.label, selected_obj.distance * 100.0);

                    if let Some(commands) = self.plan_movement(selected_obj, &objects).await? {
                        println!("   Planning: {} steps", commands.len());
                        self.current_commands = commands;
                        self.command_index = 0;
                    } else {
                        self.use_fallback_pickup(selected_obj)?;
                    }
                } else {
                    self.emg_reader.set_state(EmgState::Idle);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.config.emg_poll_interval.as_millis() as u64,
            ))
            .await;
        }

        Ok(())
    }

    async fn plan_movement(
        &mut self,
        target: &DetectedObject,
        all_objects: &[DetectedObject],
    ) -> Result<Option<Vec<MovementCommand>>> {
        if self.llm_planner.is_none() {
            return Ok(None);
        }

        #[cfg(feature = "opencv")]
        let hand_pose = {
            if let Some(ref mut tracker) = self.hand_tracker {
                let hand_frame_path = format!("temp/hand_frame_{}.jpg", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                if self.detector.save_current_frame(&hand_frame_path).is_ok() {
                    use opencv::imgcodecs;
                    if let Ok(frame) = imgcodecs::imread(&hand_frame_path, imgcodecs::IMREAD_COLOR) {
                        let depth = Some(target.distance * 100.0);
                        if let Ok(hands) = tracker.detect_hands(&frame, depth) {
                            let _ = std::fs::remove_file(&hand_frame_path);
                            hands.first().cloned()
                        } else {
                            let _ = std::fs::remove_file(&hand_frame_path);
                            None
                        }
                    } else {
                        let _ = std::fs::remove_file(&hand_frame_path);
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        #[cfg(not(feature = "opencv"))]
        let hand_pose = None;

        let other_objects: Vec<DetectedObject> = all_objects
            .iter()
            .filter(|obj| obj.label != target.label)
            .cloned()
            .collect();

        let scene = SceneState {
            target_object: target.clone(),
            object_depth_cm: target.distance,
            hand_pose,
            other_objects,
            camera_fov_horizontal: 60.0,
            camera_fov_vertical: 45.0,
        };

        if let Some(ref planner) = self.llm_planner {
            match planner.generate_movement_plan(&scene).await {
                Ok(commands) => Ok(Some(commands)),
                Err(e) => {
                    println!("   ⚠ LLM planning failed: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn execute_movement_command(&mut self, cmd: &MovementCommand) -> Result<()> {
        use crate::control::llm_planner::MovementAction;

        match cmd.action {
            MovementAction::OpenHand => {
                println!("Open hand");
                self.current_joint_angles = JointAngles::open();
                let angles = self.current_joint_angles.clone();
                self.send_joint_angles(&angles)?;
            }
            MovementAction::CloseHand => {
                println!("Close hand");
                self.current_joint_angles = JointAngles::closed();
                let angles = self.current_joint_angles.clone();
                self.send_joint_angles(&angles)?;
            }
            MovementAction::Grasp => {
                if let Some(strength) = cmd.parameters.grip_strength {
                    println!("Grasp ({:.0}%)", strength * 100.0);
                    let angle = strength * 90.0;
                    self.current_joint_angles.thumb = angle * 0.8;
                    self.current_joint_angles.index = angle;
                    self.current_joint_angles.middle = angle;
                    self.current_joint_angles.ring = angle;
                    self.current_joint_angles.pinky = angle * 0.9;
                    let angles = self.current_joint_angles.clone();
                    self.send_joint_angles(&angles)?;
                }
            }
            MovementAction::MoveToPosition => {
                if let (Some(x), Some(y), Some(z)) = (
                    cmd.parameters.target_x_cm,
                    cmd.parameters.target_y_cm,
                    cmd.parameters.target_z_cm,
                ) {
                    println!("Move to ({:.0}, {:.0}, {:.0})cm", x, y, z);
                    let target = Position3D::new(x, y, z);

                    match self.ik.solve_for_grasp_position(target, Some(self.current_joint_angles.clone())) {
                        Ok(angles) => {
                            self.current_joint_angles = angles.clone();
                            self.send_joint_angles(&angles)?;
                        }
                        Err(e) => {
                            println!("   ⚠ IK failed: {}, using direct wrist control", e);
                            if let Some(pitch) = cmd.parameters.wrist_pitch {
                                self.current_joint_angles.wrist_pitch = Some(pitch);
                            }
                            if let Some(roll) = cmd.parameters.wrist_roll {
                                self.current_joint_angles.wrist_roll = Some(roll);
                            }
                            let angles = self.current_joint_angles.clone();
                            self.send_joint_angles(&angles)?;
                        }
                    }
                }
            }
            MovementAction::RotateWrist => {
                if let (Some(pitch), Some(roll)) = (cmd.parameters.wrist_pitch, cmd.parameters.wrist_roll) {
                    println!("Rotate wrist ({:.0}°, {:.0}°)", pitch, roll);
                    self.current_joint_angles.wrist_pitch = Some(pitch);
                    self.current_joint_angles.wrist_roll = Some(roll);
                    let angles = self.current_joint_angles.clone();
                    self.send_joint_angles(&angles)?;
                }
            }
            MovementAction::Approach => {
                println!("Approach");
                self.current_joint_angles = JointAngles::open();
                let angles = self.current_joint_angles.clone();
                self.send_joint_angles(&angles)?;
            }
            MovementAction::Retreat => {
                println!("Retreat");
            }
            MovementAction::Release => {
                println!("Release");
                self.current_joint_angles = JointAngles::open();
                let angles = self.current_joint_angles.clone();
                self.send_joint_angles(&angles)?;
            }
            MovementAction::Wait => {
                if let Some(duration) = cmd.parameters.duration_ms {
                    println!("Wait {}ms", duration);
                    thread::sleep(Duration::from_millis(duration));
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    fn send_joint_angles(&mut self, angles: &JointAngles) -> Result<()> {
        let finger_pairs = [
            (Finger::Thumb, angles.thumb),
            (Finger::Index, angles.index),
            (Finger::Middle, angles.middle),
            (Finger::Ring, angles.ring),
            (Finger::Pinky, angles.pinky),
        ];

        for (finger, angle) in finger_pairs {
            if let Some(config) = self.config.servo_map.get(finger) {
                let translated_angle = config.translate_angle(angle);
                self.protocol.send_servo_command(
                    config.id,
                    finger.name(),
                    translated_angle,
                )?;
            }
        }

        if let Some(pitch) = angles.wrist_pitch {
            self.protocol.send_servo_command(10, "WristPitch", pitch)?;
        }
        if let Some(roll) = angles.wrist_roll {
            self.protocol.send_servo_command(11, "WristRoll", roll)?;
        }

        let current_position = self.fk.compute_palm_center(angles);
        self.ik.update_base_position(current_position);

        Ok(())
    }

    fn use_fallback_pickup(&mut self, _selected_obj: &DetectedObject) -> Result<()> {
        println!("   Using fallback grip");
        Ok(())
    }

    async fn check_auto_trigger(&mut self) -> Result<bool> {
        if self.emg_reader.get_state() != EmgState::Idle {
            return Ok(false);
        }

        let objects = self.detector.detect_objects()?;
        if !objects.is_empty() {
            self.emg_reader.inject_value(self.emg_reader.threshold() + 1)?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn stop(&mut self) {
        self.running = false;
        println!("\nSystem stopped");
    }
}
