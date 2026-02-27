use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  DUT Shutdown Command Test                ║");
    println!("╚════════════════════════════════════════════╝\n");

    let control_port = "COM5";

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
    println!("║  Testing DUT1 Shutdown (COM3)             ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_shutdown(&mut controller, DeviceSide::Device1, "COM3", "DUT1");

    std::thread::sleep(Duration::from_secs(2));

    // 测试 DUT2
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Testing DUT2 Shutdown (COM4)             ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_shutdown(&mut controller, DeviceSide::Device2, "COM4", "DUT2");

    println!("\n✅ All tests completed!");
}

fn test_shutdown(controller: &mut PowerController, side: DeviceSide, port: &str, name: &str) {
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

    // 3. 先发送 init_status 确认设备在线
    println!("\n📋 Step 3: Verifying device is online...");
    let init_cmd = b"[init_status,]";

    if let Err(e) = comm.write_all(init_cmd) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }
    let _ = comm.flush();

    std::thread::sleep(Duration::from_millis(500));

    let mut buffer = [0u8; 512];
    let mut got_response = false;

    for _ in 0..5 {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                got_response = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    if got_response {
        println!("  ✅ Device is online and responding");
    } else {
        println!("  ⚠️  No response to init_status, continuing anyway...");
    }

    // 清空缓冲区
    while comm.read(&mut discard).is_ok() {}

    // 4. 发送 shutdown 命令
    println!("\n🔴 Step 4: Sending '[shutdown,]' command...");
    let shutdown_cmd = b"[shutdown,]";

    if let Err(e) = comm.write_all(shutdown_cmd) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }

    if let Err(e) = comm.flush() {
        eprintln!("  ❌ Flush failed: {}", e);
        return;
    }

    println!("  ✅ Shutdown command sent: {:?}", String::from_utf8_lossy(shutdown_cmd));
    println!("     Hex: {:02X?}", shutdown_cmd);

    // 5. 等待并接收响应
    println!("\n📥 Step 5: Waiting for response (5 seconds)...\n");
    let start = std::time::Instant::now();
    let mut total_received = 0;
    let mut response_buffer = Vec::new();

    while start.elapsed() < Duration::from_secs(5) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_received += n;
                response_buffer.extend_from_slice(&buffer[..n]);

                println!("  ✅ [{:5.2}s] Received {} bytes:",
                    start.elapsed().as_secs_f32(), n);
                println!("     Hex: {:02X?}", &buffer[..n]);

                let text = String::from_utf8_lossy(&buffer[..n]);
                println!("     Text: {:?}", text);
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 6. 显示结果
    if total_received > 0 {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║  Shutdown Response for {}                 ", name);
        println!("╚════════════════════════════════════════════╝");
        println!("Total received: {} bytes", total_received);
        println!("\nComplete response:");
        println!("  Hex: {:02X?}", response_buffer);
        println!("\n  Text:\n{}", String::from_utf8_lossy(&response_buffer));
        println!("\n✅ {} acknowledged shutdown command!", name);
    } else {
        println!("\n  ℹ️  No response received");
        println!("  💡 Device may have shut down immediately without response");
    }

    // 7. 验证设备是否关闭
    println!("\n🔍 Step 6: Verifying device shutdown...");
    std::thread::sleep(Duration::from_secs(2));

    // 尝试发送init_status看设备是否还在线
    if let Err(e) = comm.write_all(init_cmd) {
        eprintln!("  ❌ Write failed: {}", e);
    } else {
        let _ = comm.flush();
        std::thread::sleep(Duration::from_millis(500));

        let mut still_online = false;
        for _ in 0..5 {
            if let Ok(n) = comm.read(&mut buffer) {
                if n > 0 {
                    still_online = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if still_online {
            println!("  ⚠️  Device is still responding - shutdown may not have completed");
        } else {
            println!("  ✅ Device is not responding - shutdown successful!");
        }
    }

    println!("\n⚡ Note: Physical power is still ON (COM5)");
    println!("   Device has shut down via software command.");
}
