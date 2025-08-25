use std::collections::BTreeMap;
mod entity;
mod id;
mod rows;
mod session;
mod session_tree;
mod statement;
use entity::{statements_to_entities, EntityId};
use id::WithId;
use session::build_session_map;
use session_tree::SessionNode;
use statement::Statement;

pub fn hello() {
    println!("hi logzet");
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
struct DateKey {
    month: u8,
    day: u8,
    year: u16,
    context: Option<String>,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, PartialEq, Ord, PartialOrd, Eq)]
struct TimeKey {
    hour: u8,
    minute: u8,
}

/// Simple representation of a date
#[allow(dead_code)]
#[derive(Default, Clone)]
struct Date {
    key: DateKey,
    title: String,
    tags: Vec<String>,
}

/// Simple representation of a time
#[allow(dead_code)]
#[derive(Default, Clone)]
struct Time {
    id: EntityId,
    key: TimeKey,
    title: String,
    tags: Vec<String>,
}

/// A single line of text
#[allow(dead_code)]
#[derive(Clone)]
struct TextLine {
    text: String,
}

/// A command
#[allow(dead_code)]
#[derive(Clone)]
struct Command {
    args: Vec<String>,
}

#[allow(dead_code)]
#[derive(Clone)]
enum Block {
    Text(String),
    PreText(String),
}

#[derive(Default, Clone)]
struct TextBlock {
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
enum BlockData {
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
            BlockData::Text(lines) => Block::Text(lines.lines.join(" ")),
        }
    }
}
type EntryData = EntryData_<BlockData>;

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Default)]
struct EntryMap<T> {
    inner: BTreeMap<TimeKey, T>,
}

#[allow(dead_code)]
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

type DateMap<T, U> = BTreeMap<DateKey, SessionWrapper<T, U>>;

#[allow(dead_code)]
#[derive(Default)]
struct SessionMap<T, U> {
    inner: DateMap<T, U>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Default)]
struct SessionInfo {
    title: String,
    tags: Vec<String>,
}

#[allow(dead_code)]
#[derive(Default)]
struct SessionWrapper<T, S> {
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
    fn insert_block(&mut self, entry_key: &TimeKey, block: &BlockData);
}

