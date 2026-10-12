//! Portable, sample-clocked squelch behind a versioned C descriptor.

#![deny(warnings)]
#![cfg_attr(coverage, feature(coverage_attribute))]

use std::{ffi::c_char, mem::size_of, ptr};

mod micor_squelch;
mod vox;

const ABI_VERSION: u32 = 2;
const CAPABILITY_NAME: &[u8] = b"rptadv.squelch\0";
const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = -1;

/// Opaque state owned by one receiver.
#[derive(Default)]
pub struct Squelch {
    micor: micor_squelch::State,
    vox: vox::State,
}

/// ABI 2 squelch function table.
#[repr(C)]
pub struct SquelchDescriptor {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
    create: unsafe extern "C" fn(*mut *mut Squelch) -> i32,
    destroy: unsafe extern "C" fn(*mut Squelch),
    noise_update: unsafe extern "C" fn(*mut Squelch, u32, f64, u32, u32) -> u32,
    vox_advance: unsafe extern "C" fn(*mut Squelch, u32, u32, u32),
    vox_finish: unsafe extern "C" fn(*mut Squelch, u32) -> u32,
}

// SAFETY: the descriptor and every address referenced by it are immutable.
unsafe impl Sync for SquelchDescriptor {}

static DESCRIPTOR: SquelchDescriptor = SquelchDescriptor {
    struct_size: size_of::<SquelchDescriptor>() as u32,
    abi_version: ABI_VERSION,
    capability_name: CAPABILITY_NAME.as_ptr().cast::<c_char>(),
    create,
    destroy,
    noise_update,
    vox_advance,
    vox_finish,
};

/// Return the immutable ABI 2 function table.
#[unsafe(no_mangle)]
pub extern "C" fn rptadv_squelch_descriptor() -> *const SquelchDescriptor {
    &DESCRIPTOR
}

unsafe extern "C" fn create(out: *mut *mut Squelch) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    *out = ptr::null_mut();
    *out = Box::into_raw(Box::new(Squelch::default()));
    OK
}

unsafe extern "C" fn destroy(handle: *mut Squelch) {
    if !handle.is_null() {
        // SAFETY: ownership returns exactly once after callback use has stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

unsafe extern "C" fn noise_update(
    handle: *mut Squelch,
    previous_closed: u32,
    sample_power: f64,
    open_level: u32,
    hysteresis: u32,
) -> u32 {
    // SAFETY: the caller holds its handle exclusively during this update.
    let Some(state) = (unsafe { handle.as_mut() }) else {
        return 1;
    };
    u32::from(micor_squelch::update(
        &mut state.micor,
        previous_closed != 0,
        sample_power,
        open_level,
        hysteresis,
    ))
}

unsafe extern "C" fn vox_advance(
    handle: *mut Squelch,
    native_offset: u32,
    qualified: u32,
    hang_frames: u32,
) {
    // SAFETY: the caller holds its handle exclusively during this update.
    if let Some(state) = unsafe { handle.as_mut() } {
        state
            .vox
            .advance(native_offset, qualified != 0, hang_frames);
    }
}

unsafe extern "C" fn vox_finish(handle: *mut Squelch, native_frames: u32) -> u32 {
    // SAFETY: the caller holds its handle exclusively during this update.
    u32::from(unsafe { handle.as_mut() }.is_some_and(|state| state.vox.finish(native_frames)))
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn descriptor_identifies_abi_and_accepts_null_lifecycle() {
        let pointer = rptadv_squelch_descriptor();
        assert_eq!(pointer, rptadv_squelch_descriptor());
        let descriptor = unsafe { &*pointer };
        assert_eq!(
            descriptor.struct_size as usize,
            size_of::<SquelchDescriptor>()
        );
        assert_eq!(descriptor.abi_version, 2);
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(descriptor.capability_name) }.to_bytes(),
            b"rptadv.squelch"
        );
        assert_eq!(
            unsafe { (descriptor.create)(ptr::null_mut()) },
            INVALID_ARGUMENT
        );
        let mut probe = ptr::null_mut();
        assert_eq!(unsafe { (descriptor.create)(&mut probe) }, OK);
        assert_eq!(
            unsafe { (descriptor.noise_update)(probe, 7, 100_000_000.0, 7_000, 500) },
            1
        );
        unsafe { (descriptor.destroy)(probe) };
        unsafe { (descriptor.destroy)(std::ptr::null_mut()) };
        assert_eq!(
            unsafe { (descriptor.noise_update)(ptr::null_mut(), 0, 0.0, 0, 0) },
            1
        );
        unsafe { (descriptor.vox_advance)(ptr::null_mut(), 0, 7, 1) };
        assert_eq!(unsafe { (descriptor.vox_finish)(ptr::null_mut(), 0) }, 0);
    }

    #[test]
    fn noise_gate_opens_after_settling_and_low_noise() {
        let descriptor = unsafe { &*rptadv_squelch_descriptor() };
        let mut handle = ptr::null_mut();
        assert_eq!(unsafe { (descriptor.create)(&mut handle) }, OK);
        assert!(!handle.is_null());
        let mut closed = 1;
        for _ in 0..480 {
            closed =
                unsafe { (descriptor.noise_update)(handle, closed, 100_000_000.0, 7_000, 500) };
            assert_eq!(closed, 1);
        }
        for _ in 0..48_000 {
            closed =
                unsafe { (descriptor.noise_update)(handle, closed, 100_000_000.0, 7_000, 500) };
        }
        for _ in 0..24_000 {
            closed = unsafe { (descriptor.noise_update)(handle, closed, 810_000.0, 7_000, 500) };
        }
        assert_eq!(closed, 0);
        unsafe { (descriptor.destroy)(handle) };
    }

    #[test]
    fn vox_hold_uses_qualifying_offset_and_expires() {
        let descriptor = unsafe { &*rptadv_squelch_descriptor() };
        let mut handle = ptr::null_mut();
        assert_eq!(unsafe { (descriptor.create)(&mut handle) }, OK);
        unsafe { (descriptor.vox_advance)(handle, 5, 7, 10) };
        assert_eq!(unsafe { (descriptor.vox_finish)(handle, 8) }, 1);
        assert_eq!(unsafe { (descriptor.vox_finish)(handle, 7) }, 0);
        unsafe { (descriptor.destroy)(handle) };
    }
}
