# Milestone 2v1 — 인증과 소유권 (방식 B)

작성일: 2026-09-17

## 요약

Web과 Desktop 어디서든 같은 계정으로 자신의 데이터에만 접근하도록 앱 내 인증을 구현했다. Milestone 2의 소유자 기준 스키마 위에 다음을 추가했다.

- 세션 테이블, Argon2id 비밀번호 해시, 로그인 시도 제한
- Web HttpOnly 쿠키 세션, Desktop Bearer 토큰(Windows 자격 증명 관리자 저장)
- 최초 계정 생성, 로그인·로그아웃 화면, 모든 경로 앞의 로그인 게이트
- Host/Origin 요청 보호, 보안 헤더, 요청 본문 크기 제한
- 서버 시작 시 DB 열기·마이그레이션

계획서에는 Milestone 2 바로 다음의 **Milestone 2v1**로 추가했다(번호 3–7은 유지). 이제 Milestone 3–4의 제품 서버 함수는 처음부터 `require_user`로 받은 소유자를 사용해 작성한다.

화면의 프로젝트·이슈는 아직 Milestone 1 예시 데이터다. 실제 데이터 연결은 Milestone 3–4에서 진행한다.

## 구현 내용

### 스키마 (`migrations/0002_sessions.sql`)

| 컬럼 | 내용 |
| --- | --- |
| `token_hash BLOB UNIQUE` | 64자리 hex 랜덤 토큰(32바이트)의 SHA-256. **원문 토큰은 저장하지 않는다** |
| `user_id` | `users(id) ON DELETE CASCADE` |
| `client` | `web` 또는 `desktop` |
| `created_at`, `expires_at` | 30일 고정 만료(자동 연장 없음) |

DB 사본이 유출되어도 저장된 값으로는 로그인 상태를 가장할 수 없다.

### 서버 (`src/server/`)

| 파일 | 역할 |
| --- | --- |
| `config.rs` | `RIVET_PUBLIC_ORIGIN`, `RIVET_ALLOW_REGISTRATION` 해석. 허용 Host·Origin 판단, 쿠키 `Secure` 여부 |
| `guard.rs` | 허용되지 않은 Host는 421, 다른 Origin의 쓰기 요청은 403. 모든 응답에 `nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: same-origin` |
| `auth.rs` | Argon2id 해시·검증, 로그인 제한, 쿠키/Bearer 토큰 추출, 쿠키 생성 |
| `sessions.rs` | 토큰 생성·해시, 세션 생성·조회·삭제, 만료 세션 정리 |
| `users.rs` | 가입 정책(`Open`/`FirstUserOnly`)을 한 SQL 문장으로 적용 |
| `mod.rs` | `AppState`(pool·config·auth), `from_env`, 계층 적용 순서(guard → body limit → extension) |

**비밀번호**

- Argon2id 기본 파라미터로 해시하며, 해시 작업은 `spawn_blocking`에서 실행한다.
- 동시 해시 작업은 2개로 제한한다. 1회에 약 19 MiB를 쓰므로 대량 요청으로 메모리·CPU를 소진시키는 공격을 줄인다.
- 없는 이메일도 더미 해시로 검증해 응답 시간으로 계정 존재 여부를 알 수 없게 했다.
- 비밀번호는 12–128자다. 상한은 해시 작업량을 제한하기 위한 것이다.

**로그인 제한:** 정규화된 이메일마다 15분에 5회 실패하면 429를 반환한다. 성공하면 기록을 지운다. 기록은 메모리에만 있어 서버를 재시작하면 초기화된다. 추적 중인 키가 1024개를 넘으면 만료된 기록을 정리한다.

**가입 정책**

- 기본값에서는 계정이 하나도 없을 때만, 그리고 **직접 들어온 로컬 요청**에서만 계정을 만들 수 있다. 로컬 요청은 Host가 loopback이고 `Forwarded`/`X-Forwarded-*`/`X-Real-IP` 헤더가 없는 요청이다.
- `RIVET_ALLOW_REGISTRATION=true`이면 누구나 가입할 수 있다.
- 두 사용자가 동시에 최초 가입을 시도해도 `INSERT … SELECT … WHERE ? OR NOT EXISTS (SELECT 1 FROM users)` 한 문장으로 처리하므로 한 명만 성공한다.

