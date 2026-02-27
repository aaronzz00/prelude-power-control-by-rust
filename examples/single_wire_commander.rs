use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Single Wire Command Tool                ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("📋 Configuration:");
    println!("  Power Control: COM5");
    println!("  DUT1: COM3");
    println!("  DUT2: COM4\n");

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

    // 初始化
    let _ = controller.power_off(DeviceSide::Both);

    loop {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║  Main Menu                                ║");
        println!("╚════════════════════════════════════════════╝");
        println!("  Power Control:");
        println!("    1. Power ON DUT1");
        println!("    2. Power OFF DUT1");
        println!("    3. Power ON DUT2");
        println!("    4. Power OFF DUT2");
        println!("    5. Power ON BOTH");
        println!("    6. Power OFF BOTH");
        println!();
        println!("  DUT1 Commands (COM3):");
        println!("    7. Send [init_status,]");
        println!("    8. Send custom command");
        println!("    9. Monitor continuously");
        println!();
        println!("  DUT2 Commands (COM4):");
        println!("   10. Send [init_status,]");
        println!("   11. Send custom command");
        println!("   12. Monitor continuously");
        println!("   13. Debug DUT2 (extended wait)");
        println!();
        println!("    0. Exit");
        println!("─────────────────────────────────────────────");
        print!("Select option: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => power_on(&mut controller, DeviceSide::Device1, "DUT1"),
            "2" => power_off(&mut controller, DeviceSide::Device1, "DUT1"),
            "3" => power_on(&mut controller, DeviceSide::Device2, "DUT2"),
            "4" => power_off(&mut controller, DeviceSide::Device2, "DUT2"),
            "5" => power_on(&mut controller, DeviceSide::Both, "BOTH"),
            "6" => power_off(&mut controller, DeviceSide::Both, "BOTH"),

            "7" => send_init_status("COM3", "DUT1"),
            "8" => send_custom_command("COM3", "DUT1"),
            "9" => monitor_continuous("COM3", "DUT1"),

            "10" => send_init_status("COM4", "DUT2"),
            "11" => send_custom_command("COM4", "DUT2"),
            "12" => monitor_continuous("COM4", "DUT2"),
            "13" => debug_dut2(&mut controller),

            "0" => {
                println!("\n👋 Exiting...");
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
            println!("⏱️  Waiting 3 seconds for boot...");
            std::thread::sleep(Duration::from_secs(3));
            println!("✅ Ready");
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

fn send_init_status(port: &str, name: &str) {
    println!("\n📤 Sending [init_status,] to {} ({})", name, port);
    println!("─────────────────────────────────────────────");

    let mut comm = match open_port(port, 9600) {
        Some(c) => c,
        None => return,
    };

    clear_buffer(&mut comm);

    let command = b"[init_status,]";
    println!("  → Sending command...");

    if let Err(e) = comm.write_all(command) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }
    let _ = comm.flush();
    println!("  ✅ Command sent");

    println!("\n  📥 Waiting for response (10 seconds)...\n");
    let received = receive_data(&mut comm, 10);

    if received == 0 {
        println!("  ❌ No response received");
        println!("\n  💡 Try:");
        println!("     • Check if device is powered ON");
        println!("     • Wait longer after power on");
        println!("     • Try option 13 for DUT2 extended debug");
    }
}

fn send_custom_command(port: &str, name: &str) {
    println!("\n📤 Send custom command to {} ({})", name, port);
    println!("─────────────────────────────────────────────");

    let mut comm = match open_port(port, 9600) {
        Some(c) => c,
        None => return,
    };

    clear_buffer(&mut comm);

    print!("Enter command (e.g., [init_status,]): ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let command = input.trim();

    println!("  → Sending: {:?}", command);

    if let Err(e) = comm.write_all(command.as_bytes()) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }
    let _ = comm.flush();
    println!("  ✅ Command sent");

    println!("\n  📥 Waiting for response (10 seconds)...\n");
    receive_data(&mut comm, 10);
}

fn monitor_continuous(port: &str, name: &str) {
    println!("\n👂 Monitoring {} ({}) for 30 seconds", name, port);
    println!("─────────────────────────────────────────────\n");

    let mut comm = match open_port(port, 9600) {
        Some(c) => c,
        None => return,
    };

    clear_buffer(&mut comm);

    let start = std::time::Instant::now();
    let mut buffer = [0u8; 512];
    let mut total = 0;
    let mut last_print = start;

    while start.elapsed() < Duration::from_secs(30) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total += n;
                println!("[{:6.2}s] {} bytes: {:02X?}",
                    start.elapsed().as_secs_f32(), n, &buffer[..n]);
                let text = String::from_utf8_lossy(&buffer[..n]);
                if !text.trim().is_empty() {
                    println!("         Text: {:?}", text);
                }
                last_print = std::time::Instant::now();
            }
            _ => {
                if last_print.elapsed() > Duration::from_secs(5) {
                    println!("[{:6.2}s] Still listening... ({} bytes so far)",
                        start.elapsed().as_secs_f32(), total);
                    last_print = std::time::Instant::now();
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    println!("\n📊 Total: {} bytes", total);
}

fn debug_dut2(controller: &mut PowerController) {
    println!("\n╔════════════════════════════════════════════╗");
    println!("║  DUT2 Extended Debug                      ║");
    println!("╚════════════════════════════════════════════╝\n");

    // 确保 DUT2 关闭
    println!("⚡ Ensuring DUT2 is OFF...");
    let _ = controller.power_off(DeviceSide::Device2);
    std::thread::sleep(Duration::from_secs(1));

    // 上电
    println!("⚡ Powering ON DUT2...");
    if let Err(e) = controller.power_on(DeviceSide::Device2) {
        eprintln!("❌ Failed: {}", e);
        return;
    }
    println!("✅ DUT2 powered ON");

    // 等待更长时间
    println!("⏱️  Waiting 5 seconds for boot (extended)...");
    std::thread::sleep(Duration::from_secs(5));

    // 打开串口
    println!("\n📡 Opening COM4...");
    let mut comm = match open_port("COM4", 9600) {
        Some(c) => c,
        None => return,
    };

    clear_buffer(&mut comm);

    // 先被动监听更长时间
    println!("\n👂 Step 1: Extended passive listening (5 seconds)...");
    let received1 = receive_data(&mut comm, 5);

    if received1 > 0 {
        println!("  ✅ Device is transmitting!");
    } else {
        println!("  ℹ️  No spontaneous data");
    }

    // 发送 init_status
    println!("\n📤 Step 2: Sending [init_status,]...");
    let command = b"[init_status,]";
    if let Err(e) = comm.write_all(command) {
        eprintln!("  ❌ Write failed: {}", e);
        return;
    }
    let _ = comm.flush();
    println!("  ✅ Command sent");

    println!("\n  📥 Waiting for response (15 seconds)...\n");
    let received2 = receive_data(&mut comm, 15);

    // 再次尝试
    if received2 == 0 {
        println!("\n🔄 Step 3: Trying again after short delay...");
        std::thread::sleep(Duration::from_secs(1));

        if let Err(e) = comm.write_all(command) {
            eprintln!("  ❌ Write failed: {}", e);
            return;
        }
        let _ = comm.flush();
        println!("  ✅ Command sent again");

        println!("\n  📥 Waiting for response (10 seconds)...\n");
        let received3 = receive_data(&mut comm, 10);

        if received3 == 0 {
            println!("\n❌ Still no response from DUT2");
            println!("\n💡 Possible issues:");
            println!("   • DUT2 hardware might be different from DUT1");
            println!("   • DUT2 might need different command");
            println!("   • DUT2 might need different baud rate");
            println!("   • DUT2 might need hardware reset");
            println!("\n💡 Try:");
            println!("   • Power cycle DUT2 (OFF then ON)");
            println!("   • Try hardware reset (option 9)");
            println!("   • Check physical connections");
        }
    }

    // 关闭
    println!("\n⚡ Powering OFF DUT2...");
    let _ = controller.power_off(DeviceSide::Device2);
}

// Helper functions
fn open_port(port: &str, baud: u32) -> Option<Box<dyn serialport::SerialPort>> {
    match serialport::new(port, baud)
        .timeout(Duration::from_millis(1000))
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

fn receive_data(comm: &mut Box<dyn serialport::SerialPort>, seconds: u64) -> usize {
    let start = std::time::Instant::now();
    let mut buffer = [0u8; 512];
    let mut total = 0;
    let mut response = Vec::new();

    while start.elapsed() < Duration::from_secs(seconds) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                total += n;
                response.extend_from_slice(&buffer[..n]);

                println!("  ✅ [{:5.2}s] {} bytes: {:02X?}",
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

    if total > 0 {
        println!("\n  📊 Total: {} bytes", total);
        println!("\n  Complete response:");
        println!("{}", String::from_utf8_lossy(&response));
    }

    total
}
