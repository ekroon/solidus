use solidus::prelude::*;
use std::pin::Pin;

// Debug function that prints the VALUE type
fn debug_func(s: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    Ok(RString::new("OK"))
}

fn init_debug(ruby: &Ruby) -> Result<(), Error> {
    // Try to register with explicit type checking
    ruby.define_global_function("debug_func", solidus::function!(debug_func, 1), 1)?;
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn Init_debug() {
    Ruby::mark_ruby_thread();
    let ruby = Ruby::get();
    if let Err(e) = init_debug(ruby) {
        e.raise();
    }
}
