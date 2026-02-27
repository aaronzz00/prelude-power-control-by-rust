use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::Read;
use std::time::Duration;

fn main() {
    println!("=== Prelude Power Controller Test ===\n");

    // 列出可用的串口
    println!("📋 Available Serial Ports:");
    match serialport::available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                println!("  ⚠️  No serial ports found!");
                println!("\n❌ Please connect your FTDI device and try again.");
                return;
            }
            for port in &ports {
                println!("  - {}", port.port_name);
            }
        }
        Err(e) => {
            eprintln!("  ❌ Error listing ports: {}", e);
            return;
        }
    }

    // 提示用户输入串口名称（或使用第一个可用的）
    println!("\n🔌 Enter serial port name (or press Enter to use first available):");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let port_name = input.trim();

    let port_name = if port_name.is_empty() {
        let ports = serialport::available_ports().unwrap();
        ports[0].port_name.clone()
    } else {
        port_name.to_string()
    };

    println!("Using port: {}\n", port_name);

    // 测试1: 电源控制功能
    println!("🔋 Test 1: Power Control");
    println!("─────────────────────────────────────");

    match test_power_control(&port_name) {
        Ok(_) => println!("✅ Power control test passed!\n"),
        Err(e) => {
            eprintln!("❌ Power control test failed: {}\n", e);
            return;
        }
    }

    // 测试2: 单线通信功能
    println!("📡 Test 2: Single Wire Communication");
    println!("─────────────────────────────────────");

    match test_single_wire_comm(&port_name) {
        Ok(_) => println!("✅ Single wire communication test passed!\n"),
        Err(e) => {
            eprintln!("❌ Single wire communication test failed: {}\n", e);
        }
    }

    println!("=== All Tests Completed ===");
}

/// 测试电源控制功能
fn test_power_control(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Connecting to device in SingleWire mode (9600 baud)...");
    let mut controller = PowerController::connect(port_name, WireMode::SingleWire)?;
    println!("  ✓ Connected successfully");

    // 测试 Device1 电源控制
    println!("\n  Testing Device1:");
    println!("    → Turning Device1 ON...");
    controller.power_on(DeviceSide::Device1)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ Device1 powered ON");

    println!("    → Turning Device1 OFF...");
    controller.power_off(DeviceSide::Device1)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ Device1 powered OFF");

    // 测试 Device2 电源控制
    println!("\n  Testing Device2:");
    println!("    → Turning Device2 ON...");
    controller.power_on(DeviceSide::Device2)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ Device2 powered ON");

    println!("    → Turning Device2 OFF...");
    controller.power_off(DeviceSide::Device2)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ Device2 powered OFF");

    // 测试同时控制两个设备
    println!("\n  Testing Both Devices:");
    println!("    → Turning Both ON...");
    controller.power_on(DeviceSide::Both)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ Both devices powered ON");

    println!("    → Turning Both OFF...");
    controller.power_off(DeviceSide::Both)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ Both devices powered OFF");

    // 测试充电器控制
    println!("\n  Testing VCHARGER:");
    println!("    → Enabling VCHARGER for Device1...");
    controller.enable_vcharger(DeviceSide::Device1)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ VCHARGER enabled");

    println!("    → Disabling VCHARGER for Device1...");
    controller.disable_vcharger(DeviceSide::Device1)?;
    std::thread::sleep(Duration::from_millis(500));
    println!("    ✓ VCHARGER disabled");

    // 测试复位功能
    println!("\n  Testing RESET:");
    println!("    → Resetting Device1...");
    controller.reset(DeviceSide::Device1)?;
    println!("    ✓ Device1 reset completed (100ms pulse)");

    Ok(())
}

/// 测试单线通信功能
fn test_single_wire_comm(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Connecting to device in SingleWire mode (9600 baud)...");
    let mut controller = PowerController::connect(port_name, WireMode::SingleWire)?;
    println!("  ✓ Connected successfully");

    // 打开设备电源
    println!("\n  → Powering ON Device1 for communication test...");
    controller.power_on(DeviceSide::Device1)?;
    std::thread::sleep(Duration::from_millis(1000));

    // 测试发送数据
    println!("  → Testing data transmission...");
    let test_data = b"TEST\n";
    controller
        .port_mut()
        .write_all(test_data)
        .map_err(|e| format!("Write error: {}", e))?;
    println!("    ✓ Sent {} bytes: {:?}", test_data.len(), test_data);

    // 测试接收数据
    println!("\n  → Testing data reception...");
    println!("    Listening for 3 seconds...");
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total_received = 0;

    while start.elapsed() < Duration::from_secs(3) {
        match controller.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_received += n;
                println!(
                    "    ✓ Received {} bytes: {:?}",
                    n,
                    String::from_utf8_lossy(&buffer[..n])
                );
            }
            Ok(_) => {
                // No data, continue
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is expected
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("    ⚠️  Read error: {}", e);
                break;
            }
        }
    }

    if total_received > 0 {
        println!("    ✓ Total received: {} bytes", total_received);
    } else {
        println!("    ℹ️  No data received (device may not be transmitting)");
    }

    // 关闭电源
    println!("\n  → Powering OFF Device1...");
    controller.power_off(DeviceSide::Device1)?;

    Ok(())
}