impl<T> InsertBlock for SessionWrapper<T, SessionInfo>
where
    T: AppendBlock,
{
    fn insert_block(&mut self, entry_key: &TimeKey, block: &BlockData) {
        self.entries.append_block(entry_key, block);
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

#[allow(dead_code)]
type SessionTreeMap = BTreeMap<DateKey, SessionNode>;

#[allow(dead_code)]
#[derive(Default, Clone)]
struct Entry {
    time: Time,
    blocks: Vec<Block>,
}

#[allow(dead_code)]
#[derive(Default)]
struct Session {
    date: Date,
    entries: Vec<Entry>,
}

#[allow(dead_code)]
// TODO: error handling, plz read that rust for rustaceans chapter
fn build_sessions(stmts: Vec<Statement>) -> Vec<Session> {
    let entities = statements_to_entities(stmts);
    let session_map = build_session_map(entities);
    session_map.into_iter().map(|s| s.into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_tryfrom_break() {
        let s = "---".to_string();
        let stmt: Option<Statement> = s.try_into().ok();
        assert!(stmt.is_some(), "Could not parse");
        assert!(
            matches!(stmt.unwrap(), Statement::Break),
            "Did not parse correctly"
        );
    }

    #[test]
    fn test_statement_tryfrom_time() {
        let timestr = "@12:34 this is a title".to_string();

        let time: Option<Statement> = timestr.try_into().ok();

        assert!(time.is_some(), "Could not parse");
        let time = time.unwrap();
        assert!(
            matches!(&time, Statement::Time(_)),
            "Did not parse correctly"
        );

        if let Statement::Time(time) = time {
            let key = &time.key;
            assert_eq!(key.hour, 12);
            assert_eq!(key.minute, 34);
            assert_eq!(&time.title, "this is a title");
        }
    }

    #[test]
    fn test_statement_tryfrom_time_with_tags() {
        let time1str = "@12:34 test title #tag1 #tag2".to_string();
        let time2str = "@12:34 test #tag3 title #tag1 #tag2  extra spaces #tag4 ...".to_string();

        let time1: Option<Statement> = time1str.try_into().ok();

        assert!(time1.is_some(), "Could not parse");
        let time1 = time1.unwrap();
        assert!(
            matches!(&time1, Statement::Time(_)),
            "Did not parse correctly"
        );

        if let Statement::Time(time) = time1 {
            assert_eq!(&time.title, "test title");
            let expected_tags: Vec<_> = ["tag1", "tag2"].into_iter().map(String::from).collect();
            assert_eq!(&time.tags, &expected_tags);
        }

        let time2: Option<Statement> = time2str.try_into().ok();

        assert!(time2.is_some(), "Could not parse");
        let time2 = time2.unwrap();
        assert!(
            matches!(&time2, Statement::Time(_)),
            "Did not parse correctly"
        );

        if let Statement::Time(time) = time2 {
            assert_eq!(&time.title, "test title  extra spaces ...");
            let expected_tags: Vec<_> = ["tag3", "tag1", "tag2", "tag4"]
                .into_iter()
                .map(String::from)
                .collect();
            assert_eq!(&time.tags, &expected_tags);
        }
    }

    #[test]
    fn test_statement_tryfrom_date() {
        let date1str = "@2025-08-18 Test Title";
        let date2str = "@2025-08-18#abc Test Title (with context name)";
        let date3str = "@2025-08-18 Title with hashtags? #tag1 #tag2";
        let date4str = "@2025-08-18#abcde Everything! #tag2 #tag1";

        let date1: Option<Statement> = date1str.to_string().try_into().ok();

        assert!(date1.is_some(), "Could not parse");
        let date1 = date1.unwrap();
        assert!(
            matches!(&date1, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date1 {
            let key = &date.key;
            assert_eq!(key.year, 2025);
            assert_eq!(key.month, 8);
            assert_eq!(key.day, 18);
            assert_eq!(&date.title, "Test Title");
        }

        let date2: Option<Statement> = date2str.to_string().try_into().ok();
        assert!(date2.is_some(), "Could not parse");
        let date2 = date2.unwrap();
        assert!(
            matches!(&date2, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date2 {
            let key = date.key;
            assert_eq!(key.year, 2025);
            assert_eq!(key.month, 8);
            assert_eq!(key.day, 18);
            assert_eq!(&date.title, "Test Title (with context name)");
            assert!(key.context.is_some());

            if let Some(context) = &key.context {
                assert_eq!(context, "abc");
            }
        }

        let date3: Option<Statement> = date3str.to_string().try_into().ok();
        assert!(date3.is_some(), "Could not parse");
        let date3 = date3.unwrap();
        assert!(
            matches!(&date3, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date3 {
            let key = date.key;
            assert_eq!(key.year, 2025);
            assert_eq!(key.month, 8);
            assert_eq!(key.day, 18);
            assert_eq!(&date.title, "Title with hashtags?");
            assert!(key.context.is_none());
            let expected_tags: Vec<_> = ["tag1", "tag2"].into_iter().map(String::from).collect();
            assert_eq!(&date.tags, &expected_tags);
        }

        let date4: Option<Statement> = date4str.to_string().try_into().ok();
        assert!(date4.is_some(), "Could not parse");
        let date4 = date4.unwrap();
        assert!(
            matches!(&date4, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date4 {
            let key = &date.key;
            assert_eq!(key.year, 2025);
            assert_eq!(key.month, 8);
            assert_eq!(key.day, 18);
            assert_eq!(&date.title, "Everything!");
            assert!(key.context.is_some());
            let expected_tags: Vec<_> = ["tag2", "tag1"].into_iter().map(String::from).collect();
            assert_eq!(&date.tags, &expected_tags);
            if let Some(context) = &key.context {
                assert_eq!(context, "abcde");
            }
        }
    }

    #[test]
    fn test_statement_tryfrom_command() {
        let cmdstr = "#! dz foo/bar";
        let cmd: Option<Statement> = cmdstr.to_string().try_into().ok();
        assert!(cmd.is_some(), "Could not parse");
        let cmd = cmd.unwrap();
        assert!(
            matches!(&cmd, Statement::Command(_)),
            "Did not parse correctly"
        );

        if let Statement::Command(cmd) = cmd {
            let expected_args: Vec<_> = ["dz", "foo/bar"].into_iter().map(String::from).collect();
            assert_eq!(&cmd.args, &expected_args);
        }
    }

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
        let block_text = match &entries[0].blocks[0] {
            Block::Text(text) => text,
            _ => panic!("Expected block text"),
        };

        // Make sure line breaking logic is being handled correctly
        assert_eq!(block_text, "I am writing some words and I am doing my task");

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
