pub struct Whoami {
    pub uid: u32,
    pub gid: u32,
    pub username: Option<String>,
}

#[cfg(windows)]
pub fn whoami() -> Whoami {
    Whoami {
        uid: 1000,
        gid: 1000,
        username: None,
    }
}

#[cfg(not(windows))]
mod unix {
    pub use std::ffi::CStr;
    pub use std::mem::MaybeUninit;
    use std::os::raw::c_char;

    // Safety: C getpwuid_r expects buffers to be uninitialized and the appropriate length
    const STRING_BUF_LEN: usize = libc::_SC_GETPW_R_SIZE_MAX as usize;

    /// A safe and easy to use zero cost wrapper for libc::passwd.
    /// Cannot be created outside this module because the inner field is private, so user can't make an invalid one.
    pub struct SafePasswd<'a>(&'a libc::passwd);

    impl<'a> SafePasswd<'a> {
        /// calls libc::getpwuid_r but forces params to be correct and enforces the lifetime contract.
        /// user cant access/move/drop the buffers until return value is dropped.
        pub fn getpwuid_r(
            uid: u32,
            pwd_buf: &'a mut MaybeUninit<libc::passwd>,
            string_buf: &'a mut MaybeUninit<[c_char; STRING_BUF_LEN]>,
        ) -> Option<SafePasswd<'a>> {
            let mut pwd_ptr = MaybeUninit::uninit(); // uninitialized pointer to libc::passwd
            unsafe {
                // Safety: getpwuid_r is reentrant and writes only to our buffers here
                // Safety: pwd_buf is the libc-defined size and shape
                // Safety: string_buf is the appropriate length and its length is passed
                let _errno = libc::getpwuid_r(
                    uid,
                    pwd_buf.as_mut_ptr(), // ptr to uninit struct
                    string_buf.as_mut_ptr() as *mut c_char, // ptr to first byte of uninit c_char array
                    STRING_BUF_LEN,
                    pwd_ptr.as_mut_ptr(), // ptr to uninit ptr to libc::passwd. *result is always written.
                );
                // Safety: by contract with getpwuid_r, pwd_ptr is now either null or a valid pointer to an initialized pwd_buf
                // Safety: if pwd_ptr non-null, C string pointers point somewhere valid inside string_buf.
                pwd_ptr.assume_init().as_ref()
            }
            // Safety: return value borrows both buffers that back it. User can't make it invalid without using unsafe.
            .map(|c| SafePasswd(c))
        }

        /// safe access to pw_name
        pub fn pw_name(&'a self) -> &'a CStr {
            // see read why this is always safe.
            unsafe { CStr::from_ptr(self.0.pw_name) }
        }
    }
}

#[cfg(not(windows))]
pub fn whoami() -> Whoami {
    let uid = unsafe { libc::geteuid() };
    let gid = unsafe { libc::getegid() };

    use unix::*;
    let mut pwd_buf = MaybeUninit::uninit();
    let mut string_buf = MaybeUninit::uninit();
    let opt_safe_pwd = SafePasswd::getpwuid_r(uid, &mut pwd_buf, &mut string_buf);
    // pwd_buf and string_buf are now borrowed by opt_safe_pwd and we can't violate any contract.

    Whoami {
        uid: uid as _,
        gid: gid as _,
        username: opt_safe_pwd.map(|pwd| pwd.pw_name().to_string_lossy().to_string()),
    }
}
