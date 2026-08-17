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
