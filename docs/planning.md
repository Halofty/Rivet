# Rivet development plan

Status: milestone 0 implementation in progress; product features have not begun.

Repository inspected and documentation researched: 2026-09-17.

This document is the source of truth for Rivet's product scope and technical direction. Implement one milestone at a time. When evidence changes a decision, update this document with the reason in the same change as the implementation. The architecture below is the target; the progress record distinguishes implemented work from future work.

## Current progress

Milestone 0 now has a single-package Dioxus scaffold, separate feature entrypoints, a shared connection-check page, request extensions, typed server-function errors, an optional desktop backend URL, transport tests, and a repeatable PowerShell validation script. The temporary `/api/health` and `/api/echo` endpoints are setup probes, not additions to the seven-function MVP interface. Replace the echo probe and development page when product flows arrive.

Rust 1.98.1, rustfmt, Clippy, and the WASM target are installed; `rust-toolchain.toml` pins that version. DX 0.7.10 is in ignored `.tools/bin`, verified against the official archive checksum. The VS 2022 C++ tools and WebView2 runtime were already present. `Cargo.lock` records the resolved dependencies. The stock DX Bare-Bones template was inspected in an ignored directory; it still specified Dioxus 0.7.1, so Rivet uses the planned exact 0.7.10 pin and adds explicit fullstack/server features instead of copying the template unchanged.

The full `scripts/check.ps1` matrix has passed in the actual workspace path: formatting, separate Web/Desktop/Server compilation and Clippy checks, and three server-function transport tests. The Web DX build also logged success. Browser hydration and interactive flows, plus Desktop launch and real server-function calls, remain unverified, so milestone 0 is not complete. The Korean work log and code review are in [done.md](done.md), including the loader-retry draft-reset limitation. Run instructions and `RIVET_SERVER_URL` configuration are in README. There is no SQLite database yet; milestone 1 is next after this gate.

## 1. Initial repository findings and technology baseline

At the initial planning inspection, the complete working-tree inventory, including hidden entries outside Git internals, contained only `.git/` and `README.md`. The only tracked file was `README.md`, containing `# Rivet`. HEAD was `a3e9bed` (`Initial commit`), and the working tree was clean before this document was added. No applicable `AGENTS.md` was found in the repository or its ancestor directories.

At that point there was no `Cargo.toml`, `Cargo.lock`, `Dioxus.toml`, source tree, asset directory, database, migration, test, CI configuration, or existing architectural decision. The project name and repository history are preserved; there was no implementation to migrate.

`rustc`, `cargo`, and `dx` were not discoverable on this session's PATH. No default `%USERPROFILE%/.cargo` directory was found. This establishes that a usable local toolchain and dependency cache were not available to inspect, not that Rust cannot exist elsewhere on the machine. No tools were installed and no application builds were attempted during planning.

The environment is Windows with PowerShell. The repository is inside OneDrive and its path contains spaces and Korean characters. Future setup must verify this path works with the build tools and keep runtime database files outside the synchronized source directory.

### Version and API decisions

