//! Build script for solidus.
//!
//! This uses rb-sys-env to configure Ruby-related build settings.

fn main() {
    // Re-run if Ruby configuration changes
    println!("cargo::rerun-if-env-changed=RUBY_ROOT");
    println!("cargo::rerun-if-env-changed=RUBY_VERSION");

    // rb-sys-env provides Ruby configuration at build time
    // The actual linking is handled by rb-sys
}
