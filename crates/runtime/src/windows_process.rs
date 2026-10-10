// SPDX-License-Identifier: MPL-2.0
//! Windows 11+: atomic Job assignment at CreateProcess, restricted inherited
//! handles, no breakaway and KILL_ON_JOB_CLOSE. Never signal a numeric PID.
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    fs::File,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    process::Command,
    ptr,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::SECURITY_ATTRIBUTES,
    System::{JobObjects::*, Pipes::CreatePipe, Threading::*},
};

pub(crate) struct Child {
    process: OwnedHandle,
    job: OwnedHandle,
    pid: u32,
    pub stdin: Option<File>,
    pub stdout: Option<File>,
    pub stderr: Option<File>,
}
fn owned(handle: HANDLE) -> Result<OwnedHandle> {
    ensure!(
        !handle.is_null() && handle != INVALID_HANDLE_VALUE,
        "Invalid Windows process handle"
    );
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}
fn check(ok: i32) -> Result<()> {
    ensure!(
        ok != 0,
        "Windows private process operation failed: {}",
        std::io::Error::last_os_error()
    );
    Ok(())
}
fn wide(s: &std::ffi::OsStr) -> Result<Vec<u16>> {
    let mut v: Vec<_> = s.encode_wide().collect();
    ensure!(!v.contains(&0), "NUL in process argument");
    v.push(0);
    Ok(v)
}
// Windows CRT argv escaping, including quotes and trailing backslashes. No cmd.exe.
fn quote(s: &std::ffi::OsStr) -> Result<Vec<u16>> {
    let input = wide(s)?;
    let mut out = vec![b'"' as u16];
    let mut slashes = 0;
    for &c in &input[..input.len() - 1] {
        if c == b'\\' as u16 {
            slashes += 1;
            continue;
        }
        out.extend(std::iter::repeat_n(
            b'\\' as u16,
            if c == b'"' as u16 {
                slashes * 2 + 1
            } else {
                slashes
            },
        ));
        slashes = 0;
        out.push(c);
    }
    out.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
    out.push(b'"' as u16);
    Ok(out)
}
fn pipe(input: bool) -> Result<(File, OwnedHandle)> {
    let mut read = ptr::null_mut();
    let mut write = ptr::null_mut();
    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    check(unsafe { CreatePipe(&mut read, &mut write, &sa, 0) })?;
    let (read, write) = (owned(read)?, owned(write)?);
    let (parent, child) = if input { (write, read) } else { (read, write) };
    check(unsafe { SetHandleInformation(parent.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) })?;
    Ok((File::from(parent), child))
}
struct Attributes {
    _storage: Vec<usize>,
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
}
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe { DeleteProcThreadAttributeList(self.list) }
    }
}
impl Child {
    pub fn spawn(command: &Command) -> Result<Self> {
        // Job handle is NOT inherited by worker: closing the last owner on app
        // death kills even detached browser descendants. Atomic job-list creation
        // removes the spawn/assign crash race. Chromium nested jobs remain allowed.
        let job = owned(unsafe { CreateJobObjectW(ptr::null(), ptr::null()) })?;
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        check(unsafe {
            SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&limits) as u32,
            )
        })?;
        let (input, child_in) = pipe(true)?;
        let (output, child_out) = pipe(false)?;
        let (error, child_err) = pipe(false)?;
        let handles = [
            child_in.as_raw_handle(),
            child_out.as_raw_handle(),
            child_err.as_raw_handle(),
        ];
        let jobs = [job.as_raw_handle()];
        let mut bytes = 0;
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut bytes);
        }
        ensure!(bytes > 0 && bytes < 65536, "Invalid process attribute size");
        let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
        let list = storage.as_mut_ptr().cast();
        check(unsafe { InitializeProcThreadAttributeList(list, 2, 0, &mut bytes) })?;
        let attributes = Attributes {
            _storage: storage,
            list,
        };
        for (kind, data, length) in [
            (
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                handles.as_ptr(),
                size_of_val(&handles),
            ),
            (
                PROC_THREAD_ATTRIBUTE_JOB_LIST,
                jobs.as_ptr(),
                size_of_val(&jobs),
            ),
        ] {
            check(unsafe {
                UpdateProcThreadAttribute(
                    list,
                    0,
                    kind as usize,
                    data.cast(),
                    length,
                    ptr::null_mut(),
                    ptr::null(),
                )
            })?;
        }
        let program = wide(command.get_program())?;
        let mut argv = quote(command.get_program())?;
        for arg in command.get_args() {
            argv.push(b' ' as u16);
            argv.extend(quote(arg)?);
        }
        argv.push(0);
        // Caller clears inherited environment. Read ONLY its explicit additions;
        // sort keys case-insensitively as required by Windows environment blocks.
        let mut env = BTreeMap::new();
        for (k, v) in command.get_envs() {
            if let Some(v) = v {
                env.insert(k.to_string_lossy().to_uppercase(), (k, v));
            }
        }
        let mut block = Vec::new();
        for (_, (k, v)) in env {
            let k = wide(k)?;
            let v = wide(v)?;
            block.extend_from_slice(&k[..k.len() - 1]);
            block.push(b'=' as u16);
            block.extend(v);
        }
        block.push(0);
        let cwd = command
            .get_current_dir()
            .map(|p| wide(p.as_os_str()))
            .transpose()?;
        let mut startup = STARTUPINFOEXW::default();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = handles[0];
        startup.StartupInfo.hStdOutput = handles[1];
        startup.StartupInfo.hStdError = handles[2];
        startup.lpAttributeList = attributes.list;
        let mut info = PROCESS_INFORMATION::default();
        check(unsafe {
            CreateProcessW(
                program.as_ptr(),
                argv.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                1,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
                block.as_ptr().cast(),
                cwd.as_ref().map_or(ptr::null(), |v| v.as_ptr()),
                &startup.StartupInfo,
                &mut info,
            )
        })?;
        let process = owned(info.hProcess)?;
        drop(owned(info.hThread)?);
        Ok(Self {
            process,
            job,
            pid: info.dwProcessId,
            stdin: Some(input),
            stdout: Some(output),
            stderr: Some(error),
        })
    }
    pub fn id(&self) -> u32 {
        self.pid
    }
    pub fn kill(&mut self) -> Result<()> {
        check(unsafe { TerminateJobObject(self.job.as_raw_handle(), 1) })
    }
    pub fn wait(&mut self) -> Result<()> {
        ensure!(
            unsafe { WaitForSingleObject(self.process.as_raw_handle(), 10000) } == WAIT_OBJECT_0,
            "Windows worker did not reap within 10 seconds"
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn job_assignment_and_only_owned_tree_termination() -> Result<()> {
        let mut command = Command::new(std::env::var_os("COMSPEC").unwrap());
        command
            .env_clear()
            .env("SystemRoot", std::env::var_os("SystemRoot").unwrap());
        command.args(["/d", "/c", "ping -n 30 127.0.0.1 >nul"]);
        let mut child = Child::spawn(&command)?;
        let mut in_job = 0;
        check(unsafe {
            IsProcessInJob(
                child.process.as_raw_handle(),
                child.job.as_raw_handle(),
                &mut in_job,
            )
        })?;
        assert_eq!(in_job, 1);
        child.kill()?;
        child.wait()?;
        Ok(())
    }
}
