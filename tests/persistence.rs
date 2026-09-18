#![cfg(feature = "server")]

use rivet::models::{
    ClientKind, CreateIssueInput, CreateProjectInput, IssueStatus, UpdateIssueInput,
};
use rivet::server::{
    db, issues, projects, sessions,
    users::{self, CreateUser, Registration, UserRecord},
};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tempfile::TempDir;

/// Each test gets its own on-disk database so reopening and migrations are exercised.
struct TestDb {
    dir: TempDir,
    pool: SqlitePool,
}

impl TestDb {
    async fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::open(&path(&dir)).await.unwrap();
        Self { dir, pool }
    }

    async fn reopen(self) -> Self {
        // Close before reopening (and before cleanup) so Windows releases the file.
        self.pool.close().await;
        let pool = db::open(&path(&self.dir)).await.unwrap();
        Self { pool, ..self }
    }

    async fn user(&self, email: &str) -> UserRecord {
        match users::create_user(&self.pool, email, "$argon2id$test-hash", Registration::Open)
            .await
            .unwrap()
        {
            CreateUser::Created(record) => record,
            _ => panic!("expected a new user"),
        }
    }
}

fn path(dir: &TempDir) -> PathBuf {
    // A nested, non-ASCII directory also covers directory creation and Unicode paths.
    dir.path().join("데이터").join("rivet.sqlite3")
}

fn project(name: &str) -> rivet::models::NewProject {
    CreateProjectInput {
        name: name.into(),
        description: "description".into(),
    }
    .validate()
    .unwrap()
}

fn issue(title: &str) -> rivet::models::NewIssue {
    CreateIssueInput {
        title: title.into(),
        description: "original".into(),
    }
    .validate()
    .unwrap()
}

#[tokio::test]
async fn fresh_database_migrates_and_repeated_startup_is_a_no_op() {
    let test = TestDb::new().await;
    let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&test.pool)
        .await
        .unwrap();
    assert_eq!(applied, db::MIGRATOR.iter().count() as i64);

    db::MIGRATOR.run(&test.pool).await.unwrap();
    let test = test.reopen().await;
    let reapplied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&test.pool)
        .await
        .unwrap();
    assert_eq!(reapplied, applied);
    test.pool.close().await;
}

#[tokio::test]
async fn data_survives_closing_and_reopening_the_database() {
    let test = TestDb::new().await;
    let owner = test.user("me@example.com").await;
    let created = projects::create_project(&test.pool, owner.id, &project("리벳 프로젝트"))
        .await
        .unwrap();
    let issue = issues::create_issue(&test.pool, owner.id, created.id, &issue("첫 이슈"))
        .await
        .unwrap()
        .unwrap();

    let test = test.reopen().await;
    let owner = users::find_user_by_email(&test.pool, "me@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        projects::get_project(&test.pool, owner.id, created.id)
            .await
            .unwrap(),
        Some(created)
    );
    assert_eq!(
        issues::get_issue(&test.pool, owner.id, issue.id)
            .await
            .unwrap(),
        Some(issue)
    );
    test.pool.close().await;
}

#[tokio::test]
async fn schema_enforces_foreign_keys_status_and_lengths() {
    let test = TestDb::new().await;
    let owner = test.user("me@example.com").await;
    let created = projects::create_project(&test.pool, owner.id, &project("Rivet"))
        .await
        .unwrap();

    let orphan = sqlx::query(
        "INSERT INTO issues (project_id, title, created_at, updated_at) VALUES (999, 'x', 1, 1)",
    )
    .execute(&test.pool)
    .await;
    assert!(orphan.is_err(), "foreign keys must be enforced");

    let bad_status = sqlx::query(
        "INSERT INTO issues (project_id, title, status, created_at, updated_at)
         VALUES (?, 'x', 'blocked', 1, 1)",
    )
    .bind(created.id)
    .execute(&test.pool)
    .await;
    assert!(bad_status.is_err(), "unknown statuses must be rejected");

    let long_name = sqlx::query(
        "INSERT INTO projects (owner_id, name, created_at, updated_at) VALUES (?, ?, 1, 1)",
    )
    .bind(owner.id.get())
    .bind("가".repeat(101))
    .execute(&test.pool)
    .await;
    assert!(long_name.is_err(), "name length must be checked");

    let orphan_project = sqlx::query(
        "INSERT INTO projects (owner_id, name, created_at, updated_at) VALUES (999, 'x', 1, 1)",
    )
    .execute(&test.pool)
    .await;
    assert!(orphan_project.is_err(), "projects need an existing owner");

    let delete_parent = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(owner.id.get())
        .execute(&test.pool)
        .await;
    assert!(delete_parent.is_err(), "owned rows restrict deletion");
    test.pool.close().await;
}

