//! Системные окна сообщений и выбора файлов.

use std::path::PathBuf;

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

use crate::Ui;

/// Набор расширений в окне выбора файла: `("Текст", &["txt", "md"])`.
pub type FileFilter<'a> = (&'a str, &'a [&'a str]);

impl Ui {
    fn message(&self, title: &str, text: &str, buttons: MessageButtons) -> MessageDialog {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title(title)
            .set_description(text)
            .set_buttons(buttons);
        match &self.owner {
            Some(owner) => dialog.set_parent(owner),
            None => dialog,
        }
    }

    fn files(&self, filters: &[FileFilter]) -> FileDialog {
        let mut dialog = FileDialog::new();
        for (name, extensions) in filters {
            dialog = dialog.add_filter(*name, extensions);
        }
        match &self.owner {
            Some(owner) => dialog.set_parent(owner),
            None => dialog,
        }
    }

    // Без окна (тесты) системные диалоги не показываются.

    /// Сообщение с кнопкой «ОК»; возвращается после закрытия.
    pub fn message_box(&mut self, title: &str, text: &str) {
        if self.owner.is_some() {
            self.message(title, text, MessageButtons::Ok).show();
        }
    }

    /// Вопрос «Да / Нет».
    pub fn confirm(&mut self, title: &str, text: &str) -> bool {
        self.owner.is_some()
            && self.message(title, text, MessageButtons::YesNo).show() == MessageDialogResult::Yes
    }

    pub fn open_file(&mut self, filters: &[FileFilter]) -> Option<PathBuf> {
        self.owner.as_ref()?;
        self.files(filters).pick_file()
    }

    pub fn save_file(&mut self, name: &str, filters: &[FileFilter]) -> Option<PathBuf> {
        self.owner.as_ref()?;
        self.files(filters).set_file_name(name).save_file()
    }

    pub fn pick_folder(&mut self) -> Option<PathBuf> {
        self.owner.as_ref()?;
        self.files(&[]).pick_folder()
    }
}
