use robot_hand::{HandConfig, HandController};
use std::thread;
use std::time::Duration;

fn main() -> robot_hand::Result<()> {
    let config = HandConfig::from_file("config/default.toml")?;

    let mut hand = HandController::new(config)?;

    hand.initialize()?;

    println!("Opening hand...");
    hand.open_hand()?;
    thread::sleep(Duration::from_secs(2));

    println!("Grasping small object (20mm)...");
    hand.grasp(20.0)?;
    thread::sleep(Duration::from_secs(2));

    println!("Opening hand...");
    hand.open_hand()?;
    thread::sleep(Duration::from_secs(2));

    println!("Grasping medium object (50mm)...");
    hand.grasp(50.0)?;
    thread::sleep(Duration::from_secs(2));

    println!("Opening hand...");
    hand.open_hand()?;
    thread::sleep(Duration::from_secs(2));

    println!("Grasping large object (80mm)...");
    hand.grasp(80.0)?;
    thread::sleep(Duration::from_secs(2));

    println!("Opening hand...");
    hand.open_hand()?;

    hand.shutdown()?;
    println!("Complete!");

    Ok(())
}
