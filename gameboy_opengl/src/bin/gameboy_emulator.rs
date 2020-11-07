#[macro_use]
extern crate clap;

extern crate gio;
extern crate gtk;

use clap::{App, Arg};
use std::fs::File;
use std::io::Read;

use gameboy_opengl::sdl_subsystems::SdlSubsystems;
use gio::prelude::*;
use gio::Menu;
use glib::clone;
use gtk::prelude::*;
use gtk::AboutDialog;
use std::sync::Arc;
use std::sync::Mutex;

fn build_system_menu(application: &gtk::Application) {
    let menu = Menu::new();
    let menu_bar = Menu::new();
    let more_menu = Menu::new();
    let switch_menu = Menu::new();
    let settings_menu = Menu::new();
    let submenu = Menu::new();

    // The first argument is the label of the menu item whereas the second is the action name. It'll
    // makes more sense when you'll be reading the "add_actions" function.
    menu.append(Some("Quit"), Some("app.quit"));

    switch_menu.append(Some("Switch"), Some("app.switch"));
    menu_bar.append_submenu(Some("_Switch"), &switch_menu);

    settings_menu.append(Some("Sub another"), Some("app.sub_another"));
    submenu.append(Some("Sub sub another"), Some("app.sub_sub_another"));
    submenu.append(Some("Sub sub another2"), Some("app.sub_sub_another2"));
    settings_menu.append_submenu(Some("Sub menu"), &submenu);
    menu_bar.append_submenu(Some("_Another"), &settings_menu);

    more_menu.append(Some("About"), Some("app.about"));
    menu_bar.append_submenu(Some("?"), &more_menu);

    application.set_app_menu(Some(&menu));
    application.set_menubar(Some(&menu_bar));
}

/// This function creates "actions" which connect on the declared actions from the menu items.
fn add_actions(application: &gtk::Application, window: &gtk::ApplicationWindow) {
    let sub_another = gio::SimpleAction::new("sub_another", None);
    sub_another.connect_activate(move |_, _| {
        println!("sub another menu item clicked");
    });
    let sub_sub_another = gio::SimpleAction::new("sub_sub_another", None);
    sub_sub_another.connect_activate(move |_, _| {
        println!("sub sub another menu item clicked");
    });
    let sub_sub_another2 = gio::SimpleAction::new("sub_sub_another2", None);
    sub_sub_another2.connect_activate(move |_, _| {
        println!("sub sub another 2 menu item clicked");
    });

    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak window => move |_, _| {
        window.close();
    }));

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(clone!(@weak window => move |_, _| {
        let p = AboutDialog::new();
        p.set_website_label(Some("gtk-rs"));
        p.set_website(Some("http://gtk-rs.org"));
        p.set_authors(&["Gtk-rs developers"]);
        p.set_title("About!");
        p.set_transient_for(Some(&window));
        p.show_all();
    }));

    // We need to add all the actions to the application so they can be taken into account.
    application.add_action(&about);
    application.add_action(&quit);
    application.add_action(&sub_another);
    application.add_action(&sub_sub_another);
    application.add_action(&sub_sub_another2);
}

fn main() -> Result<(), String> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("rom filename")
                .help("rom file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

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
        gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(move |app| {
        let sdl_subsystems = Arc::clone(&sdl_subsystems);
        let window = gtk::ApplicationWindow::new(app);

        window.set_title("System menu bar");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        build_system_menu(app);
        add_actions(app, &window);
        window.show_all();

        let rom_filename = matches.value_of("rom filename").unwrap().to_string();
        std::thread::spawn(move || {
            let mut file = File::open(rom_filename)
                .map_err(|e| format!("{:?}", e))
                .unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("{:?}", e))
                .unwrap();

            gameboy_opengl::start(buffer, sdl_subsystems).unwrap();
        });
    });

    application.run(&[]);

    Ok(())
}
