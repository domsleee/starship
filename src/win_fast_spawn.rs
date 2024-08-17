extern crate winapi;

use core::slice;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::prelude::OsStringExt;
use std::ptr::{null, null_mut};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FormatMessageW;
use winapi::um::{
    fileapi::{CreateFileW, OPEN_EXISTING},
    processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW},
    winbase::{
        FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
        STARTF_USESTDHANDLES,
    },
    winnt::{FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_WRITE},
};

pub fn fast_background_spawn(cmd: &str, args: &str) -> () {
    // changed return type
    let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
    let cmd_string = format!("{} {}", cmd, args);

    log::debug!("spawning {cmd_string}");

    let mut cmd = OsStr::new(&cmd_string)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let nul_handle = get_nul_handle();

    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    startup_info.hStdError = nul_handle;
    startup_info.hStdOutput = nul_handle;
    startup_info.dwFlags |= STARTF_USESTDHANDLES;

    let success = unsafe {
        CreateProcessW(
            null_mut(),
            cmd.as_mut_ptr(),
            null_mut(),
            null_mut(),
            false as i32,
            0,
            null_mut(),
            null_mut(),
            &mut startup_info,
            &mut process_info,
        )
    };

    if success == 0 {
        // 0 indicates failure for CreateProcessW
        log::error!("{}", get_last_error());
    }

    ()
}

fn get_nul_handle() -> *mut winapi::ctypes::c_void {
    let nul_handle = unsafe {
        CreateFileW(
            OsStr::new("NUL")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>()
                .as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };

    if nul_handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        log::error!("{}", get_last_error());
    }
    return nul_handle;
}

fn get_last_error() -> String {
    unsafe {
        let mut message_buffer: *mut u16 = null_mut();
        let error_code = GetLastError();

        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            null(),
            error_code,
            0,
            (&mut message_buffer as *mut *mut u16) as *mut _,
            0,
            null_mut(),
        );

        let error_message = OsString::from_wide(slice::from_raw_parts(message_buffer, 512))
            .to_string_lossy()
            .into_owned();

        winapi::um::winbase::LocalFree(message_buffer as *mut _);

        format!("Error (code: {}): {}", error_code, error_message.trim())
    }
}
