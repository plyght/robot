use crate::control::pickup_sequence::{create_default_finger_servo_map, PickupSequence};
use crate::emg::{EmgReader, EmgState};
use crate::error::Result;
use crate::protocol::ServoProtocol;
use crate::vision::{classify_object_type, select_best_object, GripPattern, ObjectDetector};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

pub struct VisionControllerConfig {
    pub camera_poll_interval: Duration,
    pub emg_poll_interval: Duration,
    pub finger_to_servo_map: HashMap<String, u8>,
}

impl Default for VisionControllerConfig {
    fn default() -> Self {
        Self {
            camera_poll_interval: Duration::from_millis(100),
            emg_poll_interval: Duration::from_millis(10),
            finger_to_servo_map: create_default_finger_servo_map(),
        }
    }
}

pub struct VisionController<D: ObjectDetector, P: ServoProtocol> {
    detector: D,
    emg_reader: EmgReader,
    protocol: P,
    config: VisionControllerConfig,
    current_sequence: Option<PickupSequence>,
    pub running: bool,
}

impl<D: ObjectDetector, P: ServoProtocol> VisionController<D, P> {
    pub fn new(
        detector: D,
        emg_reader: EmgReader,
        protocol: P,
        config: VisionControllerConfig,
    ) -> Self {
        Self {
            detector,
            emg_reader,
            protocol,
            config,
            current_sequence: None,
            running: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.running = true;
        println!("Vision + EMG Control System Started");
        println!("Threshold: {} | Waiting for EMG trigger...\n", 600);

        while self.running {
            if let Some(ref mut sequence) = self.current_sequence {
                if self.emg_reader.get_state() == EmgState::Executing {
                    let complete =
                        sequence.execute_step_by_step(&mut self.protocol, &self.config.finger_to_servo_map)?;

                    if complete {
                        println!("\n✓ Pickup sequence completed!\n");
                        self.current_sequence = None;
                        self.emg_reader.set_state(EmgState::Idle);
                        println!("Ready for next trigger...\n");
                    }
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }

            if self.emg_reader.poll()? {
                println!("\n🔔 EMG threshold triggered!");

                self.emg_reader.set_state(EmgState::Executing);

                let objects = self.detector.detect_objects()?;
                println!("   Detected {} objects", objects.len());

                if objects.is_empty() {
                    println!("   ⚠ No objects detected, returning to idle\n");
                    self.emg_reader.set_state(EmgState::Idle);
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }

                for (idx, obj) in objects.iter().enumerate() {
                    println!(
                        "   {}. {} (confidence: {:.1}%)",
                        idx + 1,
                        obj.label,
                        obj.confidence * 100.0
                    );
                }

                let (frame_width, frame_height) = self.detector.get_frame_size();
                let frame_center = (frame_width / 2, frame_height / 2);

                if let Some(selected_obj) = select_best_object(&objects, frame_center) {
                    println!("\n   → Selected: {}", selected_obj.label);

                    let object_type = classify_object_type(&selected_obj.label)
                        .unwrap_or("small_object");
                    println!("   → Classified as: {}", object_type);

                    let grip_pattern = GripPattern::for_object_type(object_type);
                    println!("   → Using grip: {:?}\n", grip_pattern.pattern_type);

                    let sequence = PickupSequence::new(grip_pattern);
                    self.current_sequence = Some(sequence);
                } else {
                    println!("   ⚠ Could not select object, returning to idle\n");
                    self.emg_reader.set_state(EmgState::Idle);
                }
            }

            thread::sleep(self.config.emg_poll_interval);
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
        println!("\nVision + EMG Control System Stopped");
    }

    pub fn inject_emg_trigger(&mut self, value: u16) -> Result<()> {
        self.emg_reader.inject_value(value)?;
        Ok(())
    }

    pub fn run_step(&mut self) -> Result<bool> {
        if let Some(ref mut sequence) = self.current_sequence {
            if self.emg_reader.get_state() == EmgState::Executing {
                let complete =
                    sequence.execute_step_by_step(&mut self.protocol, &self.config.finger_to_servo_map)?;

                if complete {
                    println!("\n✓ Pickup sequence completed!\n");
                    self.current_sequence = None;
                    self.emg_reader.set_state(EmgState::Idle);
                    println!("Ready for next trigger...\n");
                }
                return Ok(true);
            }
        }

        let current_state = self.emg_reader.get_state();
        
        if current_state == EmgState::Triggered {
            println!("\n🔔 Manual trigger activated!");
            self.emg_reader.set_state(EmgState::Executing);
        } else if self.emg_reader.poll()? {
            println!("\n🔔 EMG threshold triggered!");
            self.emg_reader.set_state(EmgState::Executing);
        } else {
            return Ok(self.running);
        }
        
        if self.emg_reader.get_state() != EmgState::Executing {
            return Ok(self.running);
        }

        let objects = self.detector.detect_objects()?;
        println!("   Detected {} objects", objects.len());

        if objects.is_empty() {
            println!("   ⚠ No objects detected, returning to idle\n");
            self.emg_reader.set_state(EmgState::Idle);
            return Ok(true);
        }

        for (idx, obj) in objects.iter().enumerate() {
            println!(
                "   {}. {} (confidence: {:.1}%)",
                idx + 1,
                obj.label,
                obj.confidence * 100.0
            );
        }

        let (frame_width, frame_height) = self.detector.get_frame_size();
        let frame_center = (frame_width / 2, frame_height / 2);

        if let Some(selected_obj) = select_best_object(&objects, frame_center) {
            println!("\n   → Selected: {}", selected_obj.label);

            let object_type = classify_object_type(&selected_obj.label)
                .unwrap_or("small_object");
            println!("   → Classified as: {}", object_type);

            let grip_pattern = GripPattern::for_object_type(object_type);
            println!("   → Using grip: {:?}\n", grip_pattern.pattern_type);

            let sequence = PickupSequence::new(grip_pattern);
            self.current_sequence = Some(sequence);
        } else {
            println!("   ⚠ Could not select object, returning to idle\n");
            self.emg_reader.set_state(EmgState::Idle);
        }

        Ok(self.running)
    }
}

