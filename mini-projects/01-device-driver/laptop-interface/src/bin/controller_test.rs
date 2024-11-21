use laptop_interface::controller::ControllerManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    log::info!("Starting controller test");

    // Create a new ControllerManager
    let mut manager = ControllerManager::new();

    // Specify the actual serial port where your controller is connected
    let real_port = "/dev/cu.usbmodem101"; // Example for macOS

    // Connect to the real controller
    manager.connect_controller(real_port)?;

    // Give more time for the connection to establish
    thread::sleep(Duration::from_millis(1000));

    // Check if the controller was added
    log::info!("Connected controllers: {:?}", manager.get_controller_ids());
    if manager.get_controller_ids().len() != 1 {
        return Err("Controller was not added".into());
    }

    // Get the controller ID
    let controller_id = manager.get_controller_ids()[0];

    // Switch the controller into run mode
    manager.set_controller_led(controller_id, laptop_interface::controller::LedState::AllOn)?;

    thread::sleep(Duration::from_millis(10));

    manager.start_controller(controller_id)?;

    // Give some time for the commands to take effect
    thread::sleep(Duration::from_millis(1000));

    // Wait for some data from the controller and update its state
    let mut state_updated = false;
    for _ in 0..50 {
        // Try for 5 seconds (50 * 100ms)
        manager.update_controller_state();
        if let Some(state) = manager.get_controller_state(controller_id) {
            log::info!("Updated controller state: {:?}", state);
            state_updated = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    if !state_updated {
        return Err("Controller state not updated after 5 seconds".into());
    }

    // Verify that we can get the controller state
    let final_state = manager.get_controller_state(controller_id);
    if final_state.is_none() {
        return Err("Failed to get controller state".into());
    }
    log::info!("Final controller state: {:?}", final_state.unwrap());

    // Clean up
    manager.disconnect_controller(controller_id);

    log::info!("Controller test completed successfully");
    Ok(())
}
