use std::io::Cursor;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use libc::*;

use porter_utils::StringReadExt;
use porter_utils::StructReadExt;

use crate::ProcessError;
use crate::ProcessInfo;
use crate::ProcessInfoPlatform;

#[allow(non_camel_case_types)]
type caddr_t = *const libc::c_char;
#[allow(non_camel_case_types)]
type segsz_t = i32;
#[allow(non_camel_case_types)]
type user_addr_t = u64;
#[allow(non_camel_case_types)]
type boolean_t = i32;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct kinfo_proc {
    pub kp_proc: extern_proc,
    pub kp_eproc: kinfo_proc_eproc,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct extern_proc {
    pub p_un: p_un,
    pub p_vmspace: user_addr_t,
    pub p_sigacts: user_addr_t,
    pub p_flag: libc::c_int,
    pub p_stat: libc::c_char,
    pub p_pid: libc::pid_t,
    pub p_oppid: libc::pid_t,
    pub p_dupfd: libc::c_int,
    pub user_stack: caddr_t,
    pub exit_thread: *mut libc::c_void,
    pub p_debugger: libc::c_int,
    pub sigwait: boolean_t,
    pub p_estcpu: libc::c_uint,
    pub p_cpticks: libc::c_int,
    pub p_pctcpu: u32,
    pub p_wchan: *mut libc::c_void,
    pub p_wmesg: *mut libc::c_char,
    pub p_swtime: libc::c_uint,
    pub p_slptime: libc::c_uint,
    pub p_realtimer: libc::itimerval,
    pub p_rtime: libc::timeval,
    pub p_uticks: u64,
    pub p_sticks: u64,
    pub p_iticks: u64,
    pub p_traceflag: libc::c_int,
    pub p_tracep: *mut libc::c_void,
    pub p_siglist: libc::c_int,
    pub p_textvp: *mut libc::c_void,
    pub p_holdcnt: libc::c_int,
    pub p_sigmask: libc::sigset_t,
    pub p_sigignore: libc::sigset_t,
    pub p_sigcatch: libc::sigset_t,
    pub p_priority: libc::c_uchar,
    pub p_usrpri: libc::c_uchar,
    pub p_nice: libc::c_char,
    pub p_comm: [libc::c_char; 17],
    pub p_pgrp: *mut libc::c_void,
    pub p_addr: *mut libc::c_void,
    pub p_xstat: libc::c_ushort,
    pub p_acflag: libc::c_ushort,
    pub p_ru: *mut libc::rusage,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub union p_un {
    pub p_st1: run_sleep_queue,
    pub p_starttime: libc::timeval,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct run_sleep_queue {
    p_forw: user_addr_t,
    p_back: user_addr_t,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct kinfo_proc_eproc {
    pub e_paddr: *mut libc::c_void,
    pub e_sess: *mut libc::c_void,
    pub e_pcred: pcred,
    pub e_ucred: libc::xucred,
    pub e_vm: vmspace,
    pub e_ppid: libc::pid_t,
    pub e_pgid: libc::pid_t,
    pub e_jobc: libc::c_short,
    pub e_tdev: libc::dev_t,
    pub e_tpgid: libc::pid_t,
    pub e_tsess: *mut libc::c_void,
    pub e_wmesg: [libc::c_char; 8],
    pub e_xsize: segsz_t,
    pub e_xrssize: libc::c_short,
    pub e_xccount: libc::c_short,
    pub e_xswrss: libc::c_short,
    pub e_flag: i32,
    pub e_login: [libc::c_char; 12],
    pub e_spare: [i32; 4],
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct pcred {
    pub pc_lock: [libc::c_char; 72],
    pub pc_ucred: *mut libc::xucred,
    pub p_ruid: libc::uid_t,
    pub p_svuid: libc::uid_t,
    pub p_rgid: libc::gid_t,
    pub p_svgid: libc::gid_t,
    pub p_refcnt: libc::c_int,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub struct vmspace {
    pub dummy: i32,
    pub dummy2: caddr_t,
    pub dummy3: [i32; 5],
    pub dummy4: [caddr_t; 3],
}

/// Utility to convert `timeval` to `SystemTime`.
fn timeval_to_systime(timeval: &timeval) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::new(timeval.tv_sec as u64, timeval.tv_usec as u32 * 1000)
}

impl ProcessInfoPlatform for ProcessInfo {
    fn get_processes<F: AsRef<[u64]>>(filter: F) -> Result<Vec<Self>, ProcessError> {
        let filter = filter.as_ref();

        let mut required: size_t = 0;
        let mut process_info_buffer: Vec<u8> = Vec::new();

        let mut name: [c_int; 3] = [CTL_KERN, KERN_PROC, KERN_PROC_ALL];

        loop {
            unsafe {
                sysctl(
                    name.as_mut_ptr(),
                    name.len() as c_uint,
                    std::ptr::null_mut(),
                    &mut required as *mut size_t,
                    std::ptr::null_mut(),
                    0,
                )
            };

            process_info_buffer.resize(required, 0);

            let status = unsafe {
                sysctl(
                    name.as_mut_ptr(),
                    name.len() as c_uint,
                    process_info_buffer.as_mut_ptr() as *mut c_void,
                    &mut required as *mut size_t,
                    std::ptr::null_mut(),
                    0,
                )
            };

            if status == 0 {
                break;
            }
        }

        let mut result = Vec::with_capacity(if filter.is_empty() { 256 } else { filter.len() });

        for info in process_info_buffer.chunks_exact(size_of::<kinfo_proc>()) {
            let kinfo: kinfo_proc = Cursor::new(info).read_struct()?;

            if !filter.is_empty() && !filter.contains(&(kinfo.kp_proc.p_pid as u64)) {
                continue;
            }

            let mut buffer: [u8; 1024] = [0; 1024];

            let (name, path) = if unsafe {
                proc_pidpath(
                    kinfo.kp_proc.p_pid,
                    buffer.as_mut_ptr() as _,
                    buffer.len() as _,
                )
            } != 0
            {
                let name_and_path = PathBuf::from((&buffer[0..]).read_null_terminated_string()?);

                let name = name_and_path
                    .file_stem()
                    .map(|x| x.to_string_lossy().into_owned())
                    .unwrap_or(format!("Process_{}", kinfo.kp_proc.p_pid));

                (name, Some(name_and_path))
            } else if kinfo.kp_proc.p_pid == 0 {
                (String::from("kernel_task"), None)
            } else {
                (format!("Process_{}", kinfo.kp_proc.p_pid), None)
            };

            result.push(ProcessInfo {
                pid: kinfo.kp_proc.p_pid as u64,
                name,
                path,
                started_at: timeval_to_systime(unsafe { &kinfo.kp_proc.p_un.p_starttime }),
            });

            if !filter.is_empty() && result.len() == filter.len() {
                break;
            }
        }

        Ok(result)
    }

    fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
}
