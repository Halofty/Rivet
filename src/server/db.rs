use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub static MIGRATOR: Migrator = sqlx::migrate!();

const DATABASE_PATH_VAR: &str = "RIVET_DATABASE_PATH";

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("{DATABASE_PATH_VAR} must be an absolute path, got `{0}`")]
    RelativePath(PathBuf),
    #[error("{DATABASE_PATH_VAR} is not valid Unicode")]
    InvalidPathVar,
    #[error("could not find a local application data directory; set {DATABASE_PATH_VAR}")]
    NoDataDirectory,
    #[error("could not create the database directory `{path}`: {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not open the database: {0}")]
    Open(#[from] sqlx::Error),
    #[error("could not apply database migrations: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Resolves the database file: `RIVET_DATABASE_PATH`, or `Rivet/rivet.sqlite3` in the
/// OS local application-data directory. The live database stays out of the repository.
pub fn database_path_from_env() -> Result<PathBuf, DatabaseError> {
    match std::env::var_os(DATABASE_PATH_VAR) {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.as_os_str().is_empty() {
                return Err(DatabaseError::InvalidPathVar);
            }
            if !path.is_absolute() {
                return Err(DatabaseError::RelativePath(path));
            }
            Ok(path)
        }
        None => Ok(local_data_dir()
            .ok_or(DatabaseError::NoDataDirectory)?
            .join("Rivet")
            .join("rivet.sqlite3")),
    }
}

fn local_data_dir() -> Option<PathBuf> {
    let from_var = |name| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    if cfg!(windows) {
        from_var("LOCALAPPDATA")
    } else if cfg!(target_os = "macos") {
        from_var("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        from_var("XDG_DATA_HOME").or_else(|| from_var("HOME").map(|home| home.join(".local/share")))
    }
}

/// Opens the pool and applies pending migrations before any request is served.
///
/// One connection keeps transactions easy to reason about for the single-backend MVP.
/// Foreign keys are enforced on every connection; there is no in-memory fallback.
pub async fn open(path: &Path) -> Result<SqlitePool, DatabaseError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|source| DatabaseError::CreateDirectory {
            path: parent.to_owned(),
            source,
        })?;
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5))
        .journal_mode(SqliteJournalMode::Delete);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// Current UTC time in Unix milliseconds. Timestamps are always assigned by the server.
pub(crate) fn now_millis() -> i64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
}
