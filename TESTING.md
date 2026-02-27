# Prelude Power Controller - 测试指南

## 📋 测试状态总结

### ✅ 已验证功能

#### 1. 电源控制 (COM3)
- ✅ Device1 开关控制
- ✅ Device2 开关控制
- ✅ 双设备同时控制
- ✅ VCHARGER 充电器控制
- ✅ 硬件复位功能

#### 2. 单线通信 (COM5/COM6)
- ✅ 串口连接成功
- ✅ 数据发送功能
- ⚠️ 数据接收待验证（需要实际硬件测试）

---

## 🧪 测试工具

### 1. 基础测试 - `test_power_and_comm.rs`
快速验证电源控制和通信功能：
```bash
cargo run --example test_power_and_comm
```

### 2. 分离端口测试 - `test_power_and_comm_v2.rs`
使用 COM3 控制电源，COM5 进行通信：
```bash
cargo run --example test_power_and_comm_v2
```

### 3. 通信调试工具 - `test_comm_debug.rs`
尝试多种波特率，详细诊断通信问题：
```bash
cargo run --example test_comm_debug
```

### 4. 交互式工具 - `interactive_comm.rs` ⭐推荐
提供交互式菜单，可以灵活测试各种功能：
```bash
cargo run --example interactive_comm
```

**功能菜单：**
- 电源开关控制
- 设备复位
- 通信测试
- 持续监控数据
- 发送自定义数据

---

## 🔧 使用方法

### 快速开始

1. **列出可用串口**
   ```bash
   cargo run --example interactive_comm
   ```

2. **选择端口**
   - 控制端口（电源）: COM3（默认）
   - 通信端口: COM5 或 COM6

3. **测试流程**
   ```
   1. Power ON Device1
   3. Reset Device1 (等待设备启动)
   4. Test Communication
   5. Monitor Communication (持续监控)
   ```

### 常见问题排查

#### 没有收到数据？

可能的原因：
1. **端口选择错误**: 尝试 COM6 而不是 COM5
2. **波特率不匹配**: 尝试不同的波特率（9600, 115200, 19200）
3. **设备未启动**: 等待更长时间或执行复位
4. **需要初始化命令**: 某些设备需要特定的握手序列
5. **硬件连接问题**: 检查 RX/TX 线路连接

#### 调试步骤：

1. **确认端口存在**
   ```bash
   # 列出所有端口
   cargo run --example interactive_comm
   ```

2. **尝试不同端口**
   - COM5
   - COM6
   - 其他可用端口

3. **尝试不同波特率**
   - 9600（默认单线模式）
   - 115200（常用高速率）
   - 19200, 38400, 57600

4. **使用监控模式**
   选择菜单选项 5，持续监控 30 秒，看是否有任何数据

---

## 📊 测试结果

### 当前测试环境
- **操作系统**: Windows 11
- **工具链**: x86_64-pc-windows-gnu
- **可用端口**: COM3, COM4, COM5, COM6

### 电源控制测试
```
✅ Device1 ON/OFF - 正常
✅ Device2 ON/OFF - 正常
✅ 双设备控制 - 正常
✅ VCHARGER 控制 - 正常
✅ 复位功能 - 正常
```

### 通信测试
```
⚠️ COM5 @ 9600 baud - 发送成功，未收到数据
⚠️ COM5 @ 115200 baud - 发送成功，未收到数据
⚠️ COM5 @ 其他波特率 - 发送成功，未收到数据
```

**需要进一步验证：**
- [ ] 确认 COM5 是正确的通信端口
- [ ] 尝试 COM6 端口
- [ ] 确认设备是否自动发送数据
- [ ] 测试设备是否需要特定的初始化命令

---

## 🛠️ 开发环境配置

### 依赖项
```toml
[dependencies]
serialport = "4.3"
thiserror = "1.0"

[dependencies.libftd2xx]
version = "0.33"
features = ["static"]
```

### 编译要求
- Rust 1.70+
- Windows: GNU toolchain (x86_64-pc-windows-gnu)
- 如果使用 MSVC 工具链，需要安装 Windows 10 SDK

---

## 📝 下一步

1. **验证通信端口**
   - 使用交互式工具尝试 COM6
   - 确认设备实际使用的通信端口

2. **收集日志**
   - 使用监控模式记录所有数据
   - 分析数据格式和通信协议

3. **文档更新**
   - 记录正确的端口配置
   - 更新通信协议说明

4. **代码优化**
   - 清理警告
   - 添加单元测试
   - 改进错误处理

---

## 📞 支持

如遇问题，请运行交互式工具并尝试：
1. 不同的串口组合
2. 不同的波特率
3. 复位后立即监控
4. 发送不同的测试命令
