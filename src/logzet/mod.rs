use std::collections::BTreeMap;
mod entity;
mod session;
mod statement;
use entity::{statements_to_entities, EntityId};
use session::build_session_map;
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

impl WithId for Time {
    type Id = EntityId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn with_id(mut self, id: Self::Id) -> Self {
        self.id = id;
        self
    }
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

#[derive(Default)]
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

enum BlockData {
    Text(TextBlock),
}

impl From<BlockData> for Block {
    fn from(value: BlockData) -> Block {
        match value {
            BlockData::Text(lines) => Block::Text(lines.lines.join(" ")),
        }
    }
}

impl WithId for BlockData {
    type Id = EntityId;

    fn id(&self) -> Self::Id {
        match self {
            BlockData::Text(text) => text.uuid,
        }
    }

    fn with_id(self, id: Self::Id) -> Self {
        match self {
            BlockData::Text(mut text) => {
                text.uuid = id;
                BlockData::Text(text)
            }
        }
    }
}

#[allow(dead_code)]
trait WithId {
    type Id;
    fn id(&self) -> Self::Id;
    fn with_id(self, id: Self::Id) -> Self;
}

#[allow(dead_code)]
struct EntryData {
    title: String,
    tags: Vec<String>,
    blocks: Vec<BlockData>,
}

/// An intermediate structure used for sorting time entries for a day
#[allow(dead_code)]
type EntryMap = BTreeMap<TimeKey, EntryData>;

#[allow(dead_code)]
struct SessionData {
    entries: EntryMap,
    title: String,
    tags: Vec<String>,
}
#[allow(dead_code)]
type SessionMap = BTreeMap<DateKey, SessionData>;

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
    let session_map = build_session_map(stmts);
    session_map.into_iter().map(|s| s.into()).collect()
}

/// Represents a row in a SQLite table, corresponding with the existing
/// log schema
#[allow(dead_code)]
#[derive(Default)]
struct EntryRow {
    entity_id: EntityId,
    day: String,
    time: String,
    title: String,
    //comment: String,
    position: usize,
    category: Option<String>,
    nblocks: usize,
    top_block: Option<usize>,
}

#[allow(dead_code)]
#[derive(Default)]
struct SessionRow {
    entity_id: EntityId,
    day: String,
    title: Option<String>,
    category: Option<String>,
    blurb: Option<String>,
}

#[allow(dead_code)]
#[derive(Default)]
struct BlockRow {
    entity_id: EntityId,
    parent_id: EntityId,
    position: usize,
    content: String,
}

#[allow(dead_code)]
#[derive(Default)]
struct EntityConnectionsRow {
    entity_id: EntityId,
    node: String,
}

#[allow(dead_code)]
#[derive(Default)]
struct SessionRows {
    logs: Vec<EntryRow>,
    dayblurb: SessionRow,
    blocks: Vec<BlockRow>,
    entities: Vec<EntityRow>,
    connections: Vec<EntityConnectionsRow>,
}

#[allow(dead_code)]
#[derive(Default)]
struct EntityRowId {
    date: Option<DateKey>,
    time: Option<TimeKey>,
    position: Option<usize>,
}

impl From<&EntityRow> for String {
    fn from(row: &EntityRow) -> String {
        let mut chunks: Vec<String> = Vec::new();
        let uuid = &row.uuid;

        if let Some(date) = &uuid.date {
            let mut parts: Vec<String> = Vec::new();
            let s = format!("{:04}-{:02}-{:02}", date.year, date.month, date.day);

            parts.push(s);

            if let Some(context) = &date.context {
                parts.push(context.clone());
            }
            chunks.push(parts.join("#"));
        }

        if let Some(time) = &uuid.time {
            let s = format!("{:02}:{:02}", time.hour, time.minute);
            chunks.push(s);
        }

        if let Some(position) = uuid.position {
            let s = format!("{}", position);
            chunks.push(s);
        }

        chunks.join("/")
    }
}

#[allow(dead_code)]
#[derive(Default)]
struct EntityRow {
    uuid: EntityRowId,
}

impl From<Session> for SessionRows {
    fn from(value: Session) -> SessionRows {
        let date = &value.date.key;
        let category = None;
        let mut entities = Vec::new();

        let dayblurb = SessionRow {
            day: format!("{:04}-{:02}-{:02}", date.year, date.month, date.day),
            title: Some(value.date.title),
            category: category.clone(),
            blurb: None,
            ..Default::default()
        };

        entities.push(EntityRow {
            uuid: EntityRowId {
                date: Some(date.clone()),
                ..Default::default()
            },
        });

        let logs = value
            .entries
            .iter()
            .map(|e| EntryRow {
                day: format!("{:04}-{:02}-{:02}", date.year, date.month, date.day),
                time: format!("{:02}:{:02}", e.time.key.hour, e.time.key.minute),
                category: category.clone(),
                title: e.time.title.clone(),
                position: 0,
                //comment: blocks_to_string(e.blocks),
                ..Default::default()
            })
            .collect();

        value.entries.iter().for_each(|e| {
            entities.push(EntityRow {
                uuid: EntityRowId {
                    date: Some(date.clone()),
                    time: Some(e.time.key.clone()),
                    ..Default::default()
                },
            });
            e.blocks.iter().enumerate().for_each(|(i, _)| {
                entities.push(EntityRow {
                    uuid: EntityRowId {
                        date: Some(date.clone()),
                        time: Some(e.time.key.clone()),
                        position: Some(i),
                    },
                });
            });
        });

        let blocks = Vec::new();

        SessionRows {
            dayblurb,
            logs,
            blocks,
            entities,
            ..Default::default()
        }
    }
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

