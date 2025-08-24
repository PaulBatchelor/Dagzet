use crate::logzet::{BlockData, Date, Time};
use std::collections::HashMap;

#[allow(dead_code)]
pub type EntityId = usize;

pub type DagzetPathList = Vec<String>;

#[allow(dead_code)]
pub enum Entity {
    Block(BlockData),
    Entry(Time),
    Session(Date),
}

#[allow(dead_code)]
pub struct EntityList {
    pub entities: Vec<Entity>,
    pub connections: HashMap<EntityId, DagzetPathList>,
}
