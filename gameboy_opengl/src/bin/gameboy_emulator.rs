extern crate gio;
extern crate gtk;

use std::fs::File;
use std::io::Read;

use gameboy_opengl::sdl_subsystems::SdlSubsystems;
use gio::prelude::*;
use gio::Menu;
use glib::clone;
use gtk::prelude::*;
use gtk::{FileChooserAction, FileChooserDialog, ResponseType};
use std::sync::Arc;
use std::sync::Mutex;

fn build_system_menu(application: &gtk::Application) {
    let menu_bar = Menu::new();
    let file_bar = Menu::new();
    let emulation_bar = Menu::new();
    let options_bar = Menu::new();
    let tools_bar = Menu::new();

    file_bar.append(Some("Open"), Some("app.open"));
    file_bar.append(Some("Exit"), Some("app.exit"));
    menu_bar.append_submenu(Some("_File"), &file_bar);

    emulation_bar.append(Some("Play"), Some("app.play"));
    emulation_bar.append(Some("Stop"), Some("app.stop"));
    emulation_bar.append(Some("Reset"), Some("app.reset"));
    menu_bar.append_submenu(Some("_Emulation"), &emulation_bar);

    options_bar.append(Some("Controler Settings"), Some("app.controller_settings"));
    menu_bar.append_submenu(Some("_Options"), &options_bar);

    tools_bar.append(Some("Tile Viewer"), Some("app.tile_viewer"));
    tools_bar.append(Some("Debugger"), Some("app.debugger"));
    menu_bar.append_submenu(Some("_Tools"), &tools_bar);

    application.set_menubar(Some(&menu_bar));
}

/// This function creates "actions" which connect on the declared actions from the menu items.
fn add_actions(
    application: &gtk::Application,
    window: &gtk::ApplicationWindow,
    sdl_subsystems: Arc<Mutex<SdlSubsystems>>,
) {
    let open_action = gio::SimpleAction::new("open", None);
    open_action.connect_activate(clone!(@weak window => move |_, _| {
        let dialog = FileChooserDialog::with_buttons(
            Some("open file"),
            Some(&window),
            FileChooserAction::Open,
            &[("_Cancel", ResponseType::Cancel), ("_Open", ResponseType::Accept)]
        );
        if dialog.run() == ResponseType::Accept {
            let filename_opt = dialog.get_filename();
            println!("{:?}", filename_opt);
            if let Some(filename) = filename_opt {
                let sdl_subsystems = Arc::clone(&sdl_subsystems);
                std::thread::spawn(move || {
                    let mut file = File::open(filename)
                        .map_err(|e| format!("{:?}", e))
                        .unwrap();
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .map_err(|e| format!("{:?}", e))
                        .unwrap();
                    gameboy_opengl::start(buffer, sdl_subsystems).unwrap();
                });
            }
        }
        unsafe{dialog.destroy()};
    }));

    let exit_action = gio::SimpleAction::new("exit", None);
    exit_action.connect_activate(clone!(@weak window => move |_, _| {
        window.close();
    }));

    let play_action = gio::SimpleAction::new("play", None);
    play_action.connect_activate(move |_, _| {});

    let stop_action = gio::SimpleAction::new("stop", None);
    stop_action.connect_activate(move |_, _| {});

    let reset_action = gio::SimpleAction::new("reset", None);
    reset_action.connect_activate(move |_, _| {});

    let controller_settings_action = gio::SimpleAction::new("controller_settings", None);
    controller_settings_action.connect_activate(move |_, _| {});

    let tile_viewer_action = gio::SimpleAction::new("tile_viewer", None);
    tile_viewer_action.connect_activate(move |_, _| {});

    let debugger_action = gio::SimpleAction::new("debugger", None);
    debugger_action.connect_activate(move |_, _| {});

    application.add_action(&open_action);
    application.add_action(&exit_action);
    application.add_action(&play_action);
    application.add_action(&stop_action);
    application.add_action(&reset_action);
    application.add_action(&controller_settings_action);
    application.add_action(&tile_viewer_action);
    application.add_action(&debugger_action);
}

fn main() -> Result<(), String> {
    // SDL2 subsystems need to be initialized in the main thread
    // and passed down to the GTK3 event loop via an Arc<Mutex<SdlSubsystems>>
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;
    let video_subsystem = sdl_context.video()?;
    let event_pump = sdl_context.event_pump()?;

    let sdl_subsystems = Arc::new(Mutex::new(SdlSubsystems {
        audio_subsystem,
        video_subsystem,
        event_pump,
    }));

    let application =
        gtk::Application::new(Some("com.benkonz.gameboy_emulator"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(move |app| {
        let sdl_subsystems = Arc::clone(&sdl_subsystems);
        let window = gtk::ApplicationWindow::new(app);

        window.set_title("Gameboy Emulator");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        build_system_menu(app);
        add_actions(app, &window, sdl_subsystems);
        window.show_all();
    });

    application.run(&[]);

    Ok(())
}
