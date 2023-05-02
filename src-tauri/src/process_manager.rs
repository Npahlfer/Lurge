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
// use serde::{Deserialize, Serialize as SerdeSerialize};
use std::ffi::CStr;

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

pub struct KillResult {
    success: bool,
    error: String,
}

impl Serialize for KillResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("KillResult", 2)?;
        s.serialize_field("success", &self.success)?;
        s.serialize_field("error", &self.error)?;
        s.end()
    }
}

// fn do_on_socket_kind(task: TaskAllInfo, callback: &dyn Fn(), udp_callback: &dyn Fn())

pub fn find_port_by_name(search_value: String) -> Vec<u16> {
    let info_list = get_process_info_list();
    let mut ports: Vec<u16> = Vec::new();

    for task in info_list {
        let pid = task.pbsd.pbi_pid as i32;

        if let Ok(process) = Process::new(task.pbsd.pbi_pid) {
            if let Ok(name) = process.name() {
                if name.starts_with(&search_value) {
                    // TODO dry this up
                    println!("found process: {}", name);
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

pub fn process_killer(port_to_kill: u16) -> KillResult {
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
                        };
                    }
                }
            }
        }
    }

    KillResult {
        success: false,
        error: "No processes found".to_string(),
    }
}

fn get_command_name(task: &TaskAllInfo) -> String {
    let ptr = task.pbsd.pbi_comm.as_ptr();

    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn try_to_kill(task: TaskAllInfo) -> KillResult {
    let command = get_command_name(&task);
    // let command = if let Some(pbi_comm) = task.pbsd.pbi_comm.as_ref() {
    //     let bytes = unsafe { std::slice::from_raw_parts(pbi_comm, libc::MAXCOMLEN) };
    //     String::from_utf8_lossy(bytes).to_string()
    // } else {
    //     String::new()
    // };

    if command.starts_with("com.docker") {
        return KillResult {
            success: false,
            error: "Docker process. Stop the container manually".to_string(),
        };
    } else {
        let pid = Pid::from_raw(task.pbsd.pbi_pid as i32);
        let result = kill_process(pid);
        let maybe_error = if result {
            format!("Failed to kill process {}", pid)
        } else {
            "".to_string()
        };

        return KillResult {
            success: result,
            error: maybe_error,
        };
    }
}

fn kill_process(pid: Pid) -> bool {
    if let Err(_) = signal::kill(pid, Signal::SIGKILL) {
        return false;
    }

    true
}
