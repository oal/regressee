// SPDX-License-Identifier: MPL-2.0
//! Private, local-only named pipes. No TCP or public/inherited pipe DACL.
use super::*;
use std::{ffi::c_void, ptr};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree},
    Security::{Authorization::*, *},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

struct Security {
    descriptor: *mut c_void,
}
impl Security {
    fn current_user() -> Result<Self> {
        unsafe {
            let mut token = ptr::null_mut();
            ensure!(
                OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) != 0,
                "Cannot inspect pipe user"
            );
            let mut size = 0;
            GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut size);
            let mut bytes = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
            let ok =
                GetTokenInformation(token, TokenUser, bytes.as_mut_ptr().cast(), size, &mut size);
            CloseHandle(token);
            ensure!(ok != 0, "Cannot inspect pipe SID");
            let user = &*bytes.as_ptr().cast::<TOKEN_USER>();
            let mut sid = ptr::null_mut();
            ensure!(
                ConvertSidToStringSidW(user.User.Sid, &mut sid) != 0,
                "Cannot encode pipe SID"
            );
            let mut len = 0;
            while *sid.add(len) != 0 {
                len += 1;
            }
            let sid_string = String::from_utf16(std::slice::from_raw_parts(sid, len));
            LocalFree(sid.cast());
            let text = format!("D:P(A;;GA;;;{})", sid_string?);
            let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
            let mut descriptor = ptr::null_mut();
            ensure!(
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    wide.as_ptr(),
                    SDDL_REVISION_1,
                    &mut descriptor,
                    ptr::null_mut()
                ) != 0,
                "Cannot create private pipe DACL"
            );
            Ok(Self { descriptor })
        }
    }
    fn create(&self, name: &str, first: bool) -> Result<NamedPipeServer> {
        let mut attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.descriptor,
            bInheritHandle: 0,
        };
        // At capacity there are 16 admitted connections, the newly accepted
        // instance and its replacement. Leave room for both before rejecting
        // excess clients; otherwise an overload strands a connected listener.
        Ok(unsafe {
            ServerOptions::new()
                .first_pipe_instance(first)
                .reject_remote_clients(true)
                .max_instances(crate::protocol::MAX_CONNECTIONS + 2)
                .in_buffer_size(65536)
                .out_buffer_size(65536)
                .create_with_security_attributes_raw(
                    name,
                    (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
                )?
        })
    }
}
impl Drop for Security {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.descriptor);
        }
    }
}
// The descriptor is immutable; only the listener task creates pipe instances.
unsafe impl Send for Security {}
pub struct Listener {
    next: NamedPipeServer,
    name: String,
    security: Security,
}
fn name(paths: &AppPaths) -> String {
    format!(r"\\.\pipe\regressee-engine-{}", key(paths))
}
impl Listener {
    pub fn bind(paths: &AppPaths) -> Result<Self> {
        let security = Security::current_user()?;
        let name = name(paths);
        let next = security.create(&name, true)?;
        Ok(Self {
            next,
            name,
            security,
        })
    }
    pub async fn accept(&mut self) -> Result<Stream> {
        self.next.connect().await?;
        let next = self.security.create(&self.name, false)?;
        Ok(Box::new(std::mem::replace(&mut self.next, next)))
    }
}
pub async fn connect(paths: &AppPaths) -> Result<Stream> {
    let name = name(paths);
    let stream = pipe_connect::connect(|| ClientOptions::new().open(&name), IO_TIMEOUT).await?;
    Ok(Box::new(stream))
}
