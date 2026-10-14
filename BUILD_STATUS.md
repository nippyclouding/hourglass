# 검증 상태 — 2026-10-14

## 이번 세션: 실제 설치와 Universal 배포 준비

- 권한 확대 후 GitHub DNS·HTTPS 접근 정상. 샌드박스 내 DNS 실패와 구분했습니다.
- `nippyclouding/hourglass`는 공개·빈 저장소이며 기본 브랜치는 main입니다. 기존 소스·릴리스·브랜치 보호 규칙 없음 확인.
- `/Applications/Hourglass.app` 실제 설치 및 실행 성공. ad-hoc 서명 검증 통과.
- 앱의 첫 실행에서 `autostartInitialized = true`, 50분·자동 시작 켜짐 저장 확인.
- `~/Library/LaunchAgents/io.hourglass.timer.plist`의 Label과 RunAtLoad 및 실행 경로 `/Applications/Hourglass.app/Contents/MacOS/hourglass` 확인.
- 실제 로그아웃·로그인은 수행하지 않았으며, 설정에서 끄고 재실행하는 UI 검증은 컴퓨터 사용 권한 대기 중입니다.
- Intel Rust 타깃 설치 후 로컬 Universal macOS 릴리스 빌드 성공. arm64·x86_64 포함, 번들 7.55 MiB.
- Universal 패키징의 lipo 검증 오류를 실제 아키텍처 목록 검사로 수정. ZIP 무결성·SHA-256 확인 완료.
- Homebrew `on_upgrade`를 지원 값 `:signal`로 수정. 업그레이드·재설치 시 LaunchAgent 백업·복원 훅으로 기존 로그인 선택을 보존합니다.
- JavaScript 테스트 16개, Rust 테스트 13개, Rustfmt·Clippy·로컬 설정 검사를 재확인했습니다.
- 로컬 Git에 소스·lockfile·문서·아이콘·워크플로만 스테이징했습니다. node_modules·target·dist·개인 설정·자격 증명은 제외합니다.
- GitHub Actions 기본 권한은 read지만 릴리스·Cask 워크플로는 필요한 contents: write를 선언합니다. 보호 규칙이 추가되면 정상 PR/리뷰로 반영해야 합니다.
- GitHub 서명 Secret과 로컬 Developer ID 인증서가 없습니다. 공증·Developer ID 서명은 미완료입니다.
- 원격 게시·CI 결과·공개 아카이브의 Homebrew 설치 결과는 수행 후 아래 기록을 추가합니다.

아래 기록은 이전 세션의 결과이며 현재 세션 결과와 구분합니다.

## 이전 세션: 앱 설치와 로그인 자동 실행, 오픈소스 배포

- `dist/Hourglass.app`과 더블클릭 설치기 `dist/Install Hourglass.command` 생성.
- Apple Silicon ZIP 및 SHA-256 파일 생성, 번들 ad-hoc 서명 검증 완료. 현재 앱은 약 3.58 MiB.
- 설치기는 `/Applications` 또는 `~/Applications`에 복사하고 실행합니다.
- 설치된 릴리스 앱의 첫 실행에서 공식 플러그인으로 로그인 자동 실행을 등록합니다.
- 이후 사용자가 끈 설정은 재실행에서 유지합니다. 개발·빌드 경로는 macOS 로그인 등록에서 제외합니다.
- 로그인 작업 이름을 `io.hourglass.timer`로 고정했습니다. 기존 개발 버전의 `hourglass.plist`가 있다면 별도의 과거 등록이며 새 이름과 구분됩니다.
- `npm run install:macos`, `npm run install:windows`, `npm run package:macos` 추가.
- JavaScript 테스트 16개, Rust 테스트 13개, Clippy 및 포맷 검사 통과.
- 설치·업데이트·다른 앱 보호는 임시 폴더에서 실제 복사로 검증했으며 자동 실행 항목을 변경하지 않았습니다.
- 실제 기본 설치 시도는 `mkdir: ~/Applications: Operation not permitted`로 실패했습니다. 현재 세션에서 Applications와 로그인 등록을 변경할 수 없어 실제 설치·자동 실행 검증은 미완료입니다.
- 공개 저장소는 사용자가 제공한 `https://github.com/nippyclouding/hourglass`로 설정했습니다.
- 터미널의 GitHub DNS 접근이 실패하여 공개 저장소 상태를 확인하거나 파일을 업로드하지 못했습니다.
- GitHub 태그 릴리스 워크플로는 Universal Mac ZIP과 Windows NSIS를 빌드하고 초안 릴리스를 생성하도록 준비했습니다.
- 릴리스를 공개하면 실제 공개 ZIP의 체크섬으로 Cask를 생성해 기본 브랜치에 업데이트하는 워크플로도 준비했습니다.
- Homebrew 생성기·템플릿 검증 완료. 실제 공개 릴리스, Cask 게시, `brew install`, Universal·Windows 빌드는 아직 실행하지 않았습니다.
- Developer ID와 공증 자격 증명이 없어 현재 앱은 ad-hoc 서명이며 Apple 공증은 미완료입니다.

