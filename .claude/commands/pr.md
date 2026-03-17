현재 브랜치의 변경사항을 분석하고 Pull Request를 생성해주세요.

## 절차

1. `git status`로 커밋되지 않은 변경사항이 있는지 확인하고, 있으면 먼저 커밋할지 물어보세요.
2. `git log main..HEAD`와 `git diff main...HEAD`로 현재 브랜치의 전체 변경사항을 분석하세요.
3. 변경사항을 기반으로 PR 제목과 본문을 작성하세요:
   - 제목: 70자 이내, 변경 내용을 간결하게
   - 본문: Summary (1-3 bullet points) + Test plan
4. PR 생성 전에 제목과 본문을 보여주고 확인을 받으세요.
5. 확인 후 `gh pr create`로 PR을 생성하세요.

## PR 본문 형식

```
## Summary
- 변경사항 요약 (bullet points)

## Test plan
- [ ] 테스트 항목들

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

## 규칙

- main 브랜치에서는 실행하지 마세요. 먼저 feature 브랜치를 만들라고 안내하세요.
- remote에 push되지 않은 커밋이 있으면 push 후 PR을 생성하세요.
- PR 제목에 이모지를 사용하지 마세요.
- 한국어로 소통하되, PR 제목과 본문은 영어로 작성하세요.
