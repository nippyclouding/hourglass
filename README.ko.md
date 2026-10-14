<img src="src-tauri/icons/128x128.png" alt="Hourglass 아이콘" width="64" height="64">

# Hourglass

**클릭 한 번으로 시작하는 타이머.**

macOS 메뉴바와 Windows 시스템 트레이에서 사용하는 미니멀 타이머입니다. 모래시계를 클릭하면 시작하고, 우클릭하면 시간을 설정합니다.

[English](README.md) · [설치](#설치) · [소스 빌드](#소스-빌드) · [기여하기](#기여하기) · [MIT License](LICENSE)

> **초기 MVP:** macOS 실제 설치·로그인 등록, Git 소스 설치와 Homebrew Cask 설치를 검증했습니다. [v0.1.0](https://github.com/nippyclouding/hourglass/releases/tag/v0.1.0)에 Universal macOS 앱과 Windows x64 설치기가 공개되어 있습니다. 네이티브 CI, JavaScript 테스트 16개, Rust 테스트 13개, 포맷과 Clippy가 통과했습니다. 실제 UI·로그인·Sleep 및 Windows 실행은 미검증이며 macOS 앱은 공증 없는 ad-hoc 서명입니다. 자세한 내용은 [검증 상태](BUILD_STATUS.md)를 참고하세요.

## 컨셉

Hourglass의 기본 UI는 메뉴바나 시스템 트레이에 있는 모래시계 하나입니다. 시간을 한 번 설정해두면 별도의 창이나 Start 버튼 없이 클릭 한 번으로 타이머를 시작합니다.

작은 로컬 타이머에 집중합니다. 계정, 서버, DB, 활동 기록, 포모도로 통계는 제공하지 않습니다.

## 설치

### macOS 앱

[GitHub Releases](https://github.com/nippyclouding/hourglass/releases/tag/v0.1.0)에서 Universal macOS ZIP을 다운로드하고 압축을 푼 뒤, **Hourglass.app** 옆의 **Install Hourglass.command**를 더블클릭합니다. 설치기는 앱을 `/Applications` 또는 해당 폴더에 쓸 수 없는 경우 `~/Applications`에 복사하고 실행합니다. 업데이트 전에는 기존 Hourglass를 Quit으로 종료해주세요.

설치된 릴리스 앱의 첫 실행에서 **Launch at startup**을 자동 등록합니다. 이후 Mac에 로그인하면 메뉴바에 Hourglass가 나타나며, 아이콘을 클릭하면 타이머가 시작됩니다. 설정에서 자동 실행을 끄면 이후 실행에서도 그 선택을 유지합니다.

로컬에서는 `npm run package:macos`로 `dist/`에 앱, 더블클릭 설치기, ZIP과 체크섬을 생성할 수 있습니다.

### Homebrew

공개된 Universal macOS 앱은 다음 명령으로 설치할 수 있습니다.

```sh
brew tap nippyclouding/hourglass https://github.com/nippyclouding/hourglass.git
brew install --cask nippyclouding/hourglass/hourglass
open /Applications/Hourglass.app
```

Homebrew 설치에는 Rust나 Node.js가 필요하지 않습니다. 외부 tap에 대한 Homebrew 신뢰 안내를 따라주세요. 공개 릴리스와 서명 설정은 [배포 문서](RELEASE.md)에 정리되어 있습니다. Cask 게시와 실제 공개 아카이브의 Homebrew 설치를 검증했습니다.

### Git 소스에서 설치

[빌드 도구](#필요한-도구)를 준비한 뒤 실행합니다.

```sh
git clone https://github.com/nippyclouding/hourglass.git
cd hourglass
npm ci
npm run install:macos
```

Windows에서는 마지막 명령 대신 `npm run install:windows`를 사용합니다. NSIS 설치 파일을 빌드하고 실행하며, 설치된 앱을 한 번 실행하면 로그인 자동 실행을 등록합니다.

## 사용법

| 동작 | 결과 |
| --- | --- |
| 앱 실행 | 일반 창 없이 모래시계 아이콘 표시 |
| 대기 중 왼쪽 클릭 | 저장한 시간으로 즉시 시작 |
| 실행 중 왼쪽 클릭 | 일시정지 |
| 일시정지 중 왼쪽 클릭 | 재개 |
| 오른쪽 클릭 | 작은 설정창 열기 |
| 설정창에서 **Restart timer** | 현재 타이머를 처음 시작했던 시간으로 되돌려 다시 실행 |
| 타이머 완료 | 시스템 알림 한 번, 0.5초 간격으로 최대 15초간 아이콘 깜빡임 |
| 완료 후 왼쪽 클릭 | 깜빡임과 완료 상태 해제. 다시 클릭하면 새 타이머 시작 |

기본 시간은 **50분**입니다. 설정창에 원하는 분 단위 정수를 입력하고 **Save**를 누르세요. 변경한 시간은 다음 타이머부터 적용되며, 실행 중이거나 일시정지한 타이머의 시간은 유지됩니다. 설정은 앱을 다시 실행해도 유지됩니다.

설정창에는 다음 항목이 있습니다.

- **Time:** 1–525,600분의 정수.
- **Launch at startup:** 로그인 시 Hourglass 실행. 설치된 릴리스 앱의 첫 실행에서 켜집니다.
- **Save:** 설정을 저장하고 창 닫기.
- **Restart timer:** 실행 중이거나 일시정지한 타이머를 원래 시간부터 다시 실행하고 창 닫기. 입력란의 미저장 값은 적용하지 않으며, 진행 중인 타이머가 없으면 비활성화됩니다.
- **Quit:** 앱 종료.

설정창 밖을 클릭하거나 **Escape**를 누르면 저장하지 않고 닫습니다.

실행 중에는 알갱이가 떨어지고 위쪽 모래가 아래로 옮겨가는 아이콘 애니메이션이 표시됩니다. 0.5초마다 갱신하며, 일시정지하면 멈추고 재시작하면 모래가 위쪽으로 돌아갑니다.

## 플랫폼별 동작

| | macOS | Windows |
| --- | --- | --- |
| 아이콘 위치 | 메뉴바 | 시스템 트레이 / 알림 영역 |
| 남은 시간 | 아이콘 옆 문자열과 툴팁 | 아이콘 툴팁 |
| 일반 앱 창 | 없음 | 없음 |
| Dock / 작업 표시줄 | Dock에 표시하지 않음 | 설정창을 작업 표시줄에 표시하지 않음 |
| 로그인 시 실행 | LaunchAgent | Windows 자동 시작 등록 |

Windows에서는 아이콘이 트레이의 숨겨진 아이콘 영역에 들어갈 수 있습니다. 그 영역의 모래시계를 우클릭해도 설정창이 열립니다.

## 소스 빌드

사용할 운영체제에서 빌드하세요. macOS 개발은 Apple Silicon을 우선 대상으로 하며, Windows 빌드는 MSVC 툴체인을 사용합니다.

### 필요한 도구

- **Rust stable 1.90 이상:** [rustup](https://rustup.rs/)으로 설치합니다. 저장소의 `rust-toolchain.toml`은 stable Rust와 Rustfmt, Clippy를 선택합니다.
- **Node.js와 npm:** CI 설정에서는 Node.js 24를 사용합니다. Node는 개발 CLI와 테스트에 사용하며 앱 런타임에 포함하지 않습니다.
- **macOS:** Xcode Command Line Tools. 없다면 `xcode-select --install`을 실행합니다.
- **Windows:** Microsoft C++ Build Tools의 **Desktop development with C++** 워크로드와 Windows SDK, **WebView2 Runtime**. Rust는 MSVC 툴체인을 사용합니다.

플랫폼별 설치 방법은 [Tauri 공식 필수 도구 문서](https://v2.tauri.app/start/prerequisites/)를 참고하세요. 첫 빌드에는 npm과 Cargo 의존성을 다운로드할 인터넷 연결이 필요합니다.

### 개발 실행

저장소를 클론한 뒤 프로젝트 루트 디렉터리에서 실행합니다.

```sh
npm install
npm run dev
```

메뉴바나 시스템 트레이에서 모래시계를 찾아 우클릭하면 설정창이 열립니다. **Quit**으로 종료하거나 터미널에서 **Ctrl+C**로 개발 프로세스를 중지할 수 있습니다.

### macOS 앱 빌드

```sh
npm run build -- --bundles app
open src-tauri/target/release/bundle/macos/Hourglass.app
```

DMG도 생성하려면 `npm run build`를 실행합니다. 앱 번들은 `src-tauri/target/release/bundle/macos/`, DMG는 `src-tauri/target/release/bundle/dmg/`에 생성됩니다.

앱 설치와 로그인 자동 실행 등록에는 `npm run install:macos`를 사용합니다. 설치된 최종 경로에서 한 번만 등록하며, 개발·빌드 경로는 자동 실행에 등록하지 않습니다.

### Windows 설치 파일 빌드

Windows의 PowerShell에서 실행합니다.

```powershell
npm install
npm run build -- --bundles nsis
```

설치 파일은 `src-tauri/target/release/bundle/nsis/`에 생성됩니다. 알림과 로그인 시 실행은 설치된 앱에서 확인하세요.

## 타이머 동작과 제한

타이머 시작 시 종료 시각을 저장하고 매번 **종료 시각 − 현재 시각**으로 남은 시간을 계산합니다. 콜백마다 1초씩 빼는 방식이 아니므로 Sleep이나 콜백 지연에 실제 경과 시간을 반영합니다.

- Sleep 중 시간이 끝나면 컴퓨터가 깨어난 뒤 완료를 처리합니다.
- 일시정지하면 초 미만까지 남은 시간을 보존하고, 재개 시 새 종료 시각을 계산합니다.
- 시스템 시각을 수동으로 바꾸면 남은 시간도 영향을 받습니다.
- 설정은 저장하지만 앱 종료·재시작 후 실행 중이던 타이머를 복원하지는 않습니다.
- 알림은 운영체제의 알림 허용·집중 모드 설정을 따릅니다. Windows는 [Tauri 알림 플러그인](https://v2.tauri.app/plugin/notification/)의 제약에 따라 설치된 앱에서 알림을 확인해야 하며, 개발 실행의 알림 이름·아이콘은 다를 수 있습니다.

Apple Silicon 릴리스 앱 번들은 약 3.58 MiB입니다. 재시작과 애니메이션 추가 전의 짧은 프로세스 표본에서 CPU 0.0%, RSS 약 95–100 MiB를 관찰했습니다. 현재 빌드의 자원 측정을 의미하지는 않으며, 실제 UI 동작과 애니메이션 실행 중 자원 사용은 추가 검증이 필요합니다.

## 구현 구조

**Tauri 2.12.3**, **Rust**, 순수 **HTML/CSS/JavaScript**를 사용합니다. 프론트엔드 프레임워크, 번들러, 개발 서버는 없습니다. npm 개발 의존성은 Tauri CLI 2.12.2 하나이며, 버전은 [Cargo.toml](src-tauri/Cargo.toml)과 [package.json](package.json)에 선언되어 있습니다.

Rust가 타이머, 트레이 이벤트, 알림, 설정을 처리합니다. 설정 WebView는 필요할 때 생성하고 닫으면 해제합니다. 대기·일시정지·완료 후에는 타이머 스레드가 기다립니다. 실행 중에는 미리 생성한 아이콘 프레임을 0.5초마다 바꾸며 남은 시간 표시는 값이 바뀔 때만 갱신합니다. 완료 깜빡임도 같은 스레드를 사용하고 15초로 제한합니다.

공식 Tauri 플러그인으로 로컬 설정 저장, 자동 시작, 알림, 중복 실행 방지를 처리합니다. 플랫폼별 표현 차이는 `platform.rs`에 분리했습니다.

```text
ui/                       작은 설정 화면
src-tauri/src/main.rs      앱 수명 주기와 설정 명령
src-tauri/src/timer.rs     종료 시각 기반 상태 머신과 테스트
src-tauri/src/tray.rs      트레이 이벤트, 표시, 알림, 작업 스레드
src-tauri/src/settings.rs  설정 저장과 자동 시작
src-tauri/src/platform.rs  플랫폼별 표현 차이
src-tauri/Info.plist       macOS Dock 표시 설정
assets/hourglass.svg       모래시계 아이콘 원본
scripts/generate-icons.py  재현 가능한 아이콘 생성기
tests/                    프론트엔드 동작 테스트
```

생성된 아이콘은 저장소에 포함되어 있습니다. 아이콘을 다시 생성할 때만 `python3 scripts/generate-icons.py`를 실행합니다. 생성기는 Python 표준 라이브러리만 사용합니다.

## 기여하기

버그 제보, 작은 수정, macOS/Windows 테스트를 환영합니다. 변경 범위는 클릭 한 번으로 사용하는 타이머와 작은 설정창에 집중해주세요.

버그를 제보할 때는 OS 버전, 앱 버전 또는 커밋, 개발 실행인지 설치된 앱인지, 재현 방법, 기대한 동작과 실제 동작을 함께 알려주세요.

Pull Request를 제출하기 전에 다음을 실행합니다.

```sh
npm run check
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

영향받는 플랫폼에서 빌드하고 [네이티브 테스트 체크리스트](TESTING.md)로 클릭, Sleep/재개, 깜빡임, 알림, 자동 시작을 확인해주세요. 프론트엔드 테스트는 이러한 네이티브 동작을 검증하지 않습니다.

[GitHub Actions 워크플로](.github/workflows/build.yml)는 검사와 macOS `.app`, Windows NSIS 아티팩트 생성을 수행하도록 구성되어 있습니다. 워크플로 파일이 있다는 것이 빌드 성공을 의미하지는 않습니다. 생성된 `package-lock.json`과 `src-tauri/Cargo.lock`이 포함되어 있습니다. 두 파일을 함께 버전 관리해 의존성 해석을 재현할 수 있도록 합니다.

## 라이선스

Hourglass는 [MIT License](LICENSE)로 공개하는 오픈소스 프로젝트입니다.
