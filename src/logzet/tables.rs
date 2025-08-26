use crate::logzet::rows::{
    BlockRow as InnerBlockRow, EntityConnectionsRow as InnerEntityConnectionRow,
    EntryRow as InnerEntryRow, SessionRow as InnerSessionRow, SessionRows,
};
use crate::sqlite::{escape_quotes, Param, ParamType, Row, SQLize, Table};
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

fn uuid_lookup(lookup: &HashMap<EntityId, String>, entity_id: Option<EntityId>) -> String {
    if let Some(entity_id) = entity_id {
        if let Some(uuid) = lookup.get(&entity_id) {
            format!("(SELECT rowid FROM lz_entities WHERE id IS '{}')", uuid)
        } else {
            "-2".to_string()
        }
    } else {
        "-1".to_string()
    }
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
        let top_block = uuid_lookup(self.lookup, inner.top_block);

        format!(
            "{}, '{}', '{}', '{}', {}, {}",
            id_lookup,
            day,
            escape_quotes(title),
            escape_quotes(context),
            nblocks,
            top_block
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

struct EntryTable;

struct EntryRow<'a> {
    inner: &'a InnerEntryRow,
    lookup: &'a HashMap<EntityId, String>,
}

impl<EntryTable> Row<EntryTable> for EntryRow<'_> {
    fn sqlize_values(&self) -> String {
        let inner = &self.inner;
        let id = uuid_lookup(self.lookup, Some(self.inner.entity_id));
        let day = &inner.day;
        // TODO: make title optional
        //let title = inner.title.as_deref().unwrap_or("");
        let title = &inner.title;
        let context = inner.context.as_deref().unwrap_or("");
        let nblocks = inner.nblocks;
        let top_block = uuid_lookup(self.lookup, inner.top_block);
        let position = inner.position;

        format!(
            "{}, '{}', '{}', '{}', {}, {}, {}",
            id,
            escape_quotes(day),
            escape_quotes(title),
            escape_quotes(context),
            nblocks,
            top_block,
            position
        )
    }
}

impl Default for Table<EntryTable> {
    fn default() -> Self {
        let mut con: Table<EntryTable> = Table::new("lz_entries");
        con.add_column(&Param::new("id", ParamType::Integer));
        con.add_column(&Param::new("day", ParamType::Text));
        con.add_column(&Param::new("title", ParamType::Text));
        con.add_column(&Param::new("context", ParamType::Text));
        con.add_column(&Param::new("nblocks", ParamType::Integer));
        con.add_column(&Param::new("top_block", ParamType::Integer));
        con.add_column(&Param::new("position", ParamType::Integer));
        con
    }
}

struct BlockTable;

struct BlockRow<'a> {
    inner: &'a InnerBlockRow,
    lookup: &'a HashMap<EntityId, String>,
}

impl<BlockTable> Row<BlockTable> for BlockRow<'_> {
    fn sqlize_values(&self) -> String {
        let inner = &self.inner;
        let id = uuid_lookup(self.lookup, Some(inner.entity_id));
        let parent = uuid_lookup(self.lookup, Some(inner.parent_id));
        let position = inner.position;
        let content = &inner.content;

        format!(
            "{}, {}, '{}', {}",
            id,
            parent,
            escape_quotes(content),
            position
        )
    }
}

impl Default for Table<BlockTable> {
    fn default() -> Self {
        let mut con: Table<BlockTable> = Table::new("lz_blocks");
        con.add_column(&Param::new("id", ParamType::Integer));
        con.add_column(&Param::new("parent", ParamType::Integer));
        con.add_column(&Param::new("content", ParamType::Text));
        con.add_column(&Param::new("position", ParamType::Integer));
        con
    }
}

struct EntityConnectionTable;

struct EntityConnectionRow<'a> {
    inner: &'a InnerEntityConnectionRow,
    lookup: &'a HashMap<EntityId, String>,
}

impl Default for Table<EntityConnectionTable> {
    fn default() -> Self {
        let mut con: Table<EntityConnectionTable> = Table::new("lz_connections");
        con.add_column(&Param::new("id", ParamType::Integer));
        con.add_column(&Param::new("node", ParamType::Text));
        con
    }
}

impl<EntityConnectionTable> Row<EntityConnectionTable> for EntityConnectionRow<'_> {
    fn sqlize_values(&self) -> String {
        let inner = &self.inner;
        let id = uuid_lookup(self.lookup, Some(inner.entity_id));
        let node = &inner.node;

        format!("{}, '{}'", id, escape_quotes(node),)
    }
}

#[derive(Default)]
pub struct Schemas {
    entities: Table<EntityTable>,
    sessions: Table<SessionTable>,
    entries: Table<EntryTable>,
    blocks: Table<BlockTable>,
    connections: Table<EntityConnectionTable>,
}

impl Schemas {
    pub fn generate(&self, f: &mut impl io::Write) {
        let _ = f.write_all(&self.entities.sqlize().into_bytes());
        let _ = f.write_all(&self.sessions.sqlize().into_bytes());
        let _ = f.write_all(&self.entries.sqlize().into_bytes());
        let _ = f.write_all(&self.blocks.sqlize().into_bytes());
        let _ = f.write_all(&self.connections.sqlize().into_bytes());
    }
}

impl SessionRows {
    pub fn generate_connections(&self, schemas: &Schemas, f: &mut impl io::Write) {
        for row in &self.connections {
            let s = schemas
                .connections
                .sqlize_insert(&EntityConnectionRow {
                    inner: row,
                    lookup: &self.lookup,
                })
                .to_string();
            let _ = f.write_all(&s.into_bytes());
        }
    }
    pub fn generate(&self, schemas: &Schemas, f: &mut impl io::Write) {
        // Entity List
        for row in &self.entities {
            let s = schemas.entities.sqlize_insert(row).to_string();
            let _ = f.write_all(&s.into_bytes());
        }

        // Session
        let s = schemas
            .sessions
            .sqlize_insert(&SessionRow {
                inner: &self.session,
                lookup: &self.lookup,
            })
            .to_string();
        let _ = f.write_all(&s.into_bytes());

        for row in &self.logs {
            let s = schemas
                .entries
                .sqlize_insert(&EntryRow {
                    inner: row,
                    lookup: &self.lookup,
                })
                .to_string();
            let _ = f.write_all(&s.into_bytes());
        }

        for row in &self.blocks {
            let s = schemas
                .blocks
                .sqlize_insert(&BlockRow {
                    inner: row,
                    lookup: &self.lookup,
                })
                .to_string();
            let _ = f.write_all(&s.into_bytes());
        }
    }
}
