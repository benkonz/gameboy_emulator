mod flag;

pub struct Registers {
    pub control: flag::ControlFlag,
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub scan_line: u8,
    //TODO: fix this
    pub background_palette: u8
}