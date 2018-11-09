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

#[cfg(target_arch = "x86")]
const JMP_SIZE: usize = 7;

#[cfg(target_arch = "x86_64")]
const JMP_SIZE: usize = 12;

#[cfg(target_arch = "x86")]
#[inline]
fn assemble_jmp_to_address(address: usize) -> [u8; JMP_SIZE] {
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
    ]
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn assemble_jmp_to_address(address: usize) -> [u8; JMP_SIZE] {
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
    ]
}

/// When this structure is dropped (falls out of scope), the patch will be reverted and the function will return
/// to its original state.
pub struct PatchGuard {
    ptr: *mut u8,
    data: [u8; JMP_SIZE],
}

impl Drop for PatchGuard {
    fn drop(&mut self) {
        unsafe {
            copy_to_protected_address(self.ptr, &self.data[..]);
        }
    }
}

macro_rules! define_patch {
    ($name:ident($($arguments:ident,)*)) => (
        /// Patch replaces a function with another. Accepts closures as replacement functions as long as they
        /// do not bind to the environment.
        pub fn $name<R, $($arguments,)*>(target: fn($($arguments,)*) -> R, func: fn($($arguments,)*) -> R) -> PatchGuard {
            let target = target as *mut u8;
            let patch = assemble_jmp_to_address(func as *const () as usize);
            let mut original = [0; JMP_SIZE];

            unsafe {
                core::ptr::copy(target, original.as_mut_ptr(), original.len());
            }

            unsafe {
                copy_to_protected_address(target, &patch[..]);
            }

            PatchGuard {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike, Utc};

    fn the_ultimate_question() -> u32 {
        42
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
