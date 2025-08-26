use crate::logzet::rows::{SessionRow as InnerSessionRow, SessionRows};
use crate::sqlite::{Param, ParamType, Row, SQLize, Table};
use std::collections::HashMap;

use std::io;

use super::entity::EntityId;
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

struct SessionTable;

struct SessionRow<'a> {
    inner: &'a InnerSessionRow,
    lookup: &'a HashMap<EntityId, String>,
}
impl<SessionTable> Row<SessionTable> for SessionRow<'_> {
    fn sqlize_values(&self) -> String {
        let inner = &self.inner;
        let id: String = (&inner.entity_id).into();
        let id_lookup = format!("(SELECT rowid FROM lz_entities WHERE id IS '{}')", id);
        let day = &inner.day;
        let title = inner.title.as_deref().unwrap_or("");
        let context = inner.context.as_deref().unwrap_or("");
        let nblocks = inner.nblocks;
        let top_block = if let Some(top_block) = inner.top_block {
            dbg!(&top_block);
            if let Some(uuid) = self.lookup.get(&top_block) {
                format!("(SELECT rowid FROM lz_entities WHERE id IS '{}')", uuid)
            } else {
                "-2".to_string()
            }
        } else {
            "-1".to_string()
        };

        format!(
            "{}, '{}', '{}', '{}', {}, {}",
            id_lookup, day, title, context, nblocks, top_block
        )
    }
}

impl Default for Table<SessionTable> {
    fn default() -> Self {
        let mut con: Table<SessionTable> = Table::new("lz_sessions");
        con.add_column(&Param::new("id", ParamType::Integer));
        con.add_column(&Param::new("day", ParamType::Text));
        con.add_column(&Param::new("title", ParamType::Text));
        con.add_column(&Param::new("context", ParamType::Text));
        con.add_column(&Param::new("nblocks", ParamType::Integer));
        con.add_column(&Param::new("top_block", ParamType::Integer));
        con
    }
}

#[derive(Default)]
pub struct Schemas {
    entities: Table<EntityTable>,
    sessions: Table<SessionTable>,
}

impl Schemas {
    pub fn generate(&self, f: &mut impl io::Write) {
        let _ = f.write_all(&self.entities.sqlize().into_bytes());
        let _ = f.write_all(&self.sessions.sqlize().into_bytes());
    }
}

impl SessionRows {
    pub fn generate(&self, schemas: &Schemas, f: &mut impl io::Write) {
        for row in &self.entities {
            let s = schemas.entities.sqlize_insert(row).to_string();
            let _ = f.write_all(&s.into_bytes());
        }
        let s = schemas
            .sessions
            .sqlize_insert(&SessionRow {
                inner: &self.session,
                lookup: &self.lookup,
            })
            .to_string();
        let _ = f.write_all(&s.into_bytes());
    }
}
