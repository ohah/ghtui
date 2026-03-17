# ghtui Roadmap

## 현재 구현 현황

### 탭별 현황

| # | 탭 | API | 뷰 | 주요 기능 |
|---|---|---|---|---|
| 1 | Code | - | dashboard.rs | 레포 소개 + README placeholder (파일 브라우저 미구현) |
| 2 | Issues | ✅ | issue_list, issue_detail | **Phase 1 완료** — 목록(카드UI), 상세(섹션포커스), CRUD, 필터/검색/정렬, 라벨/Assignee/Milestone, 리액션, 타임라인, 핀/잠금/이전 |
| 3 | Pull requests | ✅ | pr_list, pr_detail | 목록(필터/검색/정렬/페이지네이션), 상세(섹션포커스, 인라인편집, 라벨/Assignee 피커, Conversation/Diff/Checks), 머지, 리뷰, 리액션, Checks 실시간 조회, Diff 테마 색상 |
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

### Phase 2-1 완료 (Issues 탭 패리티)
- [x] 필터 UI (state 토글 `s`, author/label/assignee)
- [x] 정렬 UI (`o` 순환: Newest/Updated/Popular/Long-running)
- [x] PR 검색 (`/` → GitHub Search API)
- [x] 페이지네이션 UI (`n`/`p`)
- [x] PR 제목/본문 인라인 편집 (`e` on focused section)
- [x] 코멘트 추가/편집/삭제/인용답글 (`c`/`e`/`d`/`r`)
- [x] 라벨 추가/제거 (`l` → 피커)
- [x] Assignee 추가/제거 (`a` → 피커)
- [x] 리액션 (`+`/`-`)
- [x] 닫기/열기 (`x`), 브라우저에서 열기 (`o`)
- [x] 섹션 포커스 네비게이션 (Title→Labels→Assignees→Body→Comments)
- [x] 필터 초기화 (`Shift+F`)

### Phase 2-1.5 완료 (Diff/Checks 강화)
- [x] Checks 탭 실제 데이터 조회 (check-runs + commit status API)
- [x] Checks 요약 (passed/failed/pending 카운트)
- [x] Diff 테마 색상 (GitHub Primer diff_add/remove/hunk)
- [x] Diff 파일 요약 (변경 바차트, 파일 상태 뱃지)
- [x] Diff 라인 커서 (`j`/`k`로 라인별 이동, 커서 하이라이트)
- [x] Diff 블록 선택 (`J`/`K` 또는 `Shift+j/k`로 범위 선택)
- [x] Diff 파일 접기/펼치기 (`Enter`로 토글, ▸/▾ 아이콘)
- [x] Diff/Conversation 탭별 독립 스크롤
- [x] Diff race condition 수정 (detail 로드 후 diff fetch)

### Phase 2-2 완료 (Diff 리뷰 + UI 강화)
- [x] 인라인 리뷰 코멘트 표시 (diff에 ╭─│─╰ 박스로 표시)
- [x] 인라인 리뷰 코멘트 작성 (Enter on code line → 에디터 → Ctrl+Enter)
- [x] Suggestion 삽입 (Ctrl+S → ```` ```suggestion ```` 템플릿)
- [x] Suggestion 렌더링 (💡 아이콘 + 초록 하이라이트)
- [x] 파일 트리 패널 (`f` 토글, Tab 포커스 전환, Enter 파일 점프)
- [x] 파일 접기/펼치기 (`h`/`l`/Enter on header)
- [x] 탭 이름 변경 (Conversation/Commits/Checks/Files changed)
- [x] 탭 카운트 뱃지 (Commits(N), Checks(N), Files changed(N))
- [x] Commits 탭 (SHA, 메시지, 작성자, 날짜)
- [x] Base branch 변경 (`b` → 모달)
- [x] Approve (`A`), Request changes (`R` → 모달)
- [x] 하단 액션 바 (키보드 네비게이션: ←/→ 선택, Enter 실행)
- [x] 리뷰어 중복 제거 (사용자별 최신 리뷰만 표시)
- [x] Unicode width 박스 정렬 수정

### Phase 2-3 남은 기능
- [ ] PR 생성 UI (모달 폼)
- [ ] Draft 토글
- [ ] Auto-merge 활성화
- [ ] Side-by-side diff 모드
- [ ] 리뷰 스레드 resolve/unresolve
- [ ] 리뷰어 추가/제거
- [ ] Milestone 설정
- [ ] Linked issues
- [ ] 파일별 Viewed 체크
- [ ] PR Conversation 타임라인 이벤트

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
