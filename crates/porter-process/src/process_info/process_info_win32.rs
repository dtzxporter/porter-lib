use std::ffi::c_void;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::WindowsProgramming::*;

use widestring::U16CStr;

use porter_utils::StructReadExt;

use crate::ProcessError;
use crate::ProcessInfo;
use crate::ProcessInfoPlatform;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct ReservedInfo {
    reserved: [u64; 3],
    created_at: u64,
    user_time: u64,
    kernel_time: u64,
}

/// Utility to convert creation time to system time.
fn create_time_to_sys_time(create_time: u64) -> SystemTime {
    let seconds = create_time / 10_000_000;
    let nanoseconds = ((create_time % 10_000_000) * 100) as u32;

    SystemTime::UNIX_EPOCH + Duration::new(seconds, nanoseconds)
}

impl ProcessInfoPlatform for ProcessInfo {
    fn get_processes<F: AsRef<[u64]>>(filter: F) -> Result<Vec<Self>, ProcessError> {
        let filter = filter.as_ref();

        let mut required: u32 = 0;
        let mut process_info_buffer: Vec<u8> = Vec::new();

        loop {
            let status = unsafe {
                NtQuerySystemInformation(
                    SystemProcessInformation,
                    process_info_buffer.as_mut_ptr() as *mut c_void,
                    required,
                    &mut required as *mut u32,
                )
            };

            if status != STATUS_INFO_LENGTH_MISMATCH {
                break;
            }

            process_info_buffer.resize(required as usize, 0);
        }

        let mut reader = Cursor::new(process_info_buffer);
        let mut result = Vec::with_capacity(256);

        loop {
            let sys_process_info = SYSTEM_PROCESS_INFORMATION::from_io_read(&mut reader)?;
            let sys_process_next = sys_process_info.NextEntryOffset as i64
                - std::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as i64;

            if !filter.is_empty() && !filter.contains(&(sys_process_info.UniqueProcessId as u64)) {
                if sys_process_info.NextEntryOffset == 0 {
                    break;
                }

                reader.seek(SeekFrom::Current(sys_process_next))?;

                continue;
            }

            let (name, path) = if sys_process_info.ImageName.Buffer.is_null() {
                if sys_process_info.UniqueProcessId == 4 {
                    (String::from("System"), None)
                } else if sys_process_info.UniqueProcessId == 0 {
                    (String::from("Idle"), None)
                } else {
                    (
                        format!("Process_{}", sys_process_info.UniqueProcessId),
                        None,
                    )
                }
            } else {
                let wcstr = unsafe {
                    U16CStr::from_ptr_mut(
                        sys_process_info.ImageName.Buffer,
                        sys_process_info.ImageName.Length as usize / 2,
                    )?
                };

                let name_and_path = PathBuf::from(wcstr.to_string_lossy());

                let name = name_and_path
                    .file_stem()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or(format!("Process_{}", sys_process_info.UniqueProcessId));

                let path = name_and_path.parent().map(|x| x.to_path_buf());

                (name, path)
            };

            let reserve = ReservedInfo::from_byte_slice(sys_process_info.Reserved1)?;

            result.push(ProcessInfo {
                pid: sys_process_info.UniqueProcessId as u64,
                name,
                path,
                started_at: create_time_to_sys_time(reserve.created_at),
            });

            if sys_process_info.NextEntryOffset == 0 {
                break;
            }

            reader.seek(SeekFrom::Current(sys_process_next))?;
        }

        Ok(result)
    }
}
