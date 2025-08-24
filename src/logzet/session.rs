use crate::logzet::{
    statements_to_entities, Date, DateKey, Entry, EntryData, EntryMap, Session, SessionData,
    SessionMap, Statement, Time, TimeKey,
};

use crate::logzet::entity::Entity;

/// An intermediate structure used for sorting date entries in chronological order
#[allow(dead_code)]
pub fn entities_to_session_map(entities: Vec<Entity>) -> SessionMap {
    let mut session_map = SessionMap::new();
    let mut current_session: Option<DateKey> = None;
    let mut current_entry: Option<TimeKey> = None;
    for entity in entities {
        match entity {
            Entity::Session(date) => {
                // TODO: avoid clobbering
                session_map.insert(
                    date.key.clone(),
                    SessionData {
                        title: date.title,
                        tags: date.tags,
                        entries: EntryMap::new(),
                    },
                );
                current_session = Some(date.key.clone());
            }
            Entity::Entry(time) => {
                if let Some(session_key) = &current_session {
                    if let Some(session) = session_map.get_mut(session_key) {
                        session.entries.insert(
                            time.key.clone(),
                            EntryData {
                                title: time.title,
                                tags: time.tags,
                                blocks: vec![],
                            },
                        );
                        current_entry = Some(time.key);
                    }
                } else {
                    // TODO: error handling
                    panic!("No active session found");
                }
            }
            Entity::Block(block) => {
                let session_key = match &current_session {
                    Some(key) => key,
                    // TODO: error handling
                    _ => panic!("No active session found"),
                };

                let entry_key = match &current_entry {
                    Some(key) => key,
                    // TODO: error handling
                    _ => panic!("No active entry found"),
                };

                let session = match session_map.get_mut(session_key) {
                    Some(data) => data,
                    // TODO: error handling
                    _ => panic!("session not found"),
                };

                let entry = match session.entries.get_mut(entry_key) {
                    Some(data) => data,
                    // TODO: error handling
                    _ => panic!("entry not found"),
                };
                entry.blocks.push(block);
            }
        }
    }
    session_map
}

impl From<(TimeKey, EntryData)> for Entry {
    fn from(value: (TimeKey, EntryData)) -> Entry {
        let (time, data) = value;
        Entry {
            time: Time {
                key: time,
                title: data.title,
                tags: data.tags,
                ..Default::default()
            },
            blocks: data.blocks.into_iter().map(|b| b.into()).collect(),
        }
    }
}

pub fn build_session_map(stmts: Vec<Statement>) -> SessionMap {
    let entities = statements_to_entities(stmts);
    entities_to_session_map(entities.entities)
}

impl From<(DateKey, SessionData)> for Session {
    fn from(value: (DateKey, SessionData)) -> Session {
        let (date, data) = value;
        Session {
            date: Date {
                key: date,
                title: data.title,
                tags: data.tags,
            },
            entries: data.entries.into_iter().map(|e| e.into()).collect(),
        }
    }
}
