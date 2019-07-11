use crate::{
    WebViewFFI,
    webview_dialog,
    DialogFlags,
    DialogType,
};
use crate::error::WVResult;
use std::ffi::{
    CStr,
    CString,
};
use std::path::PathBuf;

const STR_BUF_SIZE: usize = 4096;

/// A builder for opening a new dialog window.
// #[derive(Debug)]
pub struct DialogBuilder {
    ffi: *mut WebViewFFI,
}

impl DialogBuilder {
    /// Creates a new dialog builder for a WebView.
    pub fn new(ffi: *mut WebViewFFI) -> DialogBuilder {
        DialogBuilder { ffi }
    }

    fn dialog(&mut self, title: String, arg: String, diag_type: DialogType, diag_flags: DialogFlags) -> WVResult<String> {
        let mut s = [0u8; STR_BUF_SIZE];
        let title_cstr = CString::new(title)?;
        let arg_cstr = CString::new(arg)?;

        unsafe {
            webview_dialog(
                self.ffi,
                diag_type,
                diag_flags,
                title_cstr.as_ptr(),
                arg_cstr.as_ptr(),
                s.as_mut_ptr() as _,
                s.len(),
            );
        }

        Ok(read_str(&s))
    }

    /// Opens a new open file dialog and returns the chosen file path.
    pub fn open_file<S, P>(&mut self, title: S, default_file: P) -> WVResult<Option<PathBuf>>
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        self.dialog(
            title.into(),
            default_file.into().to_string_lossy().into_owned(),
            DialogType::Open,
            DialogFlags::FILE,
        )
        .map(|path| {
            if path.is_empty() {
                None
            } else {
                Some(PathBuf::from(path))
            }
        })
    }

    /// Opens a new choose directory dialog as returns the chosen directory path.
    pub fn choose_directory<S, P>(&mut self, title: S, default_dir: P) -> WVResult<Option<PathBuf>>
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        self.dialog(
            title.into(),
            default_dir.into().to_string_lossy().into_owned(),
            DialogType::Open,
            DialogFlags::DIRECTORY,
        )
        .map(|path| if path.is_empty() { None } else { Some(PathBuf::from(path)) })
    }

    /// Opens an info alert dialog.
    pub fn info<TS, MS>(&mut self, title: TS, message: MS) -> WVResult
    where
        TS: Into<String>,
        MS: Into<String>,
    {
        self.dialog(
            title.into(),
            message.into(),
            DialogType::Alert,
            DialogFlags::INFO,
        )
        .map(|_| ())
    }

    /// Opens a warning alert dialog.
    pub fn warning<TS, MS>(&mut self, title: TS, message: MS) -> WVResult
    where
        TS: Into<String>,
        MS: Into<String>,
    {
        self.dialog(
            title.into(),
            message.into(),
            DialogType::Alert,
            DialogFlags::WARNING,
        )
        .map(|_| ())
    }

    /// Opens an error alert dialog.
    pub fn error<TS, MS>(&mut self, title: TS, message: MS) -> WVResult
    where
        TS: Into<String>,
        MS: Into<String>,
    {
        self.dialog(
            title.into(),
            message.into(),
            DialogType::Alert,
            DialogFlags::ERROR,
        )
        .map(|_| ())
    }
}

fn read_str(s: &[u8]) -> String {
    let end = s.iter().position(|&b| b == 0).map_or(0, |i| i + 1);
    match CStr::from_bytes_with_nul(&s[..end]) {
        Ok(s) => s.to_string_lossy().into_owned(),
        Err(_) => "".to_string(),
    }
}
