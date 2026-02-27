/// hotplug_capture.rs - v9.1（冷启动版）
///
/// 修复：先完全下电等待 2 秒，再上电，确保 DUT 经历真正的冷启动
/// 这样才能触发 DUT 的 boot ROM 串口日志输出
use libftd2xx::{BitMode, DeviceInfo, Ftdi, FtdiCommon};
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

const VCHARGER1: u8 = 0x04;
const VCHARGER2: u8 = 0x08;
const POW1: u8 = 0x10;
const POW2: u8 = 0x20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================");
    println!("  DUT Boot Log 捕获工具 v9.1（冷启动版）");
    println!("  先下电 2 秒，再上电，触发真正的 boot 序列");
    println!("  ORKA2 = DUT1 UART, ORKA3 = DUT2 UART, 9600 baud");
    println!("================================================\n");

    // =====================================================
    // Step 1: 等待设备
    // =====================================================
    println!(">>> 等待 FTDI 设备...");
    let wait_start = Instant::now();
    loop {
        if let Ok(devs) = libftd2xx::list_devices() {
            if devs
                .iter()
                .any(|d: &DeviceInfo| d.description.contains("Orka Prelude"))
            {
                println!("[{:.1}s] ✓ 设备已就绪", wait_start.elapsed().as_secs_f32());
                break;
            }
        }
        print!(".");
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(500));
    }
    sleep(Duration::from_secs(2));

    // =====================================================
    // Step 2: 初始化 Bit-Bang，先完全下电
    // =====================================================
    println!("\n=== Step 2: 初始化 Port A Bit-Bang，先下电 ===");
    let mut ft_a = Ftdi::with_description("FT4232H_Orka Prelude A")?;
    ft_a.set_usb_parameters(4096)?;
    ft_a.set_chars(0, false, 0, false)?;
    ft_a.set_timeouts(Duration::from_millis(5000), Duration::from_millis(5000))?;
    ft_a.set_latency_timer(Duration::from_millis(16))?;
    ft_a.set_flow_control_none()?;
    ft_a.set_baud_rate(62500)?;
    ft_a.set_bit_mode(0xFF, BitMode::AsyncBitbang)?;

    let mut payload = [0u8; 7];

    // 强制下电：所有引脚清零（包括 POW/VCHARGER），持续 2 秒
    println!("  [1/2] 强制全引脚清零（下电），等待 2 秒...");
    payload[6] = 0x00;
    ft_a.write_all(&payload)?;
    sleep(Duration::from_secs(2));
    println!("  [2/2] 下电完成，DUT 已完全断电");

    // =====================================================
    // Step 3: 此时打开 DUT UART 端口（在上电前打开，抢先就位）
    // =====================================================
    println!("\n=== Step 3: 打开 DUT UART 端口（上电前就绪）===");

    let open_uart = |path: &str| -> Option<Box<dyn serialport::SerialPort>> {
        let result = serialport::new(path, 9600)
            .timeout(Duration::from_millis(10))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open();
        match result {
            Ok(p) => {
                println!("  {} ... OK", path);
                Some(p)
            }
            Err(e) => {
                println!("  {} ... 失败 ({:?})", path, e.kind());
                None
            }
        }
    };

    let mut dut1 = open_uart("/dev/cu.usbserial-FT66ORKA2");
    let mut dut2 = open_uart("/dev/cu.usbserial-FT66ORKA3");

    if dut1.is_none() && dut2.is_none() {
        println!("  ⚠ 无法打开任何 DUT 端口，退出。");
        return Ok(());
    }

    // =====================================================
    // Step 4: 清空串口缓冲区中的残留数据
    // =====================================================
    println!("  清空缓冲区残留数据...");
    let mut discard = [0u8; 4096];
    for _ in 0..5 {
        if let Some(ref mut p) = dut1 {
            let _ = p.read(&mut discard);
        }
        if let Some(ref mut p) = dut2 {
            let _ = p.read(&mut discard);
        }
    }

    // =====================================================
    // Step 5: 上电 DUT（此时串口已就绪，不丢失任何数据）
    // =====================================================
    println!("\n=== Step 5: 冷启动上电 ===");
    println!("  VCHARGER1 + VCHARGER2 = HIGH...");
    payload[6] = VCHARGER1 | VCHARGER2;
    ft_a.write_all(&payload)?;
    sleep(Duration::from_millis(100));

    println!(
        "  POW1 + POW2 = HIGH → 冷启动! (0x{:02X})",
        payload[6] | POW1 | POW2
    );
    payload[6] |= POW1 | POW2;
    ft_a.write_all(&payload)?;

    // =====================================================
    // Step 6: 监听启动日志
    // =====================================================
    let listen_secs = 30u64;
    println!("\n=== Step 6: 监听 DUT Boot Log ({} 秒) ===\n", listen_secs);

    let mut buf = [0u8; 1024];
    let start = Instant::now();
    let mut bytes_dut1 = 0usize;
    let mut bytes_dut2 = 0usize;

    while start.elapsed() < Duration::from_secs(listen_secs) {
        if let Some(ref mut p) = dut1 {
            if let Ok(n) = p.read(&mut buf) {
                if n > 0 {
                    bytes_dut1 += n;
                    print!("[DUT1] {}", String::from_utf8_lossy(&buf[..n]));
                    io::stdout().flush().unwrap();
                }
            }
        }
        if let Some(ref mut p) = dut2 {
            if let Ok(n) = p.read(&mut buf) {
                if n > 0 {
                    bytes_dut2 += n;
                    print!("[DUT2] {}", String::from_utf8_lossy(&buf[..n]));
                    io::stdout().flush().unwrap();
                }
            }
        }
        sleep(Duration::from_millis(5));
    }

    // =====================================================
    // 汇总 & 下电
    // =====================================================
    println!("\n\n================================================");
    println!("  捕获完成 ({} 秒)", listen_secs);
    println!("================================================");
    println!("  DUT1 (ORKA2): {} 字节", bytes_dut1);
    println!("  DUT2 (ORKA3): {} 字节", bytes_dut2);

    if bytes_dut1 == 0 && bytes_dut2 == 0 {
        println!("\n  ⚠ 仍未收到数据。DUT 固件可能不在 9600 baud 输出 log。");
        println!("    建议：使用逻辑分析仪确认 UART 信号是否存在，并测量实际波特率。");
    }

    println!("\n=== 下电 ===");
    payload[6] = 0x00;
    ft_a.write_all(&payload)?;
    println!("已下电。");

    Ok(())
}
