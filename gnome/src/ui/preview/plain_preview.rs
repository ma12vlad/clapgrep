use gtk::glib::{self, Object};

use crate::search::SearchResult;

glib::wrapper! {
    pub struct PlainPreview(ObjectSubclass<imp::PlainPreview>)
        @extends gtk::Widget;
}

impl PlainPreview {
    pub fn new(result: &SearchResult) -> Self {
        Object::builder().property("result", result).build()
    }
}

mod imp {
    use crate::search::SearchResult;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use gtk::{glib, prelude::*, CompositeTemplate};
    use std::{cell::RefCell, fs, time::Duration};

    #[derive(CompositeTemplate, glib::Properties, Default)]
    #[template(file = "src/ui/preview/plain_preview.blp")]
    #[properties(wrapper_type = super::PlainPreview)]
    pub struct PlainPreview {
        #[property(get, set)]
        pub result: RefCell<SearchResult>,

        #[template_child]
        pub title: TemplateChild<adw::WindowTitle>,

        #[template_child]
        pub text_view: TemplateChild<sourceview5::View>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlainPreview {
        const NAME: &'static str = "ClapgrepPlainPreview";
        type Type = super::PlainPreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl PlainPreview {
        fn update_preview(&self) {
            let result = self.result.borrow();

            if !result.file().exists() {
                return;
            }

            let file = result.file();
            let file_name = file.file_name().unwrap().to_string_lossy();
            self.title.set_title(file_name.as_ref());

            let full_text =
                fs::read_to_string(&file).expect("This can fail but I don't care right now");

            let buffer = self.text_view.buffer();
            buffer.set_text(&full_text);
            let mut cursor_position = buffer.start_iter();
            cursor_position.forward_lines((result.line() - 1) as i32);
            buffer.place_cursor(&cursor_position);

            let text_view = self.text_view.clone();
            glib::timeout_add_local_once(Duration::from_millis(100), move || {
                text_view.scroll_to_iter(&mut cursor_position, 0.0, true, 0.0, 0.3);
            });
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PlainPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.connect_result_notify(|obj| {
                obj.imp().update_preview();
            });
        }
    }

    impl WidgetImpl for PlainPreview {}
}
