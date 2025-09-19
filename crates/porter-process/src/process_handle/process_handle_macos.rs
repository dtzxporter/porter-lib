use libc::*;

use mach2::mach_port::mach_port_deallocate;
use mach2::task;
use mach2::task_info::*;
use mach2::traps::mach_task_self;
use mach2::vm::*;

use crate::ProcessError;
use crate::ProcessHandle;
use crate::ProcessHandlePlatform;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy)]
pub struct task_vm_info {
    pub virtual_size: mach_vm_size_t,
    pub region_count: integer_t,
    pub page_size: integer_t,
    pub resident_size: mach_vm_size_t,
    pub resident_size_peak: mach_vm_size_t,
    pub device: mach_vm_size_t,
    pub device_peak: mach_vm_size_t,
    pub internal: mach_vm_size_t,
    pub internal_peak: mach_vm_size_t,
    pub external: mach_vm_size_t,
    pub external_peak: mach_vm_size_t,
    pub reusable: mach_vm_size_t,
    pub reusable_peak: mach_vm_size_t,
    pub purgeable_volatile_pmap: mach_vm_size_t,
    pub purgeable_volatile_resident: mach_vm_size_t,
    pub purgeable_volatile_virtual: mach_vm_size_t,
    pub compressed: mach_vm_size_t,
    pub compressed_peak: mach_vm_size_t,
    pub compressed_lifetime: mach_vm_size_t,
    pub phys_footprint: mach_vm_size_t,
    pub min_address: mach_vm_address_t,
    pub max_address: mach_vm_address_t,
}

impl ProcessHandlePlatform for ProcessHandle {
    fn open_process(pid: u64, _: bool, _: bool) -> Result<Self, ProcessError> {
        let mut handle: mach_port_t = 0;

        let result = unsafe {
            task_for_pid(
                mach_task_self(),
                pid as c_int,
                &mut handle as *mut mach_port_t,
            )
        };

        if result == KERN_SUCCESS {
            return Ok(Self {
                handle,
                can_read: true,
                can_write: true,
            });
        }

        Err(std::io::Error::last_os_error().into())
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ProcessError> {
        if !self.can_read() {
            return Err(ProcessError::AccessDenied);
        }

        let mut size_read: mach_vm_size_t = 0;

        let result = unsafe {
            mach_vm_read_overwrite(
                self.handle,
                offset as mach_vm_address_t,
                buf.len() as mach_vm_size_t,
                buf.as_mut_ptr() as mach_vm_address_t,
                &mut size_read as *mut mach_vm_size_t,
            )
        };

        if result != KERN_SUCCESS && size_read == 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(size_read as usize)
    }

    fn base_address(&self) -> Result<u64, ProcessError> {
        let mut vm_info: task_vm_info = task_vm_info::default();
        let mut count: mach_msg_type_number_t =
            (size_of::<task_vm_info>() / size_of::<natural_t>()) as mach_msg_type_number_t;

        let result = unsafe {
            task::task_info(
                self.handle,
                TASK_VM_INFO,
                &mut vm_info as *mut task_vm_info as _,
                &mut count as *mut mach_msg_type_number_t,
            )
        };

        if result != KERN_SUCCESS {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(vm_info.min_address)
    }

    fn main_module_size(&self) -> Result<u64, ProcessError> {
        unimplemented!()
    }

    fn close(&mut self) {
        unsafe { mach_port_deallocate(mach_task_self(), self.handle) };
    }
}
