use std::io;
use std::path::Path;

use crate::http;

/// Default connect timeout.
const CONNECT_TIMEOUT_MS: u32 = 300_000;
/// Default request user agent.
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

/// Progress callback type.
pub(crate) type HttpProgressCallback = Box<dyn FnMut(Option<u32>)>;

/// Simple http client for making basic requests, downloading files, etc.
pub struct HttpClient {
    pub(crate) url: String,
    pub(crate) timeout: Option<u32>,
    pub(crate) connect_timeout: u32,
    pub(crate) user_agent: String,
    pub(crate) accept: String,
    pub(crate) content_type: String,
    pub(crate) authorization: String,
    pub(crate) progress: Option<HttpProgressCallback>,
}

impl HttpClient {
    /// Construct a new instance of [HttpClient] for the given request url.
    #[must_use]
    pub fn new<U: Into<String>>(url: U) -> Self {
        Self {
            url: url.into(),
            timeout: None,
            connect_timeout: CONNECT_TIMEOUT_MS,
            user_agent: String::from(USER_AGENT),
            accept: String::new(),
            content_type: String::new(),
            authorization: String::new(),
            progress: None,
        }
    }

    /// Sets the total time until this request will timeout in milliseconds.
    ///
    /// The default is no timeout, a value of `0` will also mean no timeout.
    #[must_use]
    pub const fn timeout(mut self, ms: u32) -> Self {
        self.timeout = if ms == 0 { None } else { Some(ms) };
        self
    }

    /// Sets the time this request can spend connecting to the host before it will timeout.
    ///
    /// The default value is `300` seconds, the minimum value is `1` millisecond.
    #[must_use]
    pub const fn connect_timeout(mut self, ms: u32) -> Self {
        self.connect_timeout = if ms == 0 { 1 } else { ms };
        self
    }

    /// Sets a custom user-agent for this request.
    #[must_use]
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Sets the content type we want to accept as a response for this request.
    #[must_use]
    pub fn accept<A: AsRef<str>>(mut self, accept: A) -> Self {
        self.accept = format!("Accept: {}", accept.as_ref());
        self
    }

    /// Sets the content type of this requests body.
    #[must_use]
    pub fn content_type<C: AsRef<str>>(mut self, content_type: C) -> Self {
        self.content_type = format!("Content-type: {}", content_type.as_ref());
        self
    }

    /// Sets the authorization data for this request.
    #[must_use]
    pub fn authorization<A: AsRef<str>>(mut self, authorization: A) -> Self {
        self.authorization = format!("Authorization: {}", authorization.as_ref());
        self
    }

    /// Sets a progress callback for this request.
    #[must_use]
    pub fn on_progress<C: FnMut(Option<u32>) + 'static>(mut self, callback: C) -> Self {
        self.progress = Some(Box::new(callback));
        self
    }

    /// Starts a request and gets the response body as a string.
    pub fn download_string(self) -> Result<String, io::Error> {
        let (buffer, content_type) = http::download_memory(self)?;

        // The web is complicated and it turns out that trial and error powers most of the text decoding on the web.
        // We'll try and use content-type hints if they are available, fallback to utf-8, then latin1 encoding if not.
        if content_type.contains("charset=iso-8859-1") {
            Ok(buffer.into_iter().map(|latin1| latin1 as char).collect())
        } else if content_type.contains("charset=utf-8") {
            String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            match String::from_utf8(buffer) {
                Ok(result) => Ok(result),
                Err(error) => Ok(error
                    .into_bytes()
                    .into_iter()
                    .map(|latin1| latin1 as char)
                    .collect()),
            }
        }
    }

    /// Starts a request and gets the response body as a vector of bytes.
    pub fn download_bytes(self) -> Result<Vec<u8>, io::Error> {
        let (buffer, _) = http::download_memory(self)?;

        Ok(buffer)
    }

    /// Starts a request and writes the response body to a file.
    ///
    /// This will `overwrite` files that already exist.
    pub fn download_file<P: AsRef<Path>>(self, path: P) -> Result<(), io::Error> {
        http::download_file(self, path.as_ref())
    }
}
