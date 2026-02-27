# Prelude Power Controller: 硬件分析与测试总结 (Windows 环境交接指南)

本文档总结了此前在 macOS 上的所有逆向工程、源码分析和测试结果。由于 macOS 下无法成功捕获到 DUT 的启动日志，现将所有已知信息和工具梳理，以便在 Windows 环境下继续排查。

## 1. 硬件架构与端口映射

通过分析原厂 `PreludeController.cpp`、`ConfigManager.cpp` 以及用户提供的《Prelude 使用指南》，已确认 **Prelude 板卡 (FT4232H)** 的端口映射如下：

| 逻辑端口 | Windows 端口 | macOS 端口 | 功能说明 | 控制方式 |
|:---|:---|:---|:---|:---|
| **Port A** | `COM5` | `/dev/cu.usbserial-FT*A` | **硬件电源控制**<br>- `VCHARGER1/2`<br>- `POW1/2`<br>- `RESET1/2` | 使用 **FTDI D2XX 库**，配置为 **Asynchronous Bit-Bang 模式** (Mask=0xFF, Baud=62500)。通过操作引脚电平控制硬件。不可作为串口使用。 |
| **Port B** | `COM6` | `/dev/cu.*B` | **I2C 拓展**<br>电压/电流采样 | (暂未涉及) |
| **Port C** | `COM7` | `/dev/cu.*C` | **DUT1 UART 串口** | 标准 **VCP (虚拟串口) UART**。用于与 DUT1 通信和捕获日志。 |
| **Port D** | `COM8` | `/dev/cu.*D` | **DUT2 UART 串口** | 标准 **VCP (虚拟串口) UART**。用于与 DUT2 通信和捕获日志。 |

### 电源控制引脚定义 (Port A)
*数据包结构*: 7字节固定数组，控制指令写入 `data[6]`：
- `0x04` (`0b0000_0100`): **VCHARGER1**
- `0x08` (`0b0000_1000`): **VCHARGER2**
- `0x10` (`0b0001_0000`): **POW1**
- `0x20` (`0b0010_0000`): **POW2**
- `0x40` (`0b0100_0000`): **RESET1**
- `0x80` (`0b1000_0000`): **RESET2**

> **🚨 踩坑警告**: 绝不能使用标准的 `serialport` 库来向 Port A 写入 `\x04` 等字符。这只是发送了 UART 数据帧，并不能触动物理引脚电平。必须使用 **D2XX** 的 `FT_SetBitMode(0xFF, 0x01)`。

---

## 2. macOS 上的测试历程与结论

我们在 macOS 上编写了 Rust 版本的控制程序 (`prelude_power_controller/examples/hotplug_capture.rs`)，并进行了全方位的测试，结论如下：

1. **电源控制完全可行**：使用 `libftd2xx`  crate 可以成功将 Port A 设置为 Bit-Bang 模式，通过写入 `0x3C` (`VCHARGER1|VCHARGER2|POW1|POW2`) 成功让 DUT 上电。
2. **端口独立开启**：正确做法是在给 Port A 上电 **之前**，先把 Port C (DUT1) 和 Port D (DUT2) 以 9600 波特率作为标准 VCP 串口打开，防止错过早期启动日志。
3. **冷启动测试**：为了防止 DUT 处于已经启动的 "温态"，我们测试了强制全引脚下拉 `0x00` 下电 2 秒，再重新将 `VCHARGER`+`POW` 拉高，以强制冷启动。
4. **最终结果：0 字节**：
   - 即便使用了正确的硬件引脚上电，并一直监听 Port C / Port D 三十多秒。
   - **完全没有收到任何数据**。

---

## 3. Windows 环境下继续尝试的建议

由于在 macOS 上始终无法从正确的端口 (DUT1/DUT2) 读到日志，问题极有可能出在：
1. macOS 的 VCP 驱动对早期数据的缓冲丢失。
2. DUT 本身在当前的硬件触发下，**根本没有输出 Boot Log**。

请在 Windows 上直接编译并运行 `prelude_power_controller`，由于底层更换为 Windows 的 D2XX/VCP 驱动，可能产生不同结果。

### 建议 1: 对比原版软件 (C++ burn-cli)
- 运行原版的烧录工具，观察它在使用 Windows COM7/COM8 时，是否在没有任何额外通讯指令的情况下，**只要一上电就能看到 Boot Log**？
- 如果原版软件能看到，而 Rust 程序看不到，说明存在**隐藏的时序或初始化指令**（例如是否需要拉高一下 RESET 引脚？或者 VCHARGER 需要在 POW 之前稳定多少毫秒？）。

### 建议 2: 测试 "握手触发" 假说
- 根据 `BES_Download.h` 的分析，BES 芯片的 BootROM 默认可能是**静默**的。
- 只有当 PC 主动向它发送一个特定波特率的 Ping 数据包或 "Chip_Ramrun_Enable" 指令时，它才会以相同的波特率回复握手协议并开始输出 Log（类似 ESP32/CH340 的 Auto-Baud 检测或同步序列）。
- 在 Windows 上，可以尝试：在运行本 Rust 工具刚刚上电后，立刻向 Port C / Port D 疯狂发送 `0xBE` 或者连续空格等特定的同步字符，看是否有回音。

### 建议 3: 使用逻辑分析仪或示波器
这是终极确诊方案：
- 把逻辑分析仪接到 DUT 的 TX 线上。
- 运行 Rust 脚本进行冷启动（先断电再上电）。
- 直接观察 DUT 的 TX 线是否有高低电平跳变。如果有跳变但串口读不到，说明是波特率错误、奇偶校验不对，或是 Windows/macOS 驱动问题。如果没跳变，说明给它的上电条件不足以让它启动。

---

## 4. 仓库信息
**本项目（Rust 重制版电源控制与日志捕获）已托管至 GitHub：**
🔗 [https://github.com/aaronzz00/prelude-power-control-by-rust](https://github.com/aaronzz00/prelude-power-control-by-rust)

**核心测试代码位置：**
- `examples/hotplug_capture.rs`：包含最完整的“断电 -> 开好 UART -> BitBang 上电 -> 监听”流程（即 v9.1 冷启动版）。
- `examples/power_hold.rs`：单纯用于保持上电或下电的工具。
