/// Filter modes for image sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Nearest filtering.
    Nearest,
    /// Linear filtering.
    Linear,
}