#[tokio::test]
async fn registration_policy_allows_only_the_first_account_unless_open() {
    let test = TestDb::new().await;
    assert!(
        users::registration_open(&test.pool, Registration::FirstUserOnly)
            .await
            .unwrap()
    );
    let first = users::create_user(
        &test.pool,
        "me@example.com",
        "$argon2id$a",
        Registration::FirstUserOnly,
    )
    .await
    .unwrap();
    assert!(matches!(first, CreateUser::Created(_)));
    assert!(
        !users::registration_open(&test.pool, Registration::FirstUserOnly)
            .await
            .unwrap()
    );

    let second = users::create_user(
        &test.pool,
        "other@example.com",
        "$argon2id$b",
        Registration::FirstUserOnly,
    )
    .await
    .unwrap();
    assert!(matches!(second, CreateUser::Closed));
    let duplicate = users::create_user(
        &test.pool,
        "me@example.com",
        "$argon2id$c",
        Registration::Open,
    )
    .await
    .unwrap();
    assert!(matches!(duplicate, CreateUser::EmailTaken));
    let open = users::create_user(
        &test.pool,
        "other@example.com",
        "$argon2id$b",
        Registration::Open,
    )
    .await
    .unwrap();
    assert!(matches!(open, CreateUser::Created(_)));
    test.pool.close().await;
}

#[tokio::test]
async fn sessions_resolve_their_user_until_deleted() {
    let test = TestDb::new().await;
    let me = test.user("me@example.com").await;
    let token = sessions::create_session(&test.pool, me.id, ClientKind::Desktop)
        .await
        .unwrap();
    let found = sessions::find_session_user(&test.pool, &token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, me.id);

    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&test.pool)
        .await
        .unwrap();
    assert_ne!(stored, token.as_str().as_bytes(), "only a hash is stored");

    sqlx::query("UPDATE sessions SET created_at = 1, expires_at = 2")
        .execute(&test.pool)
        .await
        .unwrap();
    assert!(
        sessions::find_session_user(&test.pool, &token)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        sessions::delete_expired_sessions(&test.pool).await.unwrap(),
        1
    );

    let token = sessions::create_session(&test.pool, me.id, ClientKind::Web)
        .await
        .unwrap();
    sessions::delete_session(&test.pool, &token).await.unwrap();
    assert!(
        sessions::find_session_user(&test.pool, &token)
            .await
            .unwrap()
            .is_none()
    );
    test.pool.close().await;
}

#[tokio::test]
async fn projects_are_listed_newest_first_per_owner() {
    let test = TestDb::new().await;
    let me = test.user("me@example.com").await;
    let other = test.user("other@example.com").await;
    let first = projects::create_project(&test.pool, me.id, &project("First"))
        .await
        .unwrap();
    let second = projects::create_project(&test.pool, me.id, &project("Second"))
        .await
        .unwrap();
    projects::create_project(&test.pool, other.id, &project("Theirs"))
        .await
        .unwrap();

    let mine = projects::list_projects(&test.pool, me.id).await.unwrap();
    assert_eq!(mine, vec![second, first]);
    assert!(
        mine.iter()
            .all(|project| project.created_at > 0 && project.updated_at == project.created_at)
    );
    test.pool.close().await;
}

