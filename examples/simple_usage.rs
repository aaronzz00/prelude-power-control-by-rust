/// 简单使用示例
///
/// 这个例子展示了如何使用 Prelude Power Controller 的基本功能

use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{Read, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Prelude Power Controller 简单示例 ===\n");

    // ==================== 1. 连接电源控制器 ====================
    println!("1. 连接电源控制器 (COM5)...");
    let mut controller = PowerController::connect("COM5", WireMode::SingleWire)?;
    println!("   ✅ 连接成功\n");

    // ==================== 2. 开启 DUT1 ====================
    println!("2. 开启 DUT1 电源...");
    controller.power_on(DeviceSide::Device1)?;
    println!("   ✅ DUT1 已上电");
    println!("   ⏱️  等待3秒启动...");
    std::thread::sleep(Duration::from_secs(3));
    println!("   ✅ DUT1 启动完成\n");

    // ==================== 3. 获取 DUT1 设备信息 ====================
    println!("3. 获取 DUT1 设备信息...");

    // 打开通信端口
    let mut comm = serialport::new("COM3", 9600)
        .timeout(Duration::from_millis(1000))
        .open()?;

    // 清空缓冲区
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // 发送 init_status 命令
    comm.write_all(b"[init_status,]")?;
    comm.flush()?;

    std::thread::sleep(Duration::from_millis(500));

    // 接收响应
    let mut buffer = [0u8; 512];
    let mut response = Vec::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                response.extend_from_slice(&buffer[..n]);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 显示设备信息
    let text = String::from_utf8_lossy(&response);
    println!("   ✅ DUT1 设备信息:");

    for line in text.lines() {
        if line.contains("PROD SN:") {
            println!("      序列号: {}", line.replace("PROD SN:", "").trim());
        } else if line.contains("Fw0Version:") {
            println!("      固件0版本: {}", line.replace("Fw0Version:", "").trim());
        } else if line.contains("Fw1Version:") {
            println!("      固件1版本: {}", line.replace("Fw1Version:", "").trim());
        } else if line.contains("Model Name:") {
            println!("      型号: {}", line.replace("Model Name:", "").trim());
        }
    }
    println!();

    // 关闭串口
    drop(comm);

    // ==================== 4. 复位 DUT1 ====================
    println!("4. 复位 DUT1...");
    controller.reset(DeviceSide::Device1)?;
    println!("   ✅ DUT1 已复位（100ms脉冲）");
    println!("   ⏱️  等待3秒重启...");
    std::thread::sleep(Duration::from_secs(3));
    println!("   ✅ DUT1 重启完成\n");

    // ==================== 5. 关闭 DUT1 ====================
    println!("5. 关闭 DUT1 电源...");
    controller.power_off(DeviceSide::Device1)?;
    println!("   ✅ DUT1 已关闭\n");

    // ==================== 6. 测试 DUT2 ====================
    println!("6. 测试 DUT2...");
    controller.power_on(DeviceSide::Device2)?;
    println!("   ✅ DUT2 已上电");
    std::thread::sleep(Duration::from_secs(3));

    // 获取 DUT2 信息（COM4）
    let mut comm = serialport::new("COM4", 9600)
        .timeout(Duration::from_millis(1000))
        .open()?;

    while comm.read(&mut discard).is_ok() {}

    comm.write_all(b"[init_status,]")?;
    comm.flush()?;

    std::thread::sleep(Duration::from_millis(500));

    let mut response = Vec::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                response.extend_from_slice(&buffer[..n]);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let text = String::from_utf8_lossy(&response);
    println!("   ✅ DUT2 设备信息:");

    for line in text.lines() {
        if line.contains("PROD SN:") {
            println!("      序列号: {}", line.replace("PROD SN:", "").trim());
        }
    }

    drop(comm);

    controller.power_off(DeviceSide::Device2)?;
    println!("   ✅ DUT2 已关闭\n");

    // ==================== 完成 ====================
    println!("✅ 所有测试完成！\n");

    println!("📚 更多功能请参考:");
    println!("   - README_COMPLETE.md  - 完整使用指南");
    println!("   - TAURI_INTEGRATION.md - Tauri集成");
    println!("   - examples/ 目录下的其他示例");

    Ok(())
}
