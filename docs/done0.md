# Rivet Milestone 0 작업 내역 및 코드 리뷰

작성일: 2026-09-17

## 1. 현재 상태

`planning.md`의 **Milestone 0: 저장소·개발 환경·아키텍처 기반 구성**을 진행했다.
Rust/Dioxus 개발 환경, 최소 Fullstack 애플리케이션, 플랫폼별 빌드 설정과 통합 테스트를 구성했다.
포맷 검사, Web/Desktop/Server 컴파일 및 Clippy 검사, 서버 함수 통합 테스트 3개가 통과했다.

**Milestone 0 전체가 완료된 것은 아니다.** Web DX 빌드 성공 로그는 확인했지만,
실제 브라우저의 hydration·폼 동작과 Desktop 창에서의 서버 통신은 아직 검증하지 못했다.
프로젝트·이슈 관리 기능과 SQLite는 구현하지 않았다.

이 문서는 Milestone 0의 작업 결과와 검토 기록이다. 이후에도 해당 마일스톤 번호에 맞춰
`docs/done{n}.md`에 작업 내역, 검증 결과, 코드 리뷰와 남은 작업을 한국어로 기록한다.
같은 마일스톤의 후속 작업은 해당 문서에 누적한다. 개발 방향과 범위의 기준은 [planning.md](planning.md)다.

## 2. 완료한 작업

### 개발 환경 구성

- 기존 Visual Studio 2022 C++ 도구와 WebView2 런타임을 확인했다.
- Rust 1.98.1, rustfmt, Clippy, `wasm32-unknown-unknown` 타깃을 설치했다.
- `rust-toolchain.toml`에 Rust 버전과 필요한 구성 요소를 고정했다.
- 공식 Dioxus CLI 0.7.10 실행 파일을 `.tools/bin`에 내려받고 배포된 SHA-256과 대조했다.
- Dioxus 의존성을 `=0.7.10`으로 고정하고 `Cargo.lock`을 생성했다.
- DX의 Bare-Bones 템플릿을 별도 임시 디렉터리에서 확인했다. 템플릿의 0.7.1 설정을 그대로 복사하지 않고 계획에 맞는 버전과 Fullstack 기능을 적용했다.

Rust는 사용자 기본 설치 경로를 사용하며, `.tools/`는 Git에서 제외했다.
시스템의 영구 PATH는 변경하지 않았다. 현재 PowerShell에서 도구를 사용하는 방법은 README에 기록했다.

### 애플리케이션 기반 구성

| 파일 | 구현한 내용 |
| --- | --- |
| [Cargo.toml](../Cargo.toml) | 단일 패키지, 공통 의존성, `web`·`desktop`·`server` feature 분리 |
| [rust-toolchain.toml](../rust-toolchain.toml) | Rust 버전, formatter/linter, WASM 타깃 지정 |
| [Dioxus.toml](../Dioxus.toml) | 애플리케이션 이름과 Web 문서 제목 |
| [src/main.rs](../src/main.rs) | 서버에서는 `dioxus::serve`, 클라이언트에서는 `dioxus::launch` 실행 |
| [src/lib.rs](../src/lib.rs) | 공통 모듈 공개 및 서버·Desktop 모듈의 조건부 컴파일 |
| [src/app.rs](../src/app.rs) | 연결 상태 조회, 메시지 전송 폼, 로딩·오류·재시도 UI |
| [src/api/mod.rs](../src/api/mod.rs) | 임시 서버 함수와 공유 응답 타입 |
| [src/api/error.rs](../src/api/error.rs) | 직렬화 가능한 오류 타입과 HTTP 상태 코드 매핑 |
| [src/server/mod.rs](../src/server/mod.rs) | 실제 앱 라우터와 `Extension<AppState>` 제공 |
| [src/platform/desktop.rs](../src/platform/desktop.rs) | `RIVET_SERVER_URL`을 통한 Desktop 백엔드 주소 설정 |
| [assets/main.css](../assets/main.css) | 공통 스타일, 반응형 제목, 키보드 포커스 표시 |

현재 화면은 개발 환경을 확인하기 위한 임시 화면이다. 프로젝트나 이슈 데이터가 있는 것처럼 표시하지 않는다.

