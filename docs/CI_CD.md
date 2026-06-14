# CI/CD Pipelines: Automated PR Context Generation

**CodeContext AI** can be natively integrated into your Continuous Integration (CI)
pipelines (GitHub Actions, GitLab CI, etc.). This feature automatically generates
a highly optimized prompt containing the code of the files changed in a Pull/Merge
Request, and posts it as a PR comment.

---

## How it works

Normally, `CodeContext AI` scans an entire project. When you use the `--git` flag,
it runs `git diff HEAD` to find local unstaged/staged changes.

In a CI/CD environment triggered by a Pull Request, `HEAD` is usually a merge
commit, making `git diff HEAD` inaccurate. To solve this, we introduced the
`--git-base` flag.

By passing `--git-base "origin/main"`, CodeContext AI runs:

```
git diff origin/main --name-only
```

This guarantees that **only the files modified in the PR** are processed,
minified, token-optimized, and assembled into a prompt.

---

## GitHub Actions Integration

Create `.github/workflows/codecontext-pr.yml` in your repository:

```yaml
name: Generate PR Context

on:
  pull_request:
    types: [opened, synchronize, reopened]

permissions:
  pull-requests: write
  contents: read

jobs:
  codecontext-analysis:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"

      - name: Install CodeContext AI
        run: pip install codecontext-ai

      - name: Generate PR Context
        run: |
          BASE_BRANCH="origin/${{ github.base_ref }}"
          codecontext --cli \
            --path . \
            --git \
            --git-base "$BASE_BRANCH" \
            --format markdown \
            --minify true \
            --no-comments true \
            --stdout > pr_context.md

      - name: Check if Context is Empty
        id: check_context
        run: |
          if [ ! -s pr_context.md ] || ! grep -q "FILE:" pr_context.md; then
            echo "has_changes=false" >> $GITHUB_OUTPUT
          else
            echo "has_changes=true" >> $GITHUB_OUTPUT
          fi

      - name: Comment on PR
        if: steps.check_context.outputs.has_changes == 'true'
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            let context = fs.readFileSync('pr_context.md', 'utf8');
            if (context.length > 60000) {
              context = context.substring(0, 60000)
                + "\n\n... [CONTEXT TOO LARGE, TRUNCATED]";
            }
            const body = [
              "<details>",
              "<summary><b>CodeContext AI: PR Context (Copy & Paste to LLM)</b></summary>",
              "",
              "```markdown",
              context,
              "```",
              "</details>"
            ].join("\n");
            await github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

---

## GitLab CI Integration

Add the following job to your `.gitlab-ci.yml`.

> **Note:** Create a CI/CD Variable named `GITLAB_API_TOKEN`
> (Project Settings > CI/CD > Variables) with a Personal/Project Access Token
> that has `api` scope.

```yaml
codecontext_pr_analysis:
  stage: test
  image: python:3.11-slim
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
  before_script:
    - apt-get update && apt-get install -y git curl jq
    - pip install codecontext-ai
    - git fetch origin $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
  script:
    - echo "Generating context for changed files..."
    - BASE_BRANCH="origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME"
    - codecontext --cli --path . --git --git-base "$BASE_BRANCH" --format markdown --minify true --no-comments true --stdout > mr_context.md
    - |
      if [ ! -s mr_context.md ] || ! grep -q "FILE:" mr_context.md; then
        echo "No relevant code changes found."
        exit 0
      fi
    - |
      CONTENT=$(cat mr_context.md | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | awk '{printf "%s\\n", $0}')
      COMMENT_BODY="<details><summary><b>CodeContext AI: PR Context</b></summary><br><br>\`\`\`markdown<br>${CONTENT}<br>\`\`\`</details>"
      JSON_PAYLOAD=$(jq -n --arg body "$COMMENT_BODY" '{body: $body}')
    - |
      curl --request POST \
           --header "PRIVATE-TOKEN: $GITLAB_API_TOKEN" \
           --header "Content-Type: application/json" \
           --data "$JSON_PAYLOAD" \
           "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID/notes"
```

---

## Tips

| Advice | Reason |
|--------|--------|
| Always use `fetch-depth: 0` (GitHub) or `git fetch origin` (GitLab) | Without full history `git diff` cannot compare against the base branch |
| Use `--skeleton true` for very large PRs | Strips function bodies, saving up to 80% tokens |
| The `<details>` tag keeps the PR comment thread clean | Context is hidden behind a clickable spoiler |
| GitHub comment limit is 65536 characters | The workflow truncates if exceeded |
| Set `GITLAB_API_TOKEN` as a CI/CD Variable | Required for posting MR notes via GitLab API |
