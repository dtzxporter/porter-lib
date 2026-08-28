/// An in-flight asset preview request.
#[derive(Debug, Clone, Copy)]
pub struct AssetPreviewRequest {
    /// The index of the asset to preview.
    index: usize,
    /// Whether or not to force raw file preview.
    raw: bool,
    /// Unique request id.
    request_id: u64,
}

impl AssetPreviewRequest {
    /// Constructs a new asset preview request.
    pub const fn new(index: usize, raw: bool) -> Self {
        Self {
            index,
            raw,
            request_id: 0,
        }
    }

    /// Sets up the next preview request.
    pub const fn next_request(&mut self, index: usize, raw: bool) {
        self.index = index;
        self.raw = raw;
        self.request_id += 1;
    }

    /// Gets the preview request asset index.
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Gets whether or not the preview request is for the raw asset.
    pub const fn raw(&self) -> bool {
        self.raw
    }

    /// Gets the preview request id.
    pub const fn request_id(&self) -> u64 {
        self.request_id
    }
}
