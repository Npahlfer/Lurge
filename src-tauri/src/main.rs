// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, LogicalSize, Manager, Size, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tauri_plugin_positioner::{Position, WindowExt};

use dirs::home_dir;
mod process_manager;
mod response;
mod shell_commands;
use process_manager::{
    collect_many_process_instances, find_port_by_name, process_killer, ProcessManyInfo,
};
use response::{ResponseResult, ResponseValue};
use shell_commands::{
    execute_defaults_read_cmd, execute_defaults_write_cmd, relaunch_finder, DefaultsReadOption,
    DefaultsReturnType, DefaultsWriteOption,
};

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
    let ResponseResult { result, error, .. } =
        read_defaults(DefaultsReadOption::ScreenshotDirectory);
    if error != "" {
        return ResponseResult {
            success: false,
            error,
            result: ResponseValue::String("".to_string()),
        };
    }

    if let ResponseValue::String(result) = result {
        let home = home_dir().unwrap();
        let path = result;
        if path.contains(&home.to_str().unwrap().to_string()) {
            let path = path.replace(&home.to_str().unwrap().to_string(), "~");
            return ResponseResult {
                success: true,
                error: "".to_string(),
                result: ResponseValue::String(path),
            };
        }
    }

    ResponseResult {
        success: false,
        error: "Failed to get screenshot directory".to_string(),
        result: ResponseValue::String("".to_string()),
    }
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
fn proc_kill(port: &str) -> ResponseResult<()> {
    process_killer(port.to_string().parse::<u16>().unwrap())
}

#[tauri::command]
fn get_many_processes(min_length: i8, max_length: i8) -> ResponseResult<Vec<ProcessManyInfo>> {
    let black_list = vec![
        "zsh",
        "fish",
        "bash",
        "sh",
        "tmux",
        "vim",
        "nvim",
        "code",
        "mdworker_shared",
        "CategoriesServic",
    ];

    // let max_check_value = 40;
    // let min_check_value = 10;
    let res = collect_many_process_instances(min_length, max_length, black_list);

    ResponseResult {
        success: true,
        error: "".to_string(),
        result: res,
    }
}

fn main() {
    let system_tray_menu = SystemTrayMenu::new();
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let title = CustomMenuItem::new("title".to_string(), "Lurge").disabled();
    let tray_menu = system_tray_menu
        .add_item(title)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

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
        .system_tray(SystemTray::new().with_menu(tray_menu))
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
            get_many_processes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
