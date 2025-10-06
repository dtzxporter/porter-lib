use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::ops;
use std::path::Path;
use std::sync::Once;

use porter_utils::BufferWriteExt;

use curl_sys::*;

use crate::HttpClient;
use crate::HttpProgressCallback;

/// Curl handle that cleans up automatically.
#[repr(transparent)]
struct CurlHandle(pub *mut CURL);

/// Curl, and optional headers handles.
struct Request {
    curl: CurlHandle,
    headers: Option<*mut curl_slist>,
}

/// Buffer user data.
struct BufferUserdata {
    buffer: Vec<u8>,
    error: Option<io::Error>,
}

/// File user data.
struct FileUserdata {
    file: BufWriter<File>,
    error: Option<io::Error>,
}

/// Progress user data.
struct ProgressUserdata {
    is_upload: bool,
    callback: HttpProgressCallback,
}

/// Ensure that curl was initialized.
fn curl_init() {
    static CURL_INIT: Once = Once::new();

    CURL_INIT.call_once(|| {
        unsafe { curl_global_init(CURL_GLOBAL_ALL) };
    });
}

/// Downloads a request to a memory buffer.
pub fn download_memory(client: HttpClient) -> Result<(Vec<u8>, String), io::Error> {
    let (request, progress) = create_request(client)?;

    let mut userdata: BufferUserdata = BufferUserdata {
        buffer: Vec::new(),
        error: None,
    };

    unsafe {
        curl_easy_setopt(*request.curl, CURLOPT_WRITEDATA, &mut userdata);
        curl_easy_setopt(
            *request.curl,
            CURLOPT_WRITEFUNCTION,
            write_data_callback
                as for<'a> extern "C" fn(*const u8, usize, usize, &'a mut BufferUserdata) -> usize,
        )
    };

    if let Some(progress) = progress {
        let mut userdata_progress: ProgressUserdata = ProgressUserdata {
            is_upload: false,
            callback: progress,
        };

        unsafe {
            curl_easy_setopt(*request.curl, CURLOPT_NOPROGRESS, 0u32);
            curl_easy_setopt(*request.curl, CURLOPT_PROGRESSDATA, &mut userdata_progress);
            curl_easy_setopt(
                *request.curl,
                CURLOPT_PROGRESSFUNCTION,
                progress_callback
                    as for<'a> extern "C" fn(&'a mut ProgressUserdata, f64, f64, f64, f64) -> u32,
            );
        };

        make_request(&request, userdata.error)?;
    } else {
        make_request(&request, userdata.error)?;
    }

    let mut content_type: *const i8 = std::ptr::null();

    unsafe { curl_easy_getinfo(*request.curl, CURLINFO_CONTENT_TYPE, &mut content_type) };

    let content_type = if content_type.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(content_type) }
            .to_string_lossy()
            .to_lowercase()
    };

    Ok((userdata.buffer, content_type))
}

/// Downloads a request to a file buffer.
pub fn download_file(client: HttpClient, path: &Path) -> Result<(), io::Error> {
    let (request, progress) = create_request(client)?;

    let mut userdata: FileUserdata = FileUserdata {
        file: File::create(path)?.buffer_write(),
        error: None,
    };

    unsafe {
        curl_easy_setopt(*request.curl, CURLOPT_WRITEDATA, &mut userdata);
        curl_easy_setopt(
            *request.curl,
            CURLOPT_WRITEFUNCTION,
            write_file_callback
                as for<'a> extern "C" fn(*const u8, usize, usize, &'a mut FileUserdata) -> usize,
        )
    };

    if let Some(progress) = progress {
        let mut userdata_progress: ProgressUserdata = ProgressUserdata {
            is_upload: false,
            callback: progress,
        };

        unsafe {
            curl_easy_setopt(*request.curl, CURLOPT_NOPROGRESS, 0u32);
            curl_easy_setopt(*request.curl, CURLOPT_PROGRESSDATA, &mut userdata_progress);
            curl_easy_setopt(
                *request.curl,
                CURLOPT_PROGRESSFUNCTION,
                progress_callback
                    as for<'a> extern "C" fn(&'a mut ProgressUserdata, f64, f64, f64, f64) -> u32,
            )
        };

        make_request(&request, userdata.error)?;
    } else {
        make_request(&request, userdata.error)?;
    }

    userdata.file.flush()?;

    Ok(())
}

/// Creates a new curl request from the given client options.
fn create_request(
    client: HttpClient,
) -> Result<(Request, Option<HttpProgressCallback>), io::Error> {
    curl_init();

    let curl = unsafe { curl_easy_init() };

    if curl.is_null() {
        return Err(io::Error::from(io::ErrorKind::Other));
    }

    let curl = CurlHandle(curl);

    let url = CString::new(client.url)?;

    unsafe {
        curl_easy_setopt(*curl, CURLOPT_URL, url.as_ptr());
        curl_easy_setopt(*curl, CURLOPT_TIMEOUT_MS, client.timeout.unwrap_or(0));
        curl_easy_setopt(*curl, CURLOPT_CONNECTTIMEOUT_MS, client.connect_timeout);
        curl_easy_setopt(*curl, CURLOPT_FOLLOWLOCATION, 1u32);
        curl_easy_setopt(*curl, CURLOPT_FAILONERROR, 1u32)
    };

    if !client.user_agent.is_empty() {
        let user_agent = CString::new(client.user_agent)?;

        unsafe { curl_easy_setopt(*curl, CURLOPT_USERAGENT, user_agent.as_ptr()) };
    }

    let mut headers: *mut curl_slist = std::ptr::null_mut();

    if !client.accept.is_empty() {
        let accept = CString::new(client.accept)?;

        headers = unsafe { curl_slist_append(headers, accept.as_ptr()) };
    }

    if !client.content_type.is_empty() {
        let content_type = CString::new(client.content_type)?;

        headers = unsafe { curl_slist_append(headers, content_type.as_ptr()) };
    }

    if !client.authorization.is_empty() {
        let authorization = CString::new(client.authorization)?;

        headers = unsafe { curl_slist_append(headers, authorization.as_ptr()) };
    }

    if !headers.is_null() {
        unsafe { curl_easy_setopt(*curl, CURLOPT_HTTPHEADER, headers) };
    }

    Ok((
        Request {
            curl,
            headers: if headers.is_null() {
                None
            } else {
                Some(headers)
            },
        },
        client.progress,
    ))
}

