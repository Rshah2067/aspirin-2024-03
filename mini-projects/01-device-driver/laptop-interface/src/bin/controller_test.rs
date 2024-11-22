use laptop_interface::controller::ControllerManager;
use laptop_interface::controller::LedState;
use log::info;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting controller test.");

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
    manager.set_controller_led(controller_id, LedState::AllOn)?;

    manager.start_controller(controller_id)?;

    thread::sleep(Duration::from_millis(10));


    // Wait for some data from the controller and update its state
    // Wait for some data from the controller and update its state
    for _ in 0..50 {
        // Try for 5 seconds (50 * 100ms)
        manager.update_controller_state();
        if let Some(state) = manager.get_controller_state(controller_id) {
            log::debug!("Updated controller state: {:?}", state);
        }

        // Check if the controller thread has died
        if let Some(handle) = manager.join_handles.get_mut(&controller_id) {
            if let Some(thread) = handle.as_ref() {
                if thread.is_finished() {
                    log::error!("Controller thread has died unexpectedly");
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }

    if let Err(e) = manager.join_handles.get_mut(&controller_id).unwrap().take().unwrap().join() {
        log::error!("Failed to join controller thread: {:?}", e);
    }
    // Clean up
    manager.disconnect_controller(controller_id);

    log::info!("Controller test completed.");
    Ok(())
}
