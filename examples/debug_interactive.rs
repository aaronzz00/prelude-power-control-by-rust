use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Interactive Debug Tool for DUT Testing  ║");
    println!("╚════════════════════════════════════════════╝\n");

    let control_port = "COM3";

    println!("🔌 Opening control port {}...", control_port);
    let mut controller = match PowerController::connect(control_port, WireMode::SingleWire) {
        Ok(c) => {
            println!("✅ Control port opened\n");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed: {}", e);
            return;
        }
    };

    // 初始化：关闭所有设备
    let _ = controller.power_off(DeviceSide::Both);

    loop {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║  Main Menu                                ║");
        println!("╚════════════════════════════════════════════╝");
        println!("  1. Power ON DUT1");
        println!("  2. Power OFF DUT1");
        println!("  3. Reset DUT1");
        println!("  4. Test DUT1 Communication (COM5)");
        println!("  5. Monitor DUT1 (continuous)");
        println!();
        println!("  6. Power ON DUT2");
        println!("  7. Power OFF DUT2");
        println!("  8. Reset DUT2");
        println!("  9. Test DUT2 Communication (COM6)");
        println!(" 10. Monitor DUT2 (continuous)");
        println!();
        println!(" 11. Test BOTH devices simultaneously");
        println!(" 12. Advanced: Try different baud rates");
        println!("  0. Exit");
        println!("─────────────────────────────────────────────");
        print!("Select option: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => power_on(&mut controller, DeviceSide::Device1, "DUT1"),
            "2" => power_off(&mut controller, DeviceSide::Device1, "DUT1"),
            "3" => reset_device(&mut controller, DeviceSide::Device1, "DUT1"),
            "4" => test_communication("COM5", "DUT1", 9600),
            "5" => monitor_continuous("COM5", "DUT1", 9600),

            "6" => power_on(&mut controller, DeviceSide::Device2, "DUT2"),
            "7" => power_off(&mut controller, DeviceSide::Device2, "DUT2"),
            "8" => reset_device(&mut controller, DeviceSide::Device2, "DUT2"),
            "9" => test_communication("COM6", "DUT2", 9600),
            "10" => monitor_continuous("COM6", "DUT2", 9600),

            "11" => test_both_devices(&mut controller),
            "12" => test_different_bauds(),

            "0" => {
                println!("\n👋 Cleaning up and exiting...");
                let _ = controller.power_off(DeviceSide::Both);
                break;
            }
            _ => println!("❌ Invalid option"),
        }
    }
}

