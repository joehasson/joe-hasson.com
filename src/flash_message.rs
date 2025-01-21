use actix_session::Session;

pub trait Flash {
    fn set_flash(&self, flash: &str) -> Result<(), actix_session::SessionInsertError>;
    fn get_flash(&self) -> Option<String>;
    fn clear_flash(&self);
}

impl Flash for Session {
    fn set_flash(&self, flash: &str) -> Result<(), actix_session::SessionInsertError> {
        self.insert("_flash", flash)
    }

    fn get_flash(&self) -> Option<String> {
        self.get("_flash").ok().flatten() }

    fn clear_flash(&self) {
        self.remove("_flash");
    }
}

