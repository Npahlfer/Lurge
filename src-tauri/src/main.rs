// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_positioner::{Position, WindowExt};

use serde::ser::{Serialize, SerializeStruct, Serializer};
mod process_manager;
use process_manager::{find_port_by_name, process_killer, KillResult};
mod shell_commands;
use shell_commands::{
    execute_defaults_read_cmd, execute_defaults_write_cmd, relaunch_finder, DefaultsReadOption,
    DefaultsReturnType, DefaultsWriteOption,
};

enum ResponseValue {
    String(String),
    Bool(bool),
}

impl Serialize for ResponseValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseValue::String(s) => serializer.serialize_str(s),
            ResponseValue::Bool(b) => serializer.serialize_bool(*b),
        }
    }
}

struct ResponseResult<T> {
    success: bool,
    result: T,
    error: String,
}

impl<T: Serialize> Serialize for ResponseResult<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ResponseResult", 3)?;
        s.serialize_field("success", &self.success)?;
        s.serialize_field("result", &self.result)?;
        s.serialize_field("error", &self.error)?;
        s.end()
    }
}

fn read_defaults(option: DefaultsReadOption) -> ResponseResult<ResponseValue> {
    match execute_defaults_read_cmd(option) {
        DefaultsReturnType::String(r) => {
            return ResponseResult {
                success: true,
                error: "".to_string(),
                result: ResponseValue::String(r),
            }
        }
        DefaultsReturnType::Bool(r) => {
            return ResponseResult {
                success: true,
                error: "".to_string(),
                result: ResponseValue::Bool(r),
            }
        }
        DefaultsReturnType::Error(r) => {
            return ResponseResult {
                success: false,
                error: r,
                result: ResponseValue::Bool(false),
            }
        }
    }
}

fn write_defaults(option: DefaultsWriteOption) -> ResponseResult<ResponseValue> {
    match execute_defaults_write_cmd(option) {
        DefaultsReturnType::String(r) => {
            let mut error = "".to_string();
            if let Err(_) = relaunch_finder() {
                error = "Failed to relaunch Finder".to_string();
            }
            return ResponseResult {
                success: true,
                result: ResponseValue::String(r),
                error,
            };
        }
        DefaultsReturnType::Bool(r) => {
            let mut error = "".to_string();
            if let Err(_) = relaunch_finder() {
                error = "Failed to relaunch Finder".to_string();
            }
            return ResponseResult {
                success: true,
                result: ResponseValue::Bool(r),
                error,
            };
        }
        DefaultsReturnType::Error(r) => {
            return ResponseResult {
                success: false,
                error: r,
                result: ResponseValue::Bool(false),
            }
        }
    }
}

#[tauri::command]
fn handle_search(search: &str) -> ResponseResult<Vec<u16>> {
    let mut ports = find_port_by_name(search.to_string());
    ports.dedup();

    ResponseResult {
        success: true,
        result: ports,
        error: "".to_string(),
    }
}

#[tauri::command]
fn set_screenshot_directory(dir: String) -> ResponseResult<ResponseValue> {
    write_defaults(DefaultsWriteOption::ScreenshotDirectory(dir))
}

#[tauri::command]
fn set_screenshot_format(format: String) -> ResponseResult<ResponseValue> {
    write_defaults(DefaultsWriteOption::ScreenshotFormat(format))
}

#[tauri::command]
fn get_screenshot_directory() -> ResponseResult<ResponseValue> {
    read_defaults(DefaultsReadOption::ScreenshotDirectory)
}

#[tauri::command]
fn get_screenshot_format() -> ResponseResult<ResponseValue> {
    read_defaults(DefaultsReadOption::ScreenshotFormat)
}

#[tauri::command]
fn get_desktop_show() -> ResponseResult<ResponseValue> {
    read_defaults(DefaultsReadOption::CreateDesktop)
}

#[tauri::command]
fn get_desktop_hard_drives_show() -> ResponseResult<ResponseValue> {
    read_defaults(DefaultsReadOption::ShowHardDrives)
}

#[tauri::command]
fn set_desktop_show(state: bool) -> ResponseResult<ResponseValue> {
    write_defaults(DefaultsWriteOption::CreateDesktop(state))
}

#[tauri::command]
fn set_desktop_hard_drives_show(state: bool) -> ResponseResult<ResponseValue> {
    write_defaults(DefaultsWriteOption::ShowHardDrives(state))
}

#[tauri::command]
fn proc_kill(port: &str) -> KillResult {
    process_killer(port.to_string().parse::<u16>().unwrap())
}

fn main() {
    let system_tray_menu = SystemTrayMenu::new();
    tauri::Builder::default()
        .setup(|app| {
            // Remove from application switcher
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_window("main").unwrap();
            window.show().unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .system_tray(SystemTray::new().with_menu(system_tray_menu))
        .on_system_tray_event(|app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    let window = app.get_window("main").unwrap();
                    let _ = window.move_window(Position::TrayCenter);
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(is_focused) => {
                if !is_focused {
                    // event.window().hide().unwrap();
                }
            }
            // tauri::WindowEvent::CloseRequested { api, .. } => {
            //     event.window().hide().unwrap();
            //     api.prevent_close();
            // }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            proc_kill,
            handle_search,
            set_screenshot_directory,
            set_screenshot_format,
            get_screenshot_directory,
            get_screenshot_format,
            get_desktop_show,
            get_desktop_hard_drives_show,
            set_desktop_show,
            set_desktop_hard_drives_show,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
