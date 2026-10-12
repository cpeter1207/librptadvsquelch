//! Portable, sample-clocked squelch behind a versioned C descriptor.

#![deny(warnings)]
#![cfg_attr(coverage, feature(coverage_attribute))]

use std::{ffi::c_char, mem::size_of};

mod micor_squelch;
mod vox;

const ABI_VERSION: u32 = 1;
const CAPABILITY_NAME: &[u8] = b"rptadv.squelch\0";

/// Opaque state owned by one receiver.
#[derive(Default)]
pub struct Squelch {
    micor: micor_squelch::State,
    vox: vox::State,
}

/// ABI 1 squelch function table.
#[repr(C)]
pub struct SquelchDescriptor {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
    create: extern "C" fn() -> *mut Squelch,
    destroy: unsafe extern "C" fn(*mut Squelch),
    noise_update: unsafe extern "C" fn(*mut Squelch, bool, f64, u32, u32) -> bool,
    vox_advance: unsafe extern "C" fn(*mut Squelch, u32, bool, u32),
    vox_finish: unsafe extern "C" fn(*mut Squelch, u32) -> bool,
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

/// Return the immutable ABI 1 function table.
#[unsafe(no_mangle)]
pub extern "C" fn rptadv_squelch_descriptor() -> *const SquelchDescriptor {
    &DESCRIPTOR
}

extern "C" fn create() -> *mut Squelch {
    Box::into_raw(Box::new(Squelch::default()))
}

unsafe extern "C" fn destroy(handle: *mut Squelch) {
    if !handle.is_null() {
        // SAFETY: ownership returns exactly once after callback use has stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

unsafe extern "C" fn noise_update(
    handle: *mut Squelch,
    previous_closed: bool,
    sample_power: f64,
    open_level: u32,
    hysteresis: u32,
) -> bool {
    // SAFETY: the caller holds its handle exclusively during this update.
    let Some(state) = (unsafe { handle.as_mut() }) else {
        return true;
    };
    micor_squelch::update(
        &mut state.micor,
        previous_closed,
        sample_power,
        open_level,
        hysteresis,
    )
}

unsafe extern "C" fn vox_advance(
    handle: *mut Squelch,
    native_offset: u32,
    qualified: bool,
    hang_frames: u32,
) {
    // SAFETY: the caller holds its handle exclusively during this update.
    if let Some(state) = unsafe { handle.as_mut() } {
        state.vox.advance(native_offset, qualified, hang_frames);
    }
}

unsafe extern "C" fn vox_finish(handle: *mut Squelch, native_frames: u32) -> bool {
    // SAFETY: the caller holds its handle exclusively during this update.
    unsafe { handle.as_mut() }.is_some_and(|state| state.vox.finish(native_frames))
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
        assert_eq!(descriptor.abi_version, 1);
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(descriptor.capability_name) }.to_bytes(),
            b"rptadv.squelch"
        );
        unsafe { (descriptor.destroy)(std::ptr::null_mut()) };
        assert!(unsafe { (descriptor.noise_update)(std::ptr::null_mut(), false, 0.0, 0, 0) });
        unsafe { (descriptor.vox_advance)(std::ptr::null_mut(), 0, true, 1) };
        assert!(!unsafe { (descriptor.vox_finish)(std::ptr::null_mut(), 0) });
    }

    #[test]
    fn noise_gate_opens_after_settling_and_low_noise() {
        let descriptor = unsafe { &*rptadv_squelch_descriptor() };
        let handle = (descriptor.create)();
        let mut closed = true;
        for _ in 0..480 {
            closed =
                unsafe { (descriptor.noise_update)(handle, closed, 100_000_000.0, 7_000, 500) };
            assert!(closed);
        }
        for _ in 0..48_000 {
            closed =
                unsafe { (descriptor.noise_update)(handle, closed, 100_000_000.0, 7_000, 500) };
        }
        for _ in 0..24_000 {
            closed = unsafe { (descriptor.noise_update)(handle, closed, 810_000.0, 7_000, 500) };
        }
        assert!(!closed);
        unsafe { (descriptor.destroy)(handle) };
    }

    #[test]
    fn vox_hold_uses_qualifying_offset_and_expires() {
        let descriptor = unsafe { &*rptadv_squelch_descriptor() };
        let handle = (descriptor.create)();
        unsafe { (descriptor.vox_advance)(handle, 5, true, 10) };
        assert!(unsafe { (descriptor.vox_finish)(handle, 8) });
        assert!(!unsafe { (descriptor.vox_finish)(handle, 7) });
        unsafe { (descriptor.destroy)(handle) };
    }
}
