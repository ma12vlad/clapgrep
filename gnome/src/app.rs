use crate::ui;
use adw::prelude::*;
use gtk::{
    gdk,
    gio::{self, SimpleAction},
    glib::{self, clone},
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};

pub fn start(app: &adw::Application, files: &[gio::File]) {
    let app = app.downcast_ref::<adw::Application>().unwrap();

    let style_provider = gtk::CssProvider::new();
    style_provider.load_from_string(include_str!("styles.css"));
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &style_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ui::SearchWindow::new(app);

    if let Some(dir) = files.first() {
        if let Some(path) = dir.path() {
            if path.is_dir() {
                window.set_search_path(path);
            }
        }
    }

    let about_action = SimpleAction::new("about", None);
    about_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| ui::about_dialog().present(Some(&window))
    ));
    app.add_action(&about_action);

    let shortcuts_action = SimpleAction::new("shortcuts", None);
    shortcuts_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            ui::show_shortcuts(&window);
        }
    ));
    app.add_action(&shortcuts_action);

    let quit_action = SimpleAction::new("quit", None);
    quit_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            window.close();
        }
    ));
    app.add_action(&quit_action);

    app.set_accels_for_action("app.quit", &["<ctrl>q"]);
    app.set_accels_for_action("app.shortcuts", &["<ctrl>h"]);
    app.set_accels_for_action("win.start-search", &["<ctrl>Return"]);
    app.set_accels_for_action("app.stop-search", &["<ctrl>s"]);

    window.present();
}
