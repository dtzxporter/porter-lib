use std::ffi::OsStr;
use std::ffi::OsString;
use std::ffi::c_void;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::ops;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::slice::from_raw_parts_mut;
use std::time::Duration;
use std::time::Instant;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Networking::WinHttp::*;

use crate::HttpClient;
use crate::HttpProgressCallback;

/// Internet handle that cleans up automatically.
#[repr(transparent)]
struct InternetHandle(pub *mut c_void);

/// Request, connection, and session handles.
#[allow(unused)]
struct Request {
    request: InternetHandle,
    connect: InternetHandle,
    session: InternetHandle,
}

/// Downloads a request to a memory buffer.
pub fn download_memory(client: HttpClient) -> Result<(Vec<u8>, String), io::Error> {
    let (request, mut progress, timeout) = create_request(client)?;

    let mut result: Vec<u8> = Vec::new();
    let mut temp: Vec<u8> = Vec::new();
    let mut now: f64 = 0.0;

    let start = Instant::now();

    let (content_type, content_length) = make_request(&request)?;

    while let Some(buffer) = read_request(&request, &mut temp)? {
        result.try_reserve(buffer.len())?;
        result.extend_from_slice(buffer);

        now += buffer.len() as f64;

        if let Some(callback) = &mut progress {
            #[cfg(debug_assertions)]
            let start = Instant::now();

            if content_length > f64::EPSILON {
                let progress = ((now / content_length) * 100.0) as u32;

                callback(Some(progress));
            } else {
                callback(None);
            };

            #[cfg(debug_assertions)]
            if start.elapsed() > Duration::from_millis(10) {
                eprintln!("Warning: Progress callback exceeded 10ms, it should not block");
            }
        }

        if let Some(timeout) = timeout
            && start.elapsed() > timeout
        {
            return Err(io::Error::from(io::ErrorKind::TimedOut));
        }
    }

    Ok((result, content_type))
}

/// Downloads a request to a file buffer.
pub fn download_file(client: HttpClient, path: &Path) -> Result<(), io::Error> {
    let (request, mut progress, timeout) = create_request(client)?;

    let mut file = BufWriter::new(File::create(path)?);
    let mut temp: Vec<u8> = Vec::new();
    let mut now: f64 = 0.0;

    let start = Instant::now();

    let (_, content_length) = make_request(&request)?;

    while let Some(buffer) = read_request(&request, &mut temp)? {
        file.write_all(buffer)?;

        now += buffer.len() as f64;

        if let Some(callback) = &mut progress {
            #[cfg(debug_assertions)]
            let start = Instant::now();

            if content_length > f64::EPSILON {
                let progress = ((now / content_length) * 100.0) as u32;

                callback(Some(progress));
            } else {
                callback(None);
            };

            #[cfg(debug_assertions)]
            if start.elapsed() > Duration::from_millis(10) {
                eprintln!("Warning: Progress callback exceeded 10ms, it should not block");
            }
        }

        if let Some(timeout) = timeout
            && start.elapsed() > timeout
        {
            return Err(io::Error::from(io::ErrorKind::TimedOut));
        }
    }

    Ok(())
}

