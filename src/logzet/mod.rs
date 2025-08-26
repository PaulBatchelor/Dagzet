use std::collections::BTreeMap;
pub mod entity;
mod id;
pub mod rows;
mod session;
pub mod session_tree;
pub mod statement;
pub mod tables;
use entity::{statements_to_entities, EntityId};
use id::WithId;
use session::build_session_map;
use statement::Statement;

pub fn hello() {
    println!("hi logzet");
}

#[derive(Default, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub struct DateKey {
    month: u8,
    day: u8,
    year: u16,
    context: Option<String>,
}

#[derive(Default, Clone, Debug, PartialEq, Ord, PartialOrd, Eq)]
pub struct TimeKey {
    hour: u8,
    minute: u8,
}

/// Simple representation of a date
#[derive(Default, Clone)]
pub struct Date {
    key: DateKey,
    title: String,
    tags: Vec<String>,
}

/// Simple representation of a time
#[derive(Default, Clone)]
pub struct Time {
    id: EntityId,
    key: TimeKey,
    title: String,
    tags: Vec<String>,
}

/// A single line of text
#[derive(Clone)]
pub struct TextLine {
    text: String,
}

/// A command
#[derive(Clone)]
pub struct Command {
    args: Vec<String>,
}

#[derive(Clone)]
enum Block {
    Text,
}

#[derive(Default, Clone)]
pub struct TextBlock {
    uuid: EntityId,
    lines: Vec<String>,
}

