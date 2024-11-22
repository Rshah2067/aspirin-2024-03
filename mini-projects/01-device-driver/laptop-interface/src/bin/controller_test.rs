use laptop_interface::controller::ControllerManager;
use laptop_interface::controller::LedState;
use log::info;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    info!("Starting controller test.");

    // Create a new ControllerManager
    let mut manager = ControllerManager::new();

    // Specify the actual serial port where your controller is connected
    let real_port = "/dev/cu.usbmodem2101"; // Example for macOS

    // Connect to the real controller
    let id = manager
        .connect_controller(real_port)
        .expect("Failed to connect to controller");

    // Give more time for the connection to establish
    thread::sleep(Duration::from_millis(100));

    // Check if the controller was added
    log::info!("Connected controllers: {:?}", manager.get_controller_ids());

    assert_eq!(
        *manager.get_controller_ids().last().unwrap(),
        id,
        "Controller was not added"
    );

    manager
        .set_controller_led(id, LedState::AllOn)
        .expect("Failed to set controller LED");

    manager
        .start_controller(id)
        .expect("Failed to start controller");

    thread::sleep(Duration::from_millis(10));

    // Wait for some data from the controller and update its state
    for _ in 0..50 {
        // Try for 5 seconds (50 * 100ms)
        manager.update_controller_state();
        if let Some(state) = manager.get_controller_state(id) {
            log::debug!("Updated controller state: {:?}", state);
        }

        // Check if the controller thread has died
        if let Some(handle) = manager.join_handles.get_mut(&id) {
            if let Some(thread) = handle.as_ref() {
                if thread.is_finished() {
                    log::error!("Controller thread has died unexpectedly");
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    manager.disconnect_controller(id);

    if let Err(e) = manager
        .join_handles
        .get_mut(&id)
        .unwrap()
        .take()
        .unwrap()
        .join()
    {
        log::error!("Failed to join controller thread: {:?}", e);
    }

    log::info!("Controller test completed.");
}
