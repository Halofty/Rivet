use super::{
    db::now_millis,
    users::{UserId, UserRecord, user_from_row},
};
use crate::models::ClientKind;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::fmt::Write as _;

/// Sessions end 30 days after sign-in; there is no silent renewal.
pub const SESSION_LIFETIME_SECS: i64 = 30 * 24 * 60 * 60;
const TOKEN_BYTES: usize = 32;

/// A bearer secret handed to the client once. Its `Debug` output is redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    fn generate() -> Result<Self, getrandom::Error> {
        let mut bytes = [0_u8; TOKEN_BYTES];
        getrandom::fill(&mut bytes)?;
        let mut text = String::with_capacity(TOKEN_BYTES * 2);
        for byte in bytes {
            let _ = write!(text, "{byte:02x}");
        }
        Ok(Self(text))
    }

    /// Accepts only the exact shape [`SessionToken::generate`] produces, so arbitrary
    /// header or cookie values are rejected before touching the database.
    pub fn parse(value: &str) -> Option<Self> {
        (value.len() == TOKEN_BYTES * 2
            && value
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
        .then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn hash(&self) -> Vec<u8> {
        Sha256::digest(self.0.as_bytes()).to_vec()
    }
}

impl std::fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionToken(..)")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("could not generate a session token: {0}")]
    Random(getrandom::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

pub async fn create_session(
    pool: &SqlitePool,
    user: UserId,
    client: ClientKind,
) -> Result<SessionToken, SessionError> {
    let token = SessionToken::generate().map_err(SessionError::Random)?;
    let now = now_millis();
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, client, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(token.hash())
    .bind(user.get())
    .bind(client.as_str())
    .bind(now)
    .bind(now.saturating_add(SESSION_LIFETIME_SECS * 1000))
    .execute(pool)
    .await?;
    Ok(token)
}

/// The signed-in account for an unexpired session.
pub async fn find_session_user(
    pool: &SqlitePool,
    token: &SessionToken,
) -> sqlx::Result<Option<UserRecord>> {
    let row = sqlx::query(
        "SELECT users.id, users.email, users.password_hash, users.created_at
         FROM sessions JOIN users ON users.id = sessions.user_id
         WHERE sessions.token_hash = ? AND sessions.expires_at > ?",
    )
    .bind(token.hash())
    .bind(now_millis())
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(user_from_row).transpose()
}

pub async fn delete_session(pool: &SqlitePool, token: &SessionToken) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(token.hash())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_expired_sessions(pool: &SqlitePool) -> sqlx::Result<u64> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(now_millis())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_hex_and_parse_strictly() {
        let a = SessionToken::generate().unwrap();
        let b = SessionToken::generate().unwrap();
        assert_ne!(a, b);
        assert_eq!(SessionToken::parse(a.as_str()), Some(a.clone()));
        assert_eq!(a.hash().len(), 32);
        assert_eq!(format!("{a:?}"), "SessionToken(..)");

        assert!(SessionToken::parse(&a.as_str().to_uppercase()).is_none());
        assert!(SessionToken::parse(&a.as_str()[1..]).is_none());
        assert!(SessionToken::parse("' OR 1=1 --").is_none());
    }
}
