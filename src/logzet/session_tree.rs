use super::{
    entity::{BlockIndex, Entity, EntryIndex, SessionIndex},
    session::SessionBuilder,
    AppendBlock, BlockData, Date, DateKey, DateMap, EntryMap, InsertBlock, InsertEntry,
    SessionWrapper, Time, TimeKey, WithId,
};

#[allow(dead_code)]
#[derive(Default)]
pub struct EntryNode {
    pub entry: EntryIndex,
    pub blocks: Vec<BlockIndex>,
}

impl AppendBlock for EntryNode {
    fn append_block(&mut self, block: &BlockData) {
        self.blocks.push(BlockIndex(block.id()));
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct SessionNode {
    pub session: SessionIndex,
    pub entries: Vec<EntryNode>,
    pub blocks: Vec<BlockIndex>,
}

#[allow(dead_code)]
pub fn entities_to_map(entities: &[Entity]) -> DateMap<EntryNode, SessionNode> {
    SessionBuilder::<EntryNode, SessionNode>::new()
        .process(entities)
        .build()
}

impl<'a, T> InsertEntry<'a> for SessionWrapper<T, SessionNode>
where
    T: WithId<Id = usize> + From<&'a Time>,
{
    fn insert_entry(&mut self, id: usize, time: &'a Time) {
        self.entries.insert(id, time);
    }
}

impl From<&Time> for EntryNode {
    fn from(_time: &Time) -> EntryNode {
        EntryNode::default()
    }
}

impl<T> InsertBlock for SessionWrapper<T, SessionNode>
where
    T: AppendBlock,
    SessionNode: AppendBlock,
{
    fn insert_block_into_entry(&mut self, entry_key: &TimeKey, block: &BlockData) {
        self.entries.append_block(entry_key, block);
    }

    fn insert_block_into_session(&mut self, block: &BlockData) {
        self.data.append_block(block);
    }
}

impl AppendBlock for SessionNode {
    fn append_block(&mut self, block: &BlockData) {
        self.blocks.push(BlockIndex(block.id()));
    }
}

impl<T> From<&Date> for SessionWrapper<T, SessionNode> {
    fn from(_date: &Date) -> SessionWrapper<T, SessionNode> {
        SessionWrapper {
            data: SessionNode::default(),
            entries: EntryMap::new(),
        }
    }
}

impl From<(DateKey, SessionWrapper<EntryNode, SessionNode>)> for SessionNode {
    fn from(value: (DateKey, SessionWrapper<EntryNode, SessionNode>)) -> SessionNode {
        let (_date, data) = value;
        SessionNode {
            session: data.data.session,
            entries: data.entries.inner.into_values().collect(),
            blocks: data.data.blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logzet::statement::Statement;
    use crate::logzet::statements_to_entities;
    use crate::logzet::TextLine;

    #[test]
    fn test_session_tree() {
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
            // 0
            dt(Date::default()),
            // 1
            tm(Time {
                key: time1.clone(),
                title: "First task of the day".to_string(),
                tags: vec!["timelog:00:15:00".to_string()],
                ..Default::default()
            }),
            // 2
            tl(TextLine {
                text: "I am writing some words".to_string(),
            }),
            tl(TextLine {
                text: "and I am doing my task".to_string(),
            }),
            // 3
            tm(Time {
                key: time2.clone(),
                title: "Brainstorming".to_string(),
                tags: vec!["timelog:00:18:00".to_string(), "brainstorm".to_string()],
                ..Default::default()
            }),
            // 4
            tl(TextLine {
                text: "this is a thought I had".to_string(),
            }),
            br.clone(),
            // 5
            tl(TextLine {
                text: "this is a another thought I had".to_string(),
            }),
            br.clone(),
            // 6
            tl(TextLine {
                text: "one more thought".to_string(),
            }),
            // 7
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

        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].session.0, 0);
        assert_eq!(tree[1].session.0, 7);

        let entries = &tree[0].entries;
        assert_eq!(entries.len(), 2);

        let expected_entry_ids: Vec<usize> = vec![1, 3];
        let generated_entry_ids: Vec<usize> = entries.iter().map(|e| e.entry.0).collect();
        assert_eq!(expected_entry_ids, generated_entry_ids);

        let blocks = &entries[1].blocks;
        let expected_block_ids: Vec<usize> = vec![4, 5, 6];
        let generated_block_ids: Vec<usize> = blocks.iter().map(|b| b.0).collect();

        assert_eq!(expected_block_ids, generated_block_ids);
    }

    #[test]
    fn test_append_blocks_to_session() {
        let dt = Statement::Date;
        let tl = Statement::TextLine;
        let br = Statement::Break;
        let document: Vec<Statement> = vec![
            dt(Date::default()),
            tl(TextLine {
                text: "I am writing some words".to_string(),
            }),
            tl(TextLine {
                text: "and I am doing my task".to_string(),
            }),
            br.clone(),
            // 5
            tl(TextLine {
                text: "this is a another thought I had".to_string(),
            }),
            br.clone(),
            // 6
            tl(TextLine {
                text: "one more thought".to_string(),
            }),
        ];

        let entities = statements_to_entities(document);

        let session_map = entities_to_map(&entities.entities);

        let tree: Vec<SessionNode> = session_map.into_iter().map(|s| s.into()).collect();

        let tree = &tree[0];
        assert_eq!(tree.blocks.len(), 3);
    }
}
