use super::{
    entity::{BlockIndex, EntryIndex, SessionIndex},
    AppendBlock,
};

#[allow(dead_code)]
pub struct EntryNode {
    entry: EntryIndex,
    blocks: Vec<BlockIndex>,
}

impl AppendBlock<BlockIndex> for EntryNode {
    fn append_block(&mut self, block: BlockIndex) {
        self.blocks.push(block);
    }
}

#[allow(dead_code)]
pub struct SessionNode {
    session: SessionIndex,
    entries: Vec<EntryNode>,
}
