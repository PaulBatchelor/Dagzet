use crate::logzet::{
    entity::EntityList, id::WithId, BlockData, Date, DateKey, Entry, EntryData, InsertBlock,
    InsertEntry, Session, SessionData, SessionMap, Time, TimeKey,
};

use crate::logzet::entity::Entity;
use std::collections::BTreeMap;
use std::marker::PhantomData;

type DefaultSession = SessionData<EntryData<BlockData>>;

#[derive(Default)]
struct SessionBuilder<T> {
    session_map: SessionMap<T>,
    current_session: Option<DateKey>,
    current_entry: Option<TimeKey>,
    phantom: PhantomData<T>,
}

impl<T> SessionBuilder<T>
where
    T: InsertBlock + InsertEntry + WithId<Id = usize> + From<Date>,
{
    fn new() -> Self {
        SessionBuilder {
            phantom: PhantomData,
            current_session: None,
            current_entry: None,
            session_map: SessionMap::new(),
        }
    }

    fn build(self) -> BTreeMap<DateKey, T> {
        self.session_map.inner
    }

    fn insert_session(&mut self, id: usize, date: Date) {
        let date_key = date.key.clone();
        self.session_map.insert(id, date);
        self.current_session = Some(date_key);
    }

    fn insert_entry(&mut self, id: usize, time: Time) {
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

    fn insert_block(&mut self, id: usize, block: BlockData) {
        let session_key = match &self.current_session {
            Some(key) => key,
            // TODO: error handling
            _ => panic!("No active session found"),
        };

        let entry_key = match &self.current_entry {
            Some(key) => key,
            // TODO: error handling
            _ => panic!("No active entry found"),
        };

        let session = match self.session_map.get_session(session_key) {
            Some(data) => data,
            // TODO: error handling
            _ => panic!("session not found"),
        };

        session.insert_block(entry_key, block.with_id(id));
    }

    fn process(mut self, entities: Vec<Entity>) -> Self {
        for (id, entity) in entities.into_iter().enumerate() {
            match entity {
                Entity::Session(date) => {
                    self.insert_session(id, date);
                }
                Entity::Entry(time) => {
                    self.insert_entry(id, time);
                }
                Entity::Block(block) => {
                    self.insert_block(id, block);
                }
            }
        }
        self
    }
}

/// An intermediate structure used for sorting date entries in chronological order
#[allow(dead_code)]
pub fn entities_to_session_map(entities: Vec<Entity>) -> BTreeMap<DateKey, DefaultSession> {
    SessionBuilder::<DefaultSession>::new()
        .process(entities)
        .build()
}

impl From<(TimeKey, EntryData<BlockData>)> for Entry {
    fn from(value: (TimeKey, EntryData<BlockData>)) -> Entry {
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

impl<T> From<(DateKey, SessionData<T>)> for Session
where
    Entry: From<(TimeKey, T)>,
{
    fn from(value: (DateKey, SessionData<T>)) -> Session {
        let (date, data) = value;
        Session {
            date: Date {
                key: date,
                title: data.title,
                tags: data.tags,
            },
            entries: data.entries.inner.into_iter().map(|e| e.into()).collect(),
        }
    }
}
