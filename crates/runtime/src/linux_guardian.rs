// SPDX-License-Identifier: MPL-2.0
//! A per-launch kernel ancestor, not a /proc identity classifier.
//!
//! Both forks bypass libc/atfork; argv/env/cwd and error buffers are parent-owned.
//! The first child becomes a raw-syscall subreaper and clones verified Node.
//! The guardian closes all other inherited descriptors, observes app death via
//! pidfd, and on stop/Node exit kills only its kernel-confirmed direct children.
//! Detached/double-forked descendants are adopted by this *dedicated* ancestor;
//! unrelated app/UID/process-group members can never become its children.
//! No allocator, Rust destructor, mutex, or thread runtime runs after fork.
use anyhow::{Result, ensure};
use std::{
    fs::File,
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    os::unix::{ffi::OsStrExt, process::ExitStatusExt},
    process::Command,
    time::{Duration, Instant},
};

// Bypass libc entirely: syscall(2)'s libc wrapper writes errno through TLS, and
// fork(3) runs inherited atfork handlers. Linux returns negative errno directly.
// These ABIs use clone(SIGCHLD, NULL, NULL, NULL, 0) as fork, WITHOUT CLONE_VM.
#[inline(always)]
unsafe fn syscall(n: i64, a: usize, b: usize, c: usize, d: usize, e: usize) -> i64 {
    #[cfg(target_arch = "x86_64")]
    {
        let result: i64;
        unsafe {
            std::arch::asm!("syscall", inlateout("rax") n => result,
            in("rdi") a, in("rsi") b, in("rdx") c, in("r10") d, in("r8") e,
            in("r9") 0usize, lateout("rcx") _, lateout("r11") _, options(nostack));
        }
        result
    }
    #[cfg(target_arch = "aarch64")]
    {
        let result: i64;
        unsafe {
            std::arch::asm!("svc 0", in("x8") n, inlateout("x0") a as i64 => result,
            in("x1") b, in("x2") c, in("x3") d, in("x4") e, in("x5") 0usize,
            options(nostack));
        }
        result
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("Linux guardian raw syscall ABI requires x86_64 or aarch64");
}
unsafe fn close(fd: i32) {
    unsafe {
        syscall(libc::SYS_close, fd as usize, 0, 0, 0, 0);
    }
}
unsafe fn exit(status: i32) -> ! {
    unsafe {
        syscall(libc::SYS_exit_group, status as usize, 0, 0, 0, 0);
    }
    // SYS_exit_group cannot return. No panic/destructor even on a hostile filter.
    loop {
        std::hint::spin_loop();
    }
}

pub(crate) struct Launch {
    stop: OwnedFd,
    report: OwnedFd,
    child_stop: OwnedFd,
    child_report: OwnedFd,
    receipt: OwnedFd,
    registration: Option<OwnedFd>,
    token: zeroize::Zeroizing<[u8; 36]>,
    exec: Exec,
}
struct Exec {
    program: std::ffi::CString,
    _args: Vec<std::ffi::CString>,
    argv: Vec<*const libc::c_char>,
    _env: Vec<std::ffi::CString>,
    envp: Vec<*const libc::c_char>,
    cwd: Option<std::ffi::CString>,
}
impl Exec {
    fn prepare(command: &Command) -> Result<Self> {
        use std::ffi::CString;
        ensure!(
            std::path::Path::new(command.get_program()).is_absolute(),
            "guardian requires an absolute verified executable"
        );
        let program = CString::new(command.get_program().as_bytes())?;
        let args = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|a| CString::new(a.as_bytes()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut argv: Vec<_> = args.iter().map(|a| a.as_ptr()).collect();
        argv.push(std::ptr::null());
        // Product always uses env_clear. Only explicit Command environment is
        // accepted here; no ambient Node/loader injection or PATH search.
        let env = command
            .get_envs()
            .filter_map(|(key, value)| value.map(|v| (key, v)))
            .map(|(key, value)| {
                let mut b = key.as_bytes().to_vec();
                b.push(b'=');
                b.extend_from_slice(value.as_bytes());
                CString::new(b)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut envp: Vec<_> = env.iter().map(|a| a.as_ptr()).collect();
        envp.push(std::ptr::null());
        let cwd = command
            .get_current_dir()
            .map(|p| CString::new(p.as_os_str().as_bytes()))
            .transpose()?;
        Ok(Self {
            program,
            _args: args,
            argv,
            _env: env,
            envp,
            cwd,
        })
    }
}
fn above_stdio(fd: OwnedFd) -> Result<OwnedFd> {
    if fd.as_raw_fd() >= 3 {
        return Ok(fd);
    }
    let copy = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
    ensure!(copy >= 3, "reserve guardian descriptor above stdio");
    Ok(unsafe { OwnedFd::from_raw_fd(copy) })
}
fn pipe() -> Result<(OwnedFd, OwnedFd)> {
    let mut fds = [-1; 2];
    ensure!(
        unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) } == 0,
        "create guardian control pipe"
    );
    let (read, write) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
    Ok((above_stdio(read)?, above_stdio(write)?))
}
impl Launch {
    #[cfg_attr(not(test), allow(dead_code))] // Also included by independent standalone probes.
    pub(crate) fn prepare(command: &mut Command, receipt: &File, token: &[u8]) -> Result<Self> {
        Self::prepare_inner(command, receipt, token, None)
    }
    pub(crate) fn prepare_registered(
        command: &mut Command,
        receipt: &File,
        token: &[u8],
        registration: &File,
    ) -> Result<Self> {
        Self::prepare_inner(command, receipt, token, Some(registration))
    }
    fn prepare_inner(
        command: &mut Command,
        receipt: &File,
        token: &[u8],
        registration: Option<&File>,
    ) -> Result<Self> {
        ensure!(token.len() == 36, "invalid guardian settlement token");
        let (child_stop, stop) = pipe()?;
        let (report, child_report) = pipe()?;
        let copied = unsafe { libc::fcntl(receipt.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
        ensure!(copied >= 3, "reserve guardian receipt descriptor");
        let receipt = unsafe { OwnedFd::from_raw_fd(copied) };
        let registration = registration
            .map(|file| {
                let fd = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
                ensure!(fd >= 3, "reserve guardian registration descriptor");
                Ok(unsafe { OwnedFd::from_raw_fd(fd) })
            })
            .transpose()?;
        let mut receipt_token = zeroize::Zeroizing::new([0; 36]);
        receipt_token.copy_from_slice(token);
        Ok(Self {
            stop,
            report,
            child_stop,
            child_report,
            receipt,
            registration,
            token: receipt_token,
            exec: Exec::prepare(command)?,
        })
    }
    pub(crate) fn spawn(&self) -> std::io::Result<Child> {
        // All allocation/descriptor ownership and pointer arrays precede BOTH
        // raw forks. The child never returns or unwinds into parent Rust code.
        let stdio = || -> std::io::Result<(OwnedFd, OwnedFd)> {
            let mut fds = [-1; 2];
            if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let (a, b) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
            Ok((
                above_stdio(a).map_err(std::io::Error::other)?,
                above_stdio(b).map_err(std::io::Error::other)?,
            ))
        };
        let (input, stdin) = stdio()?;
        let (stdout, output) = stdio()?;
        let (stderr, diagnostics) = stdio()?;
        let (errors, error_write) = pipe().map_err(std::io::Error::other)?;
        let app = std::process::id();
        let descriptors = [
            input.as_raw_fd(),
            output.as_raw_fd(),
            diagnostics.as_raw_fd(),
            error_write.as_raw_fd(),
            self.child_stop.as_raw_fd(),
            self.child_report.as_raw_fd(),
            self.receipt.as_raw_fd(),
            self.registration.as_ref().map_or(-1, AsRawFd::as_raw_fd),
        ];
        // Block inherited application handlers around fork in THIS thread only.
        // Guardian never unblocks them; Node resets dispositions before restoring
        // the original mask. This also prevents arbitrary child signal callbacks.
        let all = u64::MAX;
        let mut old_mask = 0u64;
        let masked = unsafe {
            syscall(
                libc::SYS_rt_sigprocmask,
                libc::SIG_SETMASK as usize,
                &all as *const _ as usize,
                &mut old_mask as *mut _ as usize,
                8,
                0,
            )
        };
        if masked < 0 {
            return Err(std::io::Error::from_raw_os_error(-masked as i32));
        }
        let pid = unsafe { syscall(libc::SYS_clone, libc::SIGCHLD as usize, 0, 0, 0, 0) };
        if pid == 0 {
            unsafe { launch_child(app, descriptors, &self.exec, &self.token, old_mask) }
        }
        unsafe {
            syscall(
                libc::SYS_rt_sigprocmask,
                libc::SIG_SETMASK as usize,
                &old_mask as *const _ as usize,
                0,
                8,
                0,
            );
        }
        if pid < 0 {
            return Err(std::io::Error::from_raw_os_error(-pid as i32));
        }
        drop(input);
        drop(output);
        drop(diagnostics);
        drop(error_write);
        let handle = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
        let mut child = Child {
            pid: pid as u32,
            handle: if handle >= 0 {
                Some(unsafe { OwnedFd::from_raw_fd(handle) })
            } else {
                None
            },
            status: None,
            stdin: Some(File::from(stdin)),
            stdout: Some(File::from(stdout)),
            stderr: Some(File::from(stderr)),
        };
        let end = Instant::now() + Duration::from_secs(5);
        let mut errno = [0u8; 4];
        let mut offset = 0;
        let error = loop {
            let count = unsafe {
                libc::read(
                    errors.as_raw_fd(),
                    errno[offset..].as_mut_ptr().cast(),
                    4 - offset,
                )
            };
            if count == 0 {
                break if offset == 0 { None } else { Some(libc::EIO) };
            }
            if count > 0 {
                offset += count as usize;
                if offset == 4 {
                    break Some(i32::from_ne_bytes(errno));
                }
            } else if Instant::now() >= end {
                break Some(libc::ETIMEDOUT);
            } else {
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        if let Some(errno) = error {
            self.request_stop();
            let end = Instant::now() + Duration::from_secs(7);
            while child.try_wait()?.is_none() && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(5));
            }
            return Err(std::io::Error::from_raw_os_error(errno));
        }
        Ok(child)
    }
    fn request_stop(&self) {
        unsafe {
            libc::write(self.stop.as_raw_fd(), b"x".as_ptr().cast(), 1);
        }
    }
    pub(crate) fn started(self) -> Result<(Control, u32)> {
        drop(self.child_stop);
        drop(self.child_report);
        drop(self.receipt);
        drop(self.registration);
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut bytes = [0; 4];
        let mut offset = 0;
        while offset < bytes.len() {
            let count = unsafe {
                libc::read(
                    self.report.as_raw_fd(),
                    bytes[offset..].as_mut_ptr().cast(),
                    bytes.len() - offset,
                )
            };
            if count > 0 {
                offset += count as usize;
            } else if count == 0 {
                anyhow::bail!("guardian launch report closed");
            } else {
                let e = std::io::Error::last_os_error();
                ensure!(
                    matches!(e.raw_os_error(), Some(libc::EAGAIN | libc::EINTR)),
                    "guardian launch report: {e}"
                );
                ensure!(
                    Instant::now() < deadline,
                    "guardian launch report timed out"
                );
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        let pid = u32::from_ne_bytes(bytes);
        ensure!(
            pid > 1 && pid <= i32::MAX as u32,
            "invalid guardian child report"
        );
        Ok((Control(self.stop), pid))
    }
}

pub(crate) struct Child {
    pid: u32,
    handle: Option<OwnedFd>,
    status: Option<std::process::ExitStatus>,
    pub(crate) stdin: Option<File>,
    pub(crate) stdout: Option<File>,
    pub(crate) stderr: Option<File>,
}
impl Child {
    pub(crate) fn id(&self) -> u32 {
        self.pid
    }
    pub(crate) fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.reap(libc::WNOHANG)
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.reap(0)?
            .ok_or_else(|| std::io::Error::from_raw_os_error(libc::ECHILD))
    }
    fn reap(&mut self, flags: i32) -> std::io::Result<Option<std::process::ExitStatus>> {
        if let Some(status) = self.status {
            return Ok(Some(status));
        }
        let mut status = 0;
        loop {
            let result = unsafe { libc::waitpid(self.pid as i32, &mut status, flags) };
            if result > 0 {
                self.status = Some(std::process::ExitStatus::from_raw(status));
                return Ok(self.status);
            }
            if result == 0 {
                return Ok(None);
            }
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EINTR) {
                return Err(error);
            }
        }
    }
    pub(crate) fn kill(&mut self) -> std::io::Result<()> {
        if self.status.is_some() {
            return Ok(());
        }
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| std::io::Error::from_raw_os_error(libc::EBADF))?;
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        if unsafe {
            libc::waitid(
                libc::P_PIDFD,
                handle.as_raw_fd() as u32,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        } < 0
        {
            return Err(std::io::Error::last_os_error());
        }
        if unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                handle.as_raw_fd(),
                libc::SIGKILL,
                0usize,
                0,
            )
        } < 0
        {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

unsafe fn exec_error(fd: i32, errno: i32) -> ! {
    let bytes = errno.to_ne_bytes();
    unsafe {
        syscall(
            libc::SYS_write,
            fd as usize,
            bytes.as_ptr() as usize,
            4,
            0,
            0,
        );
        exit(125)
    }
}
unsafe fn reset_dispositions() -> i64 {
    // Linux kernel ABI (not libc sigaction layout). In particular SIGCHLD must
    // not inherit SIG_IGN/SA_NOCLDWAIT: this guardian requires explicit reaping.
    #[repr(C)]
    struct KernelSigaction {
        handler: usize,
        flags: usize,
        restorer: usize,
        mask: u64,
    }
    let action = KernelSigaction {
        handler: 0,
        flags: 0,
        restorer: 0,
        mask: 0,
    };
    let mut signal = 1;
    while signal <= 64 {
        if signal != libc::SIGKILL && signal != libc::SIGSTOP {
            let result = unsafe {
                syscall(
                    libc::SYS_rt_sigaction,
                    signal as usize,
                    &action as *const _ as usize,
                    0,
                    8,
                    0,
                )
            };
            if result < 0 {
                return result;
            }
        }
        signal += 1;
    }
    0
}
unsafe fn launch_child(app: u32, fds: [i32; 8], exec: &Exec, token: &[u8; 36], old_mask: u64) -> ! {
    let [
        input,
        output,
        diagnostics,
        error,
        stop,
        report,
        receipt,
        registration,
    ] = fds;
    let result = unsafe { reset_dispositions() };
    if result < 0 {
        unsafe { exec_error(error, -result as i32) }
    }
    if unsafe { syscall(libc::SYS_getppid, 0, 0, 0, 0, 0) } != app as i64
        || unsafe { syscall(libc::SYS_setsid, 0, 0, 0, 0, 0) } < 0
        || unsafe {
            syscall(
                libc::SYS_prctl,
                libc::PR_SET_CHILD_SUBREAPER as usize,
                1,
                0,
                0,
                0,
            )
        } < 0
    {
        unsafe { exec_error(error, libc::ECHILD) }
    }
    let parent = unsafe { syscall(libc::SYS_pidfd_open, app as usize, 0, 0, 0, 0) } as i32;
    if parent < 0 || unsafe { syscall(libc::SYS_getppid, 0, 0, 0, 0, 0) } != app as i64 {
        unsafe { exec_error(error, libc::ECHILD) }
    }
    let ancestor = unsafe { syscall(libc::SYS_getpid, 0, 0, 0, 0, 0) };
    let identity = if registration >= 0 {
        let Some(identity) = (unsafe { register_birth(registration, ancestor as u32) }) else {
            unsafe { exec_error(error, libc::EIO) }
        };
        unsafe {
            close(registration);
        }
        Some(identity)
    } else {
        None
    };
    let node = unsafe { syscall(libc::SYS_clone, libc::SIGCHLD as usize, 0, 0, 0, 0) };
    if node < 0 {
        unsafe { exec_error(error, -node as i32) }
    }
    if node == 0 {
        if unsafe {
            syscall(
                libc::SYS_prctl,
                libc::PR_SET_PDEATHSIG as usize,
                libc::SIGKILL as usize,
                0,
                0,
                0,
            )
        } < 0
            || unsafe { syscall(libc::SYS_getppid, 0, 0, 0, 0, 0) } != ancestor
        {
            unsafe { exec_error(error, libc::ECHILD) }
        }
        for (source, destination) in [(input, 0), (output, 1), (diagnostics, 2)] {
            let result = unsafe { syscall(libc::SYS_dup3, source as usize, destination, 0, 0, 0) };
            if result < 0 {
                unsafe { exec_error(error, -result as i32) }
            }
        }
        if let Some(cwd) = &exec.cwd {
            let result = unsafe { syscall(libc::SYS_chdir, cwd.as_ptr() as usize, 0, 0, 0, 0) };
            if result < 0 {
                unsafe { exec_error(error, -result as i32) }
            }
        }
        // All guardian controls, receipt, registration and unrelated inherited
        // descriptors are absent after exec, not merely "intended CLOEXEC".
        if !unsafe { close_except([0, 1, 2, error]) } {
            unsafe { exec_error(error, libc::EIO) }
        }
        let result = unsafe {
            syscall(
                libc::SYS_rt_sigprocmask,
                libc::SIG_SETMASK as usize,
                &old_mask as *const _ as usize,
                0,
                8,
                0,
            )
        };
        if result < 0 {
            unsafe { exec_error(error, -result as i32) }
        }
        let result = unsafe {
            syscall(
                libc::SYS_execve,
                exec.program.as_ptr() as usize,
                exec.argv.as_ptr() as usize,
                exec.envp.as_ptr() as usize,
                0,
                0,
            )
        };
        unsafe { exec_error(error, -result as i32) }
    }
    unsafe {
        guardian(
            node as i32,
            parent,
            stop,
            report,
            receipt,
            Settlement {
                token,
                identity: identity.as_ref(),
            },
        )
    }
}

// Fixed-buffer kernel birth registration, before any executable can launch.
// No formatting, /proc PID string construction, allocator or errno TLS.
unsafe fn register_birth(destination: i32, pid: u32) -> Option<[u8; 12]> {
    let fd = unsafe {
        syscall(
            libc::SYS_openat,
            libc::AT_FDCWD as usize,
            c"/proc/thread-self/stat".as_ptr() as usize,
            (libc::O_RDONLY | libc::O_CLOEXEC) as usize,
            0,
            0,
        )
    };
    if fd < 0 {
        return None;
    }
    let mut buffer = [0u8; 4096];
    let count = unsafe {
        syscall(
            libc::SYS_read,
            fd as usize,
            buffer.as_mut_ptr() as usize,
            buffer.len(),
            0,
            0,
        )
    };
    unsafe {
        close(fd as i32);
    }
    if count <= 0 || count as usize >= buffer.len() {
        return None;
    }
    let mut end = 0;
    let mut i = 0;
    while i < count as usize {
        if buffer[i] == b')' {
            end = i;
        }
        i += 1;
    }
    if end == 0 {
        return None;
    }
    i = end + 1;
    let mut field = 0;
    let mut birth = 0u64;
    while i < count as usize {
        while i < count as usize && buffer[i] == b' ' {
            i += 1;
        }
        let start = i;
        while i < count as usize && buffer[i] != b' ' && buffer[i] != b'\n' {
            i += 1;
        }
        if field == 19 {
            let mut j = start;
            while j < i {
                if buffer[j] < b'0' || buffer[j] > b'9' {
                    return None;
                }
                let value = birth
                    .checked_mul(10)
                    .and_then(|v| v.checked_add((buffer[j] - b'0') as u64))?;
                birth = value;
                j += 1;
            }
            break;
        }
        field += 1;
    }
    if birth == 0 {
        return None;
    }
    let mut identity = [0u8; 12];
    identity[..4].copy_from_slice(&pid.to_ne_bytes());
    identity[4..].copy_from_slice(&birth.to_ne_bytes());
    let written = unsafe {
        syscall(
            libc::SYS_write,
            destination as usize,
            identity.as_ptr() as usize,
            identity.len(),
            0,
            0,
        ) == 12
            && syscall(libc::SYS_fsync, destination as usize, 0, 0, 0, 0) == 0
    };
    written.then_some(identity)
}
pub(crate) struct Control(OwnedFd);
impl Control {
    pub(crate) fn stop(&self) {
        // One byte fits the private pipe; no worker/stdout can block this owner.
        unsafe {
            libc::write(self.0.as_raw_fd(), b"x".as_ptr().cast(), 1);
        }
    }
}

// All functions below are syscall-only, operating on fixed stack buffers.
unsafe fn close_except(mut keep: [RawFd; 4]) -> bool {
    // Fixed insertion sort, no library sort/panic/allocator path.
    let mut i = 1;
    while i < 4 {
        let mut j = i;
        while j > 0 && keep[j - 1] > keep[j] {
            keep.swap(j - 1, j);
            j -= 1;
        }
        i += 1;
    }
    let mut from = 0u32;
    for fd in keep {
        if from < fd as u32
            && unsafe {
                syscall(
                    libc::SYS_close_range,
                    from as usize,
                    fd as usize - 1,
                    0,
                    0,
                    0,
                )
            } != 0
        {
            return false;
        }
        from = fd as u32 + 1;
    }
    unsafe {
        syscall(
            libc::SYS_close_range,
            from as usize,
            u32::MAX as usize,
            0,
            0,
            0,
        ) == 0
    }
}
unsafe fn milliseconds() -> Option<i64> {
    let mut time: libc::timespec = unsafe { std::mem::zeroed() };
    if unsafe {
        syscall(
            libc::SYS_clock_gettime,
            libc::CLOCK_MONOTONIC as usize,
            &mut time as *mut _ as usize,
            0,
            0,
            0,
        )
    } < 0
    {
        return None;
    }
    time.tv_sec
        .checked_mul(1000)?
        .checked_add(time.tv_nsec / 1_000_000)
}
unsafe fn kill_children() -> bool {
    // /proc/self refers to the guardian, not the original multithreaded app.
    let fd = unsafe {
        syscall(
            libc::SYS_openat,
            libc::AT_FDCWD as usize,
            c"/proc/thread-self/children".as_ptr() as usize,
            (libc::O_RDONLY | libc::O_CLOEXEC) as usize,
            0,
            0,
        )
    } as i32;
    if fd < 0 {
        return false;
    }
    let mut bytes = [0u8; 65536];
    let count = unsafe {
        syscall(
            libc::SYS_read,
            fd as usize,
            bytes.as_mut_ptr() as usize,
            bytes.len(),
            0,
            0,
        )
    };
    unsafe {
        close(fd);
    }
    if count < 0 || count as usize == bytes.len() {
        return false;
    }
    let mut pid = 0i32;
    for byte in bytes[..count as usize]
        .iter()
        .copied()
        .chain(std::iter::once(b' '))
    {
        if byte.is_ascii_digit() {
            let Some(next) = pid
                .checked_mul(10)
                .and_then(|v| v.checked_add((byte - b'0') as i32))
            else {
                return false;
            };
            pid = next;
        } else if byte == b' ' || byte == b'\n' {
            if pid > 1 {
                let handle =
                    unsafe { syscall(libc::SYS_pidfd_open, pid as usize, 0, 0, 0, 0) } as i32;
                if handle >= 0 {
                    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
                    // P_PIDFD + waitid proves this exact kernel process is STILL
                    // our child. ECHILD (including a reused PID) grants nothing.
                    if unsafe {
                        syscall(
                            libc::SYS_waitid,
                            libc::P_PIDFD as usize,
                            handle as usize,
                            &mut info as *mut _ as usize,
                            (libc::WEXITED | libc::WNOHANG | libc::WNOWAIT) as usize,
                            0,
                        )
                    } == 0
                    {
                        unsafe {
                            syscall(
                                libc::SYS_pidfd_send_signal,
                                handle as usize,
                                libc::SIGKILL as usize,
                                0,
                                0,
                                0,
                            );
                        }
                    }
                    unsafe {
                        close(handle);
                    }
                }
            }
            pid = 0;
        } else {
            return false;
        }
    }
    true
}
struct Settlement<'a> {
    token: &'a [u8; 36],
    identity: Option<&'a [u8; 12]>,
}
unsafe fn guardian(
    node: i32,
    parent: RawFd,
    stop: RawFd,
    report: RawFd,
    receipt: RawFd,
    settlement: Settlement<'_>,
) -> ! {
    if !unsafe { close_except([parent, stop, report, receipt]) } {
        unsafe {
            exit(125);
        }
    }
    let bytes = (node as u32).to_ne_bytes();
    if unsafe {
        syscall(
            libc::SYS_write,
            report as usize,
            bytes.as_ptr() as usize,
            bytes.len(),
            0,
            0,
        )
    } != bytes.len() as i64
    {
        // Closed app report means parent launch failed: drain owned children.
    }
    unsafe {
        close(report);
    }
    let mut stopping = false;
    let mut end = 0;
    loop {
        let mut status = 0;
        let reaped = unsafe {
            syscall(
                libc::SYS_wait4,
                -1i32 as usize,
                &mut status as *mut _ as usize,
                libc::WNOHANG as usize,
                0,
                0,
            )
        };
        if reaped > 0 {
            if reaped == node as i64 {
                stopping = true;
            }
            continue;
        }
        if reaped == -(libc::ECHILD as i64) {
            // Only the kernel's zero-descendant proof grants deletion of scratch.
            let mut receipt_bytes = [0u8; 48];
            let length = if let Some(identity) = settlement.identity {
                receipt_bytes[..12].copy_from_slice(identity);
                receipt_bytes[12..].copy_from_slice(settlement.token);
                48
            } else {
                receipt_bytes[..36].copy_from_slice(settlement.token);
                36
            };
            let wrote = unsafe {
                syscall(
                    libc::SYS_write,
                    receipt as usize,
                    receipt_bytes.as_ptr() as usize,
                    length,
                    0,
                    0,
                )
            };
            let synced = unsafe { syscall(libc::SYS_fsync, receipt as usize, 0, 0, 0, 0) };
            unsafe {
                exit(if wrote == length as i64 && synced == 0 {
                    0
                } else {
                    125
                });
            }
        }
        let mut fds = [
            libc::pollfd {
                fd: parent,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: stop,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        unsafe {
            let time = libc::timespec {
                tv_sec: 0,
                tv_nsec: 10_000_000,
            };
            syscall(
                libc::SYS_ppoll,
                fds.as_mut_ptr() as usize,
                2,
                &time as *const _ as usize,
                0,
                0,
            );
        }
        stopping |= fds.iter().any(|fd| fd.revents != 0);
        if stopping {
            if end == 0 {
                end = match unsafe { milliseconds() }.and_then(|v| v.checked_add(5000)) {
                    Some(value) => value,
                    None => unsafe { exit(124) },
                };
            }
            let _ = unsafe { kill_children() };
            if unsafe { milliseconds() }.is_none_or(|now| now >= end) {
                // Never publish a receipt if a child is unkillable/unreaped.
                unsafe {
                    exit(124);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Read,
        os::unix::{fs::OpenOptionsExt, process::CommandExt},
        process::Stdio,
    };
    static TRACE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
    unsafe extern "C" fn atfork_trace() {
        let fd = TRACE.load(std::sync::atomic::Ordering::Relaxed);
        if fd >= 0 {
            let pid = unsafe { syscall(libc::SYS_getpid, 0, 0, 0, 0, 0) } as i32;
            unsafe {
                syscall(
                    libc::SYS_write,
                    fd as usize,
                    pid.to_ne_bytes().as_ptr() as usize,
                    4,
                    0,
                    0,
                );
            }
        }
    }
    #[test]
    fn inherited_atfork_callback_never_runs_in_guardian() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let trace = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(dir.path().join("trace"))?;
        TRACE.store(trace.as_raw_fd(), std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            unsafe {
                libc::pthread_atfork(Some(atfork_trace), Some(atfork_trace), Some(atfork_trace))
            },
            0
        );
        let mut control = Command::new("/bin/true");
        unsafe {
            control.pre_exec(|| Ok(()));
        }
        assert!(control.spawn()?.wait()?.success());
        let before = std::fs::read(dir.path().join("trace"))?;
        assert!(
            !before.is_empty(),
            "callbacks actually invoked by libc control"
        );
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let flag = running.clone();
        let thread = std::thread::spawn(move || {
            while flag.load(std::sync::atomic::Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        let receipt = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(dir.path().join("settled"))?;
        let token = uuid::Uuid::new_v4().to_string();
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let launch = Launch::prepare(&mut command, &receipt, token.as_bytes())?;
        let mut child = launch.spawn()?;
        let (control, _) = launch.started()?;
        control.stop();
        let status = child.wait()?;
        running.store(false, std::sync::atomic::Ordering::Release);
        thread.join().unwrap();
        TRACE.store(-1, std::sync::atomic::Ordering::Relaxed);
        assert!(status.success());
        let bytes = std::fs::read(dir.path().join("trace"))?;
        assert_eq!(
            bytes, before,
            "an inherited prepare/parent/child callback ran in raw launch"
        );
        assert_eq!(std::fs::read(dir.path().join("settled"))?, token.as_bytes());
        Ok(())
    }
    #[test]
    fn detached_mid_fork_children_are_kernel_owned_not_recaptured_by_uid() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let receipt = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(dir.path().join("settled"))?;
        let token = uuid::Uuid::new_v4().to_string();
        let mut unrelated = Command::new("/bin/sleep")
            .arg("30")
            .env("TMPDIR", dir.path())
            .spawn()?;
        // Shell is inert test input only; product always execs verified Node.
        // Every setsid/double-fork descendant belongs to this dedicated guardian,
        // including those forking while stop is requested. No polling ancestry
        // registration can miss or lose their kernel adoption.
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "echo ready; i=0; while [ $i -lt 32 ]; do setsid /bin/sh -c '/bin/sleep 30 & wait' & i=$((i+1)); done; wait"])
            .stdout(Stdio::piped()).stderr(Stdio::null());
        let launch = Launch::prepare(&mut command, &receipt, token.as_bytes())?;
        let mut child = launch.spawn()?;
        let (control, _) = launch.started()?;
        let mut byte = [0; 1];
        child.stdout.as_mut().unwrap().read_exact(&mut byte)?;
        control.stop();
        assert!(
            child.wait()?.success(),
            "guardian failed to prove/reap every child"
        );
        assert_eq!(std::fs::read(dir.path().join("settled"))?, token.as_bytes());
        assert!(
            unrelated.try_wait()?.is_none(),
            "same UID/TMPDIR is NOT ownership"
        );
        unrelated.kill()?;
        unrelated.wait()?;
        Ok(())
    }
}
