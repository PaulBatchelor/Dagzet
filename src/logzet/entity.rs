use crate::logzet::id::WithId;
use crate::logzet::statement::Statement;
use crate::logzet::{BlockData, Date, TextBlock, Time};
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockIndex(usize);

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryIndex(usize);

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionIndex(usize);

#[allow(dead_code)]
impl EntityList {
    fn get_block(&self, index: BlockIndex) -> Option<&BlockData> {
        if let Entity::Block(block) = &self.entities[index.0] {
            return Some(block);
        }
        None
    }

    fn get_entry(&self, index: EntryIndex) -> Option<&Time> {
        if let Entity::Entry(time) = &self.entities[index.0] {
            return Some(time);
        }
        None
    }

    fn get_session(&self, index: SessionIndex) -> Option<&Date> {
        if let Entity::Session(date) = &self.entities[index.0] {
            return Some(date);
        }
        None
    }
}

#[allow(dead_code)]
pub fn statements_to_entities(stmts: Vec<Statement>) -> EntityList {
    let mut entities = vec![];
    let mut curblock: Option<Vec<String>> = None;
    let mut connections: HashMap<EntityId, DagzetPathList> = HashMap::new();

    for stmt in stmts {
        if let Statement::Date(date) = stmt {
            // A new session will implicitly end the current block, if there is one
            if let Some(blk) = curblock {
                entities.push(
                    Entity::Block(BlockData::Text(TextBlock::new(blk))).with_id(entities.len()),
                );
                curblock = None;
            }
            entities.push(Entity::Session(date).with_id(entities.len()));
            continue;
        }

        if let Statement::Time(time) = stmt {
            // A new entry will implicitly end the current block, if there is one
            if let Some(blk) = curblock {
                entities.push(
                    Entity::Block(BlockData::Text(TextBlock::new(blk))).with_id(entities.len()),
                );
                curblock = None;
            }
            entities.push(Entity::Entry(time).with_id(entities.len()));
            continue;
        }

        if let Statement::TextLine(text) = stmt {
            if let Some(ref mut blk) = curblock {
                blk.push(text.text);
            } else {
                curblock = Some(vec![text.text]);
            }
            continue;
        }

        if matches!(stmt, Statement::Break) {
            if let Some(blk) = curblock {
                entities.push(
                    Entity::Block(BlockData::Text(TextBlock::new(blk))).with_id(entities.len()),
                );
                curblock = None;
            }

            continue;
        }

        if let Statement::Command(cmd) = stmt {
            let args = cmd.args;
            if args.is_empty() {
                continue;
            }

            if args[0] != "dz" {
                // TODO: error handling
                panic!("Unrecognized command: {}", args[0]);
            }

            if args.len() < 2 {
                // TODO: error handling
                panic!("Not enough args for dz");
            }

            // TODO: get ID
            let last_entity_id = match entities.last() {
                Some(entity) => entity.id(),
                // TODO: error handling
                None => panic!("No entity found"),
            };

            let con = connections.get_mut(&last_entity_id);

            if let Some(con) = con {
                con.push(args[1].clone());
            } else {
                connections.insert(last_entity_id, vec![args[1].clone()]);
            }

            continue;
        }
    }
    // Wrap up last block if it is the last thing
    if let Some(blk) = curblock {
        entities.push(Entity::Block(BlockData::Text(TextBlock::new(blk))));
    }
    EntityList {
        entities,
        connections,
    }
}
