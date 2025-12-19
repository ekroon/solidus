//! Pinned Values Example
//!
//! This example demonstrates the core pinning concepts in Solidus:
//! - Why pinning matters for Ruby GC safety
//! - Stack pinning with Context for method returns
//! - Heap boxing with `BoxValue<T>` for collections
//! - Methods with pinned arguments `Pin<&StackPinned<T>>`
//!
//! # The Problem
//!
//! Ruby's GC uses conservative stack scanning - it only looks at the C stack
//! to find live VALUE references. If a VALUE is moved to the heap (Vec, Box,
//! HashMap), the GC cannot see it and may collect the underlying Ruby object.
//!
//! # The Solution
//!
//! Solidus enforces at compile time that all Ruby values are either:
//! 1. Pinned on the stack (visible to GC)
//! 2. Explicitly boxed with `BoxValue<T>` (registered with GC)

use solidus::prelude::*;
use std::pin::Pin;

// ============================================================================
// Stack Pinning - The Common Case
// ============================================================================

/// Demonstrates a function that takes a pinned RString argument.
///
/// The `Pin<&StackPinned<RString>>` type guarantees at compile time that:
/// - The value lives on the stack (GC can see it)
/// - It cannot be moved to the heap accidentally
///
/// Use `.get()` to access the inner `&RString`.
fn process_pinned_string<'ctx>(
    ctx: &'ctx Context,
    input: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let content = input.get().to_string()?;
    let processed = content.to_uppercase();
    ctx.new_string(&format!("Processed: {}", processed))
        .map_err(Into::into)
}

/// Function with multiple pinned arguments.
///
/// Each pinned argument is independently protected on the stack.
fn concatenate_pinned<'ctx>(
    ctx: &'ctx Context,
    first: Pin<&StackPinned<RString>>,
    second: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let s1 = first.get().to_string()?;
    let s2 = second.get().to_string()?;
    ctx.new_string(&format!("{}{}", s1, s2))
        .map_err(Into::into)
}

/// Instance method example - uses `self` as first argument.
///
/// When registered with `method!`, the first argument receives `self`.
fn append_to_self<'ctx>(
    ctx: &'ctx Context,
    rb_self: RString,
    suffix: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let self_str = rb_self.to_string()?;
    let suffix_str = suffix.get().to_string()?;
    ctx.new_string(&format!("{}{}", self_str, suffix_str))
        .map_err(Into::into)
}

// ============================================================================
// Heap Boxing - For Collections and Long-Lived Values
// ============================================================================

/// Demonstrates creating and returning a BoxValue for heap storage.
///
/// `BoxValue<T>` is heap-allocated and registered with Ruby's GC via
/// `rb_gc_register_address`. This makes it safe to store in collections.
///
/// When to use BoxValue:
/// - Storing values in Vec, HashMap, etc.
/// - Caching Ruby values in Rust structs
/// - Keeping values alive across async boundaries
fn create_boxed_string<'ctx>(
    ctx: &'ctx Context,
    content: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let text = content.get().to_string()?;
    ctx.new_string(&format!("Boxed: {}", text))
        .map_err(Into::into)
}

// ============================================================================
// Collections with BoxValue - The Safe Pattern
// ============================================================================

/// StringCollector stores multiple Ruby strings safely on the heap.
///
/// This is the ONLY safe way to store Ruby values in Rust collections.
/// Each `BoxValue<RString>` is registered with Ruby's GC.
struct StringCollector {
    strings: Vec<BoxValue<RString>>,
}

impl StringCollector {
    /// Add a string to the collection.
    ///
    /// We clone the value from the pinned reference and box it.
    /// The BoxValue registration keeps it alive.
    fn add(&mut self, s: Pin<&StackPinned<RString>>) {
        let boxed = BoxValue::new(s.get().clone());
        self.strings.push(boxed);
    }

