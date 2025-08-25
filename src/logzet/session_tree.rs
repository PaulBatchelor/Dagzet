use super::{
    entity::{BlockIndex, Entity, EntryIndex, SessionIndex},
    session::SessionBuilder,
    AppendBlock, BlockData, Date, DateKey, DateMap, EntryMap, InsertBlock, InsertEntry,
    SessionWrapper, Time, TimeKey, WithId,
};

#[allow(dead_code)]
#[derive(Default)]
pub struct EntryNode {
    entry: EntryIndex,
    blocks: Vec<BlockIndex>,
}

impl AppendBlock for EntryNode {
    fn append_block(&mut self, block: &BlockData) {
        self.blocks.push(BlockIndex(block.id()));
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct SessionNode {
    session: SessionIndex,
    entries: Vec<EntryNode>,
}

#[allow(dead_code)]
pub fn entities_to_map(entities: Vec<Entity>) -> DateMap<EntryNode, SessionNode> {
    SessionBuilder::<EntryNode, SessionNode>::new()
        .process(&entities)
        .build()
}

pub type SessionTreeWrapper = SessionWrapper<EntryNode, SessionNode>;

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
{
    fn insert_block(&mut self, entry_key: &TimeKey, block: &BlockData) {
        self.entries.append_block(entry_key, block);
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

impl From<(DateKey, SessionWrapper<EntryNode, SessionNode>)> for SessionNode
where
    EntryNode: From<(TimeKey, EntryNode)>,
{
    fn from(value: (DateKey, SessionWrapper<EntryNode, SessionNode>)) -> SessionNode {
        let (_date, data) = value;
        SessionNode {
            session: data.data.session,
            entries: data.entries.inner.into_iter().map(|e| e.into()).collect(),
        }
    }
}

impl From<(TimeKey, EntryNode)> for EntryNode {
    fn from(value: (TimeKey, EntryNode)) -> EntryNode {
        let (_time, data) = value;
        EntryNode {
            entry: data.entry,
            blocks: data.blocks.into_iter().collect(),
        }
    }
}
