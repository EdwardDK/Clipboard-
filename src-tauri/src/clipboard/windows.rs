use windows_sys::Win32::{
    Foundation::HWND,
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
};

#[derive(Default)]
pub struct CaptureContext {
    pub source_application: Option<String>,
    pub window_title: Option<String>,
}

pub fn foreground_context() -> CaptureContext {
    unsafe {
        let window: HWND = GetForegroundWindow();
        if window.is_null() {
            return CaptureContext::default();
        }
        let mut title = [0u16; 512];
        let length = GetWindowTextW(window, title.as_mut_ptr(), title.len() as i32);
        let window_title =
            (length > 0).then(|| String::from_utf16_lossy(&title[..length as usize]));
        let mut process_id = 0;
        GetWindowThreadProcessId(window, &mut process_id);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if process.is_null() {
            return CaptureContext {
                source_application: None,
                window_title,
            };
        }
        let mut path = [0u16; 32768];
        let mut size = path.len() as u32;
        let source_application =
            if QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut size) != 0 {
                std::path::Path::new(&String::from_utf16_lossy(&path[..size as usize]))
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            } else {
                None
            };
        windows_sys::Win32::Foundation::CloseHandle(process);
        CaptureContext {
            source_application,
            window_title,
        }
    }
}
