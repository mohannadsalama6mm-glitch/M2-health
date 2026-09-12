use crate::{
    db::migrations,
    error::{AppError, ErrorCode},
};
use rusqlite::Connection;
use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

pub struct AppDb {
    connection: Mutex<Connection>,
}
impl AppDb {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut connection = Connection::open(path)?;
        Self::configure(&connection)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        migrations::run(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
    pub fn in_memory() -> Result<Self, AppError> {
        let mut connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        migrations::run(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
    fn configure(connection: &Connection) -> Result<(), AppError> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(())
    }
    pub fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.connection.lock().map_err(|_| {
            AppError::new(
                ErrorCode::Internal,
                "Local database access is unavailable. Restart the application.",
            )
        })
    }
}
