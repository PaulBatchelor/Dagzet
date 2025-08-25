use crate::logzet::{DateKey, EntityId, Session, TimeKey};

use super::{
    entity::{BlockIndex, Entity, EntityList},
    session_tree::{EntryNode, SessionNode},
};

/// Represents a row in a SQLite table, corresponding with the existing
/// log schema
#[allow(dead_code)]
#[derive(Default)]
pub struct EntryRow {
    pub entity_id: EntityId,
    pub day: String,
    pub time: String,
    pub title: String,
    pub category: Option<String>,
    pub nblocks: usize,
    pub top_block: Option<usize>,
    pub position: usize,
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

impl From<&DateKey> for String {
    fn from(date: &DateKey) -> String {
        let mut parts: Vec<String> = Vec::new();
        let s = format!("{:04}-{:02}-{:02}", date.year, date.month, date.day);

        parts.push(s);

        if let Some(context) = &date.context {
            parts.push(context.clone());
        }

        parts.join("#")
    }
}

impl From<&TimeKey> for String {
    fn from(time: &TimeKey) -> String {
        format!("{:02}:{:02}", time.hour, time.minute)
    }
}

impl From<&EntityRow> for String {
    fn from(row: &EntityRow) -> String {
        let mut chunks: Vec<String> = Vec::new();
        let uuid = &row.uuid;

        if let Some(date) = &uuid.date {
            chunks.push(date.into());
        }

        if let Some(time) = &uuid.time {
            chunks.push(time.into());
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

impl From<(&EntityList, &SessionNode, &EntryNode, usize)> for EntryRow {
    fn from(value: (&EntityList, &SessionNode, &EntryNode, usize)) -> EntryRow {
        let (entity_list, session_node, entry_node, position) = value;

        let entity_id = entry_node.entry.0;

        let (day, category) = if let Some(date) = entity_list.get_session(session_node.session) {
            ((&date.key).into(), date.key.context.clone())
        } else {
            (String::new(), None)
        };

        let (time, title) = if let Some(time) = entity_list.get_entry(entry_node.entry) {
            ((&time.key).into(), time.title.clone())
        } else {
            (String::new(), String::new())
        };

        let nblocks = entry_node.blocks.len();
        let top_block = if !entry_node.blocks.is_empty() {
            Some(entry_node.blocks[0].0)
        } else {
            None
        };

        EntryRow {
            entity_id,
            day,
            time,
            title,
            nblocks,
            top_block,
            category,
            position,
        }
    }
}

impl From<(&EntityList, &BlockIndex)> for BlockRow {
    fn from(_value: (&EntityList, &BlockIndex)) -> BlockRow {
        // TODO: implement
        BlockRow::default()
    }
}

impl From<&Entity> for EntityRow {
    fn from(_value: &Entity) -> EntityRow {
        // TODO: implement
        EntityRow::default()
    }
}

impl From<(&EntityList, usize, &String)> for EntityConnectionsRow {
    fn from(_value: (&EntityList, usize, &String)) -> EntityConnectionsRow {
        // TODO: implement
        EntityConnectionsRow::default()
    }
}

impl From<(&EntityList, &SessionNode)> for SessionRows {
    fn from(value: (&EntityList, &SessionNode)) -> SessionRows {
        let (entity_list, session_node) = value;

        let mut blocks: Vec<&BlockIndex> = Vec::new();

        let logs = session_node
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                e.blocks.iter().for_each(|b| blocks.push(b));
                (entity_list, session_node, e, i).into()
            })
            .collect();

        let blocks = blocks
            .into_iter()
            .map(|b| (entity_list, b).into())
            .collect();

        let entities = entity_list.entities.iter().map(|e| e.into()).collect();

        let connections = entity_list
            .connections
            .iter()
            .flat_map(|(id, nodes)| nodes.iter().map(|s| (entity_list, *id, s).into()))
            .collect();

        SessionRows {
            logs,
            blocks,
            entities,
            connections,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logzet::entity::{EntryIndex, SessionIndex};
    use crate::logzet::session_tree::entities_to_map;
    use crate::logzet::statement::Statement;
    use crate::logzet::statements_to_entities;
    use crate::logzet::{Block, BlockData, Command, Date, Entry, TextBlock, TextLine, Time};

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

    #[test]
    fn test_session_tree_rows() {
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
                args: vec!["dz".to_string(), "a/a".to_string()],
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
            cmd(Command {
                args: vec!["dz".to_string(), "a/b".to_string()],
            }),
            tl(TextLine {
                text: "this is a another thought I had".to_string(),
            }),
            br.clone(),
            cmd(Command {
                args: vec!["dz".to_string(), "a/c".to_string()],
            }),
            tl(TextLine {
                text: "one more thought".to_string(),
            }),
            dt(Date {
                key: DateKey {
                    year: 2025,
                    month: 8,
                    day: 23,
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];

        let entities = statements_to_entities(document);

        let session_map = entities_to_map(&entities.entities);

        let tree: Vec<SessionNode> = session_map.into_iter().map(|s| s.into()).collect();

        let rows: SessionRows = (&entities, &tree[0]).into();

        assert_eq!(rows.logs.len(), 2, "Incorrect number of logs");
        assert_eq!(rows.blocks.len(), 4, "Incorrect number of blocks");
        assert_eq!(
            rows.entities.len(),
            entities.entities.len(),
            "Incorrect number of entities"
        );
        assert_eq!(rows.connections.len(), 3, "Incorrect number of connections");
    }

    #[test]
    fn test_entry_node_to_row() {
        let mut entity_list = EntityList::default();
        let date = Date {
            key: DateKey {
                year: 2025,
                month: 8,
                day: 25,
                ..Default::default()
            },
            ..Default::default()
        };

        let time1 = Time::default();
        let time2 = Time {
            key: TimeKey {
                hour: 11,
                minute: 23,
            },
            id: 2,
            title: "Title for Entry 2".to_string(),
            tags: vec!["one".to_string(), "two".to_string(), "three".to_string()],
        };

        let session = SessionNode {
            session: SessionIndex(0),
            ..Default::default()
        };

        let block1 = BlockData::Text(TextBlock::default());
        let block2 = BlockData::Text(TextBlock::default());

        entity_list.entities.push(Entity::Session(date.clone()));
        entity_list.entities.push(Entity::Entry(time1.clone()));
        entity_list.entities.push(Entity::Entry(time2.clone()));
        entity_list.entities.push(Entity::Block(block1.clone()));
        entity_list.entities.push(Entity::Block(block2.clone()));

        // Manually build an entry node. This is usually automated
        let node = EntryNode {
            entry: EntryIndex(2),
            blocks: [3, 4].into_iter().map(BlockIndex).collect(),
        };

        let row: EntryRow = (&entity_list, &session, &node, 2).into();
        assert_eq!(row.entity_id, time2.id, "wrong id");
        let date_string: String = (&date.key).into();
        assert_eq!(&row.day, &date_string, "wrong date");
        let time_string: String = (&time2.key).into();
        assert_eq!(&row.time, &time_string, "wrong time");
        assert_eq!(&row.title, &time2.title, "wrong title");
        assert_eq!(row.nblocks, 2, "wrong nblocks");
        assert_eq!(row.top_block, Some(3), "wrong top block");
        assert_eq!(row.position, 2, "wrong position");
    }

    #[test]
    fn test_block_index_to_row() {
        unimplemented!();
    }

    #[test]
    fn test_entity_to_row() {
        unimplemented!();
    }
}