### Fullstack 동작 확인용 서버 함수

| 엔드포인트 | 역할 |
| --- | --- |
| `GET /api/health` | 요청의 `AppState`에서 애플리케이션 이름을 읽어 반환 |
| `POST /api/echo` | 메시지를 검증하고 서버에서 받은 내용을 반환 |

메시지는 앞뒤 공백을 제거한 뒤 Unicode scalar 값 기준 1~200자인지 검사한다.
유효하지 않은 입력은 HTTP 400과 구조화된 검증 오류를 반환한다.
이 함수들은 데이터를 저장하지 않으며, 계획에 정의한 MVP 서버 함수 7개를 대체하지 않는다.

UI는 `use_loader`로 서버 상태를 조회하고, `use_action`으로 전송 상태와 결과를 관리한다.
전송 중에는 입력과 제출 버튼을 비활성화한다. 일반적인 전송 실패 시 입력값을 지우지 않으며,
로더의 최초 조회 실패에는 재시도 버튼을 표시한다. 로더 재시도와 초안 보존의 관계는 아래 코드 리뷰에 별도로 기록했다.

### 품질 검사 및 문서

- [scripts/check.ps1](../scripts/check.ps1): 포맷 → 플랫폼별 컴파일·Clippy → 서버 테스트 순서로 실행하며, 실패하면 중단한다.
- [clippy.toml](../clippy.toml): signal borrow를 `await` 경계 너머로 유지하는 문제를 검사하도록 설정했다.
- [tests/transport.rs](../tests/transport.rs): 생성된 서버 함수 라우트를 Axum 요청/응답으로 검증한다.
- [.gitignore](../.gitignore): 빌드 결과, 임시 도구, 로컬 환경 설정, SQLite 및 저널 파일을 제외한다.
- [README.md](../README.md): 준비 사항, 실행 방법, Desktop 서버 주소 설정, 검증 방법을 정리했다.
- [planning.md](planning.md): 최초 조사 기록과 현재 구현 상태를 구분하고 진행 상황을 갱신했다.

## 3. 검증 결과

수정된 테스트를 포함하여 `scripts/check.ps1`을 다시 실행했고, 최종 종료 코드는 **0**이었다.
재실행은 이미 받은 의존성을 이용해 `CARGO_NET_OFFLINE=true`로 진행했다.

| 항목 | 결과 | 확인 범위 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 통과 | Rust 코드 포맷 |
| Web `cargo check` | 통과 | `wasm32-unknown-unknown` 타깃 컴파일 |
| Desktop `cargo check` | 통과 | Windows Desktop feature 컴파일 |
| Server `cargo check` | 통과 | 서버 feature 컴파일 |
| Web/Desktop/Server `cargo clippy` | 통과 | 경고를 오류로 취급한 검사, Server는 `--all-targets` 포함 |
| 서버 함수 통합 테스트 | 3개 통과 | 요청 컨텍스트, Unicode 응답, HTTP 400 오류 구조 |
| `dx serve --web`의 빌드 단계 | 성공 로그 확인 | DX의 Web Fullstack 빌드 및 자산 처리 |
| 브라우저 실제 동작 | 미검증 | hydration, 입력·제출, 오류 표시, 재시도 |
| Desktop 실행 및 실제 통신 | 미검증 | 창 표시, WebView 렌더링, 서버 함수 왕복 호출 |
| Release 빌드 | 미실행 | Milestone 6에서 별도 확인 |

통합 테스트의 정확한 대상은 다음과 같다.

1. `health_endpoint_receives_server_context`: 테스트용 애플리케이션 이름이 요청 컨텍스트에서 응답으로 전달되는지 확인한다.
2. `echo_endpoint_round_trips_unicode`: 한국어 메시지의 JSON 왕복과 앞뒤 공백 제거를 확인한다.
3. `invalid_echo_returns_a_structured_bad_request`: 공백만 있는 입력에 대해 HTTP 400 및 `data.Validation` 메시지를 확인한다.

라이브러리·실행 파일 단위 테스트와 문서 테스트는 현재 0개다. “테스트 3개 통과”는 위 통합 테스트를 뜻한다.
또한 이 테스트는 실제 TCP 연결이나 브라우저 클라이언트를 사용하지 않는다. Axum 라우터에 직접 요청을 전달하므로
생성된 엔드포인트와 서버 측 직렬화는 확인하지만, hydration이나 클라이언트 stub의 동작까지 입증하지 않는다.

