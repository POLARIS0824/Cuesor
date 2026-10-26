use std::{ffi::OsString, mem::size_of, os::windows::ffi::OsStrExt};

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS, FILETIME, HANDLE},
        Security::{
            GetLengthSid, GetTokenInformation, TokenIntegrityLevel, TokenUser,
            TOKEN_INFORMATION_CLASS, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TOKEN_USER,
        },
        System::{
            Console::GetConsoleWindow,
            Threading::{
                CreateMutexW, CreateProcessW, GetCurrentProcess, GetProcessTimes, OpenProcess,
                OpenProcessToken, CREATE_NO_WINDOW, PROCESS_INFORMATION,
                PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, STARTUPINFOW,
            },
        },
    },
};

use crate::platform::{wide, AppResult, OwnedHandle};

fn token_information(token: HANDLE, kind: TOKEN_INFORMATION_CLASS) -> AppResult<Vec<usize>> {
    let mut size = 0;
    unsafe {
        let _ = GetTokenInformation(token, kind, None, 0, &mut size);
    }
    if size == 0 {
        return Err("could not determine token information size".into());
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
    unsafe {
        GetTokenInformation(
            token,
            kind,
            Some(buffer.as_mut_ptr().cast()),
            size,
            &mut size,
        )?;
    }
    Ok(buffer)
}

pub fn identity() -> AppResult<String> {
    let mut raw = HANDLE::default();
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw)?;
    }
    let token = OwnedHandle(raw);
    let user = token_information(token.0, TokenUser)?;
    let integrity = token_information(token.0, TokenIntegrityLevel)?;
    let mut key = String::new();
    unsafe {
        let user_sid = (*(user.as_ptr().cast::<TOKEN_USER>())).User.Sid;
        let level_sid = (*(integrity.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()))
            .Label
            .Sid;
        for sid in [user_sid, level_sid] {
            for byte in std::slice::from_raw_parts(sid.0.cast::<u8>(), GetLengthSid(sid) as usize) {
                use std::fmt::Write;
                write!(key, "{byte:02x}")?;
            }
            key.push('-');
        }
    }
    Ok(key)
}

pub fn desktop_scope() -> AppResult<String> {
    Ok(format!("Local\\Cuesor-desktop-{}", identity()?))
}

pub(crate) fn console_lease() -> AppResult<Option<OwnedHandle>> {
    let name = wide(format!(
        "Local\\Cuesor-console-{}-{}",
        identity()?,
        unsafe { GetConsoleWindow() }.0 as usize
    ));
    let handle = OwnedHandle(unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr()))? });
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        Ok(None)
    } else {
        Ok(Some(handle))
    }
}

pub(crate) fn shell_process(pid: u32) -> AppResult<OwnedHandle> {
    Ok(OwnedHandle(unsafe {
        OpenProcess(
            PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        )?
    }))
}

pub fn created_at(process: HANDLE) -> AppResult<u64> {
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user)?;
    }
    Ok((created.dwHighDateTime as u64) << 32 | created.dwLowDateTime as u64)
}

/// Return a waitable process handle. No console, standard streams, or other handles are inherited.
pub fn spawn_hidden(arguments: &[OsString]) -> AppResult<OwnedHandle> {
    let executable: Vec<u16> = arguments
        .first()
        .ok_or("missing executable")?
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut command = command_line(arguments);
    let startup = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut process = PROCESS_INFORMATION::default();
    // Do not inherit ANY handles. Inheriting Nu's pipe handles makes `complete`
    // wait for the long-lived worker even after this short-lived launcher exits.
    unsafe {
        CreateProcessW(
            PCWSTR(executable.as_ptr()),
            PWSTR(command.as_mut_ptr()),
            None,
            None,
            false,
            CREATE_NO_WINDOW,
            None,
            None,
            &startup,
            &mut process,
        )?;
    }
    let process_handle = OwnedHandle(process.hProcess);
    let _thread = OwnedHandle(process.hThread);
    Ok(process_handle)
}

fn command_line(arguments: &[OsString]) -> Vec<u16> {
    let mut output = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        if index != 0 {
            output.push(b' ' as u16);
        }
        output.push(b'"' as u16);
        let mut slashes = 0;
        for unit in argument.encode_wide() {
            if unit == b'\\' as u16 {
                slashes += 1;
                continue;
            }
            let count = if unit == b'"' as u16 {
                slashes * 2 + 1
            } else {
                slashes
            };
            output.extend(std::iter::repeat_n(b'\\' as u16, count));
            output.push(unit);
            slashes = 0;
        }
        output.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
        output.push(b'"' as u16);
    }
    output.push(0);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_spawn_preserves_spaces_and_trailing_backslashes() {
        let actual = command_line(&[
            "D:\\工具 路径\\app.exe".into(),
            "".into(),
            "D:\\logs folder\\".into(),
            "a\"b".into(),
        ]);
        assert_eq!(
            String::from_utf16(&actual[..actual.len() - 1]).unwrap(),
            "\"D:\\工具 路径\\app.exe\" \"\" \"D:\\logs folder\\\\\" \"a\\\"b\""
        );
    }
}
