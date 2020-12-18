#![feature(const_fn_fn_ptr_basics)]

extern crate winapi;
use winapi::shared::minwindef::*;
use std::result::Result;
use std::ffi::CString;

#[allow(non_snake_case)]
struct FunctionPtrs {
    initialized: bool,
    InitializeKDMAPIStream: Option<extern "stdcall" fn() -> bool>,
    IsKDMAPIAvailable: Option<extern "stdcall" fn() -> bool>,
    SendDirectData: Option<extern "stdcall" fn(data: u32) -> u32>,
    TerminateKDMAPIStream: Option<extern "stdcall" fn() -> bool>,
}

impl FunctionPtrs {
    const fn new() -> FunctionPtrs {
        FunctionPtrs {
            initialized: false,
            InitializeKDMAPIStream: None,
            IsKDMAPIAvailable: None,
            SendDirectData: None,
            TerminateKDMAPIStream: None,
        }
    }
}

impl Default for FunctionPtrs {
    fn default() -> FunctionPtrs {
        FunctionPtrs::new()
    }
}

static mut FUNCTION_PTRS: FunctionPtrs = FunctionPtrs::new();

fn get_func(module: HMODULE, name: &str) -> Result<&__some_function, u32> {
    unsafe {
        let cstr_name = CString::new(name).unwrap();
        let proc = winapi::um::libloaderapi::GetProcAddress(module, cstr_name.as_ptr() as *const i8).as_ref();
        match proc {
            Some(x) => Ok(x),
            None => Err(winapi::um::errhandlingapi::GetLastError())
        }
    }
}

pub fn init() -> Result<(), String> {
    unsafe {
        if FUNCTION_PTRS.initialized {
            return Err("Already initialized".to_string())
        }

        // try and load OmniMIDI.dll from current directory first
        let mod_name = CString::new("OmniMIDI.dll").unwrap();
        let mut module = winapi::um::libloaderapi::LoadLibraryA(mod_name.as_ptr() as *const i8);
        if module.as_ref().is_none() {
            // then try loading it from system32
            let mut system_path = vec![0u8; 260];
            winapi::um::sysinfoapi::GetSystemDirectoryA(system_path.as_mut_ptr() as *mut i8, 260);
            let mut system_path_len: usize = 0;
            for (i, x) in system_path.iter().enumerate() {
                if *x == 0 {
                    system_path_len = i;
                    break;
                }
            }
            system_path.truncate(system_path_len);

            let system_path = std::str::from_utf8(&system_path);
            let new_path = CString::new(system_path.unwrap().to_string() + "\\OmniMIDI\\OmniMIDI.dll").unwrap();
            module = winapi::um::libloaderapi::LoadLibraryA(new_path.as_ptr() as *const i8);
            if module.as_ref().is_none() {
                return Err(format!("Failed to load OmniMIDI.dll! GLE: {:x}", winapi::um::errhandlingapi::GetLastError()))
            }
        }

        // populate all of the function pointers
        macro_rules! populate_function {
            ($func:ident, $sig:ty) => {
                let func_ptr: $sig = match get_func(module, stringify!($func)) {
                    Ok(x) => std::mem::transmute(x),
                    Err(e) => return Err(format!("Failed to find function {}! GLE: {:x}", stringify!($func), e)),
                };
                FUNCTION_PTRS.$func = Some(func_ptr);
            };
        }

        populate_function!(InitializeKDMAPIStream, extern "stdcall" fn() -> bool);
        populate_function!(IsKDMAPIAvailable, extern "stdcall" fn() -> bool);
        populate_function!(SendDirectData, extern "stdcall" fn(u32) -> u32);
        populate_function!(TerminateKDMAPIStream, extern "stdcall" fn() -> bool);

        if !FUNCTION_PTRS.InitializeKDMAPIStream.unwrap()() {
            return Err("KDMAPI failed to initialize.".to_string());
        }
        if !FUNCTION_PTRS.IsKDMAPIAvailable.unwrap()() {
            return Err("KDMAPI was able to be initialized, but is currently disabled in settings.".to_string());
        }
        FUNCTION_PTRS.initialized = true;

        Ok(())
    }
}

pub fn send_direct_data(data: u32) {
    unsafe {
        if !FUNCTION_PTRS.initialized {
            return;
        }
        FUNCTION_PTRS.SendDirectData.unwrap()(data);
    }
}

pub fn terminate() {
    unsafe {
        if !FUNCTION_PTRS.initialized {
            return;
        }
        FUNCTION_PTRS.TerminateKDMAPIStream.unwrap()();
        FUNCTION_PTRS.initialized = false;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_midi() {
        let res = crate::init();
        match res {
            Ok(()) => (),
            Err(x) => {
                println!("{}", x);
                unreachable!();
            }
        }

        // play a C4 for 1 second
        crate::send_direct_data(0x007F3090);
        std::thread::sleep(std::time::Duration::from_millis(1000));

        crate::terminate();
    }
}