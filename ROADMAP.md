# ghtui Roadmap

## 현재 구현 현황

### 탭별 현황

| # | 탭 | API | 뷰 | 주요 기능 |
|---|---|---|---|---|
| 1 | Code | - | dashboard.rs | 레포 소개 + README placeholder (파일 브라우저 미구현) |
| 2 | Issues | ✅ | issue_list, issue_detail | **Phase 1 완료** — 목록(카드UI), 상세(섹션포커스), CRUD, 필터/검색/정렬, 라벨/Assignee/Milestone, 리액션, 타임라인, 핀/잠금/이전 |
| 3 | Pull requests | ✅ | pr_list, pr_detail | 목록, 상세(Conversation/Diff/Checks), 머지, 리뷰 |
| 4 | Actions | ✅ | actions_list, action_detail | 워크플로우 목록, 잡 선택, 로그 뷰어(스크롤), 취소/재실행 |
| 5 | Security | ✅ | security.rs | Dependabot, Code Scanning, Secret Scanning (read-only) |
| 6 | Insights | ✅ | insights.rs | Contributors, Commit Activity, Traffic (read-only) |
| 7 | Settings | ✅ | settings.rs | 일반설정, 브랜치 보호, Collaborators (read-only) |

### 완료된 기반 기능

- Cargo workspace (ghtui, ghtui-api, ghtui-core, ghtui-widgets)
- Elm architecture (Message → update → Command → API → Message)
- GitHub REST API 클라이언트 (인증, LRU 캐시, rate limit, 쓰기 후 캐시 무효화)
- **GitHub GraphQL API 클라이언트** (pinIssue, transferIssue 등 mutation 지원)
- GitHub Primer 기반 테마 (Dark / Light, `t`키 토글)
- Tab / Shift-Tab / 1-7 탭 네비게이션 (서브탭 overflow → 글로벌 탭 이동)
- diff 파서, 마크다운 렌더러 (GitHub Primer 색상, 이미지→링크 표시)
- Notifications API + 뷰, Search API
- 멀티 계정 지원 (gh CLI hosts.yml, gh 2.40+ 멀티계정, `S`키 전환)
- 마우스 지원 (클릭으로 탭/리스트 선택, 스크롤)
- **TextEditor** (커서 추적, 방향키, 단어이동, Undo/Redo, 뷰포트 스크롤)
- **EditorView / InlineEditorView 위젯** (재사용 가능한 에디터 컴포넌트)
- GitHub Actions CI (check, test, clippy, fmt, RUSTFLAGS=-Dwarnings)
- rustfmt + clippy 설정

---

## Phase 1 — Issues 탭 ✅ 완성

**28/28 항목 완료**

### 리스트 기능
- [x] 필터 UI — open/closed 토글 (`s`), author/label/assignee (API 파라미터)
- [x] 정렬 UI — Newest/Updated/Comments 순환 (`o`)
- [x] 이슈 검색 (`/` → 검색어 → Enter, GitHub Search API)
- [x] 페이지네이션 (`n`/`p` 다음/이전)
- [x] 이슈 생성 (모달 `c`, Ctrl+Enter 제출)
- [x] 핀된 이슈 카드 UI (상단 2열 카드, 📌 아이콘, GraphQL 조회)
- [x] 필터 초기화 (`Shift+F`)

### 상세 기능 (섹션 포커스: j/k로 Title→Labels→Assignees→Body→Comments)
- [x] 제목 편집 (`e` on Title — 헤더 인라인, Enter 제출)
- [x] 본문 편집 (`e` on Body — 전체화면 에디터, 라인번호)
- [x] 코멘트 추가/편집/삭제/인용답글 (`c`/`e`/`d`/`r`)
- [x] 라벨 추가/제거 (`l` → 피커 → Space:토글 → s:적용)
- [x] Assignee 추가/제거 (`a` → 피커)
- [x] Milestone 설정 (`m` → 피커)
- [x] 리액션 (`+`/`-` 빠른 👍👎, 이슈/코멘트 모두)
- [x] 타임라인 이벤트 (labeled, assigned, closed, renamed, cross-referenced 등)
- [x] 이슈 닫기/열기 (`x`)
- [x] 이슈 잠금/해제 (`Shift+L`)
- [x] 이슈 핀/해제 (`Shift+P` — GraphQL mutation)
- [x] 이슈 이전 (`Shift+X` — GraphQL transferIssue)
- [x] 이슈 템플릿 (Contents API `.github/ISSUE_TEMPLATE` 조회)
- [x] 브라우저에서 열기 (`o`)
- [x] TextEditor 커서 (방향키, Home/End, Ctrl+←/→, Undo/Redo)

## Phase 2 — Pull Requests 탭 완성

현재 되는 것: 목록, 상세, 머지(merge/squash/rebase), 닫기/열기, 리뷰 제출, diff 보기

