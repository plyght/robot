use robot_hand::{HandConfig, HandController};

fn main() -> robot_hand::Result<()> {
    let config = HandConfig::from_file("config/default.toml")?;

    let mut hand = HandController::new(config)?;

    hand.initialize()?;

    println!("Moving index finger to [30, 45, 20] degrees");
    hand.move_finger(1, &[30.0, 45.0, 20.0])?;

    println!("Moving wrist to [10, 0, -5] degrees");
    hand.move_wrist([10.0, 0.0, -5.0])?;

    println!("Opening hand");
    hand.open_hand()?;

    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Closing hand");
    hand.close_hand()?;

    hand.shutdown()?;
    println!("Hand shutdown complete");

    Ok(())
}
