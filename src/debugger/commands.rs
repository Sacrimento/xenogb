#[allow(nonstandard_style)]
#[derive(Debug)]
pub enum DebuggerCommand {
    ENABLED(bool),
    CPU_CLOCK(u32),
    // STEP,
}
