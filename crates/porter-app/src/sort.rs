/// Sorting state for asset columns.
#[derive(Debug, Clone, Copy)]
pub enum Sort {
    /// Not currently sorted.
    None,
    /// Sorted in ascending order.
    Ascending,
    /// Sorted in descending order.
    Descending,
}
