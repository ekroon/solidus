fn main() {
    // Get Ruby configuration and set up linking
    let rb_config = rb_sys_build::RbConfig::current();
    rb_config.print_cargo_args();
}
