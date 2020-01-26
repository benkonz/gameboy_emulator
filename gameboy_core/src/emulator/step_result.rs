#[derive(Eq, PartialEq)]
pub enum StepResult {
    VBlank,
    AudioBufferFull,
    Nothing,
}