설치·배포 방법은 [README.ko.md](README.ko.md)와 [RELEASE.md](RELEASE.md)에 정리했습니다.
아래는 앞선 빌드 및 테스트 기록입니다.

## 최신 변경: 재시작과 모래 애니메이션

- 설정창에 Restart timer 추가. 현재 세션의 원래 시간으로 복원하고 바로 실행합니다.
- 모래 양 8단계 × 알갱이 이동 4단계의 프레임을 두 플랫폼용으로 생성했습니다.
- 기존 작업 스레드에서 0.5초마다 애니메이션을 갱신하며 일시정지하면 프레임도 멈춥니다.
- macOS 프레임 교체는 아이콘과 템플릿 플래그를 함께 설정해 메뉴바 테마를 유지합니다.
- Rust 테스트 11개, UI 테스트 12개, Clippy 및 로컬 설정·아이콘 검사가 통과했습니다.
- `npm run build -- --bundles app` 성공. 새 `.app` 번들은 약 3.60 MiB입니다.
- 새 빌드의 UI 검증도 `Computer Use permissions are not granted`로 수행하지 못했습니다.
- 기존 앱이 실행 중이면 Quit으로 종료한 뒤 새 번들을 실행해야 추가 기능이 적용됩니다.

추가 테스트는 10분 경과 후 원래 시간으로 재시작, 이전 종료 시각 폐기, 일시정지 중
재시작, 설정 시간 변경으로 세션 길이가 바뀌지 않음, 완료·대기 상태 재시작 방지,
알갱이 이동과 일시정지 프레임 보존, 미저장 입력·중복 명령 처리를 검사합니다.
아이콘 프레임 미리보기도 확인했습니다. 실제 클릭·Sleep·알림 및 애니메이션 실행 중
CPU·메모리는 수동 검증이 필요합니다.

`package-lock.json`과 `src-tauri/Cargo.lock`을 보존했으며, 아래는 이전 기본 MVP의
검증 기록입니다. 이전의 실행·자원 사용 표본을 새 빌드의 검증 결과로 해석하지 않습니다.

## 이전 기본 MVP 검증

Apple Silicon Mac에서 Rust 검사와 macOS 릴리스 앱 빌드에 성공했습니다.
생성된 앱을 실행하고 프로세스 유지 및 중복 프로세스 방지를 확인했습니다.
컴퓨터 사용 도구의 권한이 없어 메뉴바·Dock·클릭·알림의 실제 UI 동작은 미검증입니다.

