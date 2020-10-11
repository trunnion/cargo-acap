use std::ffi::CStr;
use std::os::raw::c_char;

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
pub fn whoami() -> Whoami {
    let uid = unsafe { libc::geteuid() };
    let gid = unsafe { libc::getegid() };
    let username = {
        // Safety: C expects this buffer to be uninitialized
        let mut pwd_buf: libc::passwd = unsafe { std::mem::zeroed() };
        let mut string_buf = vec![0 as c_char; libc::_SC_GETPW_R_SIZE_MAX as usize];
        let mut pwd_ptr = std::ptr::null_mut::<libc::passwd>();

        // Safety: getpwuid_r is reentrant and writes only to our buffers here
        // pwd_buf is the libc-defined size and shape
        // string_buf is the appropriate length and its length is passed
        let _errno = unsafe {
            libc::getpwuid_r(
                uid,
                &mut pwd_buf as *mut libc::passwd,
                string_buf.as_mut_ptr(),
                string_buf.len(),
                &mut pwd_ptr as *mut *mut libc::passwd,
            );
        };

        // Safety: by contract with getpwuid_r, pwd_ptr is either null or a valid pointer to pwd_buf
        unsafe { pwd_ptr.as_ref() }
            // If we have a passwd struct, get the pw_name field
            .map(|pwd| pwd.pw_name)
            // Safety: pwd.pw_name points to a position within string_buf which is valid through the
            // end of this block
            .map(|ptr| unsafe { CStr::from_ptr(ptr) })
            // If we have a name, copy (force) it into a String
            .map(|cstr| cstr.to_string_lossy().into_owned())
    };

    Whoami {
        uid: uid as _,
        gid: gid as _,
        username,
    }
}
