use crate::logzet::{
    entity::EntityList, id::WithId, BlockData, Date, DateKey, Entry, EntryData, InsertBlock,
    InsertEntry, Session, SessionInfo, SessionMap, SessionWrapper, Time, TimeKey,
};

use crate::logzet::entity::Entity;
use std::collections::BTreeMap;

type DefaultSession = SessionWrapper<EntryData, SessionInfo>;

#[derive(Default)]
pub struct SessionBuilder<T, U> {
    session_map: SessionMap<T, U>,
    current_session: Option<DateKey>,
    current_entry: Option<TimeKey>,
}

impl<'a, T, U> SessionBuilder<T, U>
where
    SessionWrapper<T, U>: InsertBlock + InsertEntry<'a> + WithId<Id = usize> + From<&'a Date>,
{
    pub fn new() -> Self {
        SessionBuilder {
            current_session: None,
            current_entry: None,
            session_map: SessionMap::new(),
        }
    }

    pub fn build(self) -> BTreeMap<DateKey, SessionWrapper<T, U>> {
        self.session_map.inner
    }

    fn insert_session(&mut self, id: usize, date: &'a Date) {
        let date_key = date.key.clone();
        self.session_map.insert(id, date);
        self.current_session = Some(date_key);
        self.current_entry = None;
    }

    fn insert_entry(&mut self, id: usize, time: &'a Time) {
        if let Some(session_key) = &self.current_session {
            if let Some(session) = self.session_map.get_session(session_key) {
                let time_key = time.key.clone();
                session.insert_entry(id, time);
                self.current_entry = Some(time_key);
            }
        } else {
            // TODO: error handling
            panic!("No active session found");
        }
    }

    fn insert_block(&mut self, block: &BlockData) {
        let session_key = match &self.current_session {
            Some(key) => key,
            // TODO: error handling
            _ => panic!("No active session found"),
        };

        let session = match self.session_map.get_session(session_key) {
            Some(data) => data,
            // TODO: error handling
            _ => panic!("session not found"),
        };

        match &self.current_entry {
            Some(key) => session.insert_block_into_entry(key, block),
            _ => session.insert_block_into_session(block),
        }
    }

    pub fn process(mut self, entities: &'a [Entity]) -> Self {
        for (id, entity) in entities.iter().enumerate() {
            match entity {
                Entity::Session(date) => {
                    self.insert_session(id, date);
                }
                Entity::Entry(time) => {
                    self.insert_entry(id, time);
                }
                Entity::Block(block) => {
                    self.insert_block(block);
                }
            }
        }
        self
    }
}

/// An intermediate structure used for sorting date entries in chronological order
pub fn entities_to_session_map(entities: Vec<Entity>) -> BTreeMap<DateKey, DefaultSession> {
    SessionBuilder::<EntryData, SessionInfo>::new()
        .process(&entities)
        .build()
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

pub fn build_session_map(entities: EntityList) -> BTreeMap<DateKey, DefaultSession> {
    entities_to_session_map(entities.entities)
}

impl<T> From<(DateKey, SessionWrapper<T, SessionInfo>)> for Session
where
    Entry: From<(TimeKey, T)>,
{
    fn from(value: (DateKey, SessionWrapper<T, SessionInfo>)) -> Session {
        let (date, data) = value;
        Session {
            date: Date {
                key: date,
                title: data.data.title,
                tags: data.data.tags,
            },
            entries: data.entries.inner.into_iter().map(|e| e.into()).collect(),
        }
    }
}