/// Creates a new winhttp request from the given client options.
fn create_request(
    client: HttpClient,
) -> Result<(Request, Option<HttpProgressCallback>, Option<Duration>), io::Error> {
    let user_agent: Vec<u16> = OsString::from(client.user_agent)
        .encode_wide()
        .chain(Some(0x0))
        .collect();

    let user_agent = if user_agent.len() > 1 {
        user_agent.as_ptr()
    } else {
        std::ptr::null()
    };

    let session = unsafe {
        WinHttpOpen(
            user_agent,
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            std::ptr::null(),
            std::ptr::null(),
            0,
        )
    };

    if session.is_null() {
        return Err(handle_last_error());
    }

    let session = InternetHandle(session);

    let mut components = URL_COMPONENTS {
        dwStructSize: size_of::<URL_COMPONENTS>() as u32,
        dwSchemeLength: u32::MAX,
        dwHostNameLength: u32::MAX,
        dwUrlPathLength: u32::MAX,
        dwExtraInfoLength: u32::MAX,
        ..unsafe { std::mem::zeroed() }
    };

    let url: Vec<u16> = OsString::from(client.url)
        .encode_wide()
        .chain(Some(0x0))
        .collect();

    let crack = unsafe { WinHttpCrackUrl(url.as_ptr(), 0, 0, &mut components) };

    if crack == 0
    // We must have a hostname for a request to be valid.
        || components.lpszHostName.is_null()
        || components.dwHostNameLength == 0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Url was malformed",
        ));
    }

    let host_name_component =
        unsafe { from_raw_parts_mut(components.lpszHostName, components.dwHostNameLength as _) };

    let host_name_component: Vec<u16> = host_name_component
        .iter()
        .copied()
        .chain(Some(0x0))
        .collect();

    let connect =
        unsafe { WinHttpConnect(*session, host_name_component.as_ptr(), components.nPort, 0) };

    if connect.is_null() {
        return Err(handle_last_error());
    }

    let connect = InternetHandle(connect);

    let method: Vec<u16> = OsStr::new("GET")
        // Eventually handle custom methods.
        .encode_wide()
        .chain(Some(0x0))
        .collect();

    let path = if !components.lpszUrlPath.is_null() && components.dwUrlPathLength > 0 {
        unsafe { from_raw_parts_mut(components.lpszUrlPath, components.dwUrlPathLength as _) }
    } else {
        &mut []
    };

    let extra = if !components.lpszExtraInfo.is_null() && components.dwExtraInfoLength > 0 {
        unsafe { from_raw_parts_mut(components.lpszExtraInfo, components.dwExtraInfoLength as _) }
    } else {
        &mut []
    };

    let path: Vec<u16> = path
        .iter()
        .copied()
        .chain(extra.iter().copied())
        .chain(Some(0x0))
        .collect();

    let request = unsafe {
        WinHttpOpenRequest(
            *connect,
            method.as_ptr(),
            path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            // Only omit secure flag if the scheme is explicitly http, otherwise, https.
            if components.nScheme == WINHTTP_INTERNET_SCHEME_HTTP {
                0
            } else {
                WINHTTP_FLAG_SECURE
            },
        )
    };

    if request.is_null() {
        return Err(handle_last_error());
    }

    let request = InternetHandle(request);

    unsafe {
        WinHttpSetTimeouts(
            *request,
            client.connect_timeout as i32,
            client.connect_timeout as i32,
            client.timeout.unwrap_or(0) as i32,
            client.timeout.unwrap_or(0) as i32,
        )
    };

    let redirect_policy: u32 = WINHTTP_OPTION_REDIRECT_POLICY_ALWAYS;

    unsafe {
        WinHttpSetOption(
            *request,
            WINHTTP_OPTION_REDIRECT_POLICY,
            &redirect_policy as *const u32 as _,
            size_of_val(&redirect_policy) as _,
        )
    };

    let decompress: u32 = WINHTTP_DECOMPRESSION_FLAG_DEFLATE | WINHTTP_DECOMPRESSION_FLAG_GZIP;

    unsafe {
        WinHttpSetOption(
            *request,
            WINHTTP_OPTION_DECOMPRESSION,
            &decompress as *const u32 as _,
            size_of_val(&decompress) as _,
        )
    };

    let mut headers: Vec<String> = Vec::new();

    if !client.accept.is_empty() {
        headers.push(client.accept);
    }

    if !client.content_type.is_empty() {
        headers.push(client.content_type);
    }

    if !client.authorization.is_empty() {
        headers.push(client.authorization);
    }

    if !headers.is_empty() {
        let headers = headers.join("\r\n");
        let headers: Vec<u16> = OsString::from(headers)
            .encode_wide()
            .chain(Some(0x0))
            .collect();

        let add = unsafe {
            WinHttpAddRequestHeaders(
                *request,
                headers.as_ptr(),
                u32::MAX,
                WINHTTP_ADDREQ_FLAG_ADD | WINHTTP_ADDREQ_FLAG_REPLACE,
            )
        };

        if add == 0 {
            return Err(handle_last_error());
        }
    }

    Ok((
        Request {
            request,
            connect,
            session,
        },
        client.progress,
        client.timeout.map(|ms| Duration::from_millis(ms as _)),
    ))
}

