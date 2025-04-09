#[allow(nonstandard_style)]
pub enum DebuggerCommand {
    ENABLED(bool),
    CPU_CLOCK(u32),
    // STEP,
}
