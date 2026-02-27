use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Prelude Dual Device Test - Full Suite   ║");
    println!("╚════════════════════════════════════════════╝\n");

    let control_port = "COM3";
    let comm_port_dut1 = "COM5";
    let comm_port_dut2 = "COM6";

    println!("📋 Configuration:");
    println!("  Control Port: {}", control_port);
    println!("  DUT1 Communication: {}", comm_port_dut1);
    println!("  DUT2 Communication: {}\n", comm_port_dut2);

    // Phase 1: 电源控制测试
    println!("╔════════════════════════════════════════════╗");
    println!("║  Phase 1: Power Control Test             ║");
    println!("╚════════════════════════════════════════════╝\n");

    let mut controller = match PowerController::connect(control_port, WireMode::SingleWire) {
        Ok(c) => {
            println!("✅ Control port opened successfully\n");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed to open control port: {}", e);
            return;
        }
    };

    // 测试 DUT1 电源控制
    println!("🔋 Testing DUT1 Power Control");
    println!("─────────────────────────────────────");
    test_power_sequence(&mut controller, DeviceSide::Device1, "DUT1");

    println!("\n🔋 Testing DUT2 Power Control");
    println!("─────────────────────────────────────");
    test_power_sequence(&mut controller, DeviceSide::Device2, "DUT2");

    // Phase 2: DUT1 通信测试
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Phase 2: DUT1 Communication Test        ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("⚡ Powering ON DUT1...");
    controller.power_on(DeviceSide::Device1).unwrap();
    println!("✅ DUT1 powered ON");
    println!("⏱️  Waiting 2 seconds for boot...\n");
    std::thread::sleep(Duration::from_secs(2));

    test_communication(&mut controller, comm_port_dut1, "DUT1");

    println!("\n⚡ Powering OFF DUT1...");
    controller.power_off(DeviceSide::Device1).unwrap();
    println!("✅ DUT1 powered OFF\n");

    // Phase 3: DUT2 通信测试
    println!("╔════════════════════════════════════════════╗");
    println!("║  Phase 3: DUT2 Communication Test        ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("⚡ Powering ON DUT2...");
    controller.power_on(DeviceSide::Device2).unwrap();
    println!("✅ DUT2 powered ON");
    println!("⏱️  Waiting 2 seconds for boot...\n");
    std::thread::sleep(Duration::from_secs(2));

    test_communication(&mut controller, comm_port_dut2, "DUT2");

    println!("\n⚡ Powering OFF DUT2...");
    controller.power_off(DeviceSide::Device2).unwrap();
    println!("✅ DUT2 powered OFF\n");

    // Phase 4: 双设备同时测试
    println!("╔════════════════════════════════════════════╗");
    println!("║  Phase 4: Dual Device Simultaneous Test  ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("⚡ Powering ON both devices...");
    controller.power_on(DeviceSide::Both).unwrap();
    println!("✅ Both devices powered ON");
    println!("⏱️  Waiting 2 seconds for boot...\n");
    std::thread::sleep(Duration::from_secs(2));

    println!("📡 Monitoring both devices simultaneously for 5 seconds...\n");
    monitor_dual_devices(comm_port_dut1, comm_port_dut2);

    println!("\n⚡ Powering OFF both devices...");
    controller.power_off(DeviceSide::Both).unwrap();
    println!("✅ Both devices powered OFF\n");

    // Final Summary
    println!("╔════════════════════════════════════════════╗");
    println!("║  Test Complete!                           ║");
    println!("╚════════════════════════════════════════════╝");
}

fn test_power_sequence(controller: &mut PowerController, side: DeviceSide, name: &str) {
    // Power ON
    print!("  → Powering ON {}... ", name);
    std::io::stdout().flush().unwrap();
    if controller.power_on(side).is_ok() {
        println!("✅");
    } else {
        println!("❌");
        return;
    }
    std::thread::sleep(Duration::from_millis(500));

    // Power OFF
    print!("  → Powering OFF {}... ", name);
    std::io::stdout().flush().unwrap();
    if controller.power_off(side).is_ok() {
        println!("✅");
    } else {
        println!("❌");
        return;
    }
    std::thread::sleep(Duration::from_millis(500));

    // VCHARGER ON
    print!("  → Enabling VCHARGER... ");
    std::io::stdout().flush().unwrap();
    if controller.enable_vcharger(side).is_ok() {
        println!("✅");
    } else {
        println!("❌");
    }
    std::thread::sleep(Duration::from_millis(500));

    // VCHARGER OFF
    print!("  → Disabling VCHARGER... ");
    std::io::stdout().flush().unwrap();
    if controller.disable_vcharger(side).is_ok() {
        println!("✅");
    } else {
        println!("❌");
    }
    std::thread::sleep(Duration::from_millis(500));

    // Reset
    print!("  → Testing RESET pulse... ");
    std::io::stdout().flush().unwrap();
    if controller.reset(side).is_ok() {
        println!("✅");
    } else {
        println!("❌");
    }
}

fn test_communication(controller: &mut PowerController, comm_port: &str, name: &str) {
    println!("📡 Testing {} Communication ({})", name, comm_port);
    println!("─────────────────────────────────────");

    let mut comm = match serialport::new(comm_port, 9600)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => {
            println!("  ✅ Port opened at 9600 baud");
            c
        }
        Err(e) => {
            eprintln!("  ❌ Failed to open port: {}", e);
            return;
        }
    };

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // Test 1: 被动监听
    println!("\n  📥 Test 1: Passive listening (3 seconds)");
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut received_passive = 0;

    while start.elapsed() < Duration::from_secs(3) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                received_passive += n;
                println!("    ✅ Received {} bytes: {:02X?}", n, &buffer[..n]);
                let text = String::from_utf8_lossy(&buffer[..n]);
                if !text.trim().is_empty() {
                    println!("       Text: {:?}", text);
                }
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if received_passive == 0 {
        println!("    ℹ️  No spontaneous data received");
    } else {
        println!("    📊 Total: {} bytes", received_passive);
    }

    // Test 2: 发送数据并等待响应
    println!("\n  📤 Test 2: Send and receive");

    let test_messages: Vec<(&str, &[u8])> = vec![
        ("HELLO", b"HELLO\n"),
        ("AT", b"AT\r\n"),
        ("TEST", b"TEST\n"),
    ];

    for (name, data) in test_messages {
        print!("    → Sending '{}'... ", name);
        std::io::stdout().flush().unwrap();

        if let Err(e) = comm.write_all(data) {
            println!("❌ Write error: {}", e);
            continue;
        }
        let _ = comm.flush();
        println!("✅");

        std::thread::sleep(Duration::from_millis(300));

        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                println!("      ✅ Response: {:02X?}", &buffer[..n]);
                println!("         Text: {:?}", String::from_utf8_lossy(&buffer[..n]));
            }
            _ => {
                println!("      ℹ️  No response");
            }
        }
    }

    // Test 3: 持续监听
    println!("\n  👂 Test 3: Extended monitoring (3 seconds)");
    let start = std::time::Instant::now();
    let mut total_extended = 0;

    while start.elapsed() < Duration::from_secs(3) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total_extended += n;
                println!("    ✅ [{:5.2}s] {} bytes: {:02X?}",
                    start.elapsed().as_secs_f32(), n, &buffer[..n]);
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if total_extended == 0 {
        println!("    ℹ️  No data received");
    }

    println!("\n  📊 Communication Summary for {}:", name);
    println!("    - Passive listening: {} bytes", received_passive);
    println!("    - Extended monitoring: {} bytes", total_extended);
    println!("    - Total: {} bytes", received_passive + total_extended);
}

