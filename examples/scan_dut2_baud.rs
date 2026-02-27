use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  DUT2 Baud Rate Scanner                   ║");
    println!("╚════════════════════════════════════════════╝\n");

    let control_port = "COM5";
    let comm_port = "COM4";

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

    // 上电 DUT2
    println!("⚡ Powering ON DUT2...");
    if let Err(e) = controller.power_on(DeviceSide::Device2) {
        eprintln!("❌ Failed: {}", e);
        return;
    }
    println!("✅ DUT2 powered ON");
    println!("⏱️  Waiting 5 seconds for boot...\n");
    std::thread::sleep(Duration::from_secs(5));

    // 测试常见波特率
    let baud_rates = vec![
        9600, 115200, 19200, 38400, 57600,
        14400, 28800, 4800, 2400, 1200,
        230400, 460800, 921600
    ];

    println!("🔍 Testing {} different baud rates...\n", baud_rates.len());
    println!("Looking for readable text response to [init_status,] command\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for (i, &baud) in baud_rates.iter().enumerate() {
        println!("[{:2}/{}] Testing {:7} baud...", i + 1, baud_rates.len(), baud);

        let result = test_baud_rate(comm_port, baud);

        match result {
            TestResult::Success(response) => {
                println!("\n🎉 ============================================");
                println!("🎉 SUCCESS! Found correct baud rate!");
                println!("🎉 ============================================");
                println!("\n✅ Baud rate: {} bps", baud);
                println!("\n📄 Response:\n{}", response);
                println!("\n💡 Use {} baud for DUT2 communication", baud);
                break;
            }
            TestResult::GotData(bytes, text) => {
                println!("  ⚠️  Got {} bytes but not readable text", bytes);
                println!("     Sample: {:?}", text);
            }
            TestResult::NoData => {
                println!("  ℹ️  No response");
            }
            TestResult::Error(e) => {
                println!("  ❌ Error: {}", e);
            }
        }

        // 短暂延迟
        std::thread::sleep(Duration::from_millis(200));
    }

    println!("\n⚡ Powering OFF DUT2...");
    let _ = controller.power_off(DeviceSide::Device2);
    println!("✅ DUT2 powered OFF");
}

enum TestResult {
    Success(String),
    GotData(usize, String),
    NoData,
    Error(String),
}

fn test_baud_rate(port: &str, baud: u32) -> TestResult {
    // 打开串口
    let mut comm = match serialport::new(port, baud)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open()
    {
        Ok(c) => c,
        Err(e) => return TestResult::Error(format!("Failed to open: {}", e)),
    };

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 发送命令
    let command = b"[init_status,]";
    if comm.write_all(command).is_err() {
        return TestResult::Error("Write failed".to_string());
    }
    let _ = comm.flush();

    // 等待响应
    std::thread::sleep(Duration::from_millis(200));

    let mut buffer = [0u8; 512];
    let mut response = Vec::new();
    let start = std::time::Instant::now();

    // 收集数据最多3秒
    while start.elapsed() < Duration::from_secs(3) {
        match comm.read(&mut buffer) {
            Ok(n) if n > 0 => {
                response.extend_from_slice(&buffer[..n]);

                // 如果收到足够数据，可以提前判断
                if response.len() > 50 {
                    break;
                }
            }
            _ => {
                // 如果已经有数据并且超过500ms没新数据，可以退出
                if !response.is_empty() && start.elapsed() > Duration::from_millis(500) {
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if response.is_empty() {
        return TestResult::NoData;
    }

    // 尝试解析为文本
    let text = String::from_utf8_lossy(&response).to_string();

    // 检查是否是可读的文本（包含常见的关键词）
    let is_readable = text.contains("Fw") ||
                      text.contains("Init") ||
                      text.contains("Version") ||
                      text.contains("Model") ||
                      text.contains("SN") ||
                      text.contains("BT") ||
                      text.contains("Aw:") ||
                      text.contains("Cw:");

    if is_readable {
        TestResult::Success(text)
    } else {
        // 检查是否大部分是可打印字符
        let printable_count = text.chars()
            .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
            .count();

        let printable_ratio = printable_count as f32 / text.len() as f32;

        if printable_ratio > 0.7 {
            TestResult::Success(text)
        } else {
            TestResult::GotData(response.len(), text.chars().take(30).collect())
        }
    }
}
