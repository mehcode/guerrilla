#![no_std]

#[cfg(windows)]
extern crate winapi;

#[cfg(any(macos, unix))]
extern crate libc;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
extern crate chrono;

#[cfg(windows)]
unsafe fn copy_to_protected_address(dst: *mut u8, src: &[u8]) {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::memoryapi::VirtualProtect;
    use winapi::um::winnt::PAGE_EXECUTE_READWRITE;

    let mut old_permissions: DWORD = 0;
    let rv = VirtualProtect(
        dst as _,
        src.len(),
        PAGE_EXECUTE_READWRITE,
        (&mut old_permissions) as _,
    );

    assert_eq!(rv, 1);

    core::ptr::copy(src.as_ptr(), dst, src.len());

    let mut temp: DWORD = 0;
    let rv = VirtualProtect(dst as _, src.len(), old_permissions, (&mut temp) as _);

    assert_eq!(rv, 1);
}

#[cfg(any(macos, unix))]
unsafe fn copy_to_protected_address(dst: *mut u8, src: &[u8]) {
    use libc::{c_void, mprotect, sysconf, PROT_EXEC, PROT_READ, PROT_WRITE, _SC_PAGESIZE};

    let page_size = sysconf(_SC_PAGESIZE) as usize;
    let page_start = ((dst as usize) & !(page_size - 1)) as *mut c_void;

    let rv = mprotect(page_start, page_size, PROT_EXEC | PROT_READ | PROT_WRITE);
    assert_eq!(rv, 0);

    core::ptr::copy(src.as_ptr(), dst, src.len());

    let rv = mprotect(page_start, page_size, PROT_EXEC | PROT_READ);
    assert_eq!(rv, 0);
}

const JMP_MAX_SIZE: usize = 12;

#[inline]
fn assemble_jmp_to_address(address: usize, relative: isize) -> ([u8; JMP_MAX_SIZE], usize) {
    if (relative >= (core::i8::MIN as isize)) && (relative <= (core::i8::MAX as isize)) {
        (
            [
                // jmp rel8
                0xEB,
                relative as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            2,
        )
    } else if cfg!(target_arch = "x86") {
        (
            [
                // mov edx, #
                0xBA,
                address as u8,
                (address >> 8) as u8,
                (address >> 16) as u8,
                (address >> 24) as u8,
                // jmp edx
                0xFF,
                0xE2,
                0,
                0,
                0,
                0,
                0,
            ],
            7,
        )
    } else if (relative >= (core::i32::MIN as isize)) && (relative <= (core::i32::MAX as isize)) {
        (
            [
                // jmp rel32
                0xE9,
                relative as u8,
                (relative >> 8) as u8,
                (relative >> 16) as u8,
                (relative >> 24) as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            5,
        )
    } else {
        (
            [
                // movabs rdx, #
                0x48,
                0xBA,
                address as u8,
                (address >> 8) as u8,
                (address >> 16) as u8,
                (address >> 24) as u8,
                (address >> 32) as u8,
                (address >> 40) as u8,
                (address >> 48) as u8,
                (address >> 56) as u8,
                // jmp rdx
                0xFF,
                0xE2,
            ],
            12,
        )
    }
}

/// When this structure is dropped (falls out of scope), the patch will be reverted and the function will return
/// to its original state.
pub struct PatchGuard {
    ptr: *mut u8,
    len: usize,
    data: [u8; JMP_MAX_SIZE],
}

impl Drop for PatchGuard {
    fn drop(&mut self) {
        unsafe {
            copy_to_protected_address(self.ptr, &self.data[..self.len]);
        }
    }
}

macro_rules! define_patch {
    ($name:ident($($arguments:ident,)*)) => (
        /// Patch replaces a function with another. Accepts closures as replacement functions as long as they
        /// do not bind to the environment.
        pub fn $name<R, $($arguments,)*>(target: fn($($arguments,)*) -> R, func: fn($($arguments,)*) -> R) -> PatchGuard {
            let target = target as *mut u8;
            let mut original = [0; JMP_MAX_SIZE];

            let target_addr = target as usize;
            let func_addr = func as usize;

            let rel = (if target_addr > func_addr {
                -((target_addr - func_addr) as isize)
            } else {
                ((func_addr - target_addr) as isize)
            }) - 5;

            let (patch, len) = assemble_jmp_to_address(func_addr, rel);

            unsafe {
                core::ptr::copy(target, original.as_mut_ptr(), len);
            }

            unsafe {
                copy_to_protected_address(target, &patch[..len]);
            }

            PatchGuard {
                ptr: target,
                len,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike, Utc};

    fn the_ultimate_question() -> u32 {
        42
    }
    fn other_question() -> u32 {
        23
    }
    fn default<T: Default>() -> T {
        T::default()
    }

    #[test]
    fn test_patch() {
        assert_eq!(the_ultimate_question(), 42);

        {
            let _guard = patch0(the_ultimate_question, || 24);

            assert_eq!(the_ultimate_question(), 24);
        }

        assert_eq!(the_ultimate_question(), 42);
    }

    #[test]
    fn test_out_of_order_drop() {
        assert_eq!(the_ultimate_question(), 42);

        let guard_a = patch0(the_ultimate_question, || 24);
        let guard_b = patch0(the_ultimate_question, || 23);

        core::mem::drop(guard_a);
        assert_eq!(the_ultimate_question(), 42);

        core::mem::drop(guard_b);
        // Uh oh.
        assert_eq!(the_ultimate_question(), 24);

        if let Err(e) = std::panic::catch_unwind(|| {
            assert_eq!(
                42,
                the_ultimate_question(),
                "Guards dropped without restoring original value!"
            );
        }) {
            // Fix it for other tests before we re-raise
            core::mem::forget(patch0(the_ultimate_question, || 42));
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_functions_independent() {
        assert_eq!(the_ultimate_question(), 42);
        assert_eq!(other_question(), 23);

        {
            let _guard = patch0(the_ultimate_question, || 32);

            assert_eq!(the_ultimate_question(), 32);
            assert_eq!(other_question(), 23);
        }

        assert_eq!(the_ultimate_question(), 42);
        assert_eq!(other_question(), 23);
    }

    #[test]
    fn test_patch_generic() {
        assert_eq!(default::<i32>(), 0);

        {
            let _guard = patch0(default::<i32>, || 1);

            assert_eq!(default::<i32>(), 1);
        }

        assert_eq!(default::<i32>(), 0);
    }

    #[test]
    fn test_patch_external() {
        let now = Utc::now();
        assert!(now.year() >= 2018);

        {
            let _guard = patch0(Utc::now, || Utc.ymd(1, 1, 1).and_hms(1, 1, 1));

            let now = Utc::now();
            assert_eq!(now.year(), 1);
            assert_eq!(now.hour(), 1);
        }

        assert!(now.year() >= 2018);
    }
}
