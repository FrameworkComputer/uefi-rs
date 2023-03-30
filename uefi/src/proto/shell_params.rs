//! `ShellParams` protocol

use core::ffi::c_void;
use crate::proto::unsafe_protocol;
use crate::{CStr16, Char16};

#[cfg(feature = "alloc")]
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

type ShellFileHandle = *const c_void;

/// The ShellParameters protocol. 
#[repr(C)]
#[unsafe_protocol("752f3136-4e16-4fdc-a22a-e5f46812f4ca")]
pub struct ShellParameters {
    /// Pointer to a list of arguments
    pub argv: *const *const Char16,
    /// Number of arguments
    pub argc: usize,
    /// Handle of the standard input
    std_in: ShellFileHandle,
    /// Handle of the standard output
    std_out: ShellFileHandle,
    /// Handle of the standard error output
    std_err: ShellFileHandle,
}

impl ShellParameters {
    #[cfg(feature = "alloc")]
    /// Get a Vec of the shell parameter arguments
    pub fn get_args(&self) -> Vec<String> {
        let mut args = vec![];
        // Skip first one, which is the executable's name
        for i in 1..self.argc {
            let str = unsafe { CStr16::from_ptr(*self.argv.add(i)) };
            let string = str.to_string();
            args.push(string);
        }
        args
    }
}