| 항목 | 결과 |
| --- | --- |
| 개발 시스템 | macOS 27.0 (26A425), Apple Silicon (`arm64`) |
| Node.js / npm | 26.10.0 / 11.19.1 |
| Xcode Command Line Tools | 설치됨, macOS SDK 27.0 |
| Rust / Cargo | 공식 rustup으로 stable 1.99.0 설치, Rustfmt·Clippy 포함 |
| 네트워크 / 의존성 | 샌드박스 내 DNS 실패, 권한 확대 후 공식 rustup·npm·crates.io 다운로드 성공 |
| `npm install` | 성공, 취약점 0개 보고 |
| 의존성 버전 | 기존 Tauri 2.12.3 / CLI 2.12.2 / 플러그인 고정 버전 유지, 다운그레이드 없음 |
| `npm run check` | 통과 |
| `npm test` | 8개 통과 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | 수행 완료, 이후 `-- --check` 통과 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 7개 통과, 수정 후 재검사 통과 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 설정 로더 반환 타입 복잡성 수정 후 통과 |
| Info.plist | 원본 형식 검사 통과, 생성된 번들에서 `LSUIElement = true` 확인 |
| `npm run build -- --bundles app` | 성공, Apple Silicon용 Mach-O arm64 릴리스 앱 생성 |
| 앱 번들 크기 | Tauri CLI 보고 3.47 MiB |
| `open src-tauri/target/release/bundle/macos/Hourglass.app` | 성공, 이후 Hourglass 프로세스 유지 확인 |
| 중복 실행 | `open -n`으로 두 번째 인스턴스 요청 후 `pgrep -x hourglass` 결과 기존 PID 8057 하나 유지. 아이콘 개수는 UI 미검증 |
| 대기 자원 사용 | 실행 약 2분 뒤 `ps` 표본: CPU 0.0%, RSS 102,416 KiB; 뒤 표본 97,568 KiB. 장기 측정 아님 |
| 실제 UI 자동화 | `cua.getApp(...)`가 `Computer Use permissions are not granted` 반환 |
| 설정 / 자동 시작 | 저장 파일 50분·자동 시작 꺼짐 확인, 테스트로 변경하지 않음. 설정 저장 UI·로그인 실행은 미검증 |
| Sleep 정확성 | 종료 시각 기반 경과 시간·지연 콜백·일시정지 테스트 통과, 실제 OS Sleep은 수행하지 않음 |
| Windows | 조건부 코드 검토 완료, Windows 빌드·실행은 미수행 |
| lockfile | `package-lock.json`, `src-tauri/Cargo.lock` 생성 및 보존 |

## 빌드 산출물과 수정

앱 경로:

```text
<project>/src-tauri/target/release/bundle/macos/Hourglass.app
```

Rustfmt를 적용하고 `src-tauri/src/settings.rs`에 `SettingsStore` 타입 별칭을
추가해 Clippy의 `type_complexity` 오류를 해결했습니다. 컴파일 오류는 발생하지
않았습니다. 기능과 의존성 버전은 유지했습니다.

개발 실행은 프로젝트 루트에서 `npm run dev`입니다.

## 남은 실제 실행 검증

컴퓨터 사용 권한을 활성화한 뒤 [TESTING.md](TESTING.md)의 UI 항목을 확인해야 합니다.
현재 아래 항목은 코드·자동 테스트와 실제 UI 관찰을 구분하여 미검증으로 남깁니다.

- 시작 시 일반 창 없음, 메뉴바 아이콘 하나, Dock 항목 숨김.
- 왼쪽 클릭 시작·일시정지·재개, 남은 시간 표시.
- 오른쪽 클릭 설정창, 저장, 창 밖 클릭, Escape, Quit.
- 1분 완료 알림 한 번, 0.5초 깜빡임, 15초 후 정지.
- 완료 상태 해제 후 다음 클릭으로 새 타이머 시작.
- 앱 재실행 뒤 설정 유지, 중복 실행 뒤 실제 아이콘 개수.
- 실제 Sleep·복귀, 로그인 자동 시작, 밝은/어두운 테마 및 보조 모니터.
- Windows 네이티브 빌드·실행 및 시스템 트레이·알림.

실제 Sleep·로그아웃·재부팅은 수행하지 않았습니다. 시간이나 자동 시작 설정을
변경하지 않았으므로 복원할 테스트 변경은 없습니다. 앱은 실행 상태로 남겨두었습니다.

## 환경 제한과 해결 결과

일반 샌드박스에서는 `rustc` / `cargo`가 없었고, `curl -I`의
`static.rust-lang.org`, `registry.npmjs.org`, `index.crates.io` 조회가
`curl: (6) Could not resolve host`로 실패했습니다. 이번 세션에서는 권한 확대가
승인되어 공식 설치와 빌드에 성공했습니다. 파일 수정·컴파일·프로세스 조회에도
읽기 전용 샌드박스 밖 실행 권한을 사용했습니다.

남은 차단은 UI 도구의 컴퓨터 사용 권한입니다. 해당 권한을 허용한 뒤 실제 앱
클릭 검증을 이어가야 합니다. 권한 차단을 다른 UI 제어 수단으로 우회하지 않았습니다.
서명·공증, Windows 코드 서명 및 GitHub Actions의 실제 성공 여부는 미검증입니다.
