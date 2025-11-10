use std::mem::{self, ManuallyDrop, MaybeUninit};

#[repr(C)]
pub struct FfiOption<T> {
    pub is_some: bool,
    pub value: MaybeUninit<T>,
}

impl<T> FfiOption<T> {
    pub const fn new_some(t: T) -> Self {
        FfiOption {
            is_some: true,
            value: MaybeUninit::new(t),
        }
    }
}

#[unsafe(no_mangle)]
pub static STACKPACK_PLUGIN_SHORT_NAME: &str = "wololooo";

#[unsafe(no_mangle)]
pub static STACKPACK_PLUGIN_DESCRIPTION: FfiOption<&str> = FfiOption::new_some("sample plugin rekt");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stackpack_plugin_drive_mutation(
    data: *const u8,
    data_len: usize,
    vec_buf_ptr: *mut *mut u8,
    vec_len: *mut usize,
    vec_cap: *mut usize,
) -> bool {
    unsafe {
        let slice = std::slice::from_raw_parts(data, data_len);
        let mut vec = Vec::from_raw_parts(*vec_buf_ptr, *vec_len, *vec_cap);

        match drive_mutation(slice, &mut vec) {
            Ok(()) => {
                *vec_buf_ptr = vec.as_mut_ptr();
                *vec_len = vec.len();
                *vec_cap = vec.capacity();
                mem::forget(vec);
                true
            }
            Err(e) => {
                eprintln!("encoding failed in wololooo plugin due to {:?}", e);
                false
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stackpack_plugin_revert_mutation(
    data: *const u8,
    data_len: usize,
    vec_buf_ptr: *mut *mut u8,
    vec_len: *mut usize,
    vec_cap: *mut usize,
) -> bool {
    unsafe {
        let slice = std::slice::from_raw_parts(data, data_len);
        let mut vec = Vec::from_raw_parts(*vec_buf_ptr, *vec_len, *vec_cap);

        match revert_mutation(slice, &mut vec) {
            Ok(()) => {
                *vec_buf_ptr = vec.as_mut_ptr();
                *vec_len = vec.len();
                *vec_cap = vec.capacity();
                mem::forget(vec);
                true
            }
            Err(e) => {
                eprintln!("decoding failed in wololooo plugin due to {:?}", e);
                false
            }
        }
    }
}

#[derive(Debug)]
pub enum MutateError {
    Drive(&'static str),
    Revert(&'static str),
}

pub fn drive_mutation(data: &[u8], vec: &mut Vec<u8>) -> Result<(), MutateError> {
    vec.clear();
    vec.reserve(data.len());
    for byte in data {
        vec.push(byte ^ 0b00000001);
    }
    Ok(())
}

pub fn revert_mutation(data: &[u8], vec: &mut Vec<u8>) -> Result<(), MutateError> {
    vec.clear();
    vec.reserve(data.len());
    for byte in data {
        vec.push(byte ^ 0b00000001);
    }
    Ok(())
}
