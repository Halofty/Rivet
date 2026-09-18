# Rivet

A lightweight project and issue manager for learning Dioxus on web and desktop.
Development follows [docs/planning.md](docs/planning.md), one milestone at a time.
작업 내역, 검증 결과와 코드 리뷰는 마일스톤 번호에 맞춰 `docs/done{n}.md`에 한국어로 기록한다.
Milestone 0의 기록은 [docs/done0.md](docs/done0.md)에서 확인할 수 있다.
Milestone 1의 기록은 [docs/done1.md](docs/done1.md)에서 확인할 수 있다.
Milestone 2(도메인·SQLite)와 2v1(인증·소유권)의 기록은 [docs/done2.md](docs/done2.md), [docs/done2v1.md](docs/done2v1.md)에 있다.

The current application requires **signing in**. The first account is created from
this machine; afterwards the same account works in the browser and the desktop app.
Behind the sign-in gate is still the milestone 1 static workspace: project cards, a
three-status issue board, and local form previews over deterministic sample data.
The persistent domain (SQLite, owner-scoped SQL) exists on the server but is not yet
connected to the pages; that happens in milestones 3–4. The connection check remains
available at `/dev/connection`.

## Tech Stack

| 구분 | 기술 | 용도 |
| --- | --- | --- |
| 언어 | Rust 1.98.1 · Edition 2024 | 클라이언트와 서버의 공통 구현 |
| UI 프레임워크 | Dioxus 0.7.10 · RSX | Web/Desktop 공통 컴포넌트와 반응형 상태 관리 |
| Web | WebAssembly · Dioxus Fullstack | 서버 렌더링(SSR) 및 hydration |
| Desktop | Dioxus Desktop · WebView2 (Windows) | 공통 UI를 사용하는 데스크톱 클라이언트 |
| 서버 | Dioxus Server Functions · Axum · Tokio | 타입을 공유하는 서버 API와 비동기 요청 처리 |
| 직렬화·오류 처리 | Serde · thiserror | 요청/응답 직렬화와 오류 타입 정의 |
| 스타일 | CSS | Web/Desktop 공통 스타일 |
| 빌드·개발 도구 | Cargo · Dioxus CLI 0.7.10 | 의존성 관리, 플랫폼별 빌드 및 개발 서버 |
| 품질 검사 | rustfmt · Clippy · Cargo Test · Tower | 포맷, 정적 분석, 서버 라우트 통합 테스트 |
| 영속성 | SQLite 3.51.3 (bundled) · SQLx 0.9 | 마이그레이션, 소유자 기준 직접 SQL |
| 인증 | Argon2id · SHA-256 세션 토큰 · keyring | 비밀번호 해시, Web HttpOnly 쿠키, Desktop 토큰을 Windows 자격 증명 관리자에 저장 |

Dioxus Router의 typed routes와 공통 layout/Outlet을 사용한다.

## Prerequisites

- Rust **1.98.1**, pinned in `rust-toolchain.toml`, with rustfmt, Clippy, and
  `wasm32-unknown-unknown`.
- Dioxus CLI **0.7.10**, matching the pinned Dioxus dependency.
- On Windows: Visual Studio C++ build tools, Windows SDK, and WebView2.

After installing Rust with rustup, it will honor this repository's toolchain file.
Install DX from the [official 0.7.10 release](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.10)
or compile it with:

```powershell
cargo install dioxus-cli --version 0.7.10 --locked
```