**전송 방식**

- **Web:** `rivet_session` 쿠키(`HttpOnly; SameSite=Lax; Path=/; Max-Age=30일`, HTTPS 공개 origin이면 `Secure`). 응답 본문에는 토큰을 넣지 않는다.
- **Desktop:** 응답 본문으로 토큰을 받고 `Authorization: Bearer`로 보낸다. 쿠키는 설정하지 않는다.
- `Authorization` 헤더가 있으면 형식이 잘못되었더라도 쿠키로 대체하지 않는다. 토큰은 정확한 64자리 소문자 hex만 받아들여 DB 조회 전에 걸러낸다.

### API (`src/api/`)

| 함수 | 경로 | 설명 |
| --- | --- | --- |
| `auth_status` | GET `/api/auth/status` | 현재 사용자와 이 요청에서의 가입 가능 여부. SSR에서도 요청 쿠키로 동작 |
| `register` | POST `/api/auth/register` | 가입 후 바로 로그인 |
| `login` | POST `/api/auth/login` | 로그인 |
| `logout` | POST `/api/auth/logout` | 현재 세션 삭제, 쿠키 제거. 세션이 없어도 성공 |
| `require_user` | (서버 내부) | 이후 모든 소유자 기준 서버 함수의 시작점. 없으면 401 |

- `ApiError`에 `Unauthorized`(401), `InvalidCredentials`(401), `RegistrationClosed`(403), `RateLimited`(429), `NotFound`(404), `Internal`(500)을 추가했다.
- `Validation`은 `ValidationError { field, message }`를 담는다.
- 내부 오류는 작업 이름과 함께 서버 로그에만 남기고, 클라이언트에는 일반 메시지만 보낸다.
- `Credentials`와 `SignedIn`의 `Debug` 출력은 비밀번호·토큰을 가린다.
- `/api/health`는 로그인 전 연결 확인용으로 공개 상태를 유지한다. `/api/echo`는 로그인이 필요하도록 바꿔 인증 요구 동작을 확인하는 데 쓴다.

### 클라이언트

- **`AppShell`:** `ErrorBoundary` → `SuspenseBoundary` → `SessionGate` 순서로 감쌌다. `auth_status` 로더 결과에 따라 로그인 화면과 작업 공간 중 하나를 보여 준다. 별도 `/login` 경로가 없어 어떤 URL로 들어와도 로그인 후 그 페이지가 바로 열린다. 서버에 연결하지 못하면 "Rivet is unavailable"과 Retry를 보여 준다.
- **`AuthPage`:** 가입 가능하면 계정 생성(비밀번호 확인 포함)을, 아니면 로그인 폼을 보여 준다. 클라이언트에서 모델의 이메일·비밀번호 규칙으로 먼저 검사하고, 처리 중에는 입력을 잠근다. 결과가 오면 비밀번호 입력을 지운다.
- **`AccountMenu`:** 사이드바에 이메일과 Sign out을 표시한다. Desktop은 서버에 연결하지 못해도 로컬 토큰을 지운다.
- **Desktop (`platform/desktop.rs`):**
  - `keyring` 4.2로 서버 URL별 자격 증명(서비스 `Rivet`)에 토큰을 저장하고, 시작 시 복원해 `set_request_headers`로 모든 요청에 붙인다.
  - `RIVET_SERVER_URL`은 loopback이 아니면 HTTPS만 허용한다.
- **`main.rs`:** `AppState::from_env()`로 설정 검사와 DB 마이그레이션을 끝낸 뒤 서비스를 시작한다. 실패하면 요청을 받지 않는다.
- 개발 중 인증 속도를 위해 `argon2`, `blake2`는 dev 프로필에서도 `opt-level = 3`으로 빌드한다.

