use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("=== Interactive Communication Test ===\n");

    // 列出所有串口
    println!("📋 Available Serial Ports:");
    let ports = match serialport::available_ports() {
        Ok(ports) => {
            for (i, port) in ports.iter().enumerate() {
                println!("  {}. {}", i + 1, port.port_name);
            }
            ports
        }
        Err(e) => {
            eprintln!("❌ Error listing ports: {}", e);
            return;
        }
    };

    if ports.is_empty() {
        println!("❌ No ports found!");
        return;
    }

    // 选择控制端口（默认 COM3）
    println!("\n🔌 Select control port (for power) [default: COM3]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let control_port = if input.trim().is_empty() {
        "COM3".to_string()
    } else {
        input.trim().to_string()
    };

    // 选择通信端口（默认 COM5）
    println!("🔌 Select communication port [default: COM5, or type 'COM6' to try COM6]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let comm_port = if input.trim().is_empty() {
        "COM5".to_string()
    } else {
        input.trim().to_string()
    };

    // 选择波特率
    println!("⚡ Select baud rate [default: 9600]:");
    println!("  1. 9600");
    println!("  2. 115200");
    println!("  3. 19200");
    println!("  4. 38400");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let baud = match input.trim() {
        "2" => 115200,
        "3" => 19200,
        "4" => 38400,
        _ => 9600,
    };

    println!("\n📝 Configuration:");
    println!("  Control Port: {}", control_port);
    println!("  Communication Port: {}", comm_port);
    println!("  Baud Rate: {} bps\n", baud);

    // 打开控制端口
    println!("🔋 Opening control port...");
    let mut controller = match PowerController::connect(&control_port, WireMode::SingleWire) {
        Ok(c) => {
            println!("  ✓ Control port opened");
            c
        }
        Err(e) => {
            eprintln!("  ❌ Failed: {}", e);
            return;
        }
    };

    // 菜单循环
    loop {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎛️  Menu:");
        println!("  1. Power ON Device1");
        println!("  2. Power OFF Device1");
        println!("  3. Reset Device1");
        println!("  4. Test Communication");
        println!("  5. Monitor Communication (continuous)");
        println!("  6. Send Custom Data");
        println!("  0. Exit");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        print!("Select option: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                println!("⚡ Powering ON Device1...");
                if let Err(e) = controller.power_on(DeviceSide::Device1) {
                    eprintln!("  ❌ Error: {}", e);
                } else {
                    println!("  ✓ Device1 powered ON");
                }
            }
            "2" => {
                println!("⚡ Powering OFF Device1...");
                if let Err(e) = controller.power_off(DeviceSide::Device1) {
                    eprintln!("  ❌ Error: {}", e);
                } else {
                    println!("  ✓ Device1 powered OFF");
                }
            }
            "3" => {
                println!("🔄 Resetting Device1...");
                if let Err(e) = controller.reset(DeviceSide::Device1) {
                    eprintln!("  ❌ Error: {}", e);
                } else {
                    println!("  ✓ Device1 reset completed");
                    println!("  ⏱️  Waiting 2 seconds for boot...");
                    std::thread::sleep(Duration::from_secs(2));
                }
            }
            "4" => {
                test_communication(&comm_port, baud);
            }
            "5" => {
                monitor_communication(&comm_port, baud);
            }
            "6" => {
                send_custom_data(&comm_port, baud);
            }
            "0" => {
                println!("👋 Exiting...");
                let _ = controller.power_off(DeviceSide::Device1);
                break;
            }
            _ => {
                println!("❌ Invalid option");
            }
        }
    }
}

fn test_communication(comm_port: &str, baud: u32) {
    println!("\n📡 Testing communication on {} at {} baud...", comm_port, baud);

    let mut comm = match serialport::new(comm_port, baud)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ❌ Failed to open port: {}", e);
            return;
        }
    };

    println!("  ✓ Port opened");

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 监听数据
    println!("\n  📥 Listening for 3 seconds...");
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total = 0;

    while start.elapsed() < Duration::from_secs(3) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total += n;
                println!("    ✓ Received {} bytes: {:02X?}", n, &buffer[..n]);
                println!("       ASCII: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if total == 0 {
        println!("    ℹ️  No data received");
    } else {
        println!("\n  📊 Total: {} bytes", total);
    }

    // 发送测试数据
    println!("\n  📤 Sending test data...");
    let tests = vec![
        ("AT\\r\\n", b"AT\r\n".to_vec()),
        ("HELLO\\n", b"HELLO\n".to_vec()),
        ("\\n", b"\n".to_vec()),
    ];

    for (name, data) in tests {
        println!("    → Sending: {}", name);
        if let Err(e) = comm.write_all(&data) {
            println!("      ❌ Write error: {}", e);
            continue;
        }
        let _ = comm.flush();

        std::thread::sleep(Duration::from_millis(300));

        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                println!("      ✓ Response: {:02X?}", &buffer[..n]);
                println!("         ASCII: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            _ => {
                println!("      ℹ️  No response");
            }
        }
    }
}

fn monitor_communication(comm_port: &str, baud: u32) {
    println!("\n📡 Monitoring {} at {} baud...", comm_port, baud);
    println!("  Press Ctrl+C to stop (or wait 30 seconds)\n");

    let mut comm = match serialport::new(comm_port, baud)
        .timeout(Duration::from_millis(100))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ❌ Failed to open port: {}", e);
            return;
        }
    };

    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total = 0;

    while start.elapsed() < Duration::from_secs(30) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total += n;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                println!(
                    "[{:6.3}s] Received {} bytes: {:02X?}",
                    start.elapsed().as_secs_f32(),
                    n,
                    &buffer[..n]
                );
                println!("           ASCII: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    println!("\n📊 Total received: {} bytes", total);
}

fn send_custom_data(comm_port: &str, baud: u32) {
    println!("\n📤 Send custom data to {} at {} baud", comm_port, baud);

    let mut comm = match serialport::new(comm_port, baud)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ❌ Failed to open port: {}", e);
            return;
        }
    };

    print!("Enter data to send (will append \\n): ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let data = format!("{}\n", input.trim());
    println!("  → Sending: {:?}", data);

    if let Err(e) = comm.write_all(data.as_bytes()) {
        eprintln!("  ❌ Write error: {}", e);
        return;
    }
    let _ = comm.flush();

    println!("  ✓ Data sent");
    println!("\n  📥 Listening for response (3 seconds)...");

    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];

    while start.elapsed() < Duration::from_secs(3) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                println!("    ✓ Response: {:02X?}", &buffer[..n]);
                println!("       ASCII: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
