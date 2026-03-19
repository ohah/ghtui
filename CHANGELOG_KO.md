# 변경 이력

[English](CHANGELOG.md)

이 프로젝트의 주요 변경사항을 기록합니다.

## [0.1.0] - 2026-03-19

### 최초 릴리즈

Rust와 ratatui로 구축한 종합 GitHub TUI. GitHub의 주요 기능을 모두 지원합니다.

### 기능

#### 탭
- **Code** — 파일 트리 브라우저, 구문 강조 파일 뷰어, 인라인 편집+커밋, 브랜치/태그 전환, 커밋 히스토리 및 상세
- **Issues** — 목록(카드 UI), 상세(섹션 포커스), 전체 CRUD, 필터/검색/정렬, 라벨/Assignee/Milestone, 리액션, 타임라인, 핀/잠금/이전
- **Pull Requests** — 4탭 상세(Conversation/Commits/Checks/Files changed), 인라인 편집, Approve/Request changes, 파일 트리, diff 리뷰 코멘트, Suggestion, CI 상태, 타임라인, PR 생성, 리뷰어 관리, Draft 토글, Auto-merge, Side-by-side diff
- **Actions** — 필터/검색/페이지네이션, ANSI 컬러 로그, 스텝 접기, 액션 바, Artifact, Workflow dispatch, 환경 승인, 실시간 로그 스트리밍
- **Security** — Dependabot, Code Scanning, Secret Scanning, Advisories (dismiss/resolve 지원)
- **Insights** — Contributors, Commit Activity, Traffic, Code Frequency, Forks, Dependency Graph
- **Settings** — 일반 설정 편집, 브랜치 보호, Collaborators, Webhooks, Deploy Keys, Visibility 토글

#### 추가 뷰
- **Notifications** — reason/type 필터, 레포별 그룹핑, 구독 해제, Done 처리, 전체 읽음
- **Search** — Code/Issues/Repos 검색 (GitHub Search API), Ctrl+K 단축키
- **Discussions** — GraphQL 기반, 카테고리/답변 표시
- **Gists** — 공개/비밀 뱃지, 파일 목록
- **Organizations** — 멤버 목록

#### 핵심 기능
- Elm 아키텍처 (Message → update → Command → API → Message)
- GitHub REST + GraphQL API 클라이언트 (LRU 캐시, rate limiting)
- GitHub Primer 테마 (Dark/Light, `t`키 토글)
- 멀티 계정 지원 (gh CLI hosts.yml, `S`키 전환)
- 마우스 지원 (탭/리스트 클릭, 스크롤)
- TextEditor (커서 추적, 단어 이동, Undo/Redo)
- EditorView / InlineEditorView 재사용 위젯
- Command palette (Ctrl+P)
- 커스텀 키바인딩 (config.toml)
- 마크다운 렌더러 (테이블, 링크, 취소선, 체크박스, 코드블록, 이미지)
- 오프라인 모드 (디스크 캐시, 24시간 TTL, 자동 fallback)
- 이미지 미리보기 (halfblock/sixel/kitty 자동 감지)
- 크로스 플랫폼 gh CLI 설정 감지 (macOS/Linux/Windows)

#### CI/CD
- GitHub Actions CI (check, test, clippy, fmt, MSRV, security audit)
- 멀티 플랫폼 릴리즈 빌드 (macOS/Linux/Windows)
- Dependabot (Cargo 및 GitHub Actions 의존성)
- Homebrew formula

[0.1.0]: https://github.com/ohah/ghtui/releases/tag/v0.1.0
