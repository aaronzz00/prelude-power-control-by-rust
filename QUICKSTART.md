# 快速开始指南

## 🚀 一键测试

### 完整自动化测试（推荐新手）

```bash
cargo run --example test_dual_devices
```

这将自动测试：
- ✅ DUT1 和 DUT2 的所有电源控制功能
- ✅ COM5 (DUT1) 通信
- ✅ COM6 (DUT2) 通信
- ✅ 双设备同时工作

**预期结果**: 3-5分钟完成，生成详细测试报告

---

## 🎮 交互式调试（推荐高级用户）

```bash
cargo run --example debug_interactive
```

**简单操作流程**:

### 测试 DUT1
```
输入: 1  → Power ON DUT1
等待3秒让设备启动
输入: 5  → Monitor DUT1 (监控30秒)
→ 查看是否有数据输出

如果无数据:
输入: 4  → Test Communication
→ 尝试发送命令看是否有响应

如果还是无数据:
输入: 12 → Try different baud rates
→ 自动扫描所有可能的波特率
```

### 测试 DUT2
```
使用相同流程，但选项改为:
6 → Power ON DUT2
10 → Monitor DUT2
9 → Test DUT2 Communication
```

### 同时测试两个设备
```
输入: 11 → Test BOTH devices
→ 同时上电并监控两个设备
```

---

## 📊 当前测试状态

### ✅ 已确认工作
- 电源控制（COM3）
- DUT1/DUT2 开关
- VCHARGER 控制
- 硬件复位
- 串口连接（COM5/COM6）
- 数据发送

### ⚠️ 待确认
- 数据接收（两个设备都未收到数据）
- 可能原因：设备需要特定命令才会响应

---

## 💡 如果没有收到数据

### 1. 检查硬件
- [ ] COM5 连接到 DUT1？
- [ ] COM6 连接到 DUT2？
- [ ] RX/TX 是否接反？

### 2. 查阅文档
- DUT 设备的通信协议是什么？
- 需要什么命令才能让设备响应？
- 默认波特率是多少？

### 3. 使用外部工具验证
用 PuTTY 或 Tera Term 连接 COM5/COM6
看是否能收到数据

---

## 🔧 其他工具

```bash
# 多波特率扫描
cargo run --example test_comm_debug

# 简化交互式工具
cargo run --example interactive_comm
```

---

## 📖 详细文档

- 完整测试结果: [TEST_RESULTS.md](TEST_RESULTS.md)
- 测试指南: [TESTING.md](TESTING.md)
- 项目 README: [README.md](README.md)