impl TextBlock {
    fn new(lines: Vec<String>) -> Self {
        TextBlock {
            lines,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub enum BlockData {
    Text(TextBlock),
}

impl From<&BlockData> for String {
    fn from(block: &BlockData) -> String {
        match block {
            BlockData::Text(text) => text.lines.join(" "),
        }
    }
}

impl Default for BlockData {
    fn default() -> BlockData {
        BlockData::Text(TextBlock::default())
    }
}

impl From<BlockData> for Block {
    fn from(value: BlockData) -> Block {
        match value {
            BlockData::Text(_) => Block::Text,
        }
    }
}
type EntryData = EntryData_<BlockData>;

#[derive(Default)]
struct EntryData_<T> {
    title: String,
    tags: Vec<String>,
    blocks: Vec<T>,
}

trait AppendBlock {
    fn append_block(&mut self, block: &BlockData);
}

impl AppendBlock for EntryData {
    fn append_block(&mut self, block: &BlockData) {
        self.blocks.push(block.clone())
    }
}

impl From<&Time> for EntryData {
    fn from(time: &Time) -> EntryData {
        EntryData {
            title: time.title.clone(),
            tags: time.tags.clone(),
            blocks: vec![],
        }
    }
}

#[derive(Default)]
struct EntryMap<T> {
    inner: BTreeMap<TimeKey, T>,
}

impl<T> EntryMap<T> {
    fn new() -> Self {
        EntryMap {
            inner: BTreeMap::new(),
        }
    }

    fn insert<'a>(&mut self, id: usize, time: &'a Time)
    where
        T: From<&'a Time> + WithId<Id = usize>,
    {
        let time_key = time.key.clone();
        let data: T = time.into();
        self.inner.insert(time_key.clone(), data.with_id(id));
    }

    fn get_entry(&mut self, entry_key: &TimeKey) -> Option<&mut T> {
        self.inner.get_mut(entry_key)
    }

    fn append_block(&mut self, entry_key: &TimeKey, block: &BlockData)
    where
        T: AppendBlock,
    {
        let entry = match self.get_entry(entry_key) {
            Some(data) => data,
            // TODO: error handling
            _ => panic!("entry not found"),
        };

        entry.append_block(block);
    }
}

pub type DateMap<T, U> = BTreeMap<DateKey, SessionWrapper<T, U>>;

#[derive(Default)]
struct SessionMap<T, U> {
    inner: DateMap<T, U>,
}

impl<T, U> SessionMap<T, U> {
    fn new() -> Self {
        SessionMap {
            inner: BTreeMap::new(),
        }
    }

    fn insert<'a>(&mut self, id: usize, date: &'a Date)
    where
        SessionWrapper<T, U>: From<&'a Date> + WithId<Id = usize>,
    {
        let date_key = date.key.clone();
        let data: SessionWrapper<T, U> = date.into();
        self.inner.insert(date_key, data.with_id(id));
    }

    fn get_session(&mut self, session_key: &DateKey) -> Option<&mut SessionWrapper<T, U>> {
        self.inner.get_mut(session_key)
    }
}

#[derive(Default)]
struct SessionInfo {
    title: String,
    tags: Vec<String>,
}

#[derive(Default)]
pub struct SessionWrapper<T, S> {
    entries: EntryMap<T>,
    data: S,
}

trait InsertEntry<'a> {
    fn insert_entry(&mut self, id: usize, time: &'a Time);
}

impl<'a, T> InsertEntry<'a> for SessionWrapper<T, SessionInfo>
where
    T: WithId<Id = usize> + From<&'a Time>,
{
    fn insert_entry(&mut self, id: usize, time: &'a Time) {
        self.entries.insert(id, time);
    }
}

trait InsertBlock {
    fn insert_block_into_entry(&mut self, entry_key: &TimeKey, block: &BlockData);
    fn insert_block_into_session(&mut self, block: &BlockData);
}

impl<T> InsertBlock for SessionWrapper<T, SessionInfo>
where
    T: AppendBlock,
{
    fn insert_block_into_entry(&mut self, entry_key: &TimeKey, block: &BlockData) {
        self.entries.append_block(entry_key, block);
    }

    fn insert_block_into_session(&mut self, _block: &BlockData) {
        // NOTE: this implementation isn't really used, so I might not actually
        // need to update this for a while
        unimplemented!()
    }
}

impl<T> From<&Date> for SessionWrapper<T, SessionInfo> {
    fn from(date: &Date) -> SessionWrapper<T, SessionInfo> {
        SessionWrapper {
            data: SessionInfo {
                title: date.title.clone(),
                tags: date.tags.clone(),
            },
            entries: EntryMap::new(),
        }
    }
}

#[derive(Default, Clone)]
struct Entry {
    time: Time,
    blocks: Vec<Block>,
}

#[derive(Default)]
struct Session {
    date: Date,
    entries: Vec<Entry>,
}

#[allow(dead_code)]
fn build_sessions(stmts: Vec<Statement>) -> Vec<Session> {
    let entities = statements_to_entities(stmts);
    let session_map = build_session_map(entities);
    session_map.into_iter().map(|s| s.into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_ordering() {
        let stmts: Vec<Statement> = vec![
            Statement::Date(Date {
                key: DateKey {
                    year: 2025,
                    month: 8,
                    day: 19,
                    context: None,
                },
                tags: vec![],
                title: "".to_string(),
            }),
            Statement::Date(Date {
                key: DateKey {
                    year: 2024,
                    month: 5,
                    day: 20,
                    context: None,
                },
                tags: vec![],
                title: "".to_string(),
            }),
            Statement::Date(Date {
                key: DateKey {
                    year: 2025,
                    month: 8,
                    day: 18,
                    context: None,
                },
                tags: vec![],
                title: "".to_string(),
            }),
        ];
        let output = build_sessions(stmts.clone());

        // sanity check
        assert_eq!(stmts.len(), output.len());

        // extract date structs from statments
        let dates: Vec<_> = stmts
            .into_iter()
            .filter_map(|s| match s {
                Statement::Date(d) => Some(d),
                _ => None,
            })
            .collect();

        let expected_order: Vec<DateKey> = [1, 2, 0]
            .into_iter()
            .map(|i| dates[i].key.clone())
            .collect();

        let generated_order: Vec<DateKey> =
            output.into_iter().map(|s| s.date.key.clone()).collect();

        assert_eq!(expected_order, generated_order);
    }

    // Make sure statement parsing logic is grouping and chunking things correctly
    #[test]
    fn test_entry_groupings() {
        let dt = Statement::Date;
        let tm = Statement::Time;
        let tl = Statement::TextLine;
        let br = Statement::Break;
        let time1 = TimeKey {
            hour: 14,
            minute: 1,
        };
        let time2 = TimeKey {
            hour: 15,
            minute: 30,
        };
        let document: Vec<Statement> = vec![
            dt(Date::default()),
            tm(Time {
                key: time1.clone(),
                title: "First task of the day".to_string(),
                tags: vec!["timelog:00:15:00".to_string()],
                ..Default::default()
            }),
            tl(TextLine {
                text: "I am writing some words".to_string(),
            }),
            tl(TextLine {
                text: "and I am doing my task".to_string(),
            }),
            tm(Time {
                key: time2.clone(),
                title: "Brainstorming".to_string(),
                tags: vec!["timelog:00:18:00".to_string(), "brainstorm".to_string()],
                ..Default::default()
            }),
            tl(TextLine {
                text: "this is a thought I had".to_string(),
            }),
            br.clone(),
            tl(TextLine {
                text: "this is a another thought I had".to_string(),
            }),
            br.clone(),
            tl(TextLine {
                text: "one more thought".to_string(),
            }),
        ];
        let sessions = build_sessions(document.clone());

        // Only one document expected
        assert_eq!(sessions.len(), 1);

        // extract that document
        let session = &sessions[0];
        assert_eq!(session.entries.len(), 2);

        // Make sure the entries are being grouped as expected
        // Also make sure blocks are being chunked properly
        let entries = &session.entries;

        assert_eq!(&entries[0].time.key, &time1);
        assert_eq!(entries[0].blocks.len(), 1);
        assert!(matches!(entries[0].blocks[0], Block::Text));

        assert_eq!(&entries[1].time.key, &time2);
        assert_eq!(entries[1].blocks.len(), 3);
    }

    #[test]
    fn test_statements_to_entries() {
        let dt = Statement::Date;
        let tm = Statement::Time;
        let tl = Statement::TextLine;
        let br = Statement::Break;
        let time1 = TimeKey {
            hour: 14,
            minute: 1,
        };
        let time2 = TimeKey {
            hour: 15,
            minute: 30,
        };
        let document: Vec<Statement> = vec![
            dt(Date::default()),
            tm(Time {
                key: time1.clone(),
                title: "First task of the day".to_string(),
                tags: vec!["timelog:00:15:00".to_string()],
                ..Default::default()
            }),
            tl(TextLine {
                text: "I am writing some words".to_string(),
            }),
            tl(TextLine {
                text: "and I am doing my task".to_string(),
            }),
            tm(Time {
                key: time2.clone(),
                title: "Brainstorming".to_string(),
                tags: vec!["timelog:00:18:00".to_string(), "brainstorm".to_string()],
                ..Default::default()
            }),
            tl(TextLine {
                text: "this is a thought I had".to_string(),
            }),
            br.clone(),
            tl(TextLine {
                text: "this is a another thought I had".to_string(),
            }),
            br.clone(),
            tl(TextLine {
                text: "one more thought".to_string(),
            }),
        ];

        let entities = statements_to_entities(document);

        assert_eq!(entities.entities.len(), 7);
    }

    #[test]
    fn test_dz_full_paths() {
        let dt = Statement::Date;
        let tm = Statement::Time;
        let tl = Statement::TextLine;
        let br = Statement::Break;
        let cmd = Statement::Command;
        let time1 = TimeKey {
            hour: 14,
            minute: 1,
        };
        let time2 = TimeKey {
            hour: 15,
            minute: 30,
        };
        let document: Vec<Statement> = vec![
            dt(Date::default()),
            tm(Time {
                key: time1.clone(),
                title: "First task of the day".to_string(),
                tags: vec!["timelog:00:15:00".to_string()],
                ..Default::default()
            }),
            cmd(Command {
                args: vec!["dz".to_string(), "a/b".to_string()],
            }),
            tl(TextLine {
                text: "I am writing some words".to_string(),
            }),
            tl(TextLine {
                text: "and I am doing my task".to_string(),
            }),
            tm(Time {
                key: time2.clone(),
                title: "Brainstorming".to_string(),
                tags: vec!["timelog:00:18:00".to_string(), "brainstorm".to_string()],
                ..Default::default()
            }),
            cmd(Command {
                args: vec!["dz".to_string(), "g/h".to_string()],
            }),
            tl(TextLine {
                text: "this is a thought I had".to_string(),
            }),
            br.clone(),
            cmd(Command {
                args: vec!["dz".to_string(), "c/d".to_string()],
            }),
            cmd(Command {
                args: vec!["dz".to_string(), "e/f".to_string()],
            }),
            tl(TextLine {
                text: "this is a another thought I had".to_string(),
            }),
            br.clone(),
            tl(TextLine {
                text: "one more thought".to_string(),
            }),
        ];

        let entities = statements_to_entities(document);
        assert_eq!(entities.connections.len(), 3);

        let mut total_connections = 0;
        for (_, con) in entities.connections {
            total_connections += con.len();
        }

        assert_eq!(total_connections, 4);
    }
}