fn power_on(controller: &mut PowerController, side: DeviceSide, name: &str) {
    println!("\n⚡ Powering ON {}...", name);
    match controller.power_on(side) {
        Ok(_) => {
            println!("✅ {} powered ON", name);
            println!("⏱️  Waiting 3 seconds for device to boot...");
            std::thread::sleep(Duration::from_secs(3));
            println!("✅ Boot time complete");
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}

fn power_off(controller: &mut PowerController, side: DeviceSide, name: &str) {
    println!("\n⚡ Powering OFF {}...", name);
    match controller.power_off(side) {
        Ok(_) => println!("✅ {} powered OFF", name),
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}

fn reset_device(controller: &mut PowerController, side: DeviceSide, name: &str) {
    println!("\n🔄 Resetting {}...", name);
    match controller.reset(side) {
        Ok(_) => {
            println!("✅ {} reset completed (100ms pulse)", name);
            println!("⏱️  Waiting 3 seconds for reboot...");
            std::thread::sleep(Duration::from_secs(3));
            println!("✅ Reboot time complete");
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}

fn test_communication(port: &str, name: &str, baud: u32) {
    println!("\n📡 Testing {} Communication ({} @ {} baud)", name, port, baud);
    println!("─────────────────────────────────────────────");

    let mut comm = match open_port(port, baud) {
        Some(c) => c,
        None => return,
    };

    // 清空缓冲区
    clear_buffer(&mut comm);

    // Test 1: 被动监听
    println!("\n📥 Passive listening for 5 seconds...");
    let received = listen_for_data(&mut comm, 5);
    if received == 0 {
        println!("ℹ️  No data received automatically");
    }

    // Test 2: 发送命令
    println!("\n📤 Sending test commands:");
    let commands = vec![
        ("AT\\r\\n", b"AT\r\n".to_vec()),
        ("HELLO\\n", b"HELLO\n".to_vec()),
        ("\\r\\n", b"\r\n".to_vec()),
        ("Empty line", b"\n".to_vec()),
    ];

    for (name, cmd) in commands {
        println!("  → {}", name);
        if comm.write_all(&cmd).is_ok() {
            let _ = comm.flush();
            std::thread::sleep(Duration::from_millis(500));

            let mut buf = [0u8; 256];
            match comm.read(&mut buf) {
                Ok(n) if n > 0 => {
                    println!("    ✅ Response: {:02X?}", &buf[..n]);
                    println!("       Text: {:?}", String::from_utf8_lossy(&buf[..n]));
                }
                _ => println!("    ℹ️  No response"),
            }
        }
    }

    println!("\n💡 Tip: If no data received, the device might:");
    println!("   - Need a specific command to start talking");
    println!("   - Use a different baud rate (try option 12)");
    println!("   - Need more boot time after power on");
}

fn monitor_continuous(port: &str, name: &str, baud: u32) {
    println!("\n👂 Continuous monitoring: {} ({} @ {} baud)", name, port, baud);
    println!("   Duration: 30 seconds");
    println!("   Press Ctrl+C to stop early\n");

    let mut comm = match open_port(port, baud) {
        Some(c) => c,
        None => return,
    };

    clear_buffer(&mut comm);

    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total = 0;
    let mut last_print = start;

    while start.elapsed() < Duration::from_secs(30) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total += n;
                let elapsed = start.elapsed().as_secs_f32();
                println!("[{:6.2}s] Received {} bytes: {:02X?}", elapsed, n, &buffer[..n]);

                let text = String::from_utf8_lossy(&buffer[..n]);
                if !text.trim().is_empty() {
                    println!("         Text: {:?}", text);
                }
                last_print = std::time::Instant::now();
            }
            _ => {
                // Print alive indicator every 5 seconds
                if last_print.elapsed() > Duration::from_secs(5) {
                    println!("[{:6.2}s] Still listening... ({} bytes so far)",
                        start.elapsed().as_secs_f32(), total);
                    last_print = std::time::Instant::now();
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    println!("\n📊 Total received: {} bytes", total);
}

fn test_both_devices(controller: &mut PowerController) {
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Testing Both Devices Simultaneously     ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("⚡ Powering ON both devices...");
    if controller.power_on(DeviceSide::Both).is_err() {
        eprintln!("❌ Failed to power on");
        return;
    }
    println!("✅ Both powered ON");
    println!("⏱️  Waiting 3 seconds for boot...\n");
    std::thread::sleep(Duration::from_secs(3));

    let mut comm1 = open_port("COM5", 9600);
    let mut comm2 = open_port("COM6", 9600);

    if comm1.is_none() && comm2.is_none() {
        println!("❌ Could not open any ports");
        return;
    }

    println!("👂 Monitoring both devices for 10 seconds...\n");

    let start = std::time::Instant::now();
    let mut buf1 = [0u8; 256];
    let mut buf2 = [0u8; 256];
    let mut total1 = 0;
    let mut total2 = 0;

    while start.elapsed() < Duration::from_secs(10) {
        if let Some(ref mut c) = comm1 {
            if let Ok(n) = c.read(&mut buf1) {
                if n > 0 {
                    total1 += n;
                    println!("[{:6.2}s] DUT1: {} bytes: {:02X?}",
                        start.elapsed().as_secs_f32(), n, &buf1[..n]);
                }
            }
        }

        if let Some(ref mut c) = comm2 {
            if let Ok(n) = c.read(&mut buf2) {
                if n > 0 {
                    total2 += n;
                    println!("[{:6.2}s] DUT2: {} bytes: {:02X?}",
                        start.elapsed().as_secs_f32(), n, &buf2[..n]);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    println!("\n📊 Summary:");
    println!("  DUT1 (COM5): {} bytes", total1);
    println!("  DUT2 (COM6): {} bytes", total2);

    println!("\n⚡ Powering OFF both devices...");
    let _ = controller.power_off(DeviceSide::Both);
}

fn test_different_bauds() {
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  Baud Rate Scanner                        ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("Select device:");
    println!("  1. DUT1 (COM5)");
    println!("  2. DUT2 (COM6)");
    print!("Choice: ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let port = match input.trim() {
        "1" => "COM5",
        "2" => "COM6",
        _ => {
            println!("❌ Invalid choice");
            return;
        }
    };

    let bauds = vec![9600, 115200, 19200, 38400, 57600, 4800, 2400];

    println!("\n🔍 Scanning {} with different baud rates...\n", port);
    println!("Make sure the device is powered ON!");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    for baud in bauds {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Testing {} baud", baud);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Some(mut comm) = open_port(port, baud) {
            clear_buffer(&mut comm);
            let received = listen_for_data(&mut comm, 2);

            if received > 0 {
                println!("✅ FOUND DATA at {} baud!", baud);
                println!("\n💡 Use this baud rate for communication!");
                return;
            }
        }
    }

    println!("\n❌ No data found at any baud rate");
    println!("💡 Device may need a command to start sending data");
}

// Helper functions
fn open_port(port: &str, baud: u32) -> Option<Box<dyn serialport::SerialPort>> {
    match serialport::new(port, baud)
        .timeout(Duration::from_millis(100))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => {
            println!("  ✅ {} opened at {} baud", port, baud);
            Some(c)
        }
        Err(e) => {
            eprintln!("  ❌ Failed to open {}: {}", port, e);
            None
        }
    }
}

fn clear_buffer(comm: &mut Box<dyn serialport::SerialPort>) {
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}
}

fn listen_for_data(comm: &mut Box<dyn serialport::SerialPort>, seconds: u64) -> usize {
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 256];
    let mut total = 0;

    while start.elapsed() < Duration::from_secs(seconds) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total += n;
                println!("  ✅ [{:5.2}s] Received {} bytes: {:02X?}",
                    start.elapsed().as_secs_f32(), n, &buffer[..n]);

                let text = String::from_utf8_lossy(&buffer[..n]);
                if !text.trim().is_empty() {
                    println!("     Text: {:?}", text);
                }
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    total
}
