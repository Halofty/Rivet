use super::{db::now_millis, users::UserId};
use crate::models::{Issue, IssuePatch, IssueStatus, NewIssue};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};

// A macro rather than a const so queries stay `&'static str` literals (SQLx `SqlSafeStr`).
macro_rules! columns {
    () => {
        "id, project_id, title, description, status, created_at, updated_at"
    };
}

/// Creates a Todo issue. `None` when the project is missing or not owned by `owner`.
///
/// Ownership check and insert are one statement, so there is no gap between them.
pub async fn create_issue(
    pool: &SqlitePool,
    owner: UserId,
    project_id: i64,
    input: &NewIssue,
) -> sqlx::Result<Option<Issue>> {
    let now = now_millis();
    let row = sqlx::query(concat!(
        "INSERT INTO issues (project_id, title, description, status, created_at, updated_at)
         SELECT id, ?, ?, ?, ?, ? FROM projects WHERE id = ? AND owner_id = ?
         RETURNING ",
        columns!()
    ))
    .bind(input.title())
    .bind(input.description())
    .bind(IssueStatus::Todo.as_str())
    .bind(now)
    .bind(now)
    .bind(project_id)
    .bind(owner.get())
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(issue_from_row).transpose()
}

/// The project's issues in creation order, or `None` if the project is not the owner's.
/// A missing project is not found rather than an empty list.
pub async fn list_project_issues(
    pool: &SqlitePool,
    owner: UserId,
    project_id: i64,
) -> sqlx::Result<Option<Vec<Issue>>> {
    let mut tx = pool.begin().await?;
    let owned = sqlx::query("SELECT 1 FROM projects WHERE id = ? AND owner_id = ?")
        .bind(project_id)
        .bind(owner.get())
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
    if !owned {
        return Ok(None);
    }
    let rows = sqlx::query(concat!(
        "SELECT ",
        columns!(),
        " FROM issues
         WHERE project_id = ?
         ORDER BY created_at ASC, id ASC"
    ))
    .bind(project_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    rows.iter()
        .map(issue_from_row)
        .collect::<sqlx::Result<_>>()
        .map(Some)
}

pub async fn get_issue(
    pool: &SqlitePool,
    owner: UserId,
    issue_id: i64,
) -> sqlx::Result<Option<Issue>> {
    let row = sqlx::query(
        "SELECT issues.id, issues.project_id, issues.title, issues.description, issues.status,
                issues.created_at, issues.updated_at
         FROM issues JOIN projects ON projects.id = issues.project_id
         WHERE issues.id = ? AND projects.owner_id = ?",
    )
    .bind(issue_id)
    .bind(owner.get())
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(issue_from_row).transpose()
}

/// Writes only the supplied fields and returns the committed issue, or `None` when the
/// issue is missing or not owned. `updated_at` never moves backwards.
pub async fn update_issue(
    pool: &SqlitePool,
    owner: UserId,
    issue_id: i64,
    patch: &IssuePatch,
) -> sqlx::Result<Option<Issue>> {
    let row = sqlx::query(concat!(
        "UPDATE issues SET
             title = COALESCE(?, title),
             description = COALESCE(?, description),
             status = COALESCE(?, status),
             updated_at = MAX(updated_at, ?)
         WHERE id = ?
           AND project_id IN (SELECT id FROM projects WHERE owner_id = ?)
         RETURNING ",
        columns!()
    ))
    .bind(patch.title())
    .bind(patch.description())
    .bind(patch.status().map(IssueStatus::as_str))
    .bind(now_millis())
    .bind(issue_id)
    .bind(owner.get())
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(issue_from_row).transpose()
}

fn issue_from_row(row: &SqliteRow) -> sqlx::Result<Issue> {
    let status: String = row.try_get("status")?;
    Ok(Issue {
        id: row.try_get("id")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        status: status.parse().map_err(|error| sqlx::Error::ColumnDecode {
            index: "status".to_owned(),
            source: Box::new(error),
        })?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