Use stable Dioxus **0.7.10** as the initial baseline, with the matching **0.7.10 Dioxus CLI**. The official release page identifies 0.7.10 as latest stable and 0.8.0-alpha.1 as a prerelease; the crate documentation also identifies Dioxus 0.7.10. Do not adopt a prerelease for this learning MVP. Pin the initial direct Dioxus dependency exactly and commit `Cargo.lock`; record the tested Rust toolchain during milestone 0. [Official releases](https://github.com/DioxusLabs/dioxus/releases), [Dioxus crate documentation](https://docs.rs/dioxus/0.7.10/dioxus/).

The official guide is labeled 0.7.0 and covers the 0.7 API line. These findings inform the plan:

| Verified API or behavior | Rivet decision |
| --- | --- |
| Fullstack builds have separate client and server variants, with `web`, `desktop`, and `server` features. | One package, explicitly separated feature builds; optional server dependencies. [Project setup](https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/). |
| `use_loader` supports hybrid client/server fetching; `use_resource` suits client-only fetching. | Use loaders for persisted page data; learn resources where data is actually client-only. [Data fetching](https://dioxuslabs.com/learn/0.7/essentials/basics/resources/). |
| A loader can be restarted and exposes loading/error state. | Explicitly refresh affected loaders after successful mutations. [Loader reference](https://docs.rs/dioxus-fullstack/0.7.10/dioxus_fullstack/struct.Loader.html). |
| `use_action` tracks event-driven asynchronous work; starting another action cancels its previous client task. | Use actions for saves, with submission disabled while pending; cancellation is not database rollback. [Async guide](https://dioxuslabs.com/learn/0.7/essentials/basics/async/). |
| Explicit `#[get]`/`#[post]` endpoints are available; desktop requires a backend URL outside the development setup. | Use stable endpoint paths and configure the desktop server URL before requests. [Native clients](https://dioxuslabs.com/learn/0.7/essentials/fullstack/native/). |
| `dioxus::serve` permits custom server initialization and an Axum router. | Initialize and migrate SQLite before serving requests. [Backend tutorial](https://dioxuslabs.com/learn/0.7/tutorial/backend/). |

Prefer versioned official documentation and the resolved crate source over older examples. Some fetched `latest` documentation pages resolved to different 0.7 patch versions, so milestone 0 must compile a minimal loader/action/server-function example against the pinned dependencies before product work. This document does not claim those APIs have been compiled locally.

## 2. Vision, scope, and success

Rivet is a small project and issue manager built to learn Dioxus through a realistic application. Success means understanding component composition, reactive state, forms, routing, asynchronous loading, fullstack boundaries, persistence, and platform differences while maintaining straightforward Rust code.

The MVP serves one person using a locally operated backend. It supports web and desktop clients using the same domain, UI, server functions, and database. Styling should make the application readable and usable; it is secondary to sound behavior.

### Required MVP capabilities

1. Create a project with a name and optional description.
2. List projects and open a project.
3. Create an issue within a project.
4. View project issues grouped into Todo, In Progress, and Done.
5. Change an issue's status using an accessible select or button action.
6. Open a stable issue detail route.
7. Edit an issue's title, description, and status.
8. Search the current project's issues and filter by status.
9. Preserve projects and issues across backend and client restarts using SQLite.
10. Complete these flows in a browser and the Windows desktop application.

The original milestone examples use “CRUD” broadly. Actual MVP scope is project create/read and issue create/read/update. Project editing, either entity's deletion, and moving an issue to another project are deferred. They are not prerequisites for the listed capabilities.

### Non-goals

The MVP excludes LLM/AI integration, authentication, multiple users, teams or organizations, cloud deployment, realtime synchronization, WebSockets, drag and drop, attachments, notifications, complex permissions, mobile, rich text, comments, labels, priorities, due dates, analytics, and external integrations. It also excludes offline replication, standalone desktop backend packaging, a generic design system, a generic repository framework, and a separate public REST client.

Do not add a feature merely because Linear or Trello has it. Every addition should satisfy the scoped product or teach a deliberate Dioxus concept.

### UI behavior

Use a header with Rivet and page actions, a project sidebar, and a main content region. The project board has three status groups in a fixed order. Narrow windows may stack the groups. Issue cards show a title and status, with a clear link to detail. Start with native form controls and one CSS asset; no styling framework is necessary.

Use labeled inputs, semantic buttons and links, visible keyboard focus, textual status labels, and readable loading/error messages. Prefer inline create forms over modal infrastructure during MVP. Search belongs on the project page; the shell must not imply unsupported global search.

## 3. Domain and invariants

| Type | Fields and meaning |
| --- | --- |
| `Project` | `id: i64`, `name: String`, `description: String`, `created_at: i64`, `updated_at: i64` |
| `Issue` | `id: i64`, `project_id: i64`, `title: String`, `description: String`, `status: IssueStatus`, `created_at: i64`, `updated_at: i64` |
| `IssueStatus` | `Todo`, `InProgress`, `Done`; serialized and stored as `todo`, `in_progress`, `done` |
| `CreateProjectInput` | Name and description only |
| `CreateIssueInput` | Title and description; parent ID is an explicit function argument; initial status is Todo |
| `UpdateIssueInput` | Optional title, description, and status; omitted fields remain unchanged |

IDs are positive, database-generated SQLite integer primary keys. This avoids introducing UUID generation for a single-database MVP. IDs are opaque identities, not issue numbers or ordering rules. Distributed/offline identifiers require a separate future decision.

Timestamps are UTC Unix milliseconds stored as SQLite INTEGER and shared as `i64`. The server sets both timestamps on creation and updates `updated_at` on successful changes. Clients cannot set IDs or timestamps. Display a deterministic UTC representation during SSR and initial hydration; local formatting can follow after hydration. Timestamps are metadata, not a concurrency token.

Proposed validation rules, kept together in shared pure functions:

- Trim names and titles; require 1–100 Unicode scalar values for a project name and 1–200 for an issue title.
- Descriptions are plain text, default to an empty string, preserve whitespace, and allow at most 10,000 Unicode scalar values.
- An issue always references an existing project. Its parent cannot change in MVP.
- Any status can transition to any other status, including reopening Done issues.
- Reject an update with no supplied fields. `Some("")` clears a description; `None` leaves it unchanged.
- Names need not be unique. Render all user content as text through RSX.

Client validation provides immediate feedback; the server always repeats it. Shared records and inputs derive the serialization and equality traits required by Dioxus and the transport. Keep SQL row conversion in the server layer rather than putting database dependencies on shared models.

## 4. Architecture and ownership

Use a single Cargo package. A multi-crate workspace adds boundaries that this two-entity application does not yet need. Dioxus feature compilation provides the needed platform separation.

Proposed structure, created incrementally when the files gain responsibilities:

```text
Cargo.toml
Cargo.lock
Dioxus.toml
rust-toolchain.toml
build.rs                   # added with embedded migrations, if needed
assets/main.css
docs/planning.md
migrations/0001_initial.sql
src/
  main.rs                  # feature-specific launch entrypoint
  lib.rs                   # module exports and App, also usable by tests
  app.rs                   # router, stylesheet, top-level boundaries
  routes/
    mod.rs                 # typed Route enum
    home.rs
    projects.rs
    project_detail.rs
    issue_detail.rs
    not_found.rs
  components/
    app_shell.rs
    project_list.rs
    project_form.rs
    issue_board.rs
    issue_card.rs
    issue_form.rs
    issue_filters.rs
    feedback.rs            # small loading, empty, and error views
  models/
    mod.rs
    project.rs
    issue.rs
    validation.rs
  api/
    mod.rs
    projects.rs            # shared server-function declarations
    issues.rs
    error.rs               # transport-facing application errors
  server/                  # compiled only with feature "server"
    mod.rs                 # startup and request context
    db.rs                  # configuration, pool, migrations
    projects.rs            # explicit SQL functions
    issues.rs
  platform/                # introduce only when configuration needs it
    mod.rs
    desktop.rs             # backend URL / later native integration
tests/
  persistence.rs
  api.rs
```

Dependency direction is `routes -> components + api + models`, then server-function bodies call `server -> models + SQLx`. Components receive values and event callbacks. Models do not depend on routes, components, SQLx, or native platform APIs.

Responsibilities:

- **App and shell:** shared navigation, the project-list loader, layout, page boundaries, and an `Outlet<Route>`.
- **Route components:** own route-specific loaders, form actions, selection/filter state, and coordination of successful saves and refreshes.
- **Reusable components:** render props and emit typed events. An issue card does not open a database or independently fetch its issue.
- **Models:** portable records, inputs, statuses, and pure validation. Shared transport errors live in `api/error.rs` because their Dioxus integration is not domain logic.
- **API modules:** server-function declarations visible in all builds; server-only bodies validate inputs, obtain request context, invoke SQL functions, and map errors.
- **Server modules:** database lifecycle and direct, parameterized SQL. Use a small `AppState` containing a clonable pool, provided through Axum request extensions and extracted inside server-function bodies. Ensure SSR receives the same context. [Dioxus/Axum integration](https://dioxuslabs.com/learn/0.7/essentials/fullstack/axum/).
- **Platform modules:** only real differences, initially desktop backend configuration. Do not create generic platform traits or empty adapters in advance.

There is no separate `services/` layer initially. Add a domain service only if a real workflow needs reusable multi-step rules; do not wrap each SQL function in a redundant forwarding method. Database functions take an explicit pool reference for straightforward testing.

### Dependencies and build boundaries

Start with Dioxus (`fullstack`, `router`) and Serde. Add SQLx with SQLite, Tokio runtime support, and migrations in milestone 2. SQLx **0.9.0** is the researched candidate; confirm its feature names, Rust requirements, and bundled SQLite version before locking it. Prefer runtime-bound queries over compile-time database introspection for this small application. [SQLx 0.9.0 documentation](https://docs.rs/sqlx/0.9.0/sqlx/).

Tokio, SQLx, and server configuration helpers must be optional dependencies enabled only by `server`. Reuse Dioxus's compatible Axum integration; if a direct Axum dependency becomes necessary, match the resolved version. Add a timestamp or application-data-directory helper only when actually needed. Avoid third-party state libraries and UI kits.

The intended Cargo feature mapping is:

```toml
# Design excerpt, not a complete manifest.
[features]
default = []
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
server = ["dioxus/server", "dep:sqlx", "dep:tokio"]
```

Apply `#[cfg(feature = "server")]` to the server module and its imports. Do not gate the whole API module: clients need generated stubs. Keep `fullstack` available to both client variants. Build each feature independently; `--all-features` is not the application validation strategy. [Fullstack feature guidance](https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/).

## 5. Routes and navigation

Define one `Route` enum with `Routable`, `Clone`, and `PartialEq`. Use typed `Link` targets and navigator calls throughout the application. `AppShell` is a router layout containing `Outlet::<Route>`. [Router tutorial](https://dioxuslabs.com/learn/0.7/tutorial/routing/).

| URL | Proposed variant | Responsibility |
| --- | --- | --- |
| `/` | `Home {}` | Small welcome/start page linking to Projects; no analytics dashboard |
| `/projects` | `Projects {}` | All projects and inline create-project form |
| `/projects/:project_id` | `ProjectDetail { project_id: i64 }` | Project heading, create-issue form, search/filter controls, grouped board |
| `/issues/:issue_id` | `IssueDetail { issue_id: i64 }` | Issue read/edit view and a link to its parent project |
| `/:..segments` | `NotFound { segments: Vec<String> }` | Unknown or malformed route fallback |

Route IDs are authoritative; do not mirror “selected project” into a writable global signal. On an issue route, derive the sidebar selection from the loaded issue's `project_id`. Validate nonpositive IDs and distinguish an unrecognized URL from a well-formed URL whose record does not exist. The latter gets a clear entity-not-found view and, for web SSR, an appropriate 404 response.

Create forms do not need their own routes. Search and status selection are local to a project page in MVP and reset when switching projects. Bookmarkable query filters can be a later refinement. Test browser refresh and back/forward navigation on detail routes, and in-app navigation/back behavior on desktop.

## 6. State, asynchronous work, and feedback

### State ownership

| State category | Owner and primitive | Examples |
| --- | --- | --- |
| Local UI | Closest component's `use_signal` | Input drafts, form visibility, status selection, search text |
| Derived UI | Pure expression or `use_memo` | Filtered issues, grouped columns, visible counts |
| Shared application UI | Small context, only when needed | Later theme and command-palette visibility |
| Server data | Route or shell `use_loader` result | Projects, project details, issues |
| Mutation lifecycle | `use_action` and form feedback | Saving, field errors, request failure |

The project list is the one initially useful shared server-data resource: the shell owns it, and the sidebar and Projects route consume the same loader through a narrow context. Expose its refresh capability to project creation. This is a loader handle, not a second globally mirrored project collection. Issue collections stay with the project route; issue details stay with the issue route.

Use `use_loader` for data required by SSR and desktop. An initial unresolved loader suspends; a failed loader propagates through Dioxus loading/error handling. Wrap the shell's list and the route content in suitable `SuspenseBoundary` and `ErrorBoundary` components so a page failure does not erase all navigation. Reserve `use_resource` for future client-only asynchronous data. [SSR and loading](https://dioxuslabs.com/learn/0.7/essentials/fullstack/ssr/).

Keep hooks in stable order. Route parameters must participate in reactive dependencies or explicitly key/remount the page data owner, so navigating between two project IDs cannot show the previous project's data. Do not seed a draft repeatedly from a loader during every render: initialize an editor for the current issue and preserve user changes during unrelated rerenders.

### Mutation and refresh contract

1. Validate the draft, then invoke an action from the form submit or status-change event.
2. Disable the relevant submit/status controls while pending. Preserve draft contents.
3. Validate again on the server and commit the write before returning the persisted entity.
4. On success, close/reset the create form or mark the edit saved, then restart affected loaders explicitly.
5. On failure, keep the form open and show actionable feedback. Do not move a card until its save succeeds.

| Mutation | Refresh behavior |
| --- | --- |
| Create project | Restart the shell's shared project list; navigate to the returned project ID |
| Create issue | Restart the current project's issue list |
| Update issue in detail | Refresh the issue detail; a later project-page mount loads fresh board data |
| Change status on board | Restart that board's issue list; a later issue-page mount loads fresh detail |

`Loader::restart()` reloads after the initial success without suspending again; `loading()` can drive a small refresh indicator. A refresh failure must still produce visible error/retry UI. If the failure occurred before a loader handle existed, reset/remount the data-loading subtree through the boundary's retry action. Verify that boundary behavior in the integration milestone. [Loader API](https://docs.rs/dioxus-fullstack/0.7.10/dioxus_fullstack/struct.Loader.html).

Do not assume server-function mutations invalidate reads automatically. There is no realtime cross-window synchronization: another browser tab or desktop window sees changes on refresh, navigation, or an explicit Reload action. Last successful writes win for overlapping edits in MVP; patch-style updates protect fields the user did not edit. Client cancellation cannot establish whether a write committed; do not blindly retry a create request after an ambiguous transport failure.

### Search and filtering

Fetch all issues for the selected project, then derive the visible list locally. Match the trimmed search text against lowercased title and description using Rust's ordinary lowercase conversion; this is substring search, not locale-aware full-text search. Apply the optional status filter with AND semantics. An empty search matches all issues. Show “no issues yet” separately from “no matches,” with a clear-filter action. Preserve fixed column order and deterministic issue ordering. No debounce, SQL FTS, pagination, or duplicate filtered-state signal is needed for the initial small dataset.

## 7. Fullstack interface and error contract

Use seven server functions, with explicit stable paths. These are proposed interface contracts, not compiled macro examples. Server-function macros own serialization and transport; callers use Rust functions directly.

| Function and input | Success payload | HTTP method/path |
| --- | --- | --- |
| `list_projects()` | `Vec<Project>` | GET `/api/projects` |
| `create_project(CreateProjectInput)` | `Project` | POST `/api/projects` |
| `get_project(project_id)` | `Project` | GET `/api/projects/{project_id}` |
| `list_project_issues(project_id)` | `Vec<Issue>` | GET `/api/projects/{project_id}/issues` |
| `create_issue(project_id, CreateIssueInput)` | `Issue` | POST `/api/projects/{project_id}/issues` |
| `get_issue(issue_id)` | `Issue` | GET `/api/issues/{issue_id}` |
| `update_issue(issue_id, UpdateIssueInput)` | `Issue` | POST `/api/issues/{issue_id}/update` |

The update function handles both basic editing and a status-only patch, avoiding a one-function-per-field interface. Listing issues for a nonexistent project returns not found, rather than an empty list. Project and issue lists have explicit stable ordering: projects by `created_at DESC, id DESC`; issues by `created_at ASC, id ASC` within displayed groups. No endpoint changes state on GET.

Use `std::result::Result<T, ApiError>` explicitly where helpful to avoid ambiguity with Dioxus's `Result` alias. Plan a small serializable transport error with validation details, not found, generic internal failure, and conversion from `ServerFnError` for transport failures. Dioxus 0.7 custom server-function errors require serialization, `From<ServerFnError>`, and `AsStatusCode`; verify the exact imports and round-trip behavior against the pinned crate. [Server-function error contract](https://dioxuslabs.com/learn/0.7/essentials/fullstack/server_functions/).

Map validation to 400, missing resources to 404, and unexpected server failures to 500. Field validation details should identify a known input field and user-readable message. Database errors are logged server-side with operation context, then mapped to a generic failure; do not serialize SQL, filesystem paths, or internal exception text. Transport failures render as a connection/retry message without discarding input.

Use parameterized SQL exclusively. Each update writes only supplied fields and returns the committed record. Multi-statement work requiring atomicity uses a short transaction. Obtain timestamps on the server. Server startup failure, including migration failure, prevents the listener from accepting requests and reports the error in logs.

## 8. SQLite strategy

Choose SQLx for asynchronous access and a built-in migration workflow. Use direct functions and explicit row mapping; no ORM, database-agnostic repository trait, or background persistence service.

### Initial schema contract

| Table | Columns and constraints |
| --- | --- |
| `projects` | `id INTEGER PRIMARY KEY`; `name TEXT NOT NULL`; `description TEXT NOT NULL DEFAULT ''`; `created_at INTEGER NOT NULL`; `updated_at INTEGER NOT NULL` |
| `issues` | `id INTEGER PRIMARY KEY`; `project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE RESTRICT`; `title TEXT NOT NULL`; `description TEXT NOT NULL DEFAULT ''`; `status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'done'))`; `created_at INTEGER NOT NULL`; `updated_at INTEGER NOT NULL` |

Add length/nonempty CHECK constraints consistent with normalized name/title inputs and description limits. Rust validation remains authoritative for Unicode trimming. Enforce foreign keys on every connection. Deletion is absent from the interface; RESTRICT makes the ownership rule safe if later code attempts it. `AUTOINCREMENT` is unnecessary while there is no deletion or distributed ID allocation.

Create one secondary index, `issues(project_id, created_at, id)`, for project-scoped listing and ordering. Primary keys already support detail lookups. Do not index status or text search yet: filtering is in memory and projects are expected to be small.

### Database lifecycle and storage

- Resolve the database location at server startup. Default to the OS local application-data directory under `Rivet/rivet.sqlite3`; on Windows this is `%LOCALAPPDATA%/Rivet/rivet.sqlite3`. Allow an explicit absolute path override via `RIVET_DATABASE_PATH`.
- Keep the live database outside this OneDrive repository. This is a project storage decision to keep runtime state and its journal files together and separate from synchronized source files. SQLite documents that WAL sidecar files are part of database state. [SQLite WAL documentation](https://www.sqlite.org/wal.html).
- Start with a small pool capped at **one connection**, explicit foreign-key enforcement, a five-second busy timeout, and the default rollback journal. This is enough for the single-user MVP and makes transaction behavior easy to study. Reassess WAL and a larger pool only after actual contention or a later concurrency milestone. [SQLx connection options](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/struct.SqliteConnectOptions.html).
- Initialize the pool once per backend process, not per render or request. Create the application-data directory/file when needed; never silently fall back to an ephemeral in-memory database after a failure.
- Commit only migrations. In milestone 0, ignore `target/`, generated bundles, local configuration, and SQLite database/journal files. Do not commit sample personal data.
- Both client types use the backend's database. Neither the browser nor the desktop UI opens this file directly.

### Migrations

Add `migrations/0001_initial.sql` in milestone 2. Use SQLx's embedded migration runner during server startup, before requests are served. Applied migration files are immutable; add a new numbered migration for each subsequent schema change. Keep the migration-history table managed by SQLx, not application code.

When using `sqlx::migrate!`, ensure changes in the migrations directory trigger recompilation, using the documented build-script `rerun-if-changed` approach. Test a fresh database and running the migrator twice; later migrations also need an upgrade test from the preceding schema. No live database is required at compile time when using runtime-bound queries. [SQLx migrations](https://docs.rs/sqlx/0.9.0/sqlx/macro.migrate.html).

Use a separate temporary on-disk database per persistence test so reopening and migration behavior are exercised. Close the pool before cleanup on Windows. Do not share an unqualified in-memory database across multiple pooled connections.

## 9. Web and Desktop strategy

The shared application consists of RSX, components, CSS, routes, validation, model types, and calls to server functions. Use Dioxus's web renderer for the browser and its desktop WebView renderer for Windows. Experimental native rendering is outside this plan. Windows desktop uses WebView2; verify runtime and compiler prerequisites during setup. [Platform guide](https://dioxuslabs.com/learn/0.7/guides/platforms/), [Getting started](https://dioxuslabs.com/learn/0.7/getting_started/).

| Concern | Web | Desktop |
| --- | --- | --- |
| Rendering | Fullstack SSR followed by hydration | Client-side rendering in a WebView |
| Backend | Same-origin requests | Explicit backend URL when outside the DX development setup |
| Routing | Address bar, direct URL loads, browser history | Shared routes with native application's in-app history |
| Storage | Server SQLite | The same server SQLite |
| Platform APIs | Browser access only after client mount when needed | Native access isolated behind the desktop feature |

The MVP desktop app is a client of a separately running local server. It is not a self-contained offline database application. Use `dx serve --desktop` during development and test the release client against an explicitly started local backend in milestone 6. Configure `dioxus::fullstack::set_server_url` before client launch when needed, isolated to the desktop client build. Stable endpoint names help keep native clients compatible, but client and server should still be built from the same revision during MVP. [Native fullstack behavior](https://dioxuslabs.com/learn/0.7/essentials/fullstack/native/).

Bind the unauthenticated MVP server to loopback and document its actual port. A loopback server supports local use without requiring cloud deployment or internet access; a stopped backend still means persisted data cannot be loaded. Do not add a process supervisor, packaged sidecar server, or offline fallback in this milestone. A backend-unavailable message with Retry is required.

Keep browser globals, local storage, current locale/time, and native window handles out of shared initial render logic. SSR and hydration must produce the same initial markup. Validate stylesheet/asset resolution and form behavior in both renderers early. Desktop smoke testing begins in milestone 0 and repeats with integration; it is not postponed entirely until milestone 6.

## 10. Dioxus learning map

| Application work | Concepts exercised | Evidence of learning |
| --- | --- | --- |
| Shell, project list, issue card | RSX, composition, props, keyed lists | Small components receive data and callbacks with stable entity keys |
| Create/edit forms | Events, controlled inputs, signals | Drafts survive failure; successful submissions reset deliberately |
| Issue filters and status groups | Signals, derived reactivity, `use_memo` | No duplicated filtered collection or manual DOM mutation |
| Project and issue pages | Router, typed parameters, layouts, navigation | Direct URLs and back navigation load the right record |
| Page reads | Async futures, loaders, suspense, error boundaries | Loading, not found, failure, and retry all work |
| Client-only asynchronous work, when introduced | `use_resource`, cancellation/lifecycle | Deliberate client-only use rather than duplicating SSR data fetching |
| Create/update actions | `use_action`, server functions, shared serialization | Pending state and explicit refresh follow confirmed writes |
| SQLite integration | Fullstack boundaries, feature gates, shared Rust types | WASM excludes database code; data survives backend restart |
| Web/Desktop parity | Renderers, launch configuration, asset handling | The same product flows pass on browser and Windows |
| Command palette, later | Global keyboard events, focus, context, filtering, conditional composition | Keyboard opens, navigates, executes, and dismisses accessibly |
| Drag and drop, later | Pointer/drag events, transient state, mutation feedback | Uses the existing update API and retains keyboard status controls |
| Realtime, later | WebSocket/SSE streams, task cleanup, resource refresh | Reconnect and unmount do not leak subscriptions |
| Desktop integrations, later | Feature gates and native APIs | Native behavior does not enter the web dependency graph |

## 11. Milestones

Each milestone should be one or several small, reviewable changes. “Complete” means its acceptance criteria have been checked and limitations recorded. See Current progress for milestone 0 evidence. Milestones 1–7 remain pending.

### Milestone 0 — Repository, tooling, and architecture

**Goal:** establish a reproducible development baseline before product features.

**Tasks:** preserve this plan; set up Rust/MSVC prerequisites, WASM target, and pinned DX; inspect the matching single-package fullstack template; add minimal Cargo/Dioxus configuration, ignore rules, and a toolchain file; verify separate web, desktop, and server builds. Compile a tiny disposable read/action example to settle exact loader, error, request-context, and desktop-backend APIs. Record versions and working commands in README.

**Dioxus concepts:** launch configuration, feature boundaries, fullstack build model, minimal async and server-function behavior.

**Completion criteria:** a minimal app launches on web and Windows desktop, a server-function round trip works from each, and the feature check matrix passes. No product feature implementation is needed for this gate. Inspection and this planning document satisfy only the planning subtask.

**Dependencies:** none.

### Milestone 1 — Static application shell

**Goal:** establish navigation and clear component boundaries.

**Tasks:** create typed routes, shell/sidebar, simple CSS, issue card/board, forms, and feedback components. Use a few deterministic fixtures only if useful. Cover unknown routes and keyboard navigation; keep fixtures out of persistence code.

**Dioxus concepts:** RSX, components, props, events, signals, typed Router, layout/Outlet.

**Completion criteria:** all intended routes render; links, back navigation, and status-group layout work in both renderers. Components have clear inputs/events and no database imports.

**Dependencies:** milestone 0.

### Milestone 2 — Domain and SQLite

**Goal:** establish the persistent domain independently of UI transport.

**Tasks:** implement shared models and validation; database configuration and pool; initial migration; direct SQL create/read/update operations; status mapping; database and validation tests. Record the resolved SQLx/SQLite versions and runtime database path.

**Dioxus concepts:** shared client/server types and feature separation; supporting Rust async and persistence skills.

**Completion criteria:** fresh migration and repeated startup pass; foreign keys and status constraints hold; edits preserve untouched fields; data survives closing/reopening the database; web compilation does not pull in SQLx.

**Dependencies:** milestone 0; follows milestone 1 in the planned sequence.

### Milestone 3 — Fullstack project flows

**Goal:** deliver project create/list/open through the real backend.

**Tasks:** initialize database before serving; provide request context; implement project server functions and errors; connect shell/project loaders and creation action; replace project fixtures; add loading, empty, not-found, retry, and validation feedback. Verify a desktop round trip now.

**Dioxus concepts:** server functions, serialization, loaders, actions, SSR/hydration, suspense/error boundaries, narrow context.

**Completion criteria:** a project can be created and opened, appears consistently in list/sidebar, and survives restart; invalid input and unavailable backend are recoverable; a direct project URL works after browser refresh.

**Dependencies:** milestones 1 and 2.

### Milestone 4 — Issue workflows

**Goal:** complete the core unit-of-work lifecycle.

**Tasks:** issue server functions; create form; project issue loader and grouped board; detail route; title/description editing and status changes; explicit loader refresh; retain draft/error state; remove issue fixtures.

**Dioxus concepts:** route-dependent async state, form actions, event callbacks, keyed lists, composition.

**Completion criteria:** create an issue in one of two projects, verify isolation, visit its detail, edit it, move it through all statuses and reopen it, then restart the backend and verify persistence. Failed saves do not imply success or clear drafts; editing status alone preserves title/description.

**Dependencies:** milestone 3 and issue persistence from milestone 2.

### Milestone 5 — Search and filtering

**Goal:** make a project's issue list easy to inspect using derived state.

**Tasks:** implement text and status controls, combined filtering, no-match state, reset action, and pure filter tests. Keep the original loaded collection as the only issue source.

**Dioxus concepts:** signals, memoized derivation, controlled inputs, conditional rendering.

**Completion criteria:** title/description substring search and status filters combine correctly; empty and case-varied searches work; clearing filters restores all issues; changes from successful saves affect the derived board.

**Dependencies:** milestone 4.

### Milestone 6 — Web and Desktop acceptance

**Goal:** verify a usable MVP across both targets and document operation.

**Tasks:** run the full quality matrix; build release variants; execute the acceptance flow on web and Windows desktop; test direct URLs, hydration, history, assets, focus, invalid input, backend outages, and restarts. Run desktop against an explicitly configured local server outside automatic DX client/server startup. Document startup commands, database location, reset procedure for disposable development data, and tested platform versions.

**Dioxus concepts:** deployment/build artifacts, renderer differences, native client configuration, error recovery.

**Completion criteria:** all MVP capabilities pass on both targets; no console hydration warnings; all required checks pass; runtime limitations are documented. Desktop installers and macOS/Linux certification are not implied by Windows acceptance.

**Dependencies:** milestones 3–5; extends the earlier smoke checks.

### Milestone 7 — Post-MVP learning experiments

**Goal:** add one purposeful Dioxus exercise at a time after the shared application is stable.

**Tasks:** first build a command palette opened by Ctrl/Cmd+K, with project/issue navigation and creation actions; add theme toggling only with actual theme support. Then consider drag and drop, realtime, a native integration, and finally offline experimentation as separate scoped changes.

**Dioxus concepts:** global keyboard handling, focus restoration, shared UI context, composition, conditional rendering, streams, platform APIs.

**Completion criteria:** each experiment has its own acceptance checks and plan update; command palette supports typing, arrow keys, Enter, Escape, sensible shortcut handling, and focus return. Later experiments must preserve all MVP flows.

**Dependencies:** milestone 6; offline work additionally requires an explicit storage/sync design review.

## 12. Testing and validation

Use a small set of meaningful tests rather than broad scaffolding or snapshot-heavy UI tests.

| Level | Checks |
| --- | --- |
| Pure unit tests | Name/title validation boundaries including Unicode and whitespace; status serialization; update presence semantics; search/status combinations |
| SQLite integration | Fresh/repeated migrations; FK and status constraints; isolated project lists; missing IDs; partial updates; created/updated timestamp ownership; persistence after reopen |
| Server-function integration | Valid and invalid requests, error/status serialization, request-context availability, missing entities, and create/read/update round trip through generated endpoints |
| Manual browser/Desktop acceptance | End-to-end workflow, loading/failure/retry, direct routes/history, keyboard operation, draft preservation, persistence and backend outage |

API tests must include transport behavior, not only direct SQL calls, because a function can work locally while its generated route or client serialization is broken. Keep tests local with a temporary database and an isolated local test server. Add browser automation only if repeated regressions justify its setup cost.

### Planned commands

The formatting, feature-check, Clippy, and test commands below are now automated by `scripts/check.ps1` and have passed. Toolchain setup is complete and the Web DX build has succeeded. Desktop runtime and release verification remain pending; see [done.md](done.md) for the precise evidence and limits.

```powershell
# Inspect the installed toolchain and target.
rustc --version
cargo --version
dx --version
rustup target list --installed

# One-time setup after Rust is available.
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked

# Formatting, feature separation, and linting.
cargo fmt --all -- --check
cargo check --locked --no-default-features --features web --target wasm32-unknown-unknown
cargo check --locked --no-default-features --features desktop
cargo check --locked --no-default-features --features server
cargo clippy --locked --no-default-features --features web --target wasm32-unknown-unknown -- -D warnings
cargo clippy --locked --no-default-features --features desktop -- -D warnings
cargo clippy --locked --no-default-features --features server --all-targets -- -D warnings

# Unit and integration tests, with server-only database support.
cargo test --locked --no-default-features --features server

# Run each development target separately; stop one before starting the next.
dx serve --web
dx serve --desktop

# Milestone 6 release verification.
dx build --web --release
dx build --desktop --release
```

Use `cargo fmt --all` to apply formatting before the check. Initial dependency resolution creates `Cargo.lock`; `--locked` applies after that file exists. `cargo check` verifies Rust compilation but does not verify DX asset processing, endpoint registration, or runtime behavior; the DX runs and acceptance checks remain necessary. The exact release-server launch path and desktop backend configuration command will be documented from the generated artifacts in milestone 6. Do not claim a release build passed merely because `cargo check` did.

### MVP acceptance script

1. Start with a fresh temporary database and load Projects: a useful empty state appears.
2. Create two projects, including one with non-ASCII text; verify list/sidebar and direct detail navigation.
3. Create several issues across the two projects; verify each board contains only its own issues.
4. Edit title/description, change statuses, and reopen a Done issue; verify board and detail after navigation.
5. Combine search and status filters, produce no results, then clear the filters.
6. Submit invalid input and stop the backend during a read/save; verify feedback, retry, and retained draft text.
7. Restart backend and client using the same database; verify all successful writes remain.
8. Repeat using desktop; compare data through explicit refresh with the web client connected to the same backend.

## 13. Risks, decisions still to verify, and expansion

| Question or risk | Current decision and resolution point |
| --- | --- |
| No usable Rust/DX toolchain found in this session | Milestone 0 installs/verifies the prerequisites and records exact versions; planning is not build verification. |
| Fast-moving framework and mixed patch-level documentation | Start at Dioxus/DX 0.7.10; compile a small example and inspect resolved crate APIs before product work. Update this plan for any necessary change. |
| SSR database context and boundary retry details | Milestone 0 validates request context in a minimal example; milestone 3 verifies real database reads, sanitized errors, hydration, and retry. |
| SQLx feature flags, compiler compatibility, bundled SQLite | SQLx 0.9.0 is the candidate, not an installed dependency. Resolve and lock it in milestone 2; record actual SQLite version and document any compatibility-driven alternative. |
| Desktop backend lifecycle expectations | MVP assumes a separately running local server. Standalone/offline packaging is deferred and must be planned explicitly if requirements change. |
| Windows tooling, WebView2, OneDrive path | Test actual workspace path and native startup early; keep runtime DB in local application data. |
| Multiple open windows can show stale data | Manual refresh and navigation are sufficient for MVP; no hidden claim of realtime consistency. |
| Dataset can outgrow client filtering | Start with complete per-project lists; measure before adding pagination, indexed search, or server-side filtering. |

Post-MVP expansion candidates, each requiring an explicit plan update:

- **Command palette:** first shared UI-state exercise, with accessible keyboard/focus behavior.
- **Drag and drop:** reuse `update_issue`; keep keyboard status controls and add rollback/error feedback if optimistic movement is introduced.
- **Realtime:** choose SSE for simple one-way invalidation or WebSockets only when bidirectional communication is needed; specify reconnect, stale-data, and subscription cleanup behavior.
- **Desktop functionality:** choose one concrete native feature, such as a shortcut, filesystem export, or notification, after parity is proven.
- **Offline/local-first:** decide database ownership, ID changes, queued operations, conflict handling, and sync before coding. A desktop WebView plus SQLite on a server does not provide offline mode.
- **Small product extensions:** consider project editing or deletion only with a real need and a defined ownership/deletion policy.

### Complexity review and working agreement

This plan intentionally stops at two entities, three statuses, one package, one database, seven server functions, a typed route enum, and Dioxus's own state primitives. It adds no generic repository/service hierarchy, global entity cache, event bus, authentication system, synchronization engine, or cross-platform abstraction layer.

Use idiomatic Rust and Dioxus; keep changes small enough to review. Add directories and dependencies when their responsibilities become real. Run checks appropriate to each change. Preserve the MVP/non-MVP boundary, and explain material deviations by updating this document. The next work after this planning task is the remaining milestone 0 setup, not unplanned product implementation.
