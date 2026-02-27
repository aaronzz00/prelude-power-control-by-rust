# Prelude Power Controller by Rust

这是一个使用 Rust 重写的 Prelude 串口电源控制器库。完全基于 `serialport` crate 构建标准的串口通信协议，并将功能封装为了清晰的高级接口，适合直接在 Tauri 后端（或任何其它 Rust 应用程序）中集成使用。

## 模块功能与特性

1. **核心逻辑隔离**: 删除了原版遗留下来的固件交互逻辑，只专注于“纯硬件电源状态控制”。
2. **状态自动跟踪**: `PowerController` 实例化后会在内部维持当前引脚的状态字数组，修改一台设备的开/关状态绝不会意外重置另一台设备。
3. **基于 C++ Bit-Bang 的提取**: 使用 7 字节 Payload，底层控制状态字被设置在 Array 的 `[6]` 号索引中发往设备。
4. **灵活连接**: 通过 `WireMode` 支持单线（9600）与双线（192000）切换。默认状态建议使用 `WireMode::SingleWire`。
5. **安全可靠异常处理**: 拒绝使用 `panic!`，由 `thiserror` 提供全面且精细的上下文类型错误系统 `PowerControllerError`。

## Tauri 整合说明

为了快速无缝接入，请参考本包内 `src/tauri_integration.rs` 示例给出的建议。您可以将此 Cargo package 直接加入您前端 Tauri 根部的 Workspace 中，或者直接复制代码进您的 `src-tauri/src` 里。

### 1. 配置依赖 (`Cargo.toml`)
确保加入以下依赖：
```toml
[dependencies]
serialport = "4.3"
thiserror = "1.0"
```

### 2. 初始化全局状态管理
Tauri 提供 `tauri::State` 来供应用在运行时进行注入和状态保持。
```rust
use prelude_power_controller::{PowerController, WireMode};
use std::sync::Mutex;

// 创建用于存放设备控制对象的状态管理器
#[derive(Default)]
pub struct PowerState {
    pub controller: Mutex<Option<PowerController>>,
}

// 在 main() 或者 builder 这侧注册
tauri::Builder::default()
    .manage(PowerState::default())
    // 注册各类 #[tauri::command] 后端端点 ...
```

### 3. Tauri 接口定义
您的前端可以使用类似如下的一系列 Command 控制底层的开关了：

```rust
use prelude_power_controller::DeviceSide;

#[tauri::command]
pub fn init_device(state: tauri::State<'_, PowerState>, port_name: String) -> Result<String, String> {
    // 默认使用单线模式
    match PowerController::connect(&port_name, WireMode::SingleWire) {
        Ok(controller) => {
            let mut managed = state.controller.lock().unwrap();
            *managed = Some(controller);
            Ok("Connected successfully".into())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn power_on_cmd(state: tauri::State<'_, PowerState>, side: String) -> Result<String, String> {
    let mut managed = state.controller.lock().unwrap();
    if let Some(ref mut controller) = *managed {
        let target = parse_side(&side)?; // Helper parsing str to DeviceSide::Device1, etc.
        controller.power_on(target).map_err(|e| e.to_string())?;
        Ok(format!("Powered ON: {}", side))
    } else {
        Err("Device not connected".into())
    }
}
```

如此这般，当您的 React 前端调用 `invoke('init_device', { portName: "COM3" })` 以及 `invoke('power_on_cmd', { side: "1" })` 时，底层物理设备的对应电源引脚就会亮起。
