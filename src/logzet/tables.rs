use crate::logzet::rows::SessionRows;
use std::io;

impl SessionRows {
    pub fn generate(&self, f: &mut impl io::Write) {
        let s = "sqlize!\n".to_string();
        let _ = f.write_all(&s.into_bytes());
    }
}