DX 로그의 `Build completed successfully ... launching app!` 메시지를 확인했다.
이 메시지만으로 브라우저 화면 표시나 서버 함수 호출 성공을 완료 처리하지 않았다.

### 테스트 구성 문제와 수정

최초 실행에서는 통합 테스트 3개가 모두 실패했다. 테스트에서 실제 애플리케이션 라우터를 생성하면
Dioxus가 테스트 실행 파일 옆의 `public` 디렉터리를 찾는데, 일반 `cargo test`는 DX 자산을 생성하지 않기 때문이다.

테스트 라우터를 `register_server_functions()`와 `FullstackState::headless()` 기반으로 변경하고,
테스트용 `Extension<AppState>`를 주입했다. 이 수정으로 자산 빌드와 독립적으로 서버 함수 계약을 검증할 수 있게 됐다.
수정 후 3개 테스트가 모두 통과했다. 실제 앱 라우터와 SSR 검증은 별도로 남아 있다.

## 4. 코드 리뷰

검토 범위는 현재 작성한 Rust 코드, Cargo feature 설정, 테스트, PowerShell 검사 스크립트다.
아래에서 **수정된 문제**, **현재 구조의 한계**, **실행 검증 공백**을 구분했다.
정적 코드 검토로 예상한 동작을 실제 UI에서 재현한 결과처럼 서술하지 않는다.

### 4.1 [P2 · 미해결] 연결 재시도 시 폼 초안이 초기화될 수 있음

**위치:** [src/app.rs](../src/app.rs)의 `App` 오류 경계와 `ConnectionCheck` 상태.

`message` signal이 로더와 같은 `ConnectionCheck` 컴포넌트에 있다.
상태 재조회가 실패하면 `use_loader(...)?`가 오류 경계로 전달되고, 재시도 버튼은 `attempt`를 증가시켜
경계의 key를 바꾼다. 이때 하위 컴포넌트가 새로 생성되므로 입력 초안도 초기값으로 돌아간다.

- **발생 경로:** 메시지 입력 → 백엔드 중단 → `Check again` → 연결 오류 → `Retry connection`.
- **영향:** 단순 전송 실패에서는 초안이 유지되지만, 연결 로더의 실패·재시도까지 초안 보존이 보장되는 것은 아니다.
- **권고:** 제품 폼을 만들 때 편집 초안을 재생성되는 로더 경계 바깥에서 소유하거나, 상태 표시와 편집 폼의 경계를 분리한다.
- **검증 상태:** 컴포넌트 소유권과 key 변경에 따른 정적 검토 결과이며, 위 경로의 UI 재현은 아직 하지 않았다.

임시 echo 화면의 제한 사항으로 기록하되, 프로젝트·이슈 편집 화면에 같은 구조를 그대로 옮기지 않는다.

### 4.2 [P2 · 검증 공백] 통합 테스트가 실제 앱 라우터와 클라이언트를 검증하지 않음

**위치:** [tests/transport.rs](../tests/transport.rs)의 `api_router`, [src/server/mod.rs](../src/server/mod.rs)의 `router`.

테스트는 독립적인 headless 라우터를 구성한다. 따라서 테스트가 통과해도 실제 앱의 요청 컨텍스트 연결,
SSR 렌더링, 자산 경로 또는 Web/Desktop 클라이언트 요청에 문제가 있을 수 있다.
이는 headless 테스트가 잘못됐다는 뜻이 아니라, 해당 테스트의 책임 범위가 제한적이라는 뜻이다.

- **권고:** 실제 DX 앱에서 초기 조회·메시지 전송·검증 오류·백엔드 중단 후 재시도를 Web과 Desktop 각각 확인한다.
- **완료 기준:** 각 플랫폼에서 서버 함수 왕복 호출을 관찰하고, 브라우저 hydration 오류 여부까지 기록한다.
- **현재 판단:** Milestone 0 완료를 선언하기 전에 해소해야 하는 검증 공백이다.

