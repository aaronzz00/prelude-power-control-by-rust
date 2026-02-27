use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  DUT Init Status Test - Single Wire      ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("📋 Configuration:");
    println!("  Power Control: COM5");
    println!("  DUT1: COM3");
    println!("  DUT2: COM4");
    println!("  Command: [init_status,]\n");

    let control_port = "COM5";

    // 打开电源控制
    println!("🔌 Opening power control port...");
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

    // 测试 DUT1
    println!("╔════════════════════════════════════════════╗");
    println!("║  Testing DUT1 (COM3)                      ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_dut(&mut controller, DeviceSide::Device1, "COM3", "DUT1");

    // 测试 DUT2
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Testing DUT2 (COM4)                      ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_dut(&mut controller, DeviceSide::Device2, "COM4", "DUT2");

    // 关闭所有设备
    println!("\n⚡ Powering OFF all devices...");
    let _ = controller.power_off(DeviceSide::Both);
    println!("✅ All devices powered OFF");
}

fn test_dut(controller: &mut PowerController, side: DeviceSide, port: &str, name: &str) {
    // 1. 上电
    println!("⚡ Step 1: Power ON {}...", name);
    if let Err(e) = controller.power_on(side) {
        eprintln!("❌ Failed to power on: {}", e);
        return;
    }
    println!("✅ {} powered ON", name);
    println!("⏱️  Waiting 3 seconds for boot...");
    std::thread::sleep(Duration::from_secs(3));

    // 2. 打开串口
    println!("\n📡 Step 2: Opening {} at 9600 baud...", port);
    let mut comm = match serialport::new(port, 9600)
        .timeout(Duration::from_millis(1000))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => {
            println!("✅ {} opened successfully", port);
            c
        }
        Err(e) => {
            eprintln!("❌ Failed to open {}: {}", port, e);
            return;
        }
    };

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 3. 先监听一下是否有自动发送的数据
    println!("\n👂 Step 3: Passive listening (2 seconds)...");
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 512];
    let mut had_data = false;

    while start.elapsed() < Duration::from_secs(2) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                had_data = true;
                println!("  ✅ Received {} bytes: {:02X?}", n, &buffer[..n]);
                let text = String::from_utf8_lossy(&buffer[..n]);
                println!("     Text: {:?}", text);
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if !had_data {
        println!("  ℹ️  No spontaneous data");
    }

    // 4. 发送 init_status 命令
    println!("\n📤 Step 4: Sending '[init_status,]' command...");
    let command = b"[init_status,]";

    if let Err(e) = comm.write_all(command) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }

    if let Err(e) = comm.flush() {
        eprintln!("  ❌ Flush failed: {}", e);
        return;
    }

    println!("  ✅ Command sent: {:?}", String::from_utf8_lossy(command));
    println!("     Hex: {:02X?}", command);

    // 5. 等待并接收响应
    println!("\n📥 Step 5: Waiting for response (10 seconds)...\n");
    let start = std::time::Instant::now();
    let mut total_received = 0;
    let mut response_buffer = Vec::new();

    while start.elapsed() < Duration::from_secs(10) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_received += n;
                response_buffer.extend_from_slice(&buffer[..n]);

                println!("  ✅ [{:5.2}s] Received {} bytes:",
                    start.elapsed().as_secs_f32(), n);
                println!("     Hex: {:02X?}", &buffer[..n]);

                let text = String::from_utf8_lossy(&buffer[..n]);
                println!("     Text: {:?}", text);

                // 如果收到完整的响应（包含结束符），可以提前退出
                let full_text = String::from_utf8_lossy(&response_buffer);
                if full_text.contains(']') || full_text.contains('\n') {
                    println!("\n  💡 Detected potential end of response");
                }
            }
            Ok(_) => {
                // 没有数据，继续等待
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // 超时是正常的
            }
            Err(e) => {
                eprintln!("  ⚠️  Read error: {}", e);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 6. 显示完整响应
    if total_received > 0 {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║  Response Summary for {}                  ", name);
        println!("╚════════════════════════════════════════════╝");
        println!("Total received: {} bytes", total_received);
        println!("\nComplete response:");
        println!("  Hex: {:02X?}", response_buffer);
        println!("\n  Text:\n{}", String::from_utf8_lossy(&response_buffer));
        println!("\n✅ SUCCESS! {} is responding!", name);
    } else {
        println!("\n❌ No response received from {}", name);
        println!("\n💡 Possible issues:");
        println!("   • Device may need more boot time");
        println!("   • Command format might be incorrect");
        println!("   • Baud rate might be wrong (try 115200)");
        println!("   • Device might need different command");
    }

    // 7. 关闭电源
    println!("\n⚡ Powering OFF {}...", name);
    if let Err(e) = controller.power_off(side) {
        eprintln!("❌ Failed to power off: {}", e);
    } else {
        println!("✅ {} powered OFF", name);
    }
}
