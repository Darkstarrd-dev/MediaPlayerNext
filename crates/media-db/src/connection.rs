use crate::migrations::run_migrations;
use crate::repositories::SqliteRepositories;
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub enum DatabaseLocation<'a> {
    InMemory,
    File(&'a Path),
}

pub struct MediaDatabase {
    connection: Connection,
}

impl MediaDatabase {
    pub fn open(location: DatabaseLocation<'_>) -> Result<Self> {
        let mut connection = match location {
            DatabaseLocation::InMemory => Connection::open_in_memory()?,
            DatabaseLocation::File(path) => Connection::open(path)?,
        };

        run_migrations(&mut connection)?;

        Ok(Self { connection })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }

    pub fn repositories(&self) -> SqliteRepositories<'_> {
        SqliteRepositories::new(&self.connection)
    }
}
