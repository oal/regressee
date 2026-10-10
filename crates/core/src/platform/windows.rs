// SPDX-License-Identifier: MPL-2.0
use anyhow::{Result, ensure};
use std::{os::windows::ffi::OsStrExt, path::Path, ptr};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree},
    Security::{Authorization::*, *},
    Storage::FileSystem::{FILE_ALL_ACCESS, GetDiskFreeSpaceExW},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};
fn wide(path: &Path) -> Result<Vec<u16>> {
    let v: Vec<_> = path.as_os_str().encode_wide().collect();
    ensure!(!v.contains(&0), "NUL in native path");
    Ok(v.into_iter().chain([0]).collect())
}
pub(super) fn available_bytes(path: &Path) -> Result<u64> {
    let name = wide(path)?;
    let mut available = 0;
    ensure!(
        unsafe {
            GetDiskFreeSpaceExW(
                name.as_ptr(),
                &mut available,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        } != 0,
        "Cannot check Windows disk space: {}",
        std::io::Error::last_os_error()
    );
    Ok(available)
}
pub(super) fn protect(path: &Path) -> Result<()> {
    let mut name = wide(path)?;
    unsafe {
        let mut token = ptr::null_mut();
        ensure!(
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) != 0,
            "Cannot read private-directory owner"
        );
        // Use aligned storage: TOKEN_USER includes pointer-width fields.
        let mut needed = 0;
        GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut needed);
        let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
        let ok = GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        );
        CloseHandle(token);
        ensure!(ok != 0, "Cannot read private-directory SID");
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        let mut owner = ptr::null_mut();
        let mut descriptor = ptr::null_mut();
        let ownership = GetNamedSecurityInfoW(
            name.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut descriptor,
        );
        let owned = ownership == 0 && !owner.is_null() && EqualSid(owner, user.User.Sid) != 0;
        if !descriptor.is_null() {
            LocalFree(descriptor.cast());
        }
        ensure!(
            owned,
            "Private path is not owned by the current Windows user"
        );
        let access = EXPLICIT_ACCESS_W {
            grfAccessPermissions: FILE_ALL_ACCESS,
            grfAccessMode: SET_ACCESS,
            grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: ptr::null_mut(),
                MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_USER,
                ptstrName: user.User.Sid.cast(),
            },
        };
        let mut acl = ptr::null_mut();
        ensure!(
            SetEntriesInAclW(1, &access, ptr::null(), &mut acl) == 0,
            "Cannot create private-directory ACL"
        );
        let result = SetNamedSecurityInfoW(
            name.as_mut_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            acl,
            ptr::null(),
        );
        LocalFree(acl.cast());
        ensure!(
            result == 0,
            "Cannot protect Windows private directory (code {result})"
        );
    }
    Ok(())
}
