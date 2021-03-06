//! This crate provides a safe wrapper around the Oniguruma regular
//! expression library.
//!
//! # Examples
//!
//! ```
//! # use onig::{options,syntax_types,Regex};
//! let regex = Regex::new(r#"hello (\w*)"#).unwrap();
//! let result = regex.search_str("hello world",
//!                               onig::options::ONIG_OPTION_NONE);
//! assert!(result.is_some());
//! ```

extern crate libc;
extern crate onig_sys;

use libc::c_int;
use std::ptr::null;

pub mod utils;
pub mod err;
pub mod region;

// re-export the onig types publically
pub use onig_sys::onig_option_type as options;
pub use onig_sys::onig_syntax_type as syntax_types;
pub use err::OnigError;
pub use region::OnigRegion;

/// Oniguruma Regular Expression
///
/// This struct is a wrapper around an Oniguruma regular expression
/// pointer. This represents a compiled regex which can be used in
/// search and match operations.
#[allow(raw_pointer_derive)]
#[derive(Debug)]
pub struct Regex {
    raw: *const onig_sys::regex_t,
}

impl Regex {
    /// Simple regular expression constructor. Compiles a new regular
    /// expression with the default options using the ruby syntax.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The regex pattern to compile
    ///
    /// # Examples
    ///
    /// ```
    /// use onig::{options,syntax_types,Regex};
    /// let r = Regex::new(r#"hello (\w+)"#);
    /// assert!(r.is_ok());
    /// ```
    pub fn new(pattern: &str) -> Result<Regex, OnigError> {
        Self::new_with_options(pattern, options::ONIG_OPTION_NONE, syntax_types::RUBY)
    }

    /// Create a new Regex
    ///
    /// Attempts to compile a pattern into a new `Regex` instance. See
    /// [`onig_sys::onig_new`][regex_new] for more information.
    ///
    /// # Arguments
    ///
    ///  * `pattern` - The regex pattern to compile.
    ///  * `options` - The regex compilation options.
    ///  * `syntax`  - The syntax which the regex is written in.
    ///
    /// # Examples
    ///
    /// ```
    /// use onig::{options,syntax_types,Regex};
    /// let r = Regex::new_with_options("hello.*world",
    ///                    options::ONIG_OPTION_NONE,
    ///                    syntax_types::RUBY);
    /// assert!(r.is_ok());
    /// ```
    ///
    /// [regex_new]: ./onig_sys/fn.onig_new.html
    pub fn new_with_options(pattern: &str,
                            option: onig_sys::OnigOptionType,
                            syntax: *const onig_sys::OnigSyntaxTypeStruct)
                            -> Result<Regex, OnigError> {

        // Convert the rust types to those required for the call to
        // `onig_new`.
        let pattern_bytes = pattern.as_bytes();
        let mut reg: *const onig_sys::regex_t = null();
        let reg_ptr = &mut reg as *mut *const onig_sys::regex_t;

        // We can use this later to get an error message to pass back
        // if regex creation fails.
        let mut error = onig_sys::OnigErrorInfo {
            enc: null(),
            par: null(),
            par_end: null(),
        };

        let err = unsafe {
            onig_sys::onig_new(reg_ptr,
                               pattern_bytes.as_ptr(),
                               pattern_bytes[pattern_bytes.len()..].as_ptr(),
                               option.bits(),
                               onig_sys::onig_encodings::UTF8,
                               syntax,
                               &mut error)
        };

        if err == 0 {
            Ok(Regex { raw: reg })
        } else {
            Err(OnigError::new(err, &error))
        }
    }

