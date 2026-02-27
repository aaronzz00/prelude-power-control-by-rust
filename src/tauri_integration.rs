use crate::power::{DeviceSide, PowerController, WireMode};
use std::sync::Mutex;

// This struct would be managed via Tauri state:
// tauri::Builder::default().manage(PowerState::default())
#[derive(Default)]
pub struct PowerState {
    pub controller: Mutex<Option<PowerController>>,
}

// ----------------------------------------------------------------------------
// Tauri Command Examples
// ----------------------------------------------------------------------------
//
// To use these in your Tauri app, you would register them like this:
// tauri::Builder::default()
//     .manage(PowerState::default())
//     .invoke_handler(tauri::generate_handler![
//         init_device,
//         power_on_cmd,
//         power_off_cmd,
//         reset_device_cmd
//     ])
//     .run(tauri::generate_context!())
//     .expect("error while running tauri application");

/*
// Uncomment `#[tauri::command]` if compiling with `tauri` feature!
// Currently commented block because `tauri` is not tightly coupled to this pure lib crate.

#[tauri::command]
pub fn init_device(state: tauri::State<'_, PowerState>, port_name: String, is_double_wire: bool) -> Result<String, String> {
    let mode = if is_double_wire {
        WireMode::DoubleWire
    } else {
        WireMode::SingleWire
    };

    match PowerController::connect(&port_name, mode) {
        Ok(controller) => {
            let mut managed_controller = state.controller.lock().unwrap();
            *managed_controller = Some(controller);
            Ok(format!("Connected to `{}` in {:?} mode", port_name, mode))
        }
        Err(e) => Err(format!("Failed to connect: {}", e)),
    }
}

#[tauri::command]
pub fn power_on_cmd(state: tauri::State<'_, PowerState>, side: String) -> Result<String, String> {
    let mut managed = state.controller.lock().unwrap();
    if let Some(ref mut controller) = *managed {
        let target = parse_side(&side)?;
        controller.power_on(target).map_err(|e| e.to_string())?;
        Ok(format!("Powered ON: {}", side))
    } else {
        Err("Device not initialized".into())
    }
}

#[tauri::command]
pub fn power_off_cmd(state: tauri::State<'_, PowerState>, side: String) -> Result<String, String> {
    let mut managed = state.controller.lock().unwrap();
    if let Some(ref mut controller) = *managed {
        let target = parse_side(&side)?;
        controller.power_off(target).map_err(|e| e.to_string())?;
        Ok(format!("Powered OFF: {}", side))
    } else {
        Err("Device not initialized".into())
    }
}

#[tauri::command]
pub fn reset_device_cmd(state: tauri::State<'_, PowerState>, side: String) -> Result<String, String> {
    let mut managed = state.controller.lock().unwrap();
    if let Some(ref mut controller) = *managed {
        let target = parse_side(&side)?;
        controller.reset(target).map_err(|e| e.to_string())?;
        Ok(format!("Reset pulse sent: {}", side))
    } else {
        Err("Device not initialized".into())
    }
}

// Internal Helper
fn parse_side(side_str: &str) -> Result<DeviceSide, String> {
    match side_str.to_lowercase().as_str() {
        "device1" | "1" => Ok(DeviceSide::Device1),
        "device2" | "2" => Ok(DeviceSide::Device2),
        "both"    | "all" => Ok(DeviceSide::Both),
        _ => Err(format!("Invalid device side: {}", side_str)),
    }
}
*/
