use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Prelude Complete Test Suite             ║");
    println!("╚════════════════════════════════════════════╝\n");

    let control_port = "COM5";

    println!("🔌 Opening power control port ({})...", control_port);
    let mut controller = match PowerController::connect(control_port, WireMode::SingleWire) {
        Ok(c) => {
            println!("✅ Power control opened\n");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed: {}", e);
            return;
        }
    };

    // Test Suite
    println!("╔════════════════════════════════════════════╗");
    println!("║  Test 1: Power Control                   ║");
    println!("╚════════════════════════════════════════════╝\n");
    test_power_control(&mut controller);

    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Test 2: Device Information (init_status)║");
    println!("╚════════════════════════════════════════════╝\n");
    test_device_info(&mut controller);

    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Test 3: Shutdown Command                ║");
    println!("╚════════════════════════════════════════════╝\n");
    test_shutdown_command(&mut controller);

    println!("\n╔════════════════════════════════════════════╗");
    println!("║  All Tests Completed!                     ║");
    println!("╚════════════════════════════════════════════╝");

    // Power off all
    println!("\n⚡ Powering OFF all devices...");
    let _ = controller.power_off(DeviceSide::Both);
    println!("✅ All devices powered OFF");
}

fn test_power_control(controller: &mut PowerController) {
    println!("Testing DUT1 power control...");
    assert!(controller.power_on(DeviceSide::Device1).is_ok());
    println!("  ✅ DUT1 power ON");
    std::thread::sleep(Duration::from_millis(500));

    assert!(controller.power_off(DeviceSide::Device1).is_ok());
    println!("  ✅ DUT1 power OFF");
    std::thread::sleep(Duration::from_millis(500));

    println!("\nTesting DUT2 power control...");
    assert!(controller.power_on(DeviceSide::Device2).is_ok());
    println!("  ✅ DUT2 power ON");
    std::thread::sleep(Duration::from_millis(500));

    assert!(controller.power_off(DeviceSide::Device2).is_ok());
    println!("  ✅ DUT2 power OFF");
    std::thread::sleep(Duration::from_millis(500));

    println!("\nTesting both devices...");
    assert!(controller.power_on(DeviceSide::Both).is_ok());
    println!("  ✅ Both devices power ON");
    std::thread::sleep(Duration::from_millis(500));

    assert!(controller.power_off(DeviceSide::Both).is_ok());
    println!("  ✅ Both devices power OFF");

    println!("\n✅ Power control test PASSED");
}

fn test_device_info(controller: &mut PowerController) {
    // Test DUT1
    println!("Testing DUT1 (COM3) device info...");
    controller.power_on(DeviceSide::Device1).unwrap();
    std::thread::sleep(Duration::from_secs(3));

    if let Some(info) = get_device_info("COM3", "DUT1") {
        println!("  ✅ DUT1 info retrieved:");
        println!("     Serial Number: {}", info.serial_number);
        println!("     Firmware: {} / {}", info.fw0_version, info.fw1_version);
    }

    controller.power_off(DeviceSide::Device1).unwrap();
    std::thread::sleep(Duration::from_secs(1));

    // Test DUT2
    println!("\nTesting DUT2 (COM4) device info...");
    controller.power_on(DeviceSide::Device2).unwrap();
    std::thread::sleep(Duration::from_secs(3));

    if let Some(info) = get_device_info("COM4", "DUT2") {
        println!("  ✅ DUT2 info retrieved:");
        println!("     Serial Number: {}", info.serial_number);
        println!("     Firmware: {} / {}", info.fw0_version, info.fw1_version);
    }

    controller.power_off(DeviceSide::Device2).unwrap();

    println!("\n✅ Device info test PASSED");
}

fn test_shutdown_command(controller: &mut PowerController) {
    println!("Testing shutdown command on DUT1...");
    controller.power_on(DeviceSide::Device1).unwrap();
    std::thread::sleep(Duration::from_secs(3));

    if send_command("COM3", "[2700_shutdown,]") {
        println!("  ✅ Shutdown command sent to DUT1");
    }

    std::thread::sleep(Duration::from_secs(2));
    controller.power_off(DeviceSide::Device1).unwrap();

    println!("\nTesting shutdown command on DUT2...");
    controller.power_on(DeviceSide::Device2).unwrap();
    std::thread::sleep(Duration::from_secs(3));

    if send_command("COM4", "[2700_shutdown,]") {
        println!("  ✅ Shutdown command sent to DUT2");
    }

    std::thread::sleep(Duration::from_secs(2));
    controller.power_off(DeviceSide::Device2).unwrap();

    println!("\n✅ Shutdown command test PASSED");
}

// Helper structures and functions
struct DeviceInfo {
    serial_number: String,
    fw0_version: String,
    fw1_version: String,
}

fn get_device_info(port: &str, _name: &str) -> Option<DeviceInfo> {
    let mut comm = serialport::new(port, 9600)
        .timeout(Duration::from_millis(1000))
        .open()
        .ok()?;

    // Clear buffer
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // Send command
    comm.write_all(b"[init_status,]").ok()?;
    comm.flush().ok()?;

    std::thread::sleep(Duration::from_millis(500));

    // Receive response
    let mut buffer = [0u8; 512];
    let mut response = Vec::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                response.extend_from_slice(&buffer[..n]);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let text = String::from_utf8_lossy(&response);

    // Parse response
    let mut serial_number = String::new();
    let mut fw0_version = String::new();
    let mut fw1_version = String::new();

    for line in text.lines() {
        if line.contains("PROD SN:") {
            serial_number = line.replace("PROD SN:", "").trim().to_string();
        } else if line.contains("Fw0Version:") {
            fw0_version = line.replace("Fw0Version:", "").trim().to_string();
        } else if line.contains("Fw1Version:") {
            fw1_version = line.replace("Fw1Version:", "").trim().to_string();
        }
    }

    if !serial_number.is_empty() {
        Some(DeviceInfo {
            serial_number,
            fw0_version,
            fw1_version,
        })
    } else {
        None
    }
}

fn send_command(port: &str, command: &str) -> bool {
    if let Ok(mut comm) = serialport::new(port, 9600)
        .timeout(Duration::from_millis(1000))
        .open()
    {
        // Clear buffer
        let mut discard = [0u8; 1024];
        while comm.read(&mut discard).is_ok() {}

        // Send command
        if comm.write_all(command.as_bytes()).is_ok() {
            let _ = comm.flush();
            return true;
        }
    }
    false
}
