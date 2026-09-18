# Milestone 2 — 도메인 모델과 SQLite

작성일: 2026-09-17

## 요약

UI 전송 계층과 독립된 영속 도메인을 구현했다. 공통 모델·검증, SQLx/SQLite 연결과 마이그레이션, 소유자 기준의 직접 SQL 함수, 도메인·영속성 테스트를 추가했다. 서버 함수와 화면은 아직 이 계층을 사용하지 않는다. 프로젝트·이슈 화면 연결은 Milestone 3–4에서 진행한다.

이번 작업 중 인증 방식을 **방식 B(앱 내 인증)**로 결정했다. 이에 따라 계획서 원안과 달리 첫 마이그레이션부터 `users` 테이블과 `projects.owner_id`를 포함했다. 모든 조회·수정 SQL은 소유자 조건을 함께 검사한다. 인증 흐름(세션, 로그인, 요청 보호)은 바로 이어지는 Milestone 2v1에서 구현하며 [done2v1.md](done2v1.md)에 기록한다.

## 구현 내용

### 공통 모델과 검증 (`src/models/`)

모든 빌드(Web·Desktop·Server)에서 사용하며 SQLx, 라우트, 컴포넌트에 의존하지 않는다.

| 파일 | 내용 |
| --- | --- |
| `project.rs` | `Project`, `CreateProjectInput`, 검증된 `NewProject` |
| `issue.rs` | `Issue`, `IssueStatus`(`todo`/`in_progress`/`done`), `CreateIssueInput`→`NewIssue`, `UpdateIssueInput`→`IssuePatch` |
| `user.rs` | 공개 가능한 `User`(비밀번호 해시 제외), 이메일 정규화 `normalize_email` |
| `validation.rs` | 길이 상수, `Field`, `ValidationError`, 공통 trim·길이 검사 |

- 프로젝트 이름(1–100자)과 이슈 제목(1–200자)은 앞뒤 공백을 제거하고 Unicode scalar value 수로 센다.
- 설명은 공백을 보존하고 최대 10,000자까지 허용한다.
- 수정 입력은 필드가 하나도 없으면 거부한다. `None`은 변경하지 않음을, `Some("")`은 설명 지우기를 뜻한다.
- `NewProject`, `NewIssue`, `IssuePatch`는 필드가 비공개이며 `validate()`로만 만들 수 있다. 따라서 SQL 함수가 검증되지 않은 입력을 받을 수 없다.
- 검증 오류는 `Field`를 포함하므로 폼이 오류 위치를 표시할 수 있다.

### SQLite 스키마 (`migrations/0001_initial.sql`)

| 테이블 | 주요 제약 |
| --- | --- |
| `users` | `email UNIQUE`, 길이 3–254, `password_hash` 비어 있지 않음 |
| `projects` | `owner_id → users(id) ON DELETE RESTRICT`, 이름 길이·공백 검사, 설명 ≤ 10,000 |
| `issues` | `project_id → projects(id) ON DELETE RESTRICT`, 제목 길이·공백 검사, `status IN ('todo','in_progress','done')` |

- 모든 테이블은 `STRICT`로 만들어 SQLite의 느슨한 타입 변환을 막았다. 모든 테이블에 `updated_at >= created_at`을 검사한다.
- 인덱스는 `projects(owner_id, created_at, id)`와 `issues(project_id, created_at, id)` 두 개다.
- `build.rs`가 `migrations` 디렉터리 변경 시 다시 컴파일하도록 한다. 적용된 마이그레이션 파일은 수정하지 않는다.

### 데이터베이스 수명 주기 (`src/server/db.rs`)

- 경로는 `RIVET_DATABASE_PATH`(절대 경로만 허용)를 우선한다. 없으면 OS 로컬 앱 데이터 경로를 사용한다(Windows: `%LOCALAPPDATA%\Rivet\rivet.sqlite3`). 실행 중인 DB는 OneDrive 저장소 밖에 둔다.
- `open()`은 상위 디렉터리를 만들고, 다음 설정으로 연결한 뒤 내장 마이그레이션을 실행한다: 연결 1개, `foreign_keys=ON`, busy timeout 5초, rollback journal(`DELETE`). 실패하면 오류를 반환하며 메모리 DB로 대체하지 않는다.
- 타임스탬프는 서버가 `now_millis()`(UTC Unix 밀리초)로 설정한다.

### 소유자 기준 SQL 함수 (`src/server/{users,projects,issues}.rs`)

| 함수 | 동작 |
| --- | --- |
| `create_user` / `find_user_by_email` | 이메일 중복이면 `ON CONFLICT DO NOTHING`으로 `None`을 반환 |
| `create_project` / `list_projects` / `get_project` | `owner_id`로 제한. 목록은 `created_at DESC, id DESC` |
| `create_issue` | `INSERT … SELECT … FROM projects WHERE id=? AND owner_id=?` 한 문장으로 소유 확인과 삽입을 원자적으로 처리 |
| `list_project_issues` | 프로젝트가 없거나 남의 것이면 빈 목록이 아니라 `None`. 목록은 `created_at ASC, id ASC` |
| `get_issue` / `update_issue` | 프로젝트와 조인해 소유자를 확인. 수정은 `COALESCE`로 제공된 필드만 쓰고 `updated_at = MAX(updated_at, now)`로 시간이 역행하지 않게 함 |

