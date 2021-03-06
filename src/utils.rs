use onig_sys;
use std::ffi::CStr;

pub fn onig_version() -> String {
    let raw_version = unsafe { CStr::from_ptr(onig_sys::onig_version()) };
    raw_version.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn utils_get_version_returns_expected_version() {
        let version = onig_version();
        assert_eq!(version, "5.9.6");
    }
}