fn monitor_dual_devices(port1: &str, port2: &str) {
    let mut comm1 = match serialport::new(port1, 9600)
        .timeout(Duration::from_millis(100))
        .open()
    {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("  ❌ Failed to open {}: {}", port1, e);
            None
        }
    };

    let mut comm2 = match serialport::new(port2, 9600)
        .timeout(Duration::from_millis(100))
        .open()
    {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("  ❌ Failed to open {}: {}", port2, e);
            None
        }
    };

    if comm1.is_none() && comm2.is_none() {
        println!("  ❌ Could not open any communication ports");
        return;
    }

    let start = std::time::Instant::now();
    let mut buffer1 = [0u8; 256];
    let mut buffer2 = [0u8; 256];
    let mut total1 = 0;
    let mut total2 = 0;

    while start.elapsed() < Duration::from_secs(5) {
        if let Some(ref mut c) = comm1 {
            if let Ok(n) = c.read(&mut buffer1) {
                if n > 0 {
                    total1 += n;
                    println!("  📥 DUT1: {} bytes: {:02X?}", n, &buffer1[..n]);
                }
            }
        }

        if let Some(ref mut c) = comm2 {
            if let Ok(n) = c.read(&mut buffer2) {
                if n > 0 {
                    total2 += n;
                    println!("  📥 DUT2: {} bytes: {:02X?}", n, &buffer2[..n]);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    println!("\n  📊 Dual Device Summary:");
    println!("    - DUT1: {} bytes", total1);
    println!("    - DUT2: {} bytes", total2);
}
