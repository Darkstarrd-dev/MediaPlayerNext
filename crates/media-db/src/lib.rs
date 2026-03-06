use anyhow::Result;
use rusqlite::Connection;

pub fn open_in_memory() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::open_in_memory;

    #[test]
    fn opens_in_memory_database() {
        let connection = open_in_memory().expect("in-memory database should open");
        let version: String = connection
            .query_row("select sqlite_version()", [], |row| row.get(0))
            .expect("sqlite version query should succeed");

        assert!(!version.is_empty());
    }
}
