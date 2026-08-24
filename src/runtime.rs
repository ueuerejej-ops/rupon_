use libc::CS;
use std::ffi::CStr;
use std::io::{self, Write};

pub struct Runtime {
    pub stdout: io::Stdout,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }

    pub fn compare_str(&mut self, ptr1: *const u8, ptr2: *const u8) -> bool {
        unsafe {
            let text1 = CStr::from_ptr(ptr1 as *const i8).to_bytes();
            let text2 = CStr::from_ptr(ptr2 as *const i8).to_bytes();

            text1 == text2
        }
    }

    pub fn print(&mut self, ptr: *const u8) {
        unsafe {
            let text = CStr::from_ptr(ptr as *const i8).to_str().unwrap();
            self.stdout.write_all(text.as_bytes()).unwrap();
            self.stdout.write_all(b"\n").unwrap();
            self.stdout.flush().unwrap();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn runtime_print(runtime: *mut Runtime, ptr: *const u8) {
    unsafe {
        let runtime = &mut *runtime;
        runtime.print(ptr);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn runtime_compare_str(
    runtime: *mut Runtime,
    ptr1: *const u8,
    ptr2: *const u8,
) -> bool {
    unsafe {
        let runtime = &mut *runtime;
        runtime.compare_str(ptr1, ptr2)
    }
}
