use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use crate::ProcessError;
use crate::ProcessHandle;
use crate::ProcessHandlePlatform;
use crate::ProcessInfo;
use crate::ProcessInfoPlatform;
use crate::ProcessReader;

/// A process that exists on the local system.
#[derive(Clone, PartialEq, Eq)]
pub struct Process {
    info: ProcessInfo,
}

impl Process {
    /// Returns a collection of all running processes.
    pub fn get_processes() -> Result<Vec<Self>, ProcessError> {
        Ok(ProcessInfo::get_processes([])?
            .into_iter()
            .map(|info| Process { info })
            .collect())
    }

    /// Returns a list of processes that match the given name, without an extension, case sensitive.
    pub fn get_processes_by_name<N: AsRef<str>>(name: N) -> Result<Vec<Self>, ProcessError> {
        Ok(ProcessInfo::get_processes([])?
            .into_iter()
            .filter(|info| info.name == name.as_ref())
            .map(|info| Process { info })
            .collect())
    }

    /// Attempts to get a process by it's unique id.
    pub fn get_process_by_id<P: TryInto<u64>>(pid: P) -> Result<Self, ProcessError> {
        let pid = pid.try_into().map_err(|_| ProcessError::NotFound)?;

        ProcessInfo::get_processes([pid])?
            .into_iter()
            .next()
            .map(|info| Process { info })
            .ok_or(ProcessError::NotFound)
    }

    /// Checks whether or not the process is still alive.
    pub fn alive(&self) -> bool {
        let Some(process) = ProcessInfo::get_processes([self.pid()])
            .ok()
            .and_then(|x| x.into_iter().next())
        else {
            return false;
        };

        process.name == self.name()
            && process.pid == self.pid()
            && process.started_at == self.started_at()
    }

    /// The name of the process executable without the extension.
    pub fn name(&self) -> &str {
        &self.info.name
    }

    /// The unique process id.
    pub fn pid(&self) -> u64 {
        self.info.pid
    }

    /// Queries the full path of the process executable on disk.
    pub fn path(&self) -> Option<PathBuf> {
        self.info.get_path()
    }

    /// The time at which the process was started.
    pub fn started_at(&self) -> SystemTime {
        self.info.started_at
    }

    /// Opens the process for reading it's memory.
    pub fn open_read(&self) -> Result<ProcessReader, ProcessError> {
        ProcessHandle::open_process(self.info.pid, true, false)
            .map(Arc::new)
            .map(ProcessReader::from_handle)
    }
}

impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("pid", &self.info.pid)
            .field("name", &self.info.name)
            .field("path", &self.info.path)
            .field("started_at", &self.info.started_at)
            .finish()
    }
}
