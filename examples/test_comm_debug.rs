use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("=== COM5 Communication Debug Tool ===\n");

    let control_port = "COM3";
    let comm_port = "COM5";

    println!("🔌 Port Configuration:");
    println!("  Control Port (Power): {}", control_port);
    println!("  Communication Port: {}\n", comm_port);

    // 打开电源控制
    println!("🔋 Opening control port...");
    let mut controller = match PowerController::connect(control_port, WireMode::SingleWire) {
        Ok(c) => {
            println!("  ✓ Control port opened");
            c
        }
        Err(e) => {
            eprintln!("  ❌ Failed to open control port: {}", e);
            return;
        }
    };

    // 打开设备电源
    println!("\n⚡ Powering ON Device1...");
    if let Err(e) = controller.power_on(DeviceSide::Device1) {
        eprintln!("  ❌ Failed to power on: {}", e);
        return;
    }
    println!("  ✓ Device1 powered ON");
    println!("  ⏱️  Waiting 2 seconds for device to boot...");
    std::thread::sleep(Duration::from_secs(2));

    // 尝试不同的波特率
    let baud_rates = vec![9600, 115200, 19200, 38400, 57600];

    for baud in baud_rates {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📡 Testing baud rate: {} bps", baud);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match test_communication(comm_port, baud) {
            Ok(received) => {
                if received > 0 {
                    println!("\n✅ SUCCESS at {} baud! Received {} bytes", baud, received);
                    break;
                }
            }
            Err(e) => {
                println!("  ❌ Error at {} baud: {}", baud, e);
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    // 关闭电源
    println!("\n⚡ Powering OFF Device1...");
    if let Err(e) = controller.power_off(DeviceSide::Device1) {
        eprintln!("  ❌ Failed to power off: {}", e);
    } else {
        println!("  ✓ Device1 powered OFF");
    }
}

fn test_communication(comm_port: &str, baud: u32) -> Result<usize, Box<dyn std::error::Error>> {
    // 打开通信端口
    println!("  → Opening port at {} baud...", baud);
    let mut comm = serialport::new(comm_port, baud)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()?;
    println!("    ✓ Port opened");

    // 清空接收缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 测试1: 只接收数据（不发送）
    println!("\n  📥 Test 1: Passive listening (2 seconds)...");
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total_received = 0;

    while start.elapsed() < Duration::from_secs(2) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_received += n;
                println!("    ✓ Received {} bytes: {:02X?}", n, &buffer[..n]);
                println!("       Text: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                println!("    ⚠️  Read error: {}", e);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if total_received > 0 {
        println!("    📊 Received {} bytes total", total_received);
    } else {
        println!("    ℹ️  No data received (device may not be auto-transmitting)");
    }

    // 测试2: 发送数据并等待响应
    println!("\n  📤 Test 2: Send and receive...");
    let test_commands = vec![
        b"AT\r\n".to_vec(),
        b"HELLO\n".to_vec(),
        b"\r\n".to_vec(),
        vec![0x00], // NULL byte
        vec![0xFF], // 0xFF
    ];

    for (i, cmd) in test_commands.iter().enumerate() {
        println!("    → Sending command {}: {:02X?}", i + 1, cmd);
        comm.write_all(cmd)?;
        comm.flush()?;

        std::thread::sleep(Duration::from_millis(300));

        let mut buf = [0u8; 256];
        match comm.read(&mut buf) {
            Ok(n) if n > 0 => {
                total_received += n;
                println!("      ✓ Response: {:02X?}", &buf[..n]);
                println!("         Text: {:?}", String::from_utf8_lossy(&buf[..n]));
            }
            _ => {
                println!("      ℹ️  No response");
            }
        }
    }

    // 测试3: 持续监听一段时间
    println!("\n  👂 Test 3: Extended listening (3 seconds)...");
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_received += n;
                println!("    ✓ Data: {:02X?}", &buffer[..n]);
                println!("       Text: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                println!("    ⚠️  Error: {}", e);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if total_received == 0 {
        println!("    ℹ️  No data received");
    }

    drop(comm);
    Ok(total_received)
}
