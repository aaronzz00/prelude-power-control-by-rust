/// probe_db0.rs
///
/// 诊断工具：捕获 FTDI Port A DB0 引脚的原始采样，
/// 分析 HIGH/LOW 电平比例和跳变模式，
/// 判断是否有 UART 信号存在以及空闲电平极性
use libftd2xx::{BitMode, DeviceInfo, Ftdi, FtdiCommon};
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

const VCHARGER1: u8 = 0x04;
const VCHARGER2: u8 = 0x08;
const POW1: u8 = 0x10;
const POW2: u8 = 0x20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================");
    println!("  DB0 引脚诊断工具");
    println!("  分析 Port A 各 bit 的电平分布和跳变");
    println!("================================================\n");

    println!(">>> 等待 FTDI 设备...");
    loop {
        if let Ok(devs) = libftd2xx::list_devices() {
            if devs
                .iter()
                .any(|d: &DeviceInfo| d.description.contains("Orka Prelude"))
            {
                println!("✓ 设备已就绪");
                break;
            }
        }
        print!(".");
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(500));
    }
    sleep(Duration::from_secs(2));

    let mut ft = Ftdi::with_description("FT4232H_Orka Prelude A")?;
    ft.set_usb_parameters(65536)?;
    ft.set_chars(0, false, 0, false)?;
    ft.set_timeouts(Duration::from_millis(500), Duration::from_millis(5000))?;
    ft.set_latency_timer(Duration::from_millis(1))?;
    ft.set_flow_control_none()?;
    ft.set_baud_rate(750_000)?;

    // Mask=0xFE: DB0=INPUT, DB1-DB7=OUTPUT
    ft.set_bit_mode(0xFE, BitMode::AsyncBitbang)?;

    // 上电
    let mut payload = [0u8; 7];
    payload[6] = VCHARGER1 | VCHARGER2 | POW1 | POW2; // 0x3C
    ft.write_all(&payload)?;
    println!("  DUT 上电 (0x{:02X})", payload[6]);
    println!("  立即采样 5 秒...\n");

    // 采样 5 秒
    let mut raw_buf = vec![0u8; 65536];
    let mut all_samples: Vec<u8> = Vec::with_capacity(20_000_000);
    let t = Instant::now();

    while t.elapsed() < Duration::from_secs(5) {
        if let Ok(n) = ft.read(&mut raw_buf) {
            if n > 0 {
                all_samples.extend_from_slice(&raw_buf[..n]);
            }
        }
    }

    println!(
        "\n采样完成: {} 字节 ({:.0} sps)",
        all_samples.len(),
        all_samples.len() as f64 / 5.0
    );

    // ==== 分析每个 bit 的 HIGH/LOW 分布 ====
    println!("\n=== 各 bit 电平分布 ===");
    println!(
        "{:<8} {:>12} {:>12} {:>10}",
        "BIT", "HIGH count", "LOW count", "HIGH%"
    );
    for bit in 0..8u8 {
        let mask = 1u8 << bit;
        let high = all_samples.iter().filter(|&&b| b & mask != 0).count();
        let low = all_samples.len() - high;
        let pct = high as f64 / all_samples.len() as f64 * 100.0;
        let note = match bit {
            0 => "← DB0/TXD (INPUT, DUT single wire)",
            4 => "← DB4/POW1 (OUTPUT HIGH)",
            5 => "← DB5/POW2 (OUTPUT HIGH)",
            2 => "← DB2/VCHARGER1 (OUTPUT HIGH)",
            3 => "← DB3/VCHARGER2 (OUTPUT HIGH)",
            _ => "",
        };
        println!(
            "  DB{:<4} {:>12} {:>12} {:>9.1}%  {}",
            bit, high, low, pct, note
        );
    }

    // ==== 分析 DB0 跳变次数（估算 UART 活跃度）====
    let mut transitions_db0 = 0usize;
    let mut prev_bit0 = all_samples[0] & 1;
    for &s in &all_samples[1..] {
        let cur_bit0 = s & 1;
        if cur_bit0 != prev_bit0 {
            transitions_db0 += 1;
        }
        prev_bit0 = cur_bit0;
    }
    println!("\n=== DB0 跳变分析 (5秒内) ===");
    println!("  总跳变次数: {}", transitions_db0);
    println!(
        "  期望值 (9600 baud, 8N1 随机数据): ~{} 次/5秒",
        9600 * 5 * 4
    );
    println!("  期望值 (全 HIGH 空闲无数据): ~0 次");

    // 打印前 100 个原始采样字节（低 4 bit）
    println!("\n=== 前 200 个采样的 DB0 bit 序列 ===");
    let prefix: String = all_samples[..200.min(all_samples.len())]
        .iter()
        .map(|&b| if b & 1 == 1 { '1' } else { '0' })
        .collect();
    println!("  {}", prefix);

    // 下电
    payload[6] = 0x00;
    ft.write_all(&payload)?;
    println!("\nDUT 已下电。");

    Ok(())
}
