/// Used to properly configure dialog windows parents.
pub trait PorterSetParentWindows {
    /// Sets the proper window parent on the windows platform.
    fn set_parent_windows(self) -> Self;
}

#[cfg(target_os = "windows")]
mod win32 {
    use raw_window_handle::HasRawWindowHandle;
    use raw_window_handle::RawWindowHandle;
    use raw_window_handle::Win32WindowHandle;

    use rfd::FileDialog;
    use rfd::MessageDialog;

    use super::PorterSetParentWindows;

    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;

    struct RawHandle(pub Win32WindowHandle);

    unsafe impl HasRawWindowHandle for RawHandle {
        fn raw_window_handle(&self) -> RawWindowHandle {
            RawWindowHandle::Win32(self.0)
        }
    }

    impl PorterSetParentWindows for FileDialog {
        fn set_parent_windows(self) -> Self {
            let hwnd = unsafe { GetActiveWindow() };
            let mut window_handle = Win32WindowHandle::empty();

            window_handle.hwnd = hwnd as *mut std::ffi::c_void;

            self.set_parent(&RawHandle(window_handle))
        }
    }

    impl PorterSetParentWindows for MessageDialog {
        fn set_parent_windows(self) -> Self {
            let hwnd = unsafe { GetActiveWindow() };
            let mut window_handle = Win32WindowHandle::empty();

            window_handle.hwnd = hwnd as *mut std::ffi::c_void;

            self.set_parent(&RawHandle(window_handle))
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod other {
    use rfd::FileDialog;
    use rfd::MessageDialog;

    use super::PorterSetParentWindows;

    impl PorterSetParentWindows for FileDialog {
        fn set_parent_windows(self) -> Self {
            self
        }
    }

    impl PorterSetParentWindows for MessageDialog {
        fn set_parent_windows(self) -> Self {
            self
        }
    }
}
