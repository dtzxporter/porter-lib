use crate::ProcessError;

/// Shared platform process info trait.
pub trait ProcessInfoPlatform
where
    Self: Sized,
{
    /// Gets a list of processes on the current system, filtering by an optional list of pids.
    fn get_processes<F: AsRef<[u64]>>(filter: F) -> Result<Vec<Self>, ProcessError>;
}
