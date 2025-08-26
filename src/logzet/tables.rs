use crate::logzet::rows::SessionRows;
use crate::sqlite::{Param, ParamType, Row, SQLize, Table};

use std::io;

use super::rows::EntityRow;
struct EntityTable;

impl<EntityTable> Row<EntityTable> for EntityRow {
    fn sqlize_values(&self) -> String {
        let uuid: String = self.into();
        format!("'{}'", uuid)
    }
}

impl Default for Table<EntityTable> {
    fn default() -> Self {
        let mut con: Table<EntityTable> = Table::new("lz_entities");
        con.add_column(&Param::new("id", ParamType::Text));
        con
    }
}

#[derive(Default)]
pub struct Schemas {
    entities: Table<EntityTable>,
}

impl Schemas {
    pub fn generate(&self, f: &mut impl io::Write) {
        let _ = f.write_all(&self.entities.sqlize().into_bytes());
    }
}

pub fn generate_schemas(f: &mut impl io::Write) {
    let entities: Table<EntityTable> = Table::default();
    let _ = f.write_all(&entities.sqlize().into_bytes());
}

impl SessionRows {
    pub fn generate(&self, schemas: &Schemas, f: &mut impl io::Write) {
        for row in &self.entities {
            let s = schemas.entities.sqlize_insert(row).to_string();
            let _ = f.write_all(&s.into_bytes());
        }
    }
}
