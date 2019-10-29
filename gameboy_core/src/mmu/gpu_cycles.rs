#[derive(Default)]
pub struct GpuCycles {
    pub cycles_counter: i32,
    pub aux_cycles_counter: i32,
    pub pixel_counter: i32,
    pub screen_enable_delay_cycles: i32,
    pub window_line: i32,
}
