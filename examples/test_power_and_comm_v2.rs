use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("=== Prelude Power Controller Test v2 ===\n");

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

    // 使用 COM3 进行电源控制，COM5 进行数据通信
    let control_port = "COM3";
    let comm_port = "COM5";

    println!("\n🔌 Port Configuration:");
    println!("  Control Port (Power): {}", control_port);
    println!("  Communication Port: {}\n", comm_port);

    // 测试1: 电源控制功能
    println!("🔋 Test 1: Power Control ({})", control_port);
    println!("─────────────────────────────────────");

    match test_power_control(control_port) {
        Ok(_) => println!("✅ Power control test passed!\n"),
        Err(e) => {
            eprintln!("❌ Power control test failed: {}\n", e);
            return;
        }
    }

    // 测试2: 单线通信功能（使用独立的通信端口）
    println!("📡 Test 2: Single Wire Communication ({})", comm_port);
    println!("─────────────────────────────────────");

    match test_single_wire_comm(control_port, comm_port) {
        Ok(_) => println!("✅ Single wire communication test passed!\n"),
        Err(e) => {
            eprintln!("❌ Single wire communication test failed: {}\n", e);
        }
    }

    println!("=== All Tests Completed ===");
}

/// 测试电源控制功能（使用 COM3）
fn test_power_control(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Connecting to control port...");
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

/// 测试单线通信功能（使用独立的通信端口 COM5）
fn test_single_wire_comm(
    control_port: &str,
    comm_port: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Opening control port for power management...");
    let mut controller = PowerController::connect(control_port, WireMode::SingleWire)?;
    println!("  ✓ Control port opened");

    // 打开设备电源
    println!("\n  → Powering ON Device1...");
    controller.power_on(DeviceSide::Device1)?;
    std::thread::sleep(Duration::from_millis(1000));
    println!("    ✓ Device1 powered ON");

    // 打开通信端口
    println!("\n  → Opening communication port ({})...", comm_port);
    let mut comm = serialport::new(comm_port, 9600)
        .timeout(Duration::from_millis(1000))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()?;
    println!("    ✓ Communication port opened at 9600 baud");

    // 测试发送数据
    println!("\n  → Testing data transmission...");
    let test_data = b"HELLO\n";
    comm.write_all(test_data)?;
    comm.flush()?;
    println!("    ✓ Sent {} bytes: {}", test_data.len(), String::from_utf8_lossy(test_data).trim());

    // 等待一下让设备处理
    std::thread::sleep(Duration::from_millis(500));

    // 测试接收数据（循环读取）
    println!("\n  → Testing data reception...");
    println!("    Listening for 5 seconds...");
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total_received = 0;
    let mut received_chunks = Vec::new();

    while start.elapsed() < Duration::from_secs(5) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_received += n;
                let data = buffer[..n].to_vec();
                let text = String::from_utf8_lossy(&data);
                println!("    ✓ Received {} bytes: {:?}", n, text);
                received_chunks.push(data);
            }
            Ok(_) => {
                // No data yet
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is expected, continue listening
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                println!("    ⚠️  Read error: {}", e);
                break;
            }
        }
    }

    if total_received > 0 {
        println!("\n  📊 Reception Summary:");
        println!("    Total received: {} bytes", total_received);
        println!("    Number of chunks: {}", received_chunks.len());

        // 显示所有接收到的数据
        if !received_chunks.is_empty() {
            println!("\n    Complete data:");
            let all_data: Vec<u8> = received_chunks.into_iter().flatten().collect();
            println!("      Hex: {:02X?}", all_data);
            println!("      Text: {:?}", String::from_utf8_lossy(&all_data));
        }
    } else {
        println!("    ℹ️  No data received");
        println!("    This could mean:");
        println!("      - Device is not transmitting");
        println!("      - Device needs more time to boot");
        println!("      - Baud rate mismatch");
        println!("      - Wrong COM port selected");
    }

    // 尝试发送更多数据并接收回显（如果设备支持回显）
    println!("\n  → Testing echo/response...");
    for i in 1..=3 {
        let test_msg = format!("TEST{}\n", i);
        comm.write_all(test_msg.as_bytes())?;
        comm.flush()?;
        println!("    → Sent: {}", test_msg.trim());

        std::thread::sleep(Duration::from_millis(200));

        let mut buf = [0u8; 128];
        match comm.read(&mut buf) {
            Ok(n) if n > 0 => {
                println!("    ✓ Response: {:?}", String::from_utf8_lossy(&buf[..n]));
            }
            _ => {
                println!("    ℹ️  No response");
            }
        }
    }

    // 关闭电源
    println!("\n  → Powering OFF Device1...");
    controller.power_off(DeviceSide::Device1)?;
    println!("    ✓ Device1 powered OFF");

    Ok(())
}
