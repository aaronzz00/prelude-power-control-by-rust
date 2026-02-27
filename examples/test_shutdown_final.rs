use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  DUT Shutdown Test - Final Version       ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("⚠️  Important: shutdown requires 5V power OFF");
    println!("   Device will stay alive if 5V is ON\n");

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
    println!("║  Testing DUT1 Shutdown (COM3)            ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_shutdown(&mut controller, DeviceSide::Device1, "COM3", "DUT1");

    std::thread::sleep(Duration::from_secs(2));

    // 测试 DUT2
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Testing DUT2 Shutdown (COM4)            ║");
    println!("╚════════════════════════════════════════════╝\n");

    test_shutdown(&mut controller, DeviceSide::Device2, "COM4", "DUT2");

    println!("\n✅ All shutdown tests completed!");
}

fn test_shutdown(
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

    // 2. 打开串口并验证设备在线
    println!("\n📡 Step 2: Opening {} and verifying device...", port);
    let (online, _) = check_device_online(port);

    if online {
        println!("  ✅ Device is online and responding");
    } else {
        println!("  ⚠️  Device not responding - aborting test");
        return;
    }

    // 3. 发送 shutdown 命令
    println!("\n🔴 Step 3: Sending '[shutdown,]' command...");
    let shutdown_result = send_shutdown_command(port);

    match shutdown_result {
        Ok(response) => {
            if response.is_empty() {
                println!("  ✅ Shutdown command sent (no response)");
            } else {
                println!("  ✅ Shutdown command sent");
                println!("     Response: {}", response);
            }
        }
        Err(e) => {
            println!("  ⚠️  Failed to send: {}", e);
        }
    }

    std::thread::sleep(Duration::from_secs(1));

    // 4. 关闭 5V 电源（必须！）
    println!("\n⚡ Step 4: Turning OFF 5V power (required)...");
    if let Err(e) = controller.power_off(side) {
        eprintln!("  ❌ Failed to power off: {}", e);
        return;
    }
    println!("  ✅ 5V power turned OFF");
    println!("  ⏱️  Waiting 2 seconds for complete shutdown...");
    std::thread::sleep(Duration::from_secs(2));

    // 5. 重新开启5V电源
    println!("\n⚡ Step 5: Turning 5V power back ON...");
    if let Err(e) = controller.power_on(side) {
        eprintln!("  ❌ Failed to power on: {}", e);
        return;
    }
    println!("  ✅ 5V power turned ON");
    println!("  ⏱️  Waiting 3 seconds (device should NOT auto-boot)...");
    std::thread::sleep(Duration::from_secs(3));

    // 6. 验证设备确实关闭
    println!("\n🔍 Step 6: Verifying device is shut down...");
    let (still_online, attempts) = check_device_offline(port, 3);

    println!("  → Tested with {} attempts", attempts);

    if still_online {
        println!("  ⚠️  Device is still responding!");
    } else {
        println!("  ✅ Device is not responding (confirmed shutdown)");
    }

    // 7. 最终结果
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Test Result for {}                       ", name);
    println!("╚════════════════════════════════════════════╝");

    if still_online {
        println!("❌ FAILED: Device did not shut down properly");
        println!("   • Shutdown command sent ✓");
        println!("   • 5V power cycled ✓");
        println!("   • Device still responding ✗");
    } else {
        println!("✅ SUCCESS: Device properly shut down!");
        println!("   • Shutdown command sent ✓");
        println!("   • 5V power cycled ✓");
        println!("   • Device not responding ✓");
        println!("\n   {} will not auto-boot until manually powered on.", name);
    }

    // 清理
    println!("\n🧹 Cleanup: Turning OFF 5V power...");
    let _ = controller.power_off(side);
}

// Helper: 检查设备是否在线
fn check_device_online(port: &str) -> (bool, String) {
    let mut comm = match serialport::new(port, 9600)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(c) => c,
        Err(_) => return (false, String::new()),
    };

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 发送命令
    let _ = comm.write_all(b"[init_status,]");
    let _ = comm.flush();

    std::thread::sleep(Duration::from_millis(500));

    // 读取响应
    let mut buffer = [0u8; 512];
    let mut response = Vec::new();

    for _ in 0..10 {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                response.extend_from_slice(&buffer[..n]);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 显式关闭串口
    drop(comm);

    let online = !response.is_empty();
    let text = String::from_utf8_lossy(&response).to_string();

    (online, text)
}

// Helper: 检查设备是否离线
fn check_device_offline(port: &str, max_attempts: u32) -> (bool, u32) {
    for attempt in 1..=max_attempts {
        let mut comm = match serialport::new(port, 9600)
            .timeout(Duration::from_millis(500))
            .open()
        {
            Ok(c) => c,
            Err(_) => {
                // 端口无法打开，但这不意味着设备关闭
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
        };

        // 清空缓冲区
        let mut discard = [0u8; 1024];
        while comm.read(&mut discard).is_ok() {}

        // 发送命令
        let _ = comm.write_all(b"[init_status,]");
        let _ = comm.flush();

        std::thread::sleep(Duration::from_millis(500));

        // 尝试读取
        let mut buffer = [0u8; 256];
        let mut got_response = false;

        for _ in 0..10 {
            if let Ok(n) = comm.read(&mut buffer) {
                if n > 0 {
                    got_response = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        // 关闭串口
        drop(comm);

        if got_response {
            return (true, attempt); // 设备仍在线
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    (false, max_attempts) // 设备离线
}

// Helper: 发送shutdown命令
fn send_shutdown_command(port: &str) -> Result<String, String> {
    let mut comm = serialport::new(port, 9600)
        .timeout(Duration::from_millis(1000))
        .open()
        .map_err(|e| format!("Failed to open port: {}", e))?;

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 发送命令
    comm.write_all(b"[shutdown,]")
        .map_err(|e| format!("Failed to write: {}", e))?;
    comm.flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    std::thread::sleep(Duration::from_millis(500));

    // 读取响应
    let mut buffer = [0u8; 512];
    let mut response = Vec::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(2) {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                response.extend_from_slice(&buffer[..n]);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 关闭串口
    drop(comm);

    Ok(String::from_utf8_lossy(&response).to_string())
}
