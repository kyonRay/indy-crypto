use libc::c_char;

use std::ffi::CStr;
use std::str::Utf8Error;
use std::ffi::CString;

pub struct CTypesUtils {}

impl CTypesUtils {
    pub fn c_str_to_string(cstr: *const c_char) -> Result<Option<String>, Utf8Error> {
        if cstr.is_null() {
            return Ok(None);
        }

        unsafe {
            match CStr::from_ptr(cstr).to_str() {
                Ok(str) => Ok(Some(str.to_string())),
                Err(err) => Err(err)
            }
        }
    }

    pub fn string_to_cstring(s: String) -> CString {
        CString::new(s).unwrap()
    }
}

macro_rules! check_useful_c_reference {
    ($ptr:ident, $type:ty, $err:expr) => {
        if $ptr.is_null() {
            return $err
        }

        let $ptr: &$type = unsafe { &*($ptr as *const $type) };;
    }
}

macro_rules! check_useful_mut_c_reference {
    ($ptr:ident, $type:ty, $err:expr) => {
        if $ptr.is_null() {
            return $err
        }

        let $ptr: &mut $type = unsafe { &mut *($ptr as *mut $type) };;
    }
}

macro_rules! check_useful_c_ptr {
    ($ptr:ident, $err1:expr) => {
        if $ptr.is_null() {
            return $err1
        }
    }
}

macro_rules! check_useful_c_str {
    ($x:ident, $e:expr) => {
        let $x = match CTypesUtils::c_str_to_string($x) {
            Ok(Some(val)) => val,
            _ => return $e,
        };

        if $x.is_empty() {
            return $e
        }
    }
}

macro_rules! check_useful_opt_c_str {
    ($x:ident, $e:expr) => {
        let $x = match CTypesUtils::c_str_to_string($x) {
            Ok(opt_val) => opt_val,
            Err(_) => return $e
        };
    }
}

macro_rules! check_useful_c_callback {
    ($x:ident, $e:expr) => {
        let $x = match $x {
            Some($x) => $x,
            None => return $e
        };
    }
}