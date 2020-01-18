use gameboy_core::{Cartridge, Emulator, Rtc};
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::web::window;

pub fn load_ram_save_data(cartridge: &mut Cartridge) {
    if let Some(ram_str) = window().local_storage().get(cartridge.get_name()) {
        let chars: Vec<char> = ram_str.chars().collect();
        let bytes: Vec<u8> = chars
            .chunks(2)
            .map(|chunk| {
                let byte: String = chunk.iter().collect();
                u8::from_str_radix(&byte, 16).unwrap()
            })
            .collect();
        cartridge.set_ram(bytes);
    }
}

pub fn load_timestamp_data(cartridge: &mut Cartridge) {
    let key = format!("{}-timestamp", cartridge.get_name());
    if let Some(timestamp_str) = window().local_storage().get(&key) {
        let chars: Vec<char> = timestamp_str.chars().collect();
        let bytes: Vec<u8> = chars
            .chunks(2)
            .map(|chunk| {
                let byte: String = chunk.iter().collect();
                u8::from_str_radix(&byte, 16).unwrap()
            })
            .collect();
        let rtc = Rtc::from_bytes(&bytes[..5]);
        let mut timestamp_data = [0; 8];
        timestamp_data.copy_from_slice(&bytes[5..]);
        let timestamp = u64::from_ne_bytes(timestamp_data);
        cartridge.set_last_timestamp(rtc, timestamp);
    }
}

pub fn save_ram_data(
    emulator: &Emulator,
    ram_str: Rc<RefCell<String>>,
    should_save_to_local: Rc<RefCell<bool>>,
) {
    if *should_save_to_local.borrow() && emulator.get_cartridge().has_battery() {
        let name = emulator.get_cartridge().get_name();
        window()
            .local_storage()
            .insert(&name, &ram_str.borrow())
            .unwrap();
        *should_save_to_local.borrow_mut() = false;
    }
}

pub fn save_timestamp_data(emulator: &Emulator) {
    if emulator.get_cartridge().has_battery() {
        let name = format!("{}-timestamp", emulator.get_cartridge().get_name());
        let (rtc_data, last_timestamp) = emulator.get_cartridge().get_last_timestamp();
        let mut rtc_bytes = rtc_data.to_bytes().to_vec();
        let mut last_timestamp_bytes = u64::to_ne_bytes(last_timestamp).to_vec();
        rtc_bytes.append(&mut last_timestamp_bytes);

        let timestamp_data_str: String = rtc_bytes
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        window()
            .local_storage()
            .insert(&name, &timestamp_data_str)
            .unwrap();
    }
}