    /// Get the number of stored strings.
    fn len(&self) -> usize {
        self.strings.len()
    }

    /// Concatenate all strings with a separator.
    fn join(&self, sep: &str) -> Result<String, Error> {
        let parts: Result<Vec<String>, Error> =
            self.strings.iter().map(|s| s.to_string()).collect();
        Ok(parts?.join(sep))
    }

    /// Convert to a Ruby array.
    fn to_ruby_array<'ctx>(
        &self,
        ctx: &'ctx Context,
    ) -> Result<Pin<&'ctx StackPinned<RArray>>, Error> {
        let array = ctx.new_array()?;
        for s in &self.strings {
            // Get the RString from BoxValue (via Deref) and push a clone to the array
            // RString implements IntoValue, so we can push it directly
            array.get().push((**s).clone());
        }
        Ok(array)
    }
}

// Global collector for demonstration (in real code, use TypedData wrapper)
// Using std::cell::UnsafeCell for Rust 2024 compatibility (avoids static mut references)
use std::cell::UnsafeCell;

struct CollectorHolder {
    collector: UnsafeCell<StringCollector>,
}

// SAFETY: Ruby is single-threaded, so this is safe for Ruby extension use.
// In production, prefer TypedData to wrap stateful objects.
unsafe impl Sync for CollectorHolder {}

static COLLECTOR_HOLDER: CollectorHolder = CollectorHolder {
    collector: UnsafeCell::new(StringCollector {
        strings: Vec::new(),
    }),
};

fn get_collector() -> &'static mut StringCollector {
    // SAFETY: Ruby is single-threaded, so only one thread accesses this.
    // This pattern avoids mutable static references which are UB in Rust 2024.
    unsafe { &mut *COLLECTOR_HOLDER.collector.get() }
}

// ============================================================================
// Ruby-Exposed Functions
// ============================================================================

/// Global function: process a string with pinned argument
fn ruby_process_string<'ctx>(
    ctx: &'ctx Context,
    s: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    process_pinned_string(ctx, s)
}

/// Global function: concatenate two strings
fn ruby_concat_strings<'ctx>(
    ctx: &'ctx Context,
    first: Pin<&StackPinned<RString>>,
    second: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    concatenate_pinned(ctx, first, second)
}

/// Global function: demonstrate boxing a value
fn ruby_box_string<'ctx>(
    ctx: &'ctx Context,
    s: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    create_boxed_string(ctx, s)
}

/// Global function: add a string to the collector
fn ruby_collect_string(
    _ctx: &Context,
    s: Pin<&StackPinned<RString>>,
) -> Result<i64, Error> {
    let collector = get_collector();
    collector.add(s);
    Ok(collector.len() as i64)
}

/// Global function: get the count of collected strings
fn ruby_collector_count(_ctx: &Context) -> Result<i64, Error> {
    Ok(get_collector().len() as i64)
}

/// Global function: join all collected strings
fn ruby_collector_join<'ctx>(
    ctx: &'ctx Context,
    sep: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let sep_str = sep.get().to_string()?;
    let joined = get_collector().join(&sep_str)?;
    ctx.new_string(&joined).map_err(Into::into)
}

/// Global function: get collected strings as Ruby array
fn ruby_collector_to_array<'ctx>(
    ctx: &'ctx Context,
) -> Result<Pin<&'ctx StackPinned<RArray>>, Error> {
    get_collector().to_ruby_array(ctx)
}

/// Global function: clear the collector
fn ruby_collector_clear(_ctx: &Context) -> Result<i64, Error> {
    let collector = get_collector();
    let count = collector.len();
    collector.strings.clear();
    Ok(count as i64)
}

