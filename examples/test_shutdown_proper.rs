use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  DUT Proper Shutdown Test                ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("⚠️  Important: shutdown command requires power OFF");
    println!("   to complete properly (5V keeps device alive)\n");

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
    println!("║  Testing DUT1 Proper Shutdown (COM3)     ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_proper_shutdown(&mut controller, DeviceSide::Device1, "COM3", "DUT1");

    std::thread::sleep(Duration::from_secs(2));

    // 测试 DUT2
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Testing DUT2 Proper Shutdown (COM4)     ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_proper_shutdown(&mut controller, DeviceSide::Device2, "COM4", "DUT2");

    println!("\n✅ All shutdown tests completed!");
}

fn test_proper_shutdown(
    controller: &mut PowerController,
    side: DeviceSide,
    port: &str,
    name: &str,
) {
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

    // 3. 验证设备在线
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

    for _ in 0..10 {
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
        println!("  ⚠️  No response to init_status");
        return;
    }

    // 清空缓冲区
    while comm.read(&mut discard).is_ok() {}

    // 4. 发送 shutdown 命令
    println!("\n🔴 Step 4: Sending '[2700_shutdown,]' command...");
    let shutdown_cmd = b"[2700_shutdown,]";

    if let Err(e) = comm.write_all(shutdown_cmd) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }

    if let Err(e) = comm.flush() {
        eprintln!("  ❌ Flush failed: {}", e);
        return;
    }

    println!("  ✅ Shutdown command sent");

    // 5. 等待响应
    println!("\n📥 Step 5: Waiting for shutdown acknowledgment (3 seconds)...");
    let start = std::time::Instant::now();
    let mut total_received = 0;
    let mut response_buffer = Vec::new();

    while start.elapsed() < Duration::from_secs(3) {
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

    if total_received > 0 {
        println!("\n  ✅ Shutdown command acknowledged ({} bytes)", total_received);
        println!("     Response: {}", String::from_utf8_lossy(&response_buffer));
    } else {
        println!("\n  ℹ️  No acknowledgment received (device may shutdown silently)");
    }

    // 6. 关闭 5V 电源（关键步骤！）
    println!("\n⚡ Step 6: Turning OFF 5V power (required for shutdown)...");
    if let Err(e) = controller.power_off(side) {
        eprintln!("  ❌ Failed to power off: {}", e);
        return;
    }
    println!("  ✅ 5V power turned OFF");
    println!("  ⏱️  Waiting 2 seconds for device to fully shut down...");
    std::thread::sleep(Duration::from_secs(2));

    // 7. 重新开启5V电源（但设备应该保持关闭状态）
    println!("\n⚡ Step 7: Turning 5V power back ON (device should stay OFF)...");
    if let Err(e) = controller.power_on(side) {
        eprintln!("  ❌ Failed to power on: {}", e);
        return;
    }
    println!("  ✅ 5V power turned ON");
    println!("  ⏱️  Waiting 3 seconds to see if device auto-boots...");
    std::thread::sleep(Duration::from_secs(3));

    // 8. 验证设备确实关闭（不响应命令）
    println!("\n🔍 Step 8: Verifying device is truly shut down...");

    // 重新打开串口（之前的可能已关闭）
    let mut comm = match serialport::new(port, 9600)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ❌ Failed to reopen port: {}", e);
            return;
        }
    };

    // 清空缓冲区
    while comm.read(&mut discard).is_ok() {}

    // 尝试多次发送命令
    let mut device_responded = false;

    for attempt in 1..=3 {
        println!("  → Attempt {}/3: Sending init_status...", attempt);

        if let Err(e) = comm.write_all(init_cmd) {
            println!("    ⚠️  Write failed: {}", e);
            continue;
        }
        let _ = comm.flush();

        std::thread::sleep(Duration::from_millis(500));

        // 尝试读取响应
        let mut got_data = false;
        for _ in 0..10 {
            if let Ok(n) = comm.read(&mut buffer) {
                if n > 0 {
                    got_data = true;
                    device_responded = true;
                    println!("    ⚠️  Device responded! ({} bytes)", n);
                    println!("        Data: {:02X?}", &buffer[..n]);
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        if !got_data {
            println!("    ✓ No response");
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    // 9. 最终结果
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Shutdown Test Result for {}             ", name);
    println!("╚════════════════════════════════════════════╝");

    if device_responded {
        println!("❌ FAILED: Device is still responding after shutdown!");
        println!("   Possible issues:");
        println!("   • Shutdown command may not be working");
        println!("   • Device may have auto-booted");
        println!("   • Need different shutdown procedure");
    } else {
        println!("✅ SUCCESS: Device properly shut down!");
        println!("   • Shutdown command sent ✓");
        println!("   • 5V power cycled ✓");
        println!("   • Device not responding ✓");
        println!("\n   {} is truly shut down and will not auto-boot.", name);
    }

    // 10. 清理：最后关闭电源
    println!("\n🧹 Cleanup: Turning OFF 5V power...");
    let _ = controller.power_off(side);
    println!("✅ Cleanup complete");
}
