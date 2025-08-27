use crate::logzet::id::WithId;
use crate::logzet::statement::Statement;
use crate::logzet::{BlockData, Date, TextBlock, Time};
use std::collections::BTreeMap;

pub type EntityId = usize;

pub type DagzetPathList = Vec<String>;

pub enum Entity {
    Block(BlockData),
    Entry(Time),
    Session(Date),
}

pub type ConnectionMap = BTreeMap<EntityId, DagzetPathList>;

#[derive(Default)]
pub struct EntityList {
    pub entities: Vec<Entity>,
    pub connections: ConnectionMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EntryIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SessionIndex(pub usize);

impl EntityList {
    pub fn get_block(&self, index: BlockIndex) -> Option<&BlockData> {
        if let Entity::Block(block) = &self.entities[index.0] {
            return Some(block);
        }
        None
    }

    pub fn get_entry(&self, index: EntryIndex) -> Option<&Time> {
        if let Entity::Entry(time) = &self.entities[index.0] {
            return Some(time);
        }
        None
    }

    pub fn get_session(&self, index: SessionIndex) -> Option<&Date> {
        if let Entity::Session(date) = &self.entities[index.0] {
            return Some(date);
        }
        None
    }
}

pub fn statements_to_entities(stmts: Vec<Statement>) -> EntityList {
    let mut entities = vec![];
    let mut curblock: Option<Vec<String>> = None;
    let mut connections: BTreeMap<EntityId, DagzetPathList> = BTreeMap::new();
    let mut last_node: Option<String> = None;

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

            let node = if args[1].starts_with('$') {
                if let Some(last_node) = &last_node {
                    if let Some(parts) = args[1].split_once("/") {
                        let (_, suffix) = parts;
                        format!("{}/{}", last_node, suffix)
                    } else {
                        last_node.clone()
                    }
                } else {
                    // TODO: error handling
                    panic!("$ not set");
                }
            } else {
                last_node = Some(args[1].clone());
                args[1].clone()
            };

            if let Some(con) = con {
                con.push(node);
            } else {
                connections.insert(last_entity_id, vec![node]);
            }

            continue;
        }
    }
    // Wrap up last block if it is the last thing
    if let Some(blk) = curblock {
        entities.push(Entity::Block(BlockData::Text(TextBlock::new(blk))).with_id(entities.len()));
    }
    EntityList {
        entities,
        connections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logzet::Command;
    #[test]
    fn test_dz_prev_operator() {
        let stmts: Vec<Statement> = vec![
            Statement::Date(Date::default()),
            Statement::Time(Time::default().hour(12).minute(34)),
            Statement::Command(Command {
                args: ["dz", "a/b"].into_iter().map(String::from).collect(),
            }),
            Statement::Time(Time::default().hour(13).minute(37)),
            Statement::Command(Command {
                args: ["dz", "$"].into_iter().map(String::from).collect(),
            }),
            Statement::Command(Command {
                args: ["dz", "$/c"].into_iter().map(String::from).collect(),
            }),
        ];
        let entities = statements_to_entities(stmts);
        let connections = entities.connections;
        let generated: Vec<(usize, Vec<String>)> = connections.into_iter().collect();
        let expected: Vec<(usize, Vec<String>)> = [(1, vec!["a/b"]), (2, vec!["a/b", "a/b/c"])]
            .into_iter()
            .map(|(id, nodes)| (id, nodes.into_iter().map(String::from).collect()))
            .collect();
        assert_eq!(generated, expected);
    }
}
