/// scan_all_ports.rs
///
/// 被动扫描工具：不使用 D2XX Bit-Bang，而是以标准 VCP 模式打开所有 4 个 FTDI 串口
/// (ORKA0/1/2/3)，把能打开的全部以 9600 baud 监听 10 秒，看 DUT 上电后数据来自哪个端口实际到达
///
/// 使用场景：
///   - DUT 已预先上电（或手动处于上电状态）
///   - 用于定位哪个端口有 boot log 输出
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================");
    println!("  FTDI 全端口被动扫描工具");
    println!("  监听 ORKA0 / ORKA1 / ORKA2 / ORKA3");
    println!("  波特率: 9600 baud，时长: 15 秒");
    println!("================================================\n");

    // 候选端口：全部 4 个 FTDI VCP（A/B/C/D）
    let candidates = [
        ("A (ORKA0)", "/dev/cu.usbserial-FT66ORKA0"),
        ("B (ORKA1)", "/dev/cu.usbserial-FT66ORKA1"),
        ("C (ORKA2)", "/dev/cu.usbserial-FT66ORKA2"),
        ("D (ORKA3)", "/dev/cu.usbserial-FT66ORKA3"),
    ];

    println!("=== 打开所有候选端口 (9600 baud) ===");
    let mut listeners: Vec<(&str, Box<dyn serialport::SerialPort>)> = Vec::new();

    for (label, path) in &candidates {
        print!("  Port {} ({})... ", label, path);
        io::stdout().flush().unwrap();
        match serialport::new(*path, 9600)
            .timeout(Duration::from_millis(20))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open()
        {
            Ok(port) => {
                println!("OK");
                listeners.push((label, port));
            }
            Err(e) => println!("跳过 ({:?})", e.kind()),
        }
    }

    if listeners.is_empty() {
        println!("\n⚠ 无法打开任何端口，程序退出。");
        return Ok(());
    }

    println!("\n已打开 {} 个端口，开始监听 15 秒...", listeners.len());
    println!("（如 DUT 已上电，应在此期间看到启动 log）\n");

    let mut buffer = [0u8; 512];
    let start = Instant::now();
    let duration = Duration::from_secs(15);
    let mut total: Vec<usize> = vec![0; listeners.len()];

    while start.elapsed() < duration {
        for (i, (label, port)) in listeners.iter_mut().enumerate() {
            match port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    total[i] += n;
                    let text = String::from_utf8_lossy(&buffer[..n]);
                    print!("[Port {}] {}", label, text);
                    io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(5));
    }

    println!("\n\n================================================");
    println!("  扫描完成 ({}秒)", duration.as_secs());
    println!("================================================");
    for (i, (label, _)) in listeners.iter().enumerate() {
        let status = if total[i] > 0 {
            format!("✓ {} 字节  ← 日志端口!", total[i])
        } else {
            "0 字节".to_string()
        };
        println!("  Port {}: {}", label, status);
    }

    Ok(())
}
