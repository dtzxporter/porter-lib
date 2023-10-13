use libc::*;

use crate::ProcessError;
use crate::ProcessHandle;
use crate::ProcessHandlePlatform;

impl ProcessHandlePlatform for ProcessHandle {
    fn open_process(pid: u64, _: bool, _: bool) -> Result<Self, ProcessError> {
        Ok(Self {
            handle: pid as pid_t,
            can_read: true,
            can_write: true,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ProcessError> {
        let iovec_out: iovec = iovec {
            iov_base: buf.as_mut_ptr() as *mut c_void,
            iov_len: buf.len() as size_t,
        };

        let iovec_in: iovec = iovec {
            iov_base: offset as *mut c_void,
            iov_len: buf.len() as size_t,
        };

        let read = unsafe {
            process_vm_readv(
                self.handle,
                &iovec_out as *const iovec,
                1,
                &iovec_in as *const iovec,
                1,
                0,
            )
        };

        if read > -1 {
            return Ok(read as usize);
        }

        Err(std::io::Error::last_os_error().into())
    }

    fn base_address(&self) -> Result<u64, ProcessError> {
        unimplemented!()
    }

    fn close(&mut self) {
        // Nothing, there is no open handle on linux, just the pid.
    }
}