- [ ] 필터 UI (author, reviewer, label, milestone)
- [ ] 정렬 UI
- [ ] PR 검색
- [ ] PR 생성 UI (모달 폼)
- [ ] PR 제목/본문 편집
- [ ] 코멘트 추가 UI (모달)
- [ ] 코멘트 편집/삭제
- [ ] Draft 토글
- [ ] Auto-merge 활성화
- [ ] Side-by-side diff 모드
- [ ] 인라인 코멘트 작성
- [ ] Suggested changes 보기/적용
- [ ] 리뷰 스레드 resolve/unresolve
- [ ] Checks/CI 상태 데이터 연동
- [ ] 리뷰어 추가/제거
- [ ] 라벨 추가/제거
- [ ] Assignee 추가/제거
- [ ] Milestone 설정
- [ ] Linked issues
- [ ] 리액션
- [ ] 파일 트리 in diff
- [ ] 파일별 Viewed 체크
- [ ] 타임라인 이벤트
- [ ] 페이지네이션 UI

## Phase 3 — Actions 탭 완성

현재 되는 것: 런 목록, 잡 선택, 로그 보기(스크롤), 취소, 재실행

- [ ] 필터 UI (status, branch, event, actor, workflow)
- [ ] ANSI 컬러 로그 렌더링 완성
- [ ] 잡 스텝별 접기/펼치기
- [ ] 실패한 잡만 재실행
- [ ] Workflow dispatch (수동 트리거 + 입력값)
- [ ] Artifact 목록
- [ ] Artifact 다운로드
- [ ] 워크플로우 파일 보기
- [ ] Environment 승인
- [ ] 런 삭제
- [ ] 실시간 로그 스트리밍

## Phase 4 — Notifications 완성

현재 되는 것: 알림 목록, 개별 읽음 처리

- [ ] 해당 PR/이슈로 이동 (Enter)
- [ ] 필터 (reason, type)
- [ ] 레포별 그룹핑
- [ ] 구독 해제
- [ ] Done 처리
- [ ] 전체 읽음 UI

## Phase 5 — Search 완성 (API 이미 구현됨)

- [ ] Search 뷰 구현
- [ ] 코드 검색 결과 + 하이라이트
- [ ] 이슈/PR 검색 (GitHub 검색 문법)
- [ ] 레포 검색
- [ ] 최근 검색 히스토리

## Phase 6 — Security 탭 나머지

- [x] Dependabot alerts API 연동
- [x] 취약점 목록 (severity별 필터)
- [ ] 취약점 상세 (영향 받는 패키지, 권고사항)
- [x] Code scanning alerts
- [x] Secret scanning alerts
- [ ] Security advisories

## Phase 7 — Insights 탭 나머지

- [x] Contributors API (커밋 수, additions/deletions)
- [x] 커밋 활동 그래프 (ascii chart)
- [x] 트래픽 (clones, views)
- [ ] Code frequency
- [ ] Dependency graph
- [ ] Forks 네트워크

## Phase 8 — Settings 탭 나머지

- [x] Repository API (read-only 우선)
- [x] 일반 설정 (이름, description, visibility)
- [x] 브랜치 보호 규칙 보기
- [x] Collaborators 목록
- [ ] Webhooks 목록
- [ ] Deploy keys

## Phase 9 — Code 탭 (작업량 최대)

- [ ] GitHub Contents API 연동
- [ ] 파일 트리 브라우저 (접기/펼치기)
- [ ] 파일 내용 뷰어 (syntax highlighting)
- [ ] 간단한 파일 편집 (인라인 수정 → commit)
- [ ] 브랜치/태그 전환
- [ ] 커밋 히스토리 (log with graph)
- [ ] 커밋 상세 (diff, 메시지, 메타데이터)
- [ ] README.md 실제 렌더링

## Phase 10 — UX 개선

- [ ] Command palette (Ctrl-P, fuzzy search)
- [ ] 커스텀 키바인딩 설정 (config.toml)
- [x] 마우스 지원 (클릭, 스크롤)
- [ ] 반응형 레이아웃 (좁은 터미널)
- [ ] 이미지 미리보기 (sixel/kitty protocol)

## Phase 11 — 배포 & 에코시스템

- [ ] `cargo install ghtui`
- [ ] Homebrew formula
- [ ] GitHub Releases (macOS/Linux/Windows)
- [x] CI/CD (GitHub Actions: test, lint, release)
- [ ] crates.io 게시
- [ ] CHANGELOG.md 자동 생성

## Phase 12 — 고급 기능

- [x] 멀티 계정 지원 (gh CLI hosts.yml)
- [x] GraphQL API 지원 (pinIssue, transferIssue 등)
- [ ] GitHub Enterprise Server 지원
- [ ] 멀티 레포 대시보드
- [ ] Discussions 탭
- [ ] Gists 뷰
- [ ] Organization 탐색 (팀, 멤버)
- [ ] 오프라인 모드 (캐시 기반)
- [ ] 플러그인 시스템

## 보류 (UI에서 제거됨)

> 사용 빈도가 낮아 UI에서 제거. 수요가 생기면 재추가 예정.

- ~~Projects 탭~~ (GraphQL 필수, 사용 빈도 낮음)
- ~~Wiki 탭~~ (공식 REST API 없음, 사용 빈도 낮음)
