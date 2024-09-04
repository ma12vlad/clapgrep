mod imp;

use clapgrep_core::fileinfo::FileInfo;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use imp::Section;

use crate::search_result::SearchResult;

glib::wrapper! {
    pub struct SearchModel(ObjectSubclass<imp::SearchModel>)
        @implements gio::ListModel, gtk::SectionModel;
}

impl SearchModel {
    pub fn new() -> SearchModel {
        glib::Object::new()
    }

    pub fn clear(&self) {
        let imp = self.imp();
        let len = imp.data.borrow().len();
        imp.data.borrow_mut().clear();
        self.items_changed(0, len as u32, 0)
    }

    fn append_file_info_impl(&self, file_info: &FileInfo) -> Section {
        let search_results = file_info
            .matches
            .iter()
            .map(|m| SearchResult::new(&file_info.path, m.line, &m.content, &m.ranges));

        let mut data = self.imp().data.borrow_mut();
        let start = data.len() as u32;
        data.extend(search_results);
        let end = data.len() as u32;
        drop(data);

        let section = Section { start, end };
        self.imp().sections.borrow_mut().push(section);

        section
    }

    pub fn append_file_info(&self, file_info: &FileInfo) {
        let Section { start, end } = self.append_file_info_impl(file_info);
        self.items_changed(start, 0, end - start);
    }

    pub fn extend_with_results(&self, results: &[FileInfo]) {
        let start = self.imp().data.borrow().len() as u32;
        for file_info in results {
            self.append_file_info_impl(file_info);
        }
        let end = self.imp().data.borrow().len() as u32;

        self.items_changed(start, 0, end - start);
    }
}

impl Default for SearchModel {
    fn default() -> Self {
        Self::new()
    }
}
