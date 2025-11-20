use robot_hand::ServoProtocol;
use std::time::Duration;

#[cfg(feature = "serial")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let port = args
        .nth(1)
        .unwrap_or_else(|| "/dev/cu.usbmodem1101".to_string());

    println!("Connecting to: {}", port);
    let mut controller = robot_hand::TextSerialController::new(&port, 115200)?;

    println!("Sending test command: servo3 test 90");
    controller.send_servo_command(3, "test", 90.0)?;

    std::thread::sleep(Duration::from_millis(100));

    println!("Reading response...");
    let mut buffer = [0u8; 64];

    #[cfg(feature = "serial")]
    {
        use std::io::Read;
        if let Ok(n) = std::io::stdin().read(&mut buffer) {
            if n > 0 {
                let response = String::from_utf8_lossy(&buffer[..n]);
                println!("Received: {}", response);
            } else {
                println!("No response received");
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "serial"))]
fn main() {
    eprintln!("Requires serial feature");
}
