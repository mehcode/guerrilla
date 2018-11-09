extern crate libc;

use libc::{sysconf, _SC_PAGESIZE, mprotect, PROT_EXEC, PROT_READ, PROT_WRITE};
use std::{ptr, mem::transmute};

unsafe fn mprotect_ptr(dst: *mut u8, prot: i32) {
    let page_size = sysconf(_SC_PAGESIZE) as usize;
    let page_start = (dst as usize) & !(page_size - 1);
    let rv = mprotect(transmute(page_start), page_size, prot);

    // TODO: Make this a proper error
    assert_eq!(rv, 0);
}

unsafe fn copy_into_exec(dst: *mut u8, src: &[u8]) {
    mprotect_ptr(dst, PROT_READ | PROT_WRITE | PROT_EXEC);
    ptr::copy(src.as_ptr(), dst, src.len());
    mprotect_ptr(dst, PROT_READ | PROT_EXEC);
}

#[cfg(any(unix, macos))]
fn jmp_to_function_value(to: usize) -> Vec<u8> {
    vec![
        // movabs rdx, {to}
        0x48,
        0xBA,
        to as u8,
        (to >> 8) as u8,
        (to >> 16) as u8,
        (to >> 24) as u8,
        (to >> 32) as u8,
        (to >> 40) as u8,
        (to >> 48) as u8,
        (to >> 56) as u8,
        // jmp rdx
        0xFF,
        0xe2,
    ]
}

pub struct Patch {
    ptr: *mut u8,
    data: Vec<u8>,
}

impl Drop for Patch {
    fn drop(&mut self) {
        unsafe {
            copy_into_exec(self.ptr, &self.data);
        }
    }
}

macro_rules! define_patch {
    ($name:ident($($arguments:ident,)*)) => (
        pub fn $name<R, $($arguments,)*>(target: fn($($arguments,)*) -> R, func: fn($($arguments,)*) -> R) -> Patch {
            let target = target as *mut u8;
            let patch = jmp_to_function_value(func as *const () as usize);

            let mut original = vec![0; patch.len()];

            unsafe {
                ptr::copy(target, original.as_mut_ptr(), original.len());
            }

            unsafe {
                copy_into_exec(target, &patch);
            }

            Patch {
                ptr: target,
                data: original,
            }
        }
    );
}

define_patch!(patch0());
define_patch!(patch1(A,));
define_patch!(patch2(A, B,));
define_patch!(patch3(A, B, C,));
define_patch!(patch4(A, B, C, D,));
define_patch!(patch5(A, B, C, D, E,));
define_patch!(patch6(A, B, C, D, E, F,));
define_patch!(patch7(A, B, C, D, E, F, G,));
define_patch!(patch8(A, B, C, D, E, F, G, H,));
define_patch!(patch9(A, B, C, D, E, F, G, H, I,));
