use crate::response::ResponseResult;
use libproc::libproc::file_info::pidfdinfo;
use libproc::libproc::file_info::{ListFDs, ProcFDType};
use libproc::libproc::net_info::{SocketFDInfo, SocketInfoKind};
use libproc::libproc::proc_pid::{listpidinfo, pidinfo};
use libproc::libproc::task_info::TaskAllInfo;
use libproc::processes::{pids_by_type, ProcFilter};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use psutil::process::Process;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::ffi::CStr;

pub struct ProcessManyInfo {
    name: String,
    count: i8,
}

impl Serialize for ProcessManyInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ProcessManyInfo", 2)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("count", &self.count)?;
        s.end()
    }
}

fn get_process_info_list() -> Vec<TaskAllInfo> {
    let mut info_list = Vec::new();

    if let Ok(pids) = pids_by_type(ProcFilter::All) {
        for id in pids {
            if let Ok(t) = pidinfo::<TaskAllInfo>(id as i32, 0) {
                info_list.push(t);
            }
        }
    }

    info_list
}

pub fn collect_many_process_instances(
    min_check_value: i8,
    max_check_value: i8,
    black_list: Vec<&str>,
) -> Vec<ProcessManyInfo> {
    let info_list = get_process_info_list();
    let mut process_list: Vec<String> = Vec::new();

    for task in info_list {
        // let pid = task.pbsd.pbi_pid as i32;

        if let Ok(process) = Process::new(task.pbsd.pbi_pid) {
            if let Ok(name) = process.name() {
                process_list.push(name.clone());
            }
        }
    }

    process_list.sort();

    let mut process_many_running: Vec<ProcessManyInfo> = Vec::new();

    for (i, process) in process_list.iter().enumerate() {
        if black_list.iter().any(|b| b == process) {
            continue;
        }

        if process_many_running.iter().any(|p| &p.name == process) {
            continue;
        }

        let mut _has_pushed = false;
        let mut count = 0;

        for j in i..process_list.len() {
            count += 1;

            if j == process_list.len() - 1
                || count > max_check_value - 1
                || !_has_pushed && process_list[j] != process_list[j + 1]
            {
                if count > min_check_value {
                    process_many_running.push(ProcessManyInfo {
                        name: process.clone(),
                        count,
                    });
                }

                _has_pushed = true;
                break;
            }
        }
    }

    process_many_running
}

pub fn find_port_by_name(search_value: String) -> Vec<u16> {
    let info_list = get_process_info_list();
    let mut ports: Vec<u16> = Vec::new();

    for task in info_list {
        let pid = task.pbsd.pbi_pid as i32;

        if let Ok(process) = Process::new(task.pbsd.pbi_pid) {
            if let Ok(name) = process.name() {
                if name.starts_with(&search_value) {
                    // TODO dry this up
                    if let Ok(descriptors) =
                        listpidinfo::<ListFDs>(pid, task.pbsd.pbi_nfiles as usize)
                    {
                        for descriptor in descriptors {
                            if let ProcFDType::Socket = descriptor.proc_fdtype.into() {
                                if let Ok(socket) =
                                    pidfdinfo::<SocketFDInfo>(pid, descriptor.proc_fd)
                                {
                                    match socket.psi.soi_kind.into() {
                                        SocketInfoKind::In => {
                                            if socket.psi.soi_protocol == libc::IPPROTO_UDP {
                                                let info = unsafe { socket.psi.soi_proto.pri_in };
                                                ports.push(u16::from_be(info.insi_lport as u16));
                                            }
                                        }
                                        SocketInfoKind::Tcp => {
                                            let info = unsafe { socket.psi.soi_proto.pri_tcp };
                                            ports.push(u16::from_be(
                                                info.tcpsi_ini.insi_lport as u16,
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ports
}

pub fn process_killer(port_to_kill: u16) -> ResponseResult<()> {
    let info_list = get_process_info_list();

    for task in info_list {
        let pid = task.pbsd.pbi_pid as i32;

        // TODO dry this up
        if let Ok(descriptors) = listpidinfo::<ListFDs>(pid, task.pbsd.pbi_nfiles as usize) {
            for descriptor in descriptors {
                if let ProcFDType::Socket = descriptor.proc_fdtype.into() {
                    if let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, descriptor.proc_fd) {
                        match socket.psi.soi_kind.into() {
                            SocketInfoKind::In => {
                                if socket.psi.soi_protocol == libc::IPPROTO_UDP {
                                    let info = unsafe { socket.psi.soi_proto.pri_in };
                                    let port = u16::from_be(info.insi_lport as u16);
                                    println!("port: {}", port);
                                    if port == port_to_kill {
                                        return try_to_kill(task);
                                    }
                                }
                            }
                            SocketInfoKind::Tcp => {
                                let info = unsafe { socket.psi.soi_proto.pri_tcp };
                                let port = u16::from_be(info.tcpsi_ini.insi_lport as u16);

                                println!("port: {}", port);
                                if port == port_to_kill {
                                    return try_to_kill(task);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    ResponseResult {
        success: false,
        error: "No processes found".to_string(),
        result: (),
    }
}

fn get_command_name(task: &TaskAllInfo) -> String {
    let ptr = task.pbsd.pbi_comm.as_ptr();

    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn try_to_kill(task: TaskAllInfo) -> ResponseResult<()> {
    let command = get_command_name(&task);

    if command.starts_with("com.docker") {
        ResponseResult {
            success: false,
            error: "Docker process. Stop the container manually".to_string(),
            result: (),
        }
    } else {
        let pid = Pid::from_raw(task.pbsd.pbi_pid as i32);
        let result = kill_process(pid);
        let maybe_error = if result {
            format!("Failed to kill process {}", pid)
        } else {
            "".to_string()
        };

        ResponseResult {
            success: result,
            error: maybe_error,
            result: (),
        }
    }
}

fn kill_process(pid: Pid) -> bool {
    if signal::kill(pid, Signal::SIGKILL).is_err() {
        return false;
    }

    true
}