### 4.3 [P3 · 개선 권고] 여러 서버 함수 오류를 연결 실패 메시지로 표시함

**위치:** [src/api/error.rs](../src/api/error.rs)의 `ApiError::Transport`.

`ServerFnError`에는 네트워크 오류뿐 아니라 응답 역직렬화, 미들웨어, 서버 처리 오류 등도 들어갈 수 있다.
현재는 이들을 모두 “서버에 연결할 수 없다”는 메시지로 표현한다.
내부 오류의 상세 내용을 사용자에게 노출하지 않는 장점은 있지만, 실제 원인과 안내가 일치하지 않을 수 있다.

- **권고:** 제품 API 도입 시 연결 실패와 내부 처리 실패를 구분하고, 서버 내부 정보는 로그에 남긴다.
- **범위:** 현재 확인된 검증 오류는 별도 `Validation` 타입으로 HTTP 400에 매핑된다. 그 동작은 통합 테스트가 확인한다.

### 4.4 [수정 완료] 테스트가 DX 빌드 자산에 의존하던 문제

**위치:** [tests/transport.rs](../tests/transport.rs).

일반 `cargo test`에서 존재하지 않는 `public` 디렉터리를 읽으려던 구성을 headless 라우터로 변경했다.
수정 후 테스트 3개가 통과했으며, 실제 앱 라우터의 런타임 검증은 별도 항목으로 남겼다.

### 유지할 만한 설계

- 서버 함수 선언은 클라이언트에도 남기고, 서버 구현과 import는 feature로 분리했다.
- 전송 상태는 Dioxus의 `use_action`, 서버 조회는 `use_loader`가 소유한다. 별도의 전역 데이터 캐시는 없다.
- 테스트에 다른 `AppState.application` 값을 주입해 응답이 상수가 아니라 요청 컨텍스트에서 나온다는 점을 확인한다.
- 입력 검증을 서버에서 수행하고, 구조화된 오류의 HTTP 상태 코드와 내용을 테스트한다.
- `value.leak()`은 Dioxus가 요구하는 정적 URL을 시작 시 한 번 보관하기 위한 것이다. 반복 요청마다 할당하는 구조는 아니며, 이 용도로 한정한다.
- 현재 규모에 불필요한 repository trait, service 계층, 멀티 크레이트 workspace를 추가하지 않았다.

## 5. 아직 하지 않은 작업

- 실제 브라우저·Desktop UI 검증과 연결 복구 확인.
- 제품용 앱 레이아웃, Sidebar, typed routes, 프로젝트·이슈 컴포넌트.
- Project/Issue 도메인 모델과 SQLite 마이그레이션·영속성.
- 프로젝트 생성·조회, 이슈 생성·편집·상태 변경.
- 검색과 상태 필터.
- Release 빌드 및 배포·패키징 검증.

Git 커밋, 원격 저장소 push, 서비스 배포도 수행하지 않았다.

## 6. 다음 진행 순서

1. Web 앱에서 최초 조회, 메시지 전송, 빈 입력 오류, 재조회·재시도를 확인한다.
2. Desktop 앱을 실행하고 동일한 서버 함수 왕복 호출 및 서버 URL 설정을 확인한다.
3. 코드 리뷰의 초안 보존 문제를 제품 폼의 상태 경계 설계에 반영한다.
4. 확인한 실행 결과를 이 문서와 `planning.md`에 기록한 뒤 Milestone 0을 완료 처리한다.
5. Milestone 1의 정적 앱 셸과 typed routes를 구현한다.

새 작업을 마칠 때는 실행한 검사와 결과를 함께 추가한다. 계획, 구현, 검증을 같은 의미의 “완료”로 취급하지 않는다.


## Milestone 1 후속 확인

정적 셸과 typed routes 구현 및 최신 실행 검증은 [done1.md](done1.md)에 기록했다. 연결 점검은 /dev/connection으로 옮겼고 폼 상태를 로더 재시도 경계 밖으로 분리했다. Web의 실제 한국어 서버 왕복과 오류 복구, Desktop 실행 및 health 조회를 확인했다. Desktop 최종 echo 제출은 사용자의 Computer Use 중단으로 확인하지 못했으며, 남은 검증을 통과로 처리하지 않는다.