#[tokio::test]
async fn issues_stay_in_their_project_and_missing_ids_are_not_found() {
    let test = TestDb::new().await;
    let me = test.user("me@example.com").await;
    let a = projects::create_project(&test.pool, me.id, &project("A"))
        .await
        .unwrap();
    let b = projects::create_project(&test.pool, me.id, &project("B"))
        .await
        .unwrap();
    let a1 = issues::create_issue(&test.pool, me.id, a.id, &issue("a1"))
        .await
        .unwrap()
        .unwrap();
    let a2 = issues::create_issue(&test.pool, me.id, a.id, &issue("a2"))
        .await
        .unwrap()
        .unwrap();
    let b1 = issues::create_issue(&test.pool, me.id, b.id, &issue("b1"))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(a1.status, IssueStatus::Todo);
    assert_eq!(
        issues::list_project_issues(&test.pool, me.id, a.id)
            .await
            .unwrap(),
        Some(vec![a1, a2])
    );
    assert_eq!(
        issues::list_project_issues(&test.pool, me.id, b.id)
            .await
            .unwrap(),
        Some(vec![b1])
    );

    assert_eq!(
        issues::list_project_issues(&test.pool, me.id, 999)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        issues::create_issue(&test.pool, me.id, 999, &issue("x"))
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        projects::get_project(&test.pool, me.id, 999).await.unwrap(),
        None
    );
    assert_eq!(
        issues::get_issue(&test.pool, me.id, 999).await.unwrap(),
        None
    );
    test.pool.close().await;
}

#[tokio::test]
async fn other_owners_cannot_read_create_or_update_through_ids() {
    let test = TestDb::new().await;
    let me = test.user("me@example.com").await;
    let intruder = test.user("intruder@example.com").await;
    let mine = projects::create_project(&test.pool, me.id, &project("Private"))
        .await
        .unwrap();
    let secret = issues::create_issue(&test.pool, me.id, mine.id, &issue("secret"))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        projects::get_project(&test.pool, intruder.id, mine.id)
            .await
            .unwrap(),
        None
    );
    assert!(
        projects::list_projects(&test.pool, intruder.id)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        issues::list_project_issues(&test.pool, intruder.id, mine.id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        issues::get_issue(&test.pool, intruder.id, secret.id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        issues::create_issue(&test.pool, intruder.id, mine.id, &issue("planted"))
            .await
            .unwrap(),
        None
    );
    let patch = UpdateIssueInput {
        title: Some("defaced".into()),
        ..Default::default()
    }
    .validate()
    .unwrap();
    assert_eq!(
        issues::update_issue(&test.pool, intruder.id, secret.id, &patch)
            .await
            .unwrap(),
        None
    );

    assert_eq!(
        issues::list_project_issues(&test.pool, me.id, mine.id)
            .await
            .unwrap(),
        Some(vec![secret])
    );
    test.pool.close().await;
}

#[tokio::test]
async fn updates_write_only_supplied_fields_and_own_timestamps() {
    let test = TestDb::new().await;
    let me = test.user("me@example.com").await;
    let parent = projects::create_project(&test.pool, me.id, &project("P"))
        .await
        .unwrap();
    let created = issues::create_issue(&test.pool, me.id, parent.id, &issue("Title"))
        .await
        .unwrap()
        .unwrap();

    let status_only = UpdateIssueInput {
        status: Some(IssueStatus::Done),
        ..Default::default()
    }
    .validate()
    .unwrap();
    let done = issues::update_issue(&test.pool, me.id, created.id, &status_only)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(done.status, IssueStatus::Done);
    assert_eq!(done.title, "Title");
    assert_eq!(done.description, "original");
    assert_eq!(done.created_at, created.created_at);
    assert!(done.updated_at >= created.updated_at);

    let edit = UpdateIssueInput {
        title: Some("  Renamed  ".into()),
        description: Some(String::new()),
        status: Some(IssueStatus::Todo),
    }
    .validate()
    .unwrap();
    let reopened = issues::update_issue(&test.pool, me.id, created.id, &edit)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reopened.title, "Renamed");
    assert_eq!(reopened.description, "");
    assert_eq!(reopened.status, IssueStatus::Todo);
    assert_eq!(reopened.project_id, parent.id);

    assert_eq!(
        issues::get_issue(&test.pool, me.id, created.id)
            .await
            .unwrap(),
        Some(reopened)
    );
    test.pool.close().await;
}
