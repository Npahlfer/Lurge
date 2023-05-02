use std::process::Command;

pub enum DefaultsReadOption {
    CreateDesktop,
    ShowHardDrives,
    ScreenshotDirectory,
    ScreenshotFormat,
}

pub enum DefaultsWriteOption {
    ScreenshotDirectory(String),
    ScreenshotFormat(String),
    CreateDesktop(bool),
    ShowHardDrives(bool),
}

pub enum DefaultsReturnType {
    String(String),
    Bool(bool),
    Error(String),
}

pub fn execute_defaults_read_cmd(option: DefaultsReadOption) -> DefaultsReturnType {
    match option {
        DefaultsReadOption::CreateDesktop => {
            execute_defaults_read("com.apple.finder", "CreateDesktop")
        }
        DefaultsReadOption::ShowHardDrives => {
            execute_defaults_read("com.apple.finder", "ShowHardDrivesOnDesktop")
        }
        DefaultsReadOption::ScreenshotDirectory => {
            execute_defaults_read("com.apple.screencapture", "location")
        }
        DefaultsReadOption::ScreenshotFormat => {
            execute_defaults_read("com.apple.screencapture", "type")
        }
    }
}

pub fn execute_defaults_write_cmd(option: DefaultsWriteOption) -> DefaultsReturnType {
    match option {
        DefaultsWriteOption::ScreenshotDirectory(dir) => {
            execute_defaults_write("com.apple.screencapture", vec!["location", dir.as_str()])
        }
        DefaultsWriteOption::ScreenshotFormat(format) => {
            execute_defaults_write("com.apple.screencapture", vec!["type", format.as_str()])
        }
        DefaultsWriteOption::CreateDesktop(should_create) => {
            let should_create_str = if should_create { "true" } else { "false" };
            execute_defaults_write(
                "com.apple.finder",
                vec!["CreateDesktop", "-bool", should_create_str],
            )
        }
        DefaultsWriteOption::ShowHardDrives(show) => {
            let show_str = if show { "true" } else { "false" };
            execute_defaults_write(
                "com.apple.finder",
                vec!["ShowHardDrivesOnDesktop", "-bool", show_str],
            )
        }
    }
}

pub fn relaunch_finder() -> Result<(), &'static str> {
    if let Err(_) = Command::new("killall").arg("Finder").status() {
        return Err("Failed to execute killall command");
    }

    Ok(())
}

pub fn execute_defaults_read(uri: &str, arg: &str) -> DefaultsReturnType {
    if let Ok(o) = Command::new("defaults").args(["read", uri, arg]).output() {
        let res = String::from_utf8_lossy(&o.stdout);
        let result = res.trim();

        if result.eq("1") || result.eq("true") {
            DefaultsReturnType::Bool(true)
        } else if result.eq("0") || result.eq("false") {
            DefaultsReturnType::Bool(false)
        } else {
            DefaultsReturnType::String(result.to_owned())
        }
    } else {
        DefaultsReturnType::Error("Failed to execute defaults read command".to_string())
    }
}

pub fn execute_defaults_write<'a>(uri: &'a str, args: Vec<&'a str>) -> DefaultsReturnType {
    if let Err(_) = Command::new("defaults")
        .args(["write", uri].iter().chain(args.iter()))
        .output()
    {
        DefaultsReturnType::Error("Failed to execute defaults write command".to_string())
    } else {
        DefaultsReturnType::Bool(true)
    }
}