## 의존성

| 크레이트 | 사용 범위 |
| --- | --- |
| `argon2` 0.6, `sha2` 0.10, `getrandom` 0.3 | `server` feature 전용 |
| `keyring` 4.2 | `desktop` feature 전용 |
| `tokio`에 `sync` 기능 추가 | 해시 동시성 제한용 `Semaphore` |

## 검증 결과

### 자동 검사

`scripts/check.ps1`을 프록시 헤더 수정까지 반영한 최종 코드로 실행해 **모두 통과**했다.

| 검사 | 결과 |
| --- | --- |
| rustfmt / Web(wasm32)·Desktop·Server check / Clippy `-D warnings` | 통과 |
| 단위 테스트 14개 | 모델 8개, 쿠키·Bearer 추출, 쿠키 속성, 로그인 제한, Host/Origin 설정 2개, 토큰 형식 |
| SQLite 통합 테스트 9개 | Milestone 2 항목에 더해: 가입 정책(최초 1명/중복/닫힘/개방), 세션 해시 저장·만료·삭제 |
| route 테스트 2개 | 통과 |
| transport 테스트 9개 | 아래 목록 |

transport 테스트는 실제 서버 함수와 운영용 계층(`with_layers`)을 함께 거친다.

1. health 공개, 보안 헤더 3종
2. 모르는 Host와 Host 없는 요청은 421
3. 외부 Origin의 로그인 POST는 403이고 쿠키 없음. loopback Origin은 성공
4. 세션 없음·위조 토큰 쿠키로 보호된 API 호출 시 401
5. Web 최초 가입: HttpOnly·SameSite 쿠키, 본문에 토큰 없음, 쿠키로 상태·echo(한국어)·검증 오류(`field: input`), 로그아웃 후 401
6. 약한 비밀번호·잘못된 이메일은 필드별 400이며 계정이 생기지 않음
7. 공개 Host 가입 403, **loopback Host라도 `X-Forwarded-For`가 있으면 가입 불가**, 로컬 최초 가입 성공(HTTPS 공개 origin이므로 `Secure` 쿠키), 두 번째 가입 403
8. Desktop 로그인: 대소문자가 다른 이메일로도 로그인, 토큰 반환·쿠키 없음, Bearer로 상태 조회, 로그아웃 후 같은 토큰은 401
9. 없는 계정 로그인 401, 같은 계정 5회 실패 후 올바른 비밀번호도 429

### Web 실제 동작 (`dx serve --web`, 스크래치 경로의 임시 DB)

| 확인 항목 | 결과 |
| --- | --- |
| 새 DB로 시작 시 마이그레이션 0001·0002 적용, 인증 API 4개 등록 | 로그 확인 |
| `/projects/2` 직접 접속 | 계정 생성 화면 표시 |
| 비밀번호 확인 불일치 | "The passwords do not match." |
| 계정 생성 | 요청했던 `/projects/2` 보드로 바로 이동, 사이드바에 이메일·Sign out |
| `document.cookie` | 빈 문자열 (HttpOnly 확인) |
| `/dev/connection` 새로고침(SSR) | 로그인 유지, "Rivet received: 로그인 확인" |
| 프로젝트 폼 빈 제출 | 공통 모델 메시지 "Enter a project name of 1–100 characters.", "3 sample projects" 계산값 표시 |
| Sign out | 로그인 화면, 가입이 닫혀 "Create an account" 링크 없음 |
| 틀린 비밀번호 | "Email or password is incorrect.", 이메일 유지·비밀번호 비움 |
| 올바른 비밀번호 | `/projects` 작업 공간 복귀 |
| 외부 Origin 로그인 POST (curl) | 403 |
| 세션 없는 echo (curl) | 401 `Unauthorized` |
| 페이지 응답 보안 헤더 | 3종 확인 |
| 375px 로그인 화면 | 가로 스크롤 없음 |
| 이전 DB의 쿠키로 새 DB 서버 접속 | 로그인 화면 (무효 세션 거부) |
| DX 프록시 경유 최초 가입 | 가능 (DX 개발 프록시는 forwarded 헤더를 추가하지 않음) |
| `X-Forwarded-For`가 있는 상태 조회 | `registration_open: false` |