    /// Match Str
    ///
    /// Match the regex against a string. This method will start at
    /// the beginning of the string and try and match the regex. If
    /// the regex matches then the return value is the number of
    /// characers which matched. If the regex doesn't match the return
    /// is `None`.
    ///
    /// # Arguments
    ///
    /// * `self` - The regex to match
    /// * `str` - The string slice to match against.
    /// * `options` - The regex match options.
    ///
    /// # Returns
    ///
    /// `Some(len)` if the regex matched, with `len` being the number
    /// of bytes matched. `None` if the regex doesn't match.
    ///
    /// # Examples
    ///
    /// ```
    /// use onig::Regex;
    /// use onig::{options,syntax_types};
    ///
    /// let r = Regex::new(".*").unwrap();
    /// let res = r.match_str("hello", options::ONIG_OPTION_NONE);
    /// assert!(res.is_some()); // it matches
    /// assert!(res.unwrap() == 5); // 5 characters matched
    /// ```
    pub fn match_str(&self, str: &str, options: onig_sys::OnigOptionType) -> Option<i32> {
        let ret = unsafe {
            let end = Self::str_end(str);
            onig_sys::onig_match(self.raw,
                                 str.as_ptr(),
                                 end,
                                 str.as_ptr(),
                                 0 as *mut onig_sys::OnigRegion,
                                 options.bits())
        };
        Self::result_to_match(ret)
    }

    /// Search Str
    ///
    /// Search for matches the regex in a string. This method will return the
    /// index of the first match of the regex within the string, if
    /// there is one.
    ///
    /// # Arguments
    ///
    ///  * `self` - The regex to search for
    ///  * `str` - The string to search in.
    ///  * `options` - The options for the search.
    ///
    /// # Returns
    ///
    /// `Some(pos)` if the regex matches, where `pos` is the
    /// byte-position of the start of the match. `None` if the regex
    /// doesn't match anywhere in `str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use onig::Regex;
    /// use onig::{options,syntax_types};
    ///
    /// let r = Regex::new("l{1,2}").unwrap();
    /// let res = r.search_str("hello", options::ONIG_OPTION_NONE);
    /// assert!(res.is_some()); // it matches
    /// assert!(res.unwrap() == 2); // match starts at character 3
    /// ```
    pub fn search_str(&self, str: &str, options: onig_sys::OnigOptionType) -> Option<i32> {
        let ret = unsafe {
            let start = str.as_ptr();
            let end = Self::str_end(str);
            onig_sys::onig_search(self.raw,
                                  start,
                                  end,
                                  start,
                                  end,
                                  0 as *mut onig_sys::OnigRegion,
                                  options.bits())
        };
        Self::result_to_match(ret)
    }

    unsafe fn str_end(str: &str) -> *const u8 {
        str.as_ptr().offset(str.len() as isize)
    }

    fn result_to_match(res: libc::c_int) -> Option<i32> {
        if res < 0 {
            None
        } else {
            Some(res)
        }
    }
}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe {
            onig_sys::onig_free(self.raw);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_regex(regex: &str) -> Regex {
        Regex::new(regex).unwrap()
    }

    #[test]
    fn test_regex_create() {
        Regex::new_with_options(".*", options::ONIG_OPTION_NONE, syntax_types::RUBY).unwrap();
        Regex::new(r#"a \w+ word"#).unwrap();
    }

    #[test]
    #[should_panic(expected = "Oniguruma error(-223): invalid character property name {foo}")]
    fn test_regex_invalid() {
        create_regex("\\p{foo}");
    }

    #[test]
    fn test_simple_match() {
        let r = create_regex(".*");

        let res = r.match_str("hello wolrld", options::ONIG_OPTION_NONE);

        assert!(res.is_some());
        assert!(res.unwrap() == 12);
    }

    #[test]
    fn test_failed_match() {
        let r = create_regex("foo");

        let res = r.match_str("bar", options::ONIG_OPTION_NONE);
        assert!(res.is_none());
    }

    #[test]
    fn test_partial_match() {
        let r = create_regex("hello");
        let res = r.match_str("hello world", options::ONIG_OPTION_NONE);

        assert!(res.is_some());
        assert!(res.unwrap() == 5);
    }

    #[test]
    fn test_simple_search() {
        let r = create_regex("hello");
        let res = r.search_str("just came to say hello :-)", options::ONIG_OPTION_NONE);

        assert!(res.is_some());
        assert!(res.unwrap() == 17);
    }
}
