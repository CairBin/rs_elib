use sea_orm::DatabaseConnection;
use crate::format::FormatParserRegister;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub upload_dir: std::path::PathBuf,
    pub parser_register: FormatParserRegister,
}

impl AppState {
    pub fn new(db: DatabaseConnection, upload_dir: std::path::PathBuf, parser_register: FormatParserRegister) -> Self {
        Self { db, upload_dir, parser_register }
    }
}