브라우저 콘솔에는 로그아웃 후 상태 확인과 틀린 비밀번호 요청의 401 리소스 오류만 있었으며, 이는 예상된 응답이다.

## 코드 리뷰 및 수정

- **프록시 Host 재작성 문제 발견 및 수정:** 실제 개발 서버에서 `Host: attacker.example`로 요청했는데 200이 반환되었다. DX 개발 프록시가 Host를 내부 loopback 주소로 바꿔 전달하기 때문이다. 운영 환경의 리버스 프록시도 같은 방식으로 설정될 수 있다. 그 경우 "loopback Host만 최초 가입 허용" 규칙이 무력화되어, 막 공개된 서버의 첫 계정을 외부에서 차지할 수 있다. 그래서 프록시 전달 헤더가 하나라도 있으면 최초 가입을 막도록 수정하고 회귀 테스트를 추가했다. README와 계획서에 "프록시는 원래 Host를 전달해야 하고, 공개 전에 로컬에서 계정을 먼저 만들 것"을 명시했다.
- **Host 검사의 한계:** 위와 같은 이유로 **DX 개발 프록시 뒤에서는 Host 거부(421)를 실제 서버로 확인하지 못했다.** 앱 서버 내부 포트에 직접 요청해 보려 했지만 연결 방식이 달라 응답을 받지 못했다. 이 동작은 운영용 계층을 그대로 쓰는 transport 테스트로만 검증했다.
- **Enter 제출:** 이번 브라우저 도구에서는 Enter 키가 폼 제출을 일으키지 않았다. 변경하지 않은 Milestone 0 연결 폼에서도 똑같이 동작하지 않았으므로 도구의 한계로 판단했다. 폼은 제출 버튼 클릭으로 검증했다. Milestone 1에서는 다른 브라우저 환경으로 Enter 제출을 확인한 기록이 있다.
- **이메일 입력:** `type="email"`의 브라우저 기본 검증은 비ASCII 로컬 파트(예: `리벳@…`)를 막는다. 서버 규칙은 허용하지만 Web 폼에서는 입력할 수 없다. 필요하면 이후 `type="text"`와 `inputmode="email"`로 바꾸는 것을 검토한다.
- **Milestone 2 함수 변경:** `create_user`는 이제 가입 정책을 인자로 받고 `CreateUser::{Created, EmailTaken, Closed}`를 반환한다. [done2.md](done2.md)에 적힌 "`None` 반환" 설명은 Milestone 2 당시 기준이다.
- **범위 밖으로 남긴 것:** 비밀번호 변경·이메일 재설정, 다른 기기 세션 목록·전체 로그아웃, IP 기준 시도 제한, CSP 헤더, 세션 토큰 교체.

## 미확인 항목

- **Desktop 실제 동작은 확인하지 못했다.** Desktop 빌드는 check·Clippy만 통과했다. 다음 항목은 실제 Windows 창에서 확인해야 한다: 로그인 후 자격 증명 관리자 저장, 재시작 후 로그인 유지, 로그아웃 후 토큰 삭제, `RIVET_SERVER_URL` HTTPS 강제. 이전 마일스톤에서 남은 Desktop echo·복구 확인도 함께 필요하다.
- 실제 리버스 프록시(Caddy 등)와 HTTPS 공개 origin 배포는 테스트하지 않았다. 설정 규칙만 단위·transport 테스트로 검증했다.
- DX release 빌드는 실행하지 않았다.

## 다음 단계

Milestone 3: `require_user`를 사용하는 프로젝트 서버 함수와 로더·액션을 구현하고, 사이드바·프로젝트 목록의 예시 데이터를 실제 데이터로 교체한다. 그 전에 위 Desktop 인증 확인을 끝내는 것을 권장한다.
