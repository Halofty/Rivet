use super::db::now_millis;
use crate::models::User;
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};

/// The identity every owned query is scoped by.
///
/// It is created only from a stored user row, so data functions cannot be called
/// with an arbitrary integer or with a project/issue ID in the wrong position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UserId(i64);

impl UserId {
    pub fn get(self) -> i64 {
        self.0
    }
}

/// A stored account, including the secret needed only for sign-in checks.
#[derive(Clone)]
pub struct UserRecord {
    pub id: UserId,
    pub user: User,
    pub password_hash: String,
}

/// Who may create an account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Registration {
    /// Anyone who can reach the server.
    Open,
    /// Only while the database has no account: the initial setup of a personal server.
    FirstUserOnly,
}

pub enum CreateUser {
    Created(UserRecord),
    EmailTaken,
    Closed,
}

/// Inserts an account if `policy` allows it. `email` must come from
/// [`crate::models::normalize_email`] and `password_hash` must be a PHC-format hash.
///
/// The policy check and insert are one statement, so two concurrent first sign-ups
/// cannot both succeed.
pub async fn create_user(
    pool: &SqlitePool,
    email: &str,
    password_hash: &str,
    policy: Registration,
) -> sqlx::Result<CreateUser> {
    let now = now_millis();
    let row = sqlx::query(
        "INSERT INTO users (email, password_hash, created_at, updated_at)
         SELECT ?, ?, ?, ? WHERE ? OR NOT EXISTS (SELECT 1 FROM users)
         ON CONFLICT (email) DO NOTHING
         RETURNING id, email, password_hash, created_at",
    )
    .bind(email)
    .bind(password_hash)
    .bind(now)
    .bind(now)
    .bind(policy == Registration::Open)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = row {
        return user_from_row(&row).map(CreateUser::Created);
    }
    let taken: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = ?)")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(if taken {
        CreateUser::EmailTaken
    } else {
        CreateUser::Closed
    })
}

pub async fn registration_open(pool: &SqlitePool, policy: Registration) -> sqlx::Result<bool> {
    if policy == Registration::Open {
        return Ok(true);
    }
    sqlx::query_scalar("SELECT NOT EXISTS (SELECT 1 FROM users)")
        .fetch_one(pool)
        .await
}

pub async fn find_user_by_email(
    pool: &SqlitePool,
    email: &str,
) -> sqlx::Result<Option<UserRecord>> {
    let row = sqlx::query("SELECT id, email, password_hash, created_at FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    row.as_ref().map(user_from_row).transpose()
}

pub(crate) fn user_from_row(row: &SqliteRow) -> sqlx::Result<UserRecord> {
    let id: i64 = row.try_get("id")?;
    Ok(UserRecord {
        id: UserId(id),
        user: User {
            id,
            email: row.try_get("email")?,
            created_at: row.try_get("created_at")?,
        },
        password_hash: row.try_get("password_hash")?,
    })
}