On the initial development machine, Rust was installed in `%USERPROFILE%\.cargo`
and the official DX executable was downloaded to the ignored `.tools/bin`
directory and checked against its published SHA-256. No persistent PATH change
was made. From the repository root, enable those tools in the current PowerShell:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$PWD\.tools\bin;$env:Path"
rustc --version
dx --version
```

## Run

Run one development target at a time. DX starts both the client and backend.

```powershell
dx serve --web --addr 127.0.0.1 --port 8080
```

Or, after stopping the web development command:

```powershell
dx serve --desktop --addr 127.0.0.1 --port 8080
```

Explore the project cards, issue details, and form previews. The **Next chapter**
project demonstrates an empty board. Invalid IDs and unknown paths show a
not-found view. Keyboard users can use Tab/Enter and the **Skip to content** link.

Open **Connection check** in the sidebar (`/dev/connection`). The page should say
**Rivet is connected**. Send a message and check the echoed
response. Submit an empty message to verify a server-side validation error;
correct it and retry. **Check again** reloads the server status. Initial loading
failures offer **Retry connection**.

On first start, open the app, create your account (email and a password of at least
12 characters), and you are signed in. Setup is accepted only from this machine and only
while no account exists.

### Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `RIVET_DATABASE_PATH` | `%LOCALAPPDATA%\Rivet\rivet.sqlite3` | Absolute database path. Keep it outside OneDrive. |
| `RIVET_PUBLIC_ORIGIN` | unset (loopback only) | Public HTTPS origin such as `https://rivet.example.com` when exposed through a reverse proxy. |
| `RIVET_ALLOW_REGISTRATION` | `false` | `true` lets anyone who can reach the server create an account. |
| `RIVET_SERVER_URL` | DX-managed | Desktop client backend. Must be HTTPS unless it is a loopback address. |

For a desktop client connected to a separately running backend, set its URL before
launching the desktop process:

```powershell
$env:RIVET_SERVER_URL = 'http://127.0.0.1:8080'
```

Remove that override with `Remove-Item Env:RIVET_SERVER_URL` when returning to
DX-managed development. The desktop app stores its session in Windows Credential
Manager (service `Rivet`), so it stays signed in across restarts until you sign out.

### Using Rivet away from this machine

Keep the backend bound to loopback and put an HTTPS reverse proxy (for example Caddy)
in front of it on the same host. Set `RIVET_PUBLIC_ORIGIN` to the public origin.
The proxy must **forward the original `Host` header** (Caddy does by default; nginx
needs `proxy_set_header Host $host;`) and should add `X-Forwarded-For`. Create your
account locally before exposing the server. The server rejects unknown host names and
cross-origin browser writes, and marks cookies `Secure` for HTTPS origins. There is no
password reset by email yet.

## Validate

Run the complete compilation/lint/test matrix:

```powershell
./scripts/check.ps1
```

This checks formatting, independently checks and lints `web`, `desktop`, and
`server`, and runs model, SQLite persistence, typed-route, and server transport tests. It stops at the first failure. Use
`cargo fmt --all` to apply formatting. Do not use `--all-features` to validate
mutually separate platform builds. Cargo checks do not replace running the UI.

The transport tests exercise real generated routes through the production Axum
layers: sign-up, cookie and bearer sessions, logout, throttling, Host/Origin guarding,
and typed HTTP errors. Persistence tests use a temporary on-disk SQLite database per
test, including cross-account access attempts. No external service is needed.

## Structure

- `src/app.rs`: shared root, stylesheet, and typed Router.
- `src/routes/`: overview, projects, project board, issue detail, fallback, and connection check.
- `src/components/`: shell, cards, board, forms, and feedback.
- `src/demo.rs`: UI-only deterministic fixtures; no persistence.
- `src/models/`: shared records, inputs, and validation used by every build.
- `src/api/`: shared server-function declarations (including authentication) and transport errors.
- `src/server/`: server-only startup, configuration, request guard, authentication, sessions, and owner-scoped SQL.
- `src/platform/desktop.rs`: desktop backend URL and credential-store session.
- `migrations/`: SQLite schema migrations embedded at build time.
- `assets/main.css`: shared stylesheet.
- `tests/persistence.rs`: SQLite schema, ownership, and session checks.
- `tests/transport.rs`: endpoint and security-layer integration checks.
- `tests/routes.rs`: typed URL round trips and fallback routing.

The planning document records milestone progress and verification evidence.