    #[derive(Default, Debug, PartialEq)]
    struct TestEntry {
        hour: u8,
        minute: u8,
        title: String,
        blocks: Vec<String>,
    }

    impl TestEntry {
        fn new(hour: u8, minute: u8, title: &str) -> Self {
            Self {
                hour,
                minute,
                title: title.to_string(),
                ..Default::default()
            }
        }

        fn with_blocks(mut self, blocks: Vec<&str>) -> Self {
            self.blocks = blocks.into_iter().map(|s| s.to_string()).collect();
            self
        }
    }

    impl From<&TestEntry> for Entry {
        fn from(val: &TestEntry) -> Self {
            let time = Time {
                key: TimeKey {
                    hour: val.hour,
                    minute: val.minute,
                },
                title: val.title.clone(),
                ..Default::default()
            };
            let blocks = val.blocks.iter().map(|s| Block::Text(s.clone())).collect();
            Entry { time, blocks }
        }
    }

    impl From<&EntryRow> for TestEntry {
        fn from(val: &EntryRow) -> TestEntry {
            let parts: Vec<_> = val.time.split(':').collect();
            let hour = str::parse::<u8>(parts[0]).unwrap();
            let minute = str::parse::<u8>(parts[1]).unwrap();
            let title = val.title.clone();
            let mut blocks: Vec<String> = Vec::new();

            // Currently don't have a great way to retrieve
            // these blocks.
            for _ in 0..val.nblocks {
                blocks.push(String::new());
            }

            TestEntry {
                hour,
                minute,
                title,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_session_rows() {
        let entry_data: Vec<TestEntry> = vec![
            TestEntry::new(10, 30, "entry A"),
            TestEntry::new(14, 30, "entry B"),
            TestEntry::new(15, 55, "entry C"),
        ];
        let date = Date {
            key: DateKey {
                month: 8,
                day: 20,
                year: 2025,
                context: None,
            },
            title: "Title for Day".to_string(),
            tags: vec![],
        };
        let entries: Vec<Entry> = entry_data.iter().map(|e| e.into()).collect();
        let session = Session {
            date: date.clone(),
            entries: entries.clone(),
        };
        let session_rows: SessionRows = session.into();

        // Check dayblurb entry
        let dayblurb = &session_rows.dayblurb;
        assert_eq!(&dayblurb.day, "2025-08-20");
        assert!(&dayblurb.title.is_some());
        if let Some(title) = &dayblurb.title {
            assert_eq!(title, "Title for Day");
        }

        assert_eq!(session_rows.logs.len(), entries.len());

        let generated_entries: Vec<TestEntry> =
            session_rows.logs.iter().map(|e| e.into()).collect();

        assert_eq!(&generated_entries, &entry_data);
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

    #[test]
    fn test_session_row_entity_uuids() {
        let entry_data: Vec<TestEntry> = vec![
            TestEntry::new(10, 30, "entry A"),
            TestEntry::new(14, 30, "entry B").with_blocks(vec!["block 1 in entry B"]),
            TestEntry::new(15, 55, "entry C")
                .with_blocks(vec!["block 1 in entry C", "block 2 in entry C"]),
        ];
        let date = Date {
            key: DateKey {
                month: 8,
                day: 20,
                year: 2025,
                context: None,
            },
            title: "Title for Day".to_string(),
            tags: vec![],
        };
        let entries: Vec<Entry> = entry_data.iter().map(|e| e.into()).collect();
        let session = Session {
            date: date.clone(),
            entries: entries.clone(),
        };
        let session_rows: SessionRows = session.into();
        // 1 title, 3 entries, 0 blocks
        assert_eq!(session_rows.entities.len(), 7);
        let expected_entity_uuids: Vec<String> = [
            "2025-08-20",
            "2025-08-20/10:30",
            "2025-08-20/14:30",
            "2025-08-20/14:30/0",
            "2025-08-20/15:55",
            "2025-08-20/15:55/0",
            "2025-08-20/15:55/1",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let generated_uuids: Vec<String> = session_rows.entities.iter().map(|e| e.into()).collect();

        assert_eq!(generated_uuids.len(), expected_entity_uuids.len());
        assert_eq!(generated_uuids, expected_entity_uuids);
    }
}
