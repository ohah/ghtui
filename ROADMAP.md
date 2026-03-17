# ghtui Roadmap

## 현재 구현 현황

### 완료된 탭

| # | 탭 | API | 뷰 | 주요 기능 |
|---|---|---|---|---|
| 1 | Code | - | dashboard.rs | 레포 소개 + README placeholder (파일 브라우저 미구현) |
| 2 | Issues | ✅ | issue_list, issue_detail | 목록(필터), 상세(마크다운+코멘트), 생성, 닫기/열기, 코멘트 |
| 3 | Pull requests | ✅ | pr_list, pr_detail | 목록, 상세(Conversation/Diff/Checks), 머지, 리뷰, 코멘트 |
| 4 | Actions | ✅ | actions_list, action_detail | 워크플로우 목록, 잡 목록, 로그(ANSI), 취소/재실행 |
| 5 | Projects | - | placeholder | 미구현 |
| 6 | Wiki | - | placeholder | 미구현 |
| 7 | Security | - | placeholder | 미구현 |
| 8 | Insights | - | placeholder | 미구현 |
| 9 | Settings | - | placeholder | 미구현 |

### 완료된 기반 기능

- Cargo workspace 구조 (ghtui, ghtui-api, ghtui-core, ghtui-widgets)
- Elm architecture (Message → update → Command → API → Message)
- GitHub API 클라이언트 (인증, LRU 캐시, rate limit 추적)
- GitHub Primer 기반 테마 시스템 (Dark / Light)
- Tab / Shift-Tab / 1-9 탭 네비게이션
- diff 파서 (라인 번호, 파일 상태, 다중 hunk)
- 마크다운 렌더러 (heading, bold, italic, code block, list, blockquote, link)
- Notifications API + 뷰
- Search API (repos, issues, code)
- 96개 테스트

---

## Phase 1 — Code 탭 완성

- [ ] GitHub Contents API 연동 (`/repos/{owner}/{repo}/contents/{path}`)
- [ ] 파일 트리 브라우저 (접기/펼치기)
- [ ] 파일 내용 뷰어 (syntax highlighting via syntect)
- [ ] 브랜치/태그 전환
- [ ] 커밋 히스토리 (log with graph)
- [ ] 커밋 상세 (diff, 메시지, 메타데이터)
- [ ] README.md 실제 렌더링

## Phase 2 — Projects 탭

- [ ] GitHub Projects v2 GraphQL API 연동
- [ ] 프로젝트 목록
- [ ] 보드 뷰 (컬럼 + 카드)
- [ ] 아이템 상세 보기
- [ ] 아이템 상태 변경 (드래그 대신 키보드)

## Phase 3 — Security 탭

- [ ] Dependabot alerts API 연동
- [ ] 취약점 목록 (severity별 필터)
- [ ] 취약점 상세 (영향 받는 패키지, 권고사항)
- [ ] Code scanning alerts
- [ ] Secret scanning alerts
- [ ] Security advisories

## Phase 4 — Insights 탭

- [ ] Contributors API 연동 (커밋 수, additions/deletions)
- [ ] 커밋 활동 그래프 (주간/월간 ascii chart)
- [ ] 트래픽 (clones, views) — push access 필요
- [ ] Code frequency
- [ ] Dependency graph
- [ ] Forks 네트워크

## Phase 5 — Wiki 탭

- [ ] Wiki pages API 연동 (git-based)
- [ ] 위키 페이지 목록
- [ ] 위키 페이지 마크다운 렌더링
- [ ] 위키 페이지 생성/편집

## Phase 6 — Settings 탭

- [ ] Repository API 연동 (read-only 우선)
- [ ] 일반 설정 (이름, description, visibility)
- [ ] 브랜치 보호 규칙 보기
- [ ] Collaborators 목록
- [ ] Webhooks 목록
- [ ] Deploy keys

## Phase 7 — Search 완성

- [ ] Search 뷰 구현 (현재 placeholder)
- [ ] 코드 검색 결과 + 하이라이트
- [ ] 이슈/PR 검색 (GitHub 검색 문법 지원)
- [ ] 레포 검색
- [ ] 최근 검색 히스토리

## Phase 8 — UX 개선

- [ ] Command palette (Ctrl-P, fuzzy search)
- [ ] 커스텀 키바인딩 설정 (config.toml)
- [ ] 마우스 지원 (클릭, 스크롤)
- [ ] 반응형 레이아웃 (좁은 터미널 대응)
- [ ] inline diff 코멘트 작성
- [ ] side-by-side diff 모드
- [ ] 파일 트리 in diff view
- [ ] 이미지 미리보기 (sixel/kitty protocol)

## Phase 9 — 배포 & 에코시스템

- [ ] `cargo install ghtui`로 설치 가능
- [ ] Homebrew formula
- [ ] GitHub Releases (자동 빌드, macOS/Linux/Windows)
- [ ] CI/CD (GitHub Actions: test, lint, release)
- [ ] crates.io 게시
- [ ] man page / `--help` 개선
- [ ] CHANGELOG.md 자동 생성

## Phase 10 — 고급 기능

- [ ] GitHub Enterprise Server 지원 (커스텀 base URL)
- [ ] 멀티 레포 대시보드
- [ ] Discussions 탭
- [ ] Gists 뷰
- [ ] Organization 탐색 (팀, 멤버)
- [ ] 실시간 Actions 로그 스트리밍 (WebSocket)
- [ ] 오프라인 모드 (캐시 기반)
- [ ] 플러그인 시스템
