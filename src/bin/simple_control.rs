use robot_hand::{Result, ServoProtocol, TextSerialController};

#[cfg(feature = "serial")]
fn main() -> Result<()> {
    use std::io::{self, Write};
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <port> [command] [args...]", args[0]);
        eprintln!("\nIf no command provided, enters interactive mode (keeps port open)");
        eprintln!("\nCommands:");
        eprintln!("  open                    - Open hand (all to 0)");
        eprintln!("  close                   - Close hand (all to 90)");
        eprintln!("  all <angle>             - Move all fingers to angle");
        eprintln!("  <finger> <angle>        - Move finger to angle (0-180)");
        eprintln!("  q                       - Quit (interactive mode only)");
        eprintln!("\nFingers (left to right):");
        eprintln!("  1 or pinky             - Pinky (Servo ID 3)");
        eprintln!("  2 or index             - Index/Pointer (Servo ID 4, inverted)");
        eprintln!("  3 or middle             - Middle (Servo ID 2)");
        eprintln!("  4 or ring              - Ring (Servo ID 1)");
        eprintln!("\nExamples:");
        eprintln!("  {} /dev/cu.usbmodem1101 open", args[0]);
        eprintln!("  {} /dev/cu.usbmodem1101 1 90", args[0]);
        eprintln!("  {} /dev/cu.usbmodem1101        # Interactive mode", args[0]);
        std::process::exit(1);
    }

    let port = &args[1];
    let mut controller = TextSerialController::new(port, 9600)?;
    
    // Physical finger mapping (left to right): 3, 1, 2, 4
    //   Finger 1 (leftmost) = Pinky = Servo ID 3 ✓
    //   Finger 2 = Index/Pointer = Servo ID 4 ✓ (inverted: 180=open, 0=closed)
    //   Finger 3 = Middle = Servo ID 2 ✓
    //   Finger 4 (rightmost) = Ring = Servo ID 1 ✓
    let mut finger_map = std::collections::HashMap::new();
    finger_map.insert("1".to_string(), (3, false));  // Leftmost = Pinky = Servo ID 3 (normal) ✓
    finger_map.insert("2".to_string(), (4, true));   // Index/Pointer = Servo ID 4 (inverted) ✓
    finger_map.insert("3".to_string(), (2, false));  // Middle = Servo ID 2 (normal) ✓
    finger_map.insert("4".to_string(), (1, false));  // Rightmost = Ring = Servo ID 1 (normal) ✓
    finger_map.insert("left".to_string(), (3, false));
    finger_map.insert("pinky".to_string(), (3, false));
    finger_map.insert("index".to_string(), (4, true));
    finger_map.insert("pointer".to_string(), (4, true));
    finger_map.insert("middle".to_string(), (2, false));
    finger_map.insert("ring".to_string(), (1, false));
    finger_map.insert("right".to_string(), (1, false));
    
    // If no command provided, enter interactive mode
    if args.len() < 3 {
        println!("Interactive mode - port stays open (no resets between commands)");
        println!("Type 'q' to quit\n");
        
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let cmd = input.trim().to_lowercase();
            
            if cmd == "q" || cmd == "quit" {
                break;
            }
            
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "open" => {
                    for (finger_name, &(servo_id, inverted)) in &finger_map {
                        let angle = if inverted { 180.0 } else { 0.0 };
                        controller.send_servo_command(servo_id, finger_name, angle)?;
                    }
                    println!("✓ Hand opened");
                }
                "close" => {
                    for (finger_name, &(servo_id, inverted)) in &finger_map {
                        let angle = if inverted { 0.0 } else { 180.0 };
                        controller.send_servo_command(servo_id, finger_name, angle)?;
                    }
                    println!("✓ Hand closed");
                }
                "all" => {
                    if parts.len() < 2 {
                        println!("Usage: all <angle>");
                        continue;
                    }
                    if let Ok(angle) = parts[1].parse::<f32>() {
                        for (finger_name, &(servo_id, inverted)) in &finger_map {
                            let final_angle = if inverted { 180.0 - angle } else { angle };
                            controller.send_servo_command(servo_id, finger_name, final_angle)?;
                        }
                        println!("✓ All fingers moved to {}°", angle);
                    } else {
                        println!("Invalid angle: {}", parts[1]);
                    }
                }
                finger_name => {
                    if parts.len() < 2 {
                        println!("Usage: <finger> <angle>");
                        continue;
                    }
                    if let Some(&(servo_id, inverted)) = finger_map.get(finger_name) {
                        if let Ok(angle) = parts[1].parse::<f32>() {
                            let final_angle = if inverted { 180.0 - angle } else { angle };
                            controller.send_servo_command(servo_id, finger_name, final_angle)?;
                            println!("✓ Finger {} (servo {}) moved to {}° (sent: {}°)", 
                                     finger_name, servo_id, angle, final_angle);
                        } else {
                            println!("Invalid angle: {}", parts[1]);
                        }
                    } else {
                        println!("Unknown finger: {}", finger_name);
                    }
                }
            }
        }
        return Ok(());
    }
    
    // Single command mode
    let cmd = args[2].to_lowercase();
    println!("Connected to: {}", port);
    println!("Executing: {}", args[2..].join(" "));

    match cmd.as_str() {
        "open" => {
            for (finger_name, &(servo_id, inverted)) in &finger_map {
                let angle = if inverted { 180.0 } else { 0.0 };
                controller.send_servo_command(servo_id, finger_name, angle)?;
            }
            println!("✓ Hand opened");
        }
        "close" => {
            for (finger_name, &(servo_id, inverted)) in &finger_map {
                let angle = if inverted { 0.0 } else { 180.0 };
                controller.send_servo_command(servo_id, finger_name, angle)?;
            }
            println!("✓ Hand closed");
        }
        "all" => {
            if args.len() < 4 {
                eprintln!("Error: 'all' requires an angle argument");
                eprintln!("Usage: {} <port> all <angle>", args[0]);
                std::process::exit(1);
            }
            if let Ok(angle) = args[3].parse::<f32>() {
                for (finger_name, &(servo_id, inverted)) in &finger_map {
                    let final_angle = if inverted { 180.0 - angle } else { angle };
                    controller.send_servo_command(servo_id, finger_name, final_angle)?;
                }
                println!("✓ All fingers moved to {}°", angle);
            } else {
                eprintln!("Error: Invalid angle: {}", args[3]);
                std::process::exit(1);
            }
        }
        finger_name => {
            if args.len() < 4 {
                eprintln!("Error: Finger command requires an angle argument");
                eprintln!("Usage: {} <port> <finger> <angle>", args[0]);
                std::process::exit(1);
            }
            if let Some(&(servo_id, inverted)) = finger_map.get(finger_name) {
                if let Ok(angle) = args[3].parse::<f32>() {
                    let final_angle = if inverted { 180.0 - angle } else { angle };
                    controller.send_servo_command(servo_id, finger_name, final_angle)?;
                    println!("✓ Finger {} (servo {}) moved to {}° (sent: {}°)", 
                             finger_name, servo_id, angle, final_angle);
                } else {
                    eprintln!("Error: Invalid angle: {}", args[3]);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Unknown finger: {}", finger_name);
                eprintln!("Valid fingers: 1, 2, 3, 4, pinky, index, middle, ring");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "serial"))]
fn main() -> Result<()> {
    eprintln!("This program requires the 'serial' feature");
    eprintln!("Run with: cargo run --bin simple_control --features serial");
    Ok(())
}
