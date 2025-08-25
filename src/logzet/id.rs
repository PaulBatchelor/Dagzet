use crate::logzet::entity::Entity;
use crate::logzet::session_tree::{EntryNode, SessionNode};
use crate::logzet::{BlockData, EntityId, EntryData, SessionInfo, SessionWrapper, Time};

use super::entity::{EntryIndex, SessionIndex};

#[allow(dead_code)]
pub trait WithId {
    type Id;
    fn id(&self) -> Self::Id;
    fn with_id(self, id: Self::Id) -> Self;
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

impl WithId for Entity {
    type Id = EntityId;
    fn id(&self) -> Self::Id {
        match self {
            Entity::Block(block) => block.id(),
            Entity::Entry(entry) => entry.id(),
            Entity::Session(_session) => unimplemented!(),
        }
    }
    fn with_id(self, id: Self::Id) -> Self {
        match self {
            Entity::Block(block) => Entity::Block(block.with_id(id)),
            Entity::Entry(entry) => Entity::Entry(entry.with_id(id)),
            Entity::Session(ref _session) => self,
        }
    }
}

impl<T> WithId for SessionWrapper<T, SessionInfo> {
    type Id = EntityId;
    fn id(&self) -> Self::Id {
        0
    }
    fn with_id(self, _id: Self::Id) -> Self {
        self
    }
}

impl WithId for EntryData {
    type Id = EntityId;
    fn id(&self) -> Self::Id {
        0
    }
    fn with_id(self, _id: Self::Id) -> Self {
        self
    }
}

impl<T> WithId for SessionWrapper<T, SessionNode> {
    type Id = EntityId;
    fn id(&self) -> Self::Id {
        self.data.session.0
    }
    fn with_id(mut self, id: Self::Id) -> Self {
        self.data.session = SessionIndex(id);
        self
    }
}

impl WithId for EntryNode {
    type Id = EntityId;

    fn id(&self) -> Self::Id {
        self.entry.0
    }

    fn with_id(mut self, id: Self::Id) -> Self {
        self.entry = EntryIndex(id);
        self
    }
}
