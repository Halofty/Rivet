use super::{db::now_millis, users::UserId};
use crate::models::{NewProject, Project};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};

// A macro rather than a const so queries stay `&'static str` literals (SQLx `SqlSafeStr`).
macro_rules! columns {
    () => {
        "id, name, description, created_at, updated_at"
    };
}

pub async fn create_project(
    pool: &SqlitePool,
    owner: UserId,
    input: &NewProject,
) -> sqlx::Result<Project> {
    let now = now_millis();
    let row = sqlx::query(concat!(
        "INSERT INTO projects (owner_id, name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)
         RETURNING ",
        columns!()
    ))
    .bind(owner.get())
    .bind(input.name())
    .bind(input.description())
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;
    project_from_row(&row)
}

/// The owner's projects, newest first.
pub async fn list_projects(pool: &SqlitePool, owner: UserId) -> sqlx::Result<Vec<Project>> {
    sqlx::query(concat!(
        "SELECT ",
        columns!(),
        " FROM projects
         WHERE owner_id = ?
         ORDER BY created_at DESC, id DESC"
    ))
    .bind(owner.get())
    .fetch_all(pool)
    .await?
    .iter()
    .map(project_from_row)
    .collect()
}

/// `None` when the project does not exist or belongs to someone else; callers
/// report both as not found so IDs cannot be probed across accounts.
pub async fn get_project(
    pool: &SqlitePool,
    owner: UserId,
    project_id: i64,
) -> sqlx::Result<Option<Project>> {
    let row = sqlx::query(concat!(
        "SELECT ",
        columns!(),
        " FROM projects WHERE id = ? AND owner_id = ?"
    ))
    .bind(project_id)
    .bind(owner.get())
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(project_from_row).transpose()
}

fn project_from_row(row: &SqliteRow) -> sqlx::Result<Project> {
    Ok(Project {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