/// Makes the winhttp request, handles errors, and extracts headers.
fn make_request(request: &Request) -> Result<(String, f64), io::Error> {
    let send = unsafe {
        WinHttpSendRequest(
            *request.request,
            std::ptr::null(),
            0,
            std::ptr::null(),
            0,
            0,
            0,
        )
    };

    if send == 0 {
        return Err(handle_last_error());
    }

    let response = unsafe { WinHttpReceiveResponse(*request.request, std::ptr::null_mut()) };

    if response == 0 {
        return Err(handle_last_error());
    }

    // Check the status code for any errors.
    let mut code: u32 = 0;
    let mut code_size: u32 = size_of_val(&code) as _;

    unsafe {
        WinHttpQueryHeaders(
            *request.request,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            std::ptr::null(),
            &mut code as *mut u32 as _,
            &mut code_size,
            std::ptr::null_mut(),
        )
    };

    // An error has occurred and we must exit.
    if code >= 400 {
        let error = match code {
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
        };

        return Err(error);
    }

    let mut content_type: [u16; 256] = [0; 256];
    let mut content_type_length: u32 = content_type.len() as u32 * 2;

    let result = unsafe {
        WinHttpQueryHeaders(
            *request.request,
            WINHTTP_QUERY_CONTENT_TYPE,
            std::ptr::null(),
            content_type.as_mut_ptr() as _,
            &mut content_type_length,
            std::ptr::null_mut(),
        )
    };

    if result == 0 {
        content_type_length = 0;
    }

    let content_type = &content_type[0..content_type_length as usize / 2];
    let content_type = OsString::from_wide(content_type)
        .to_string_lossy()
        .to_lowercase();

    let mut content_length: [u16; 64] = [0; 64];
    let mut content_length_length: u32 = content_length.len() as u32 * 2;

    let result = unsafe {
        WinHttpQueryHeaders(
            *request.request,
            WINHTTP_QUERY_CONTENT_LENGTH,
            std::ptr::null(),
            content_length.as_mut_ptr() as _,
            &mut content_length_length,
            std::ptr::null_mut(),
        )
    };

    if result == 0 {
        content_length_length = 0;
    }

    let content_length = &content_length[0..content_length_length as usize / 2];
    let content_length: f64 = OsString::from_wide(content_length)
        .to_string_lossy()
        .trim()
        .parse()
        .unwrap_or_default();

    Ok((content_type, content_length))
}

/// Reads a block of data from a request response body.
fn read_request<'a>(
    request: &Request,
    temp: &'a mut Vec<u8>,
) -> Result<Option<&'a [u8]>, io::Error> {
    let mut size: u32 = 0;

    let data = unsafe { WinHttpQueryDataAvailable(*request.request, &mut size) };
    let size = size as usize;

    if data == 0 {
        return Err(handle_last_error());
    }

    if size == 0 {
        return Ok(None);
    }

    if size > temp.len() {
        temp.try_reserve_exact(size - temp.len())?;
        temp.resize(size, 0);
    } else {
        temp.resize(size, 0);
    }

    let mut size_read: u32 = 0;

    let read = unsafe {
        WinHttpReadData(
            *request.request,
            temp.as_mut_ptr() as _,
            size as u32,
            &mut size_read,
        )
    };

    if read == 0 {
        return Err(handle_last_error());
    }

    Ok(Some(&temp[0..size_read as usize]))
}

/// Handles the last os error.
fn handle_last_error() -> io::Error {
    let error = unsafe { GetLastError() };

    match error {
        // Errors, making request.
        ERROR_WINHTTP_INVALID_URL | ERROR_WINHTTP_UNRECOGNIZED_SCHEME => {
            io::Error::new(io::ErrorKind::InvalidInput, "Url was malformed")
        }
        ERROR_WINHTTP_CANNOT_CONNECT => io::Error::from(io::ErrorKind::HostUnreachable),
        ERROR_WINHTTP_CONNECTION_ERROR => io::Error::from(io::ErrorKind::ConnectionReset),
        ERROR_WINHTTP_NAME_NOT_RESOLVED => io::Error::from(io::ErrorKind::NetworkUnreachable),
        ERROR_WINHTTP_TIMEOUT => io::Error::from(io::ErrorKind::TimedOut),
        // Error, unknown.
        _ => io::Error::from_raw_os_error(error as i32),
    }
}

impl Drop for InternetHandle {
    fn drop(&mut self) {
        unsafe { WinHttpCloseHandle(self.0) };
    }
}

impl ops::Deref for InternetHandle {
    type Target = *mut c_void;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
