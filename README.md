# Rivet

A lightweight project and issue manager for learning Dioxus on web and desktop.
Development follows [docs/planning.md](docs/planning.md), one milestone at a time.
작업 내역, 검증 결과와 코드 리뷰는 [docs/done.md](docs/done.md)에 한국어로 기록한다.

The current application is the **milestone 0 connection-check scaffold**. It
exercises server reads, actions, validation, loading, retry, and shared request
context. The echo form does not save data. Project and issue features come later.

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

도입 예정: **SQLite + SQLx**를 Milestone 2에서 영속성 계층에 추가한다.
Dioxus Router는 의존성에 활성화되어 있으며, typed routes는 Milestone 1에서 구현한다.

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

The page should say **Rivet is connected**. Send a message and check the echoed
response. Submit an empty message to verify a server-side validation error;
correct it and retry. **Check again** reloads the server status. Initial loading
failures offer **Retry connection**.

For a desktop client connected to a separately running backend, set its URL
before launching the desktop process:

```powershell
$env:RIVET_SERVER_URL = 'http://127.0.0.1:8080'
```

Remove that override with `Remove-Item Env:RIVET_SERVER_URL` when returning to
DX-managed development. Server functions run on the backend in both platforms;
the desktop client is not an offline application. The unauthenticated development
server should remain bound to loopback.

## Validate

Run the complete compilation/lint/test matrix:

```powershell
./scripts/check.ps1
```

This checks formatting, independently checks and lints `web`, `desktop`, and
`server`, and runs server transport tests. It stops at the first failure. Use
`cargo fmt --all` to apply formatting. Do not use `--all-features` to validate
mutually separate platform builds. Cargo checks do not replace running the UI.

The transport tests exercise real generated routes through the Axum router,
including request-context extraction, Unicode JSON round trips, and typed HTTP
400 errors. They use no external service or database.

## Structure

- `src/app.rs`: shared development page, loader, action, and boundaries.
- `src/api/`: shared server-function declarations and transport errors.
- `src/server/`: server-only initialization and request context.
- `src/platform/desktop.rs`: optional desktop backend URL configuration.
- `assets/main.css`: shared stylesheet.
- `tests/transport.rs`: endpoint integration checks.

SQLite and domain models arrive in milestone 2. Runtime database files will live
in local application data, outside this OneDrive source directory. The planning
document records milestone progress and verification evidence.