/// Makes the curl request, and handles errors.
fn make_request(request: &Request, write_error: Option<io::Error>) -> Result<(), io::Error> {
    let result = unsafe { curl_easy_perform(*request.curl) };

    let error = match result {
        // Success, returned 2XX status code.
        CURLE_OK => return Ok(()),
        // Errors, making request.
        CURLE_URL_MALFORMAT => io::Error::new(io::ErrorKind::InvalidInput, "Url was malformed"),
        CURLE_OPERATION_TIMEDOUT => io::Error::from(io::ErrorKind::TimedOut),
        CURLE_COULDNT_RESOLVE_HOST => io::Error::from(io::ErrorKind::NetworkUnreachable),
        CURLE_COULDNT_CONNECT => io::Error::from(io::ErrorKind::HostUnreachable),
        CURLE_OUT_OF_MEMORY => io::Error::from(io::ErrorKind::OutOfMemory),
        CURLE_WRITE_ERROR => write_error.unwrap_or(io::Error::from(io::ErrorKind::StorageFull)),
        // Errors, returned 4XX status codes.
        CURLE_HTTP_RETURNED_ERROR | CURLE_RECV_ERROR => {
            let mut code: u32 = 0;

            unsafe { curl_easy_getinfo(*request.curl, CURLINFO_RESPONSE_CODE, &mut code) };

            match code {
                // Note: There is a bug with curl http2 and fail on error where we get a recv error instead.
                // We detect that here and return a connection error if we have not received the http status code yet.
                0 if result == CURLE_RECV_ERROR => io::Error::from(io::ErrorKind::ConnectionReset),
                // 4XX status codes.
                400 => io::Error::from(io::ErrorKind::InvalidInput),
                403 => io::Error::from(io::ErrorKind::PermissionDenied),
                404 => io::Error::from(io::ErrorKind::NotFound),
                408 => io::Error::from(io::ErrorKind::TimedOut),
                429 => io::Error::from(io::ErrorKind::QuotaExceeded),
                // 5XX status codes.
                501 => io::Error::from(io::ErrorKind::Unsupported),
                503 => io::Error::from(io::ErrorKind::ResourceBusy),
                504 => io::Error::from(io::ErrorKind::TimedOut),
                _ => io::Error::other(format!("Received {code} status code")),
            }
        }
        // Error, unknown.
        _ => {
            let message = unsafe { CStr::from_ptr(curl_easy_strerror(result)) }
                .to_string_lossy()
                .into_owned();

            let message = format!("Request failed with code: {result} {message:?}");

            io::Error::other(message)
        }
    };

    Err(error)
}

/// Progress callback.
extern "C" fn progress_callback(
    userdata: &mut ProgressUserdata,
    dltotal: f64,
    dlnow: f64,
    ultotal: f64,
    ulnow: f64,
) -> u32 {
    let (total, now) = if userdata.is_upload {
        (ultotal, ulnow)
    } else {
        (dltotal, dlnow)
    };

    let progress = if total > f64::EPSILON {
        let progress = ((now / total) * 100.0) as u32;

        Some(progress)
    } else {
        None
    };

    #[cfg(debug_assertions)]
    let start = std::time::Instant::now();

    (userdata.callback)(progress);

    #[cfg(debug_assertions)]
    if start.elapsed() > std::time::Duration::from_millis(10) {
        eprintln!("Warning: Progress callback exceeded 10ms, it should not block");
    }

    0
}

/// Write data buffer callback.
extern "C" fn write_data_callback(
    data: *const u8,
    size: usize,
    nmemb: usize,
    userdata: &mut BufferUserdata,
) -> usize {
    if data.is_null() {
        return 0;
    }

    let total_size = size * nmemb;
    let slice = unsafe { std::slice::from_raw_parts(data, total_size) };

    if let Err(error) = userdata.buffer.try_reserve(slice.len()) {
        userdata.error = Some(io::Error::from(error));
        return 0;
    }

    userdata.buffer.extend_from_slice(slice);

    total_size
}

/// Write file callback.
extern "C" fn write_file_callback(
    data: *const u8,
    size: usize,
    nmemb: usize,
    userdata: &mut FileUserdata,
) -> usize {
    if data.is_null() {
        return 0;
    }

    let total_size = size * nmemb;
    let slice = unsafe { std::slice::from_raw_parts(data, total_size) };

    if let Err(error) = userdata.file.write_all(slice) {
        userdata.error = Some(error);
        return 0;
    }

    total_size
}

impl Drop for CurlHandle {
    fn drop(&mut self) {
        unsafe { curl_easy_cleanup(self.0) };
    }
}

impl ops::Deref for CurlHandle {
    type Target = *mut CURL;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        if let Some(headers) = self.headers.take() {
            unsafe { curl_slist_free_all(headers) };
        }
    }
}
