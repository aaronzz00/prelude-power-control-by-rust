use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // macOS FTDI first port is usually ending in '0' or 'A', matching "FT4232H_Orka Prelude A"
    let port_name = "/dev/cu.usbserial-FT66ORKA0";

    println!("Connecting to {}...", port_name);

    // Connect using default mode (SingleWire)
    let mut controller = PowerController::connect(port_name, WireMode::SingleWire)?;
    println!("Connected successfully!");

    println!("--------------------------------");
    println!("Test: Power ON Device 1...");
    controller.power_on(DeviceSide::Device1)?;
    sleep(Duration::from_secs(2));

    println!("Test: Enable VCHARGER for Device 1...");
    controller.enable_vcharger(DeviceSide::Device1)?;
    sleep(Duration::from_secs(2));

    println!("Test: Power OFF Device 1...");
    controller.disable_vcharger(DeviceSide::Device1)?;
    controller.power_off(DeviceSide::Device1)?;
    sleep(Duration::from_secs(1));

    println!("--------------------------------");
    println!("Test: Power ON Device 2...");
    controller.power_on(DeviceSide::Device2)?;
    sleep(Duration::from_secs(2));

    println!("Test: Power OFF Device 2...");
    controller.power_off(DeviceSide::Device2)?;
    sleep(Duration::from_secs(1));

    println!("--------------------------------");
    println!("Test: Resetting both devices...");
    controller.reset(DeviceSide::Both)?;
    println!("Reset pulse sent.");

    println!("All tests completed.");
    Ok(())
}
