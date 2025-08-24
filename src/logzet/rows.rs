use crate::logzet::{DateKey, EntityId, Session, TimeKey};

/// Represents a row in a SQLite table, corresponding with the existing
/// log schema
#[allow(dead_code)]
#[derive(Default)]
pub struct EntryRow {
    pub entity_id: EntityId,
    pub day: String,
    pub time: String,
    pub title: String,
    pub position: usize,
    pub category: Option<String>,
    pub nblocks: usize,
    pub top_block: Option<usize>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct SessionRow {
    pub entity_id: EntityId,
    pub day: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub blurb: Option<String>,
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
pub struct SessionRows {
    logs: Vec<EntryRow>,
    dayblurb: SessionRow,
    blocks: Vec<BlockRow>,
    entities: Vec<EntityRow>,
    connections: Vec<EntityConnectionsRow>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct EntityRowId {
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
    use crate::logzet::{Block, Date, Entry, Time};

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
