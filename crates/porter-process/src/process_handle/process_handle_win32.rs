use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Diagnostics::Debug::*;
use windows_sys::Win32::System::ProcessStatus::*;
use windows_sys::Win32::System::Threading::*;

use crate::ProcessError;
use crate::ProcessHandle;
use crate::ProcessHandlePlatform;

impl ProcessHandlePlatform for ProcessHandle {
    fn open_process(pid: u64, read: bool, write: bool) -> Result<Self, ProcessError> {
        let mut access: PROCESS_ACCESS_RIGHTS =
            PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE;

        if read {
            access |= PROCESS_VM_READ;
        }

        if write {
            access |= PROCESS_VM_WRITE;
        }

        let result: HANDLE = unsafe { OpenProcess(access, FALSE, pid as _) };

        if result.is_null() {
            match unsafe { GetLastError() } {
                ERROR_INVALID_PARAMETER => return Err(ProcessError::NotFound),
                ERROR_ACCESS_DENIED => return Err(ProcessError::AccessDenied),
                _ => return Err(std::io::Error::last_os_error().into()),
            }
        }

        Ok(ProcessHandle {
            can_read: read,
            can_write: write,
            handle: result,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ProcessError> {
        if !self.can_read() {
            return Err(ProcessError::AccessDenied);
        }

        let mut size_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                offset as _,
                buf.as_mut_ptr() as _,
                buf.len(),
                &mut size_read,
            )
        };

        if result == 0 {
            match unsafe { GetLastError() } {
                ERROR_INVALID_PARAMETER => return Err(ProcessError::NotFound),
                ERROR_ACCESS_DENIED => return Err(ProcessError::AccessDenied),
                ERROR_PARTIAL_COPY => {
                    // Nothing, size read was size read.
                }
                _ => return Err(std::io::Error::last_os_error().into()),
            }
        }

        Ok(size_read)
    }

    fn base_address(&self) -> Result<u64, ProcessError> {
        let mut modules: [HMODULE; 256] = [std::ptr::null_mut(); 256];
        let mut size_needed: u32 = 0;

        let result = unsafe {
            EnumProcessModules(
                self.handle,
                modules.as_mut_ptr(),
                size_of_val(&modules) as _,
                &mut size_needed,
            )
        };

        if result == 0 {
            match unsafe { GetLastError() } {
                ERROR_INVALID_PARAMETER => return Err(ProcessError::NotFound),
                ERROR_ACCESS_DENIED => return Err(ProcessError::AccessDenied),
                _ => return Err(std::io::Error::last_os_error().into()),
            }
        }

        Ok(modules[0] as _)
    }

    fn main_module_size(&self) -> Result<u64, ProcessError> {
        let mut modules: [HMODULE; 256] = [std::ptr::null_mut(); 256];
        let mut size_needed: u32 = 0;

        let result = unsafe {
            EnumProcessModules(
                self.handle,
                modules.as_mut_ptr(),
                size_of_val(&modules) as _,
                &mut size_needed,
            )
        };

        if result == 0 {
            match unsafe { GetLastError() } {
                ERROR_INVALID_PARAMETER => return Err(ProcessError::NotFound),
                ERROR_ACCESS_DENIED => return Err(ProcessError::AccessDenied),
                _ => return Err(std::io::Error::last_os_error().into()),
            }
        }

        let mut module_info: MODULEINFO = unsafe { std::mem::zeroed() };

        let result = unsafe {
            GetModuleInformation(
                self.handle,
                modules[0],
                &mut module_info,
                size_of_val(&module_info) as _,
            )
        };

        if result == 0 {
            match unsafe { GetLastError() } {
                ERROR_INVALID_PARAMETER => return Err(ProcessError::NotFound),
                ERROR_ACCESS_DENIED => return Err(ProcessError::AccessDenied),
                _ => return Err(std::io::Error::last_os_error().into()),
            }
        }

        Ok(module_info.SizeOfImage as _)
    }

    fn close(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}
