# Tauri 应用集成指南

## 📋 目录

1. [快速开始](#快速开始)
2. [Tauri Commands](#tauri-commands)
3. [前端集成](#前端集成)
4. [完整示例](#完整示例)
5. [最佳实践](#最佳实践)

---

## 🚀 快速开始

### 1. 添加依赖到 Cargo.toml

在你的 Tauri 项目的 `src-tauri/Cargo.toml` 中添加：

```toml
[dependencies]
prelude_power_controller = { path = "../path/to/prelude-rust" }
# 或者如果发布到 crates.io：
# prelude_power_controller = "0.1.0"

serialport = "4.3"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
tauri = { version = "1.5", features = ["shell-open"] }
```

### 2. 项目结构

```
your-tauri-app/
├── src/               # 前端代码 (React/Vue/Svelte)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs    # Tauri 入口
│   │   ├── commands.rs # Tauri commands
│   │   └── state.rs   # 应用状态管理
│   ├── Cargo.toml
│   └── tauri.conf.json
└── prelude-rust/      # 本项目（作为子模块或依赖）
```

---

## 💻 Tauri Commands

### 创建 `src-tauri/src/commands.rs`

```rust
use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;

// 应用状态
pub struct AppState {
    pub controller: Mutex<Option<PowerController>>,
}

// 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub serial_number: String,
    pub fw0_version: String,
    pub fw1_version: String,
    pub model_name: String,
    pub bt_address: String,
    pub ble_address: String,
    pub calib: String,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// ==================== Power Control Commands ====================

#[tauri::command]
pub fn connect_controller(state: State<AppState>) -> ApiResponse<String> {
    let mut controller_guard = state.controller.lock().unwrap();

    match PowerController::connect("COM5", WireMode::SingleWire) {
        Ok(controller) => {
            *controller_guard = Some(controller);
            ApiResponse {
                success: true,
                data: Some("Power controller connected".to_string()),
                error: None,
            }
        }
        Err(e) => ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to connect: {}", e)),
        },
    }
}

#[tauri::command]
pub fn power_on_device(state: State<AppState>, device: String) -> ApiResponse<String> {
    let mut controller_guard = state.controller.lock().unwrap();

    if let Some(controller) = controller_guard.as_mut() {
        let side = match device.as_str() {
            "DUT1" => DeviceSide::Device1,
            "DUT2" => DeviceSide::Device2,
            "BOTH" => DeviceSide::Both,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid device".to_string()),
                }
            }
        };

        match controller.power_on(side) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(format!("{} powered ON", device)),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to power on: {}", e)),
            },
        }
    } else {
        ApiResponse {
            success: false,
            data: None,
            error: Some("Controller not connected".to_string()),
        }
    }
}

#[tauri::command]
pub fn power_off_device(state: State<AppState>, device: String) -> ApiResponse<String> {
    let mut controller_guard = state.controller.lock().unwrap();

    if let Some(controller) = controller_guard.as_mut() {
        let side = match device.as_str() {
            "DUT1" => DeviceSide::Device1,
            "DUT2" => DeviceSide::Device2,
            "BOTH" => DeviceSide::Both,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid device".to_string()),
                }
            }
        };

        match controller.power_off(side) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(format!("{} powered OFF", device)),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to power off: {}", e)),
            },
        }
    } else {
        ApiResponse {
            success: false,
            data: None,
            error: Some("Controller not connected".to_string()),
        }
    }
}

#[tauri::command]
pub fn reset_device(state: State<AppState>, device: String) -> ApiResponse<String> {
    let mut controller_guard = state.controller.lock().unwrap();

    if let Some(controller) = controller_guard.as_mut() {
        let side = match device.as_str() {
            "DUT1" => DeviceSide::Device1,
            "DUT2" => DeviceSide::Device2,
            "BOTH" => DeviceSide::Both,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid device".to_string()),
                }
            }
        };

        match controller.reset(side) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(format!("{} reset completed", device)),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to reset: {}", e)),
            },
        }
    } else {
        ApiResponse {
            success: false,
            data: None,
            error: Some("Controller not connected".to_string()),
        }
    }
}

#[tauri::command]
pub fn enable_vcharger(state: State<AppState>, device: String) -> ApiResponse<String> {
    let mut controller_guard = state.controller.lock().unwrap();

    if let Some(controller) = controller_guard.as_mut() {
        let side = match device.as_str() {
            "DUT1" => DeviceSide::Device1,
            "DUT2" => DeviceSide::Device2,
            "BOTH" => DeviceSide::Both,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid device".to_string()),
                }
            }
        };

        match controller.enable_vcharger(side) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(format!("{} VCHARGER enabled", device)),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to enable VCHARGER: {}", e)),
            },
        }
    } else {
        ApiResponse {
            success: false,
            data: None,
            error: Some("Controller not connected".to_string()),
        }
    }
}

#[tauri::command]
pub fn disable_vcharger(state: State<AppState>, device: String) -> ApiResponse<String> {
    let mut controller_guard = state.controller.lock().unwrap();

    if let Some(controller) = controller_guard.as_mut() {
        let side = match device.as_str() {
            "DUT1" => DeviceSide::Device1,
            "DUT2" => DeviceSide::Device2,
            "BOTH" => DeviceSide::Both,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid device".to_string()),
                }
            }
        };

        match controller.disable_vcharger(side) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(format!("{} VCHARGER disabled", device)),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to disable VCHARGER: {}", e)),
            },
        }
    } else {
        ApiResponse {
            success: false,
            data: None,
            error: Some("Controller not connected".to_string()),
        }
    }
}

// ==================== Communication Commands ====================

#[tauri::command]
pub fn get_device_info(device: String) -> ApiResponse<DeviceInfo> {
    let port = match device.as_str() {
        "DUT1" => "COM3",
        "DUT2" => "COM4",
        _ => {
            return ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid device".to_string()),
            }
        }
    };

    match fetch_device_info(port) {
        Ok(info) => ApiResponse {
            success: true,
            data: Some(info),
            error: None,
        },
        Err(e) => ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get device info: {}", e)),
        },
    }
}

#[tauri::command]
pub fn send_shutdown_command(device: String) -> ApiResponse<String> {
    let port = match device.as_str() {
        "DUT1" => "COM3",
        "DUT2" => "COM4",
        _ => {
            return ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid device".to_string()),
            }
        }
    };

    match send_command(port, "[shutdown,]") {
        Ok(_) => ApiResponse {
            success: true,
            data: Some(format!("Shutdown command sent to {}", device)),
            error: None,
        },
        Err(e) => ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to send shutdown: {}", e)),
        },
    }
}

#[tauri::command]
pub fn send_custom_command(device: String, command: String) -> ApiResponse<String> {
    let port = match device.as_str() {
        "DUT1" => "COM3",
        "DUT2" => "COM4",
        _ => {
            return ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid device".to_string()),
            }
        }
    };

    match send_command(port, &command) {
        Ok(response) => ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        },
        Err(e) => ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to send command: {}", e)),
        },
    }
}

// ==================== Helper Functions ====================

fn fetch_device_info(port: &str) -> Result<DeviceInfo, String> {
    let mut comm = serialport::new(port, 9600)
        .timeout(Duration::from_millis(1000))
        .open()
        .map_err(|e| format!("Failed to open port: {}", e))?;

    // Clear buffer
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // Send command
    comm.write_all(b"[init_status,]")
        .map_err(|e| format!("Failed to write: {}", e))?;
    comm.flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    std::thread::sleep(Duration::from_millis(500));

    // Receive response
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

    let text = String::from_utf8_lossy(&response);

    // Parse response
    let mut info = DeviceInfo {
        serial_number: String::new(),
        fw0_version: String::new(),
        fw1_version: String::new(),
        model_name: String::new(),
        bt_address: String::new(),
        ble_address: String::new(),
        calib: String::new(),
        mode: String::new(),
    };

    for line in text.lines() {
        if line.contains("PROD SN:") {
            info.serial_number = line.replace("PROD SN:", "").trim().to_string();
        } else if line.contains("Fw0Version:") {
            info.fw0_version = line.replace("Fw0Version:", "").trim().to_string();
        } else if line.contains("Fw1Version:") {
            info.fw1_version = line.replace("Fw1Version:", "").trim().to_string();
        } else if line.contains("Model Name:") {
            info.model_name = line.replace("Model Name:", "").trim().to_string();
        } else if line.starts_with("BT:") {
            info.bt_address = line.replace("BT:", "").trim().to_string();
        } else if line.starts_with("BLE:") {
            info.ble_address = line.replace("BLE:", "").trim().to_string();
        } else if line.contains("Calib:") {
            info.calib = line.replace("Calib:", "").trim().to_string();
        } else if line.contains("Mode0:") {
            info.mode = line.replace("Mode0:", "").trim().to_string();
        }
    }

    if info.serial_number.is_empty() {
        Err("Failed to parse device info".to_string())
    } else {
        Ok(info)
    }
}

fn send_command(port: &str, command: &str) -> Result<String, String> {
    let mut comm = serialport::new(port, 9600)
        .timeout(Duration::from_millis(1000))
        .open()
        .map_err(|e| format!("Failed to open port: {}", e))?;

    // Clear buffer
    let mut discard = [0u8; 1024];
    while comm.read(&mut discard).is_ok() {}

    // Send command
    comm.write_all(command.as_bytes())
        .map_err(|e| format!("Failed to write: {}", e))?;
    comm.flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    std::thread::sleep(Duration::from_millis(500));

    // Receive response
    let mut buffer = [0u8; 512];
    let mut response = Vec::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(2) {
        if let Ok(n) = comm.read(&mut buffer) {
            if n > 0 {
                response.extend_from_slice(&buffer[..n]);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(String::from_utf8_lossy(&response).to_string())
}
```

---

### 更新 `src-tauri/src/main.rs`

```rust
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use commands::AppState;
use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            controller: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect_controller,
            commands::power_on_device,
            commands::power_off_device,
            commands::reset_device,
            commands::enable_vcharger,
            commands::disable_vcharger,
            commands::get_device_info,
            commands::send_shutdown_command,
            commands::send_custom_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 🌐 前端集成

### TypeScript 类型定义

创建 `src/types/prelude.ts`:

```typescript
export interface DeviceInfo {
  serial_number: string;
  fw0_version: string;
  fw1_version: string;
  model_name: string;
  bt_address: string;
  ble_address: string;
  calib: string;
  mode: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export type DeviceId = 'DUT1' | 'DUT2' | 'BOTH';
```

### API 封装

创建 `src/api/prelude.ts`:

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import type { ApiResponse, DeviceId, DeviceInfo } from '../types/prelude';

export class PreludeAPI {
  // Power Control
  static async connectController(): Promise<ApiResponse<string>> {
    return await invoke('connect_controller');
  }

  static async powerOn(device: DeviceId): Promise<ApiResponse<string>> {
    return await invoke('power_on_device', { device });
  }

  static async powerOff(device: DeviceId): Promise<ApiResponse<string>> {
    return await invoke('power_off_device', { device });
  }

  static async reset(device: DeviceId): Promise<ApiResponse<string>> {
    return await invoke('reset_device', { device });
  }

  static async enableVCharger(device: DeviceId): Promise<ApiResponse<string>> {
    return await invoke('enable_vcharger', { device });
  }

  static async disableVCharger(device: DeviceId): Promise<ApiResponse<string>> {
    return await invoke('disable_vcharger', { device });
  }

  // Communication
  static async getDeviceInfo(device: DeviceId): Promise<ApiResponse<DeviceInfo>> {
    return await invoke('get_device_info', { device });
  }

  static async sendShutdown(device: DeviceId): Promise<ApiResponse<string>> {
    return await invoke('send_shutdown_command', { device });
  }

  static async sendCustomCommand(
    device: DeviceId,
    command: string
  ): Promise<ApiResponse<string>> {
    return await invoke('send_custom_command', { device, command });
  }
}
```

---

## 📱 完整示例

### React 示例

```typescript
import React, { useState, useEffect } from 'react';
import { PreludeAPI } from './api/prelude';
import type { DeviceInfo } from './types/prelude';

function App() {
  const [connected, setConnected] = useState(false);
  const [dut1Info, setDut1Info] = useState<DeviceInfo | null>(null);
  const [dut2Info, setDut2Info] = useState<DeviceInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnect = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await PreludeAPI.connectController();
      if (result.success) {
        setConnected(true);
      } else {
        setError(result.error || 'Connection failed');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handlePowerOn = async (device: 'DUT1' | 'DUT2') => {
    setLoading(true);
    setError(null);
    try {
      const result = await PreludeAPI.powerOn(device);
      if (!result.success) {
        setError(result.error || 'Power on failed');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleGetInfo = async (device: 'DUT1' | 'DUT2') => {
    setLoading(true);
    setError(null);
    try {
      const result = await PreludeAPI.getDeviceInfo(device);
      if (result.success && result.data) {
        if (device === 'DUT1') {
          setDut1Info(result.data);
        } else {
          setDut2Info(result.data);
        }
      } else {
        setError(result.error || 'Failed to get device info');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleShutdown = async (device: 'DUT1' | 'DUT2') => {
    setLoading(true);
    setError(null);
    try {
      const result = await PreludeAPI.sendShutdown(device);
      if (!result.success) {
        setError(result.error || 'Shutdown failed');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="app">
      <h1>Prelude Power Controller</h1>

      {error && <div className="error">{error}</div>}

      {!connected ? (
        <button onClick={handleConnect} disabled={loading}>
          Connect to Controller
        </button>
      ) : (
        <div className="controls">
          <h2>DUT1 Controls</h2>
          <button onClick={() => handlePowerOn('DUT1')} disabled={loading}>
            Power ON
          </button>
          <button onClick={() => handleGetInfo('DUT1')} disabled={loading}>
            Get Info
          </button>
          <button onClick={() => handleShutdown('DUT1')} disabled={loading}>
            Shutdown
          </button>

          {dut1Info && (
            <div className="device-info">
              <h3>DUT1 Information</h3>
              <p>Serial Number: {dut1Info.serial_number}</p>
              <p>Firmware: {dut1Info.fw0_version} / {dut1Info.fw1_version}</p>
              <p>Model: {dut1Info.model_name}</p>
              <p>BT: {dut1Info.bt_address}</p>
            </div>
          )}

          <hr />

          <h2>DUT2 Controls</h2>
          <button onClick={() => handlePowerOn('DUT2')} disabled={loading}>
            Power ON
          </button>
          <button onClick={() => handleGetInfo('DUT2')} disabled={loading}>
            Get Info
          </button>
          <button onClick={() => handleShutdown('DUT2')} disabled={loading}>
            Shutdown
          </button>

          {dut2Info && (
            <div className="device-info">
              <h3>DUT2 Information</h3>
              <p>Serial Number: {dut2Info.serial_number}</p>
              <p>Firmware: {dut2Info.fw0_version} / {dut2Info.fw1_version}</p>
              <p>Model: {dut2Info.model_name}</p>
              <p>BT: {dut2Info.bt_address}</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default App;
```

---

## ✅ 最佳实践

### 1. 错误处理

```typescript
async function safeInvoke<T>(fn: () => Promise<ApiResponse<T>>): Promise<T | null> {
  try {
    const result = await fn();
    if (result.success && result.data) {
      return result.data;
    } else {
      console.error('API Error:', result.error);
      return null;
    }
  } catch (err) {
    console.error('Unexpected Error:', err);
    return null;
  }
}
```

### 2. 启动时连接

```typescript
useEffect(() => {
  // Auto-connect on app start
  PreludeAPI.connectController()
    .then((result) => {
      if (result.success) {
        console.log('Controller connected');
      }
    })
    .catch(console.error);
}, []);
```

### 3. 设备状态管理

```typescript
interface DeviceState {
  isPowered: boolean;
  info: DeviceInfo | null;
  lastUpdated: Date;
}

const [dut1State, setDut1State] = useState<DeviceState>({
  isPowered: false,
  info: null,
  lastUpdated: new Date(),
});
```

### 4. 日志记录

```typescript
const logAction = (action: string, device: string, result: boolean) => {
  const timestamp = new Date().toISOString();
  console.log(`[${timestamp}] ${action} on ${device}: ${result ? 'SUCCESS' : 'FAILED'}`);
};
```

---

## 📚 可用的 Tauri Commands

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `connect_controller` | - | `ApiResponse<String>` | 连接电源控制器 |
| `power_on_device` | `device: String` | `ApiResponse<String>` | 打开设备电源 |
| `power_off_device` | `device: String` | `ApiResponse<String>` | 关闭设备电源 |
| `reset_device` | `device: String` | `ApiResponse<String>` | 复位设备 |
| `enable_vcharger` | `device: String` | `ApiResponse<String>` | 启用充电器 |
| `disable_vcharger` | `device: String` | `ApiResponse<String>` | 禁用充电器 |
| `get_device_info` | `device: String` | `ApiResponse<DeviceInfo>` | 获取设备信息 |
| `send_shutdown_command` | `device: String` | `ApiResponse<String>` | 发送关机命令 |
| `send_custom_command` | `device: String, command: String` | `ApiResponse<String>` | 发送自定义命令 |

---

## 🔧 调试技巧

### 1. 启用 Tauri 开发者工具

在 `tauri.conf.json` 中：

```json
{
  "build": {
    "devPath": "http://localhost:3000",
    "distDir": "../dist",
    "withGlobalTauri": true
  }
}
```

### 2. 查看 Rust 日志

```rust
use log::{info, warn, error};

#[tauri::command]
pub fn power_on_device(state: State<AppState>, device: String) -> ApiResponse<String> {
    info!("Power on request for device: {}", device);
    // ... rest of code
}
```

### 3. 前端控制台日志

```typescript
PreludeAPI.powerOn('DUT1')
  .then((result) => {
    console.log('Power on result:', result);
  })
  .catch((err) => {
    console.error('Power on error:', err);
  });
```

---

## 🚀 部署

### 构建应用

```bash
# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

### Windows 安装包

构建完成后，安装包位于：
```
src-tauri/target/release/bundle/msi/your-app.msi
```

---

**文档版本**: 1.0
**最后更新**: 2026-02-27
**兼容版本**: Tauri 1.5+, Rust 1.70+
