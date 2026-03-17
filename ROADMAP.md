# ghtui Roadmap

## 현재 구현 현황

### 탭별 현황

| # | 탭 | API | 뷰 | 주요 기능 |
|---|---|---|---|---|
| 1 | Code | - | dashboard.rs | 레포 소개 + README placeholder (파일 브라우저 미구현) |
| 2 | Issues | ✅ | issue_list, issue_detail | 목록, 상세, 닫기/열기 |
| 3 | Pull requests | ✅ | pr_list, pr_detail | 목록, 상세(Conversation/Diff/Checks), 머지, 리뷰 |
| 4 | Actions | ✅ | actions_list, action_detail | 워크플로우 목록, 잡 목록, 로그, 취소/재실행 |
| 5 | Projects | - | placeholder | 미구현 |
| 6 | Wiki | - | placeholder | 미구현 |
| 7 | Security | ✅ | security.rs | Dependabot, Code Scanning, Secret Scanning (read-only) |
| 8 | Insights | ✅ | insights.rs | Contributors, Commit Activity, Traffic (read-only) |
| 9 | Settings | ✅ | settings.rs | 일반설정, 브랜치 보호, Collaborators (read-only) |

### 완료된 기반 기능

- Cargo workspace (ghtui, ghtui-api, ghtui-core, ghtui-widgets)
- Elm architecture (Message → update → Command → API → Message)
- GitHub API 클라이언트 (인증, LRU 캐시, rate limit)
- GitHub Primer 기반 테마 (Dark / Light, `t`키 토글)
- Tab / Shift-Tab / 1-9 탭 네비게이션
- diff 파서, 마크다운 렌더러
- Notifications API + 뷰, Search API
- rustfmt + clippy 설정, 96개 테스트

---

## Phase 1 — Issues 탭 완성

현재 되는 것: 목록(open/closed), 상세 보기, 닫기/열기

- [ ] 필터 UI (author, label, milestone, assignee)
- [ ] 정렬 UI (newest, oldest, most commented)
- [ ] 이슈 검색
- [ ] 이슈 생성 UI (모달 폼)
- [ ] 이슈 제목/본문 편집
- [ ] 코멘트 추가 UI (모달)
- [ ] 코멘트 편집/삭제
- [ ] 라벨 추가/제거
- [ ] Assignee 추가/제거
- [ ] Milestone 설정
- [ ] 리액션 (👍 등)
- [ ] 타임라인 이벤트 (assigned, labeled 등)
- [ ] 교차 참조 (linked PRs)
- [ ] 이슈 잠금
- [ ] 이슈 핀
- [ ] 이슈 이전 (transfer)
- [ ] 이슈 템플릿
- [ ] 페이지네이션 UI (다음/이전)

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

현재 되는 것: 런 목록, 잡 목록, 로그 보기, 취소, 재실행

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

## Phase 6 — Security 탭

- [x] Dependabot alerts API 연동
- [x] 취약점 목록 (severity별 필터)
- [ ] 취약점 상세 (영향 받는 패키지, 권고사항)
- [x] Code scanning alerts
- [x] Secret scanning alerts
- [ ] Security advisories

## Phase 7 — Insights 탭

- [x] Contributors API (커밋 수, additions/deletions)
- [x] 커밋 활동 그래프 (ascii chart)
- [x] 트래픽 (clones, views)
- [ ] Code frequency
- [ ] Dependency graph
- [ ] Forks 네트워크

## Phase 8 — Settings 탭

- [x] Repository API (read-only 우선)
- [x] 일반 설정 (이름, description, visibility)
- [x] 브랜치 보호 규칙 보기
- [x] Collaborators 목록
- [ ] Webhooks 목록
- [ ] Deploy keys

## Phase 9 — Projects 탭

- [ ] GitHub Projects v2 GraphQL API 연동
- [ ] 프로젝트 목록
- [ ] 보드 뷰 (컬럼 + 카드)
- [ ] 아이템 상세 보기
- [ ] 아이템 상태 변경

## Phase 10 — Wiki 탭

- [ ] Wiki pages API (git-based)
- [ ] 위키 페이지 목록
- [ ] 위키 페이지 마크다운 렌더링
- [ ] 위키 페이지 생성/편집

## Phase 11 — Code 탭 (작업량 최대)

- [ ] GitHub Contents API 연동
- [ ] 파일 트리 브라우저 (접기/펼치기)
- [ ] 파일 내용 뷰어 (syntax highlighting)
- [ ] 간단한 파일 편집 (인라인 수정 → commit)
- [ ] 브랜치/태그 전환
- [ ] 커밋 히스토리 (log with graph)
- [ ] 커밋 상세 (diff, 메시지, 메타데이터)
- [ ] README.md 실제 렌더링

## Phase 12 — UX 개선

- [ ] Command palette (Ctrl-P, fuzzy search)
- [ ] 커스텀 키바인딩 설정 (config.toml)
- [ ] 마우스 지원 (클릭, 스크롤)
- [ ] 반응형 레이아웃 (좁은 터미널)
- [ ] 이미지 미리보기 (sixel/kitty protocol)

## Phase 13 — 배포 & 에코시스템

- [ ] `cargo install ghtui`
- [ ] Homebrew formula
- [ ] GitHub Releases (macOS/Linux/Windows)
- [ ] CI/CD (GitHub Actions: test, lint, release)
- [ ] crates.io 게시
- [ ] CHANGELOG.md 자동 생성

## Phase 14 — 고급 기능

- [ ] GitHub Enterprise Server 지원
- [ ] 멀티 레포 대시보드
- [ ] Discussions 탭
- [ ] Gists 뷰
- [ ] Organization 탐색 (팀, 멤버)
- [ ] 오프라인 모드 (캐시 기반)
- [ ] 플러그인 시스템