/// Global function: demonstrate why pinning matters
///
/// This function creates multiple values and shows that they all
/// remain valid because they're properly pinned on the stack.
fn ruby_demo_stack_pinning<'ctx>(
    ctx: &'ctx Context,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    // Each value is pinned in the context - GC can see all of them
    let s1 = ctx.new_string("Stack")?;
    let s2 = ctx.new_string("pinning")?;
    let s3 = ctx.new_string("keeps")?;
    let s4 = ctx.new_string("values")?;
    let s5 = ctx.new_string("safe!")?;

    // All five values are visible to the GC during this call
    // Even if GC runs, none of these will be collected
    let result = format!(
        "{} {} {} {} {}",
        s1.get().to_string()?,
        s2.get().to_string()?,
        s3.get().to_string()?,
        s4.get().to_string()?,
        s5.get().to_string()?,
    );

    ctx.new_string(&result).map_err(Into::into)
}

/// Global function: demonstrate heap boxing for collections
fn ruby_demo_heap_boxing<'ctx>(
    ctx: &'ctx Context,
) -> Result<Pin<&'ctx StackPinned<RArray>>, Error> {
    // Create several BoxValue instances - safe for heap storage
    let mut boxed_values: Vec<BoxValue<RString>> = Vec::new();

    // Each value is boxed with GC registration
    boxed_values.push(RString::new_boxed("These"));
    boxed_values.push(RString::new_boxed("are"));
    boxed_values.push(RString::new_boxed("heap"));
    boxed_values.push(RString::new_boxed("stored"));
    boxed_values.push(RString::new_boxed("safely!"));

    // All values remain valid because BoxValue registers with GC
    let array = ctx.new_array()?;
    for boxed in &boxed_values {
        // Get the RString from BoxValue (via Deref) and push a clone to the array
        // RString implements IntoValue, so we can push it directly
        array.get().push((**boxed).clone());
    }

    // BoxValues are automatically unregistered when dropped
    Ok(array)
}

// ============================================================================
// Initialization
// ============================================================================

fn init_solidus(ruby: &Ruby) -> Result<(), Error> {
    // ========================================================================
    // Global functions for pinned value operations
    // ========================================================================

    // Stack pinning demos
    ruby.define_global_function(
        "process_string",
        solidus::function!(ruby_process_string, 1),
        1,
    )?;
    ruby.define_global_function(
        "concat_strings",
        solidus::function!(ruby_concat_strings, 2),
        2,
    )?;

    // Boxing demo
    ruby.define_global_function("box_string", solidus::function!(ruby_box_string, 1), 1)?;

    // String collector (demonstrates Vec<BoxValue<T>>)
    ruby.define_global_function(
        "collect_string",
        solidus::function!(ruby_collect_string, 1),
        1,
    )?;
    ruby.define_global_function(
        "collector_count",
        solidus::function!(ruby_collector_count, 0),
        0,
    )?;
    ruby.define_global_function(
        "collector_join",
        solidus::function!(ruby_collector_join, 1),
        1,
    )?;
    ruby.define_global_function(
        "collector_to_array",
        solidus::function!(ruby_collector_to_array, 0),
        0,
    )?;
    ruby.define_global_function(
        "collector_clear",
        solidus::function!(ruby_collector_clear, 0),
        0,
    )?;

    // Demonstration functions
    ruby.define_global_function(
        "demo_stack_pinning",
        solidus::function!(ruby_demo_stack_pinning, 0),
        0,
    )?;
    ruby.define_global_function(
        "demo_heap_boxing",
        solidus::function!(ruby_demo_heap_boxing, 0),
        0,
    )?;

    // ========================================================================
    // String class extension with instance method
    // ========================================================================

    let string_class = RClass::try_convert(ruby.class_string())?;
    string_class.define_method("append_solidus", solidus::method!(append_to_self, 1), 1)?;

    Ok(())
}

// Ruby extension entry point
#[no_mangle]
pub unsafe extern "C" fn Init_pinned_values() {
    Ruby::mark_ruby_thread();

    let ruby = Ruby::get();

    if let Err(e) = init_solidus(ruby) {
        e.raise();
    }
}
