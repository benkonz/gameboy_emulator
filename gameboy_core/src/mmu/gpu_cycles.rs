pub struct GpuCycles {
    pub cycles_counter: i32,
    pub aux_cycles_counter: i32,
    pub pixel_counter: i32,
    pub screen_enable_delay_cycles: i32,
    pub window_line: i32
}

impl GpuCycles {
    pub fn new() -> GpuCycles {
        GpuCycles {
            cycles_counter: 0,
            aux_cycles_counter: 0,
            pixel_counter: 0,
            screen_enable_delay_cycles: 0,
            window_line: 0
        }
    }
}
