//! Garbage collection utilities.
//!
//! This module provides functions for interacting with Ruby's garbage collector,
//! including registering and unregistering value locations for GC scanning.

use crate::value::Value;

/// Register a VALUE location with the GC.
///
/// After calling this function, the GC will consider the VALUE at the given
/// address as a root, preventing the referenced Ruby object from being collected.
///
/// # Safety
///
/// - The pointer must point to a valid `rb_sys::VALUE`.
/// - The pointer must remain valid until [`unregister_address`] is called.
/// - The caller is responsible for calling [`unregister_address`] before
///   the memory is deallocated.
#[inline]
pub unsafe fn register_address(addr: *mut rb_sys::VALUE) {
    // SAFETY: Caller guarantees the pointer is valid
    unsafe {
        rb_sys::rb_gc_register_address(addr);
    }
}

/// Unregister a VALUE location from the GC.
///
/// After calling this function, the GC will no longer consider the VALUE at
/// the given address as a root.
///
/// # Safety
///
/// - The address must have been previously registered with [`register_address`].
/// - The pointer must still be valid.
#[inline]
pub unsafe fn unregister_address(addr: *mut rb_sys::VALUE) {
    // SAFETY: Caller guarantees the address was previously registered
    unsafe {
        rb_sys::rb_gc_unregister_address(addr);
    }
}

/// Mark a value during GC marking phase.
///
/// This should be called from TypedData mark functions to tell the GC
/// that a value is reachable.
#[inline]
pub fn mark(value: Value) {
    // SAFETY: rb_gc_mark is safe to call with any VALUE
    unsafe {
        rb_sys::rb_gc_mark(value.as_raw());
    }
}

/// Permanently prevent a value from being garbage collected.
///
/// Use this sparingly - values registered this way will never be freed.
/// This is useful for values that need to live for the entire lifetime
/// of the Ruby process (e.g., class definitions, constant symbols).
///
/// # Warning
///
/// This creates a permanent GC root. Only use for truly permanent values.
#[inline]
pub fn register_mark_object(value: Value) {
    // SAFETY: rb_gc_register_mark_object is safe to call with any VALUE
    unsafe {
        rb_sys::rb_gc_register_mark_object(value.as_raw());
    }
}

/// Request a garbage collection run.
///
/// This is primarily useful for testing to ensure GC-related bugs manifest.
#[inline]
pub fn start() {
    // SAFETY: rb_gc_start is always safe to call
    unsafe {
        rb_sys::rb_gc_start();
    }
}

/// Disable garbage collection.
///
/// Returns the previous state (true if GC was already disabled).
///
/// # Warning
///
/// Disabling GC can lead to memory exhaustion. Always re-enable GC
/// as soon as possible.
#[inline]
pub fn disable() -> bool {
    // SAFETY: rb_gc_disable is always safe to call
    unsafe { rb_sys::rb_gc_disable() != 0 }
}

/// Enable garbage collection.
///
/// Returns the previous state (true if GC was disabled).
#[inline]
pub fn enable() -> bool {
    // SAFETY: rb_gc_enable is always safe to call
    unsafe { rb_sys::rb_gc_enable() != 0 }
}
