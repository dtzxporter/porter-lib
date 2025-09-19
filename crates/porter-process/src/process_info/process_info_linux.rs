use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use libc::*;

use procfs::current_system_info;
use procfs::process::*;

use crate::ProcessError;
use crate::ProcessInfo;
use crate::ProcessInfoPlatform;

impl ProcessInfoPlatform for ProcessInfo {
    fn get_processes<F: AsRef<[u64]>>(filter: F) -> Result<Vec<Self>, ProcessError> {
        let filter = filter.as_ref();
        let system_info = current_system_info();
        let system_boot_secs = system_info.boot_time_secs().unwrap_or(0);
        let system_ticks_per_sec = system_info.ticks_per_second();

        let processes: Vec<_> = if filter.is_empty() {
            all_processes()?.filter_map(|x| x.ok()).collect()
        } else {
            filter
                .iter()
                .map(|x| Process::new(*x as pid_t))
                .filter_map(|x| x.ok())
                .collect()
        };

        let mut result = Vec::with_capacity(processes.len());

        for process in processes {
            let (name, path) =
                if let Ok(Some(name)) = process.cmdline().map(|x| x.into_iter().next()) {
                    let name_and_path = PathBuf::from(name);

                    let name = name_and_path
                        .file_stem()
                        .map(|x| x.to_string_lossy().into_owned())
                        .unwrap_or(format!("Process_{}", process.pid()));

                    (name, Some(name_and_path))
                } else {
                    (format!("Process_{}", process.pid()), None)
                };

            let start_time = process.stat().map(|x| x.starttime).unwrap_or(0);
            let start_time = start_time / system_ticks_per_sec;

            result.push(ProcessInfo {
                name,
                path,
                pid: process.pid() as u64,
                started_at: SystemTime::UNIX_EPOCH
                    + Duration::from_secs(system_boot_secs + start_time),
            });
        }

        Ok(result)
    }

    fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
}
