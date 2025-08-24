use super::{
    entity::{BlockIndex, EntryIndex, SessionIndex},
    AppendBlock, BlockData, WithId,
};

#[allow(dead_code)]
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
pub struct SessionNode {
    session: SessionIndex,
    entries: Vec<EntryNode>,
}
