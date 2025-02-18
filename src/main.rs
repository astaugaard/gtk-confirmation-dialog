use std::{os::unix::process::CommandExt, process::Command};

use clap::Parser;
use gio::{
    glib::{Char, OptionArg, OptionFlags, Propagation},
    prelude::*,
    ApplicationFlags,
};
use gtk4::{
    gdk::{Display, Key},
    prelude::*,
    Align, CssProvider, EventControllerKey, GestureClick,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short = 's', long)]
    css: Option<String>,

    #[arg(short, long)]
    message: Option<String>,

    #[arg(short, long)]
    command: String,
}

fn activate(application: &gtk4::Application, message: &Option<String>, command: &str) {
    let window = gtk4::ApplicationWindow::new(application);

    window.init_layer_shell();

    window.set_layer(Layer::Top);

    window.fullscreen();

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

    let anchors = [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, true),
        (Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }

    let click_gesture = GestureClick::builder().button(0).build();
    let key = EventControllerKey::new();

    let win2 = window.clone();
    let command = command.to_owned();

    key.connect_key_pressed(move |_, key, _, _| {
        win2.close();
        match key {
            Key::KP_Enter | Key::Return | Key::ISO_Enter => {
                Command::new("sh").arg("-c").arg(&command).exec();
            }
            _ => {}
        }

        Propagation::Stop
    });

    window.add_controller(key);

    let win2 = window.clone();

    click_gesture.connect_pressed(move |_, _, _, _| {
        win2.close();
    });

    window.add_controller(click_gesture);

    let label = gtk4::Label::builder()
        .label(format!(
            "{}\npress enter to confirm",
            message.as_ref().map(|s| s.as_str()).unwrap_or("confirm?")
        ))
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    window.set_child(Some(&label));

    window.set_visible(true)
}

fn main() {
    let Args {
        css,
        message,
        command,
    } = dbg!(Args::parse());

    let mut flags = ApplicationFlags::empty();

    flags.set(ApplicationFlags::HANDLES_COMMAND_LINE, true);

    let application = gtk4::Application::builder().build();

    application.add_main_option(
        "command",
        Char::from(b'c'),
        OptionFlags::NONE,
        OptionArg::String,
        "command to execute if confirmed",
        Some("COMMAND"),
    );

    application.add_main_option(
        "css",
        Char::from(b's'),
        OptionFlags::NONE,
        OptionArg::String,
        "style sheet to use",
        Some("STYLE"),
    );

    application.add_main_option(
        "message",
        Char::from(b'm'),
        OptionFlags::NONE,
        OptionArg::String,
        "message to display for confirmation",
        Some("MESSAGE"),
    );

    application.connect_command_line(|_app, _cli| 0); // idk why I have to do this tbh

    application.connect_startup(move |_| {
        let provider = CssProvider::new();

        match &css {
            Some(file) => {
                provider.load_from_path(file);
            }
            None => {
                provider.load_from_string(include_str!("style.css"));
            }
        };

        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        )
    });

    application.connect_activate(move |app| {
        activate(app, &message, &command);
    });

    application.run();
}