- 소유자 인자는 `i64`가 아닌 `UserId` newtype이다. 저장된 사용자 행에서만 만들 수 있으므로 프로젝트/이슈 ID와 자리가 바뀌거나 임의 정수가 들어갈 수 없다.
- 존재하지 않는 ID와 다른 사용자의 ID는 모두 `None`이다. API 계층에서 둘 다 404로 응답해 다른 계정의 ID 존재 여부를 알 수 없게 한다(IDOR 방지).
- SQLx 0.9는 동적으로 만든 SQL 문자열을 거부한다(`SqlSafeStr`). 컬럼 목록은 `macro_rules!`와 `concat!`으로 정적 문자열을 만들고, 값은 모두 `bind`로 전달한다.
- `sqlx::Error`는 이 계층에서 그대로 반환한다. 사용자에게 보여 줄 오류로의 변환(내부 정보 숨김)은 API 계층의 책임이다.

### 기존 UI 정리

- `demo::PreviewStatus`를 제거하고 컴포넌트와 예시 데이터가 `models::IssueStatus`를 사용하게 했다. Milestone 3–4에서 예시 데이터를 실제 데이터로 바꿀 때 컴포넌트 수정 범위가 줄어든다.
- `ProjectForm`과 `IssueForm`의 하드코딩된 길이 검사를 `CreateProjectInput::validate()`, `CreateIssueInput::validate()`로 바꿨다. 클라이언트와 서버가 같은 규칙을 사용한다.
- 프로젝트 목록의 `"3 sample projects"`를 `PROJECTS.len()`으로 계산하게 했다.

## 의존성과 버전

| 항목 | 값 |
| --- | --- |
| SQLx | 0.9.0 (`runtime-tokio`, `sqlite-bundled`, `migrate`, `macros`), `server` feature 전용 optional |
| libsqlite3-sys / 내장 SQLite | 0.37.0 / SQLite 3.51.3 |
| 개발 의존성 | `tempfile` 3 (테스트별 임시 DB 디렉터리) |

## 검증 결과

| 검사 | 결과 |
| --- | --- |
| `cargo fmt --all` | 적용 |
| Server `cargo clippy --all-targets -D warnings` | 통과 |
| Web(wasm32) / Desktop `cargo clippy -D warnings` | 통과 |
| 모델 단위 테스트 | 7개 통과: trim·Unicode 길이, 설명 공백 보존, 상태 직렬화, 수정 입력의 필드 유무 구분, 이메일 정규화 |
| SQLite 통합 테스트 (`tests/persistence.rs`) | 8개 통과 |
| 기존 route·transport 테스트 | 5개 통과 |
| Web/Desktop 의존성 트리에서 `sqlx`/`libsqlite3` 검색 | 0건 (WASM 빌드에 DB 코드 없음) |

SQLite 통합 테스트는 테스트마다 한글이 포함된 중첩 임시 경로에 실제 파일 DB를 만든다.

1. 새 DB 마이그레이션, 반복 실행과 재시작 시 추가 적용 없음
2. 연결을 닫고 다시 열어도 데이터 유지
3. 외래 키, 상태 CHECK, 이름 길이, 소유자 없는 프로젝트, 소유 데이터가 있는 사용자 삭제 거부
4. 중복 이메일은 오류 대신 `None`
5. 프로젝트 목록의 최신순 정렬과 소유자 분리, 생성 시 타임스탬프
6. 프로젝트 간 이슈 분리, 없는 프로젝트·이슈 ID는 `None`
7. **다른 사용자가 ID로 조회·목록·이슈 생성·수정을 시도하면 모두 `None`이고 원본 데이터는 변하지 않음**
8. 상태만 수정하면 제목·설명 유지, 설명 `Some("")` 지우기, 제목 trim, Done에서 Todo로 되돌리기, `created_at` 불변

## 코드 리뷰 메모

- **계획 변경:** 원래 계획은 인증 없는 단일 사용자 스키마였다. 방식 B 채택으로 소유자 컬럼을 첫 마이그레이션에 넣었다. 데이터가 생긴 뒤 NOT NULL 소유자 컬럼을 추가하는 마이그레이션을 피하기 위해서다. `planning.md`에 이유와 함께 반영했다.
- **이메일 대소문자:** SQLite `lower()`는 ASCII만 처리한다. 그래서 DB CHECK가 아닌 Rust `normalize_email`에서 소문자로 바꾼다. 이메일 중복 판정은 정규화된 값에 대한 `UNIQUE` 제약이다.
- **`list_project_issues`의 트랜잭션:** 소유 확인과 목록 조회를 한 트랜잭션으로 묶었다. 현재 삭제 기능은 없지만 연결 수가 늘어나도 일관된 결과를 보장한다.
- **UI 수동 확인:** 이번 변경은 폼 검증 메시지와 상태 enum 교체뿐이며, 컴파일과 Clippy로 확인했다. 실제 브라우저 동작은 Milestone 2v1의 로그인 화면 검증과 함께 확인한다. 이전 마일스톤에서 남은 Desktop echo·복구 확인도 아직 미확인이다.

## 다음 단계

Milestone 2v1: 세션 테이블, Argon2id 비밀번호 해시, Web 쿠키·Desktop 토큰 인증, 로그인/최초 계정 생성 화면, Host·Origin 요청 보호.
