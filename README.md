<div align="center">

[🇷🇺 Русский](l10n/README.ru.md) · [🇫🇷 Français](l10n/README.fr.md) · [🇩🇪 Deutsch](l10n/README.de.md) · [🇨🇳 中文](l10n/README.zh.md) · [🇪🇸 Español](l10n/README.es.md) · [🇮🇹 Italiano](l10n/README.it.md) · [🇸🇦 العربية](l10n/README.ar.md) · [🇧🇷 Português](l10n/README.pt.md) · [🇯🇵 日本語](l10n/README.ja.md) · [🇰🇷 한국어](l10n/README.ko.md) · [🇮🇳 हिन्दी](l10n/README.hi.md) · [🇹🇷 Türkçe](l10n/README.tr.md) · [🇳🇱 Nederlands](l10n/README.nl.md) · [🇵🇱 Polski](l10n/README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-powered codebase analysis & prompt preparation tool**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.27.0-blue?style=flat-square)](VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 About</h2>

<p><b>CodeContext AI</b> is a powerful desktop tool for preparing your codebase to work with Large Language Models (LLMs). It scans project folders, analyzes structure, builds dependency graphs, and generates a single, perfectly structured prompt — optimized for token consumption and architectural clarity.</p>

<h3>❓ Why?</h3>
<p>When working with AI, developers face context window token limits — LLMs "lose" architectural coherence when code is copied in parts. <b>CodeContext AI solves this</b>: collect your entire project into one structured prompt in a few clicks, saving up to 80% on tokens.</p>

<hr>

<h2>🚀 Features</h2>

<table>
<thead><tr><th>Feature</th><th>CodeContext AI</th><th>Manual</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify</td><td><b>Up to 80%</b> token reduction — strips whitespace & blank lines</td><td>Manual copy-paste</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Preview & apply JSON patches</td><td>Not available</td></tr>
<tr><td>✅ LLM Checker</td><td>Auto-verify code before saving</td><td>Not available</td></tr>
<tr><td>🔗 AST dependency graph</td><td>Python, JS/TS, Vue</td><td>File listing only</td></tr>
<tr><td>🖱️ Context menu</td><td>Windows / Linux / macOS</td><td>None</td></tr>
<tr><td>🎨 Themes</td><td>Apple, Modern, custom JSON</td><td>Fixed UI</td></tr>
<tr><td>⚙️ UI customization (v1.14+)</td><td>Premiere Pro-style</td><td>Fixed UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 languages, system auto-detect</td><td>Single language</td></tr>
<tr><td>♻️ Dedup (v1.23+)</td><td>Detects & skips files with identical content</td><td>Manual check</td></tr>
<tr><td>⚡ Aggressive minify (v1.23+)</td><td>Extra compression — eliminates trailing whitespace on every line</td><td>Manual delete</td></tr>
<tr><td>📌 Checkpoints (v1.23+)</td><td>Save before/after snapshots for debugging</td><td>Not available</td></tr>
<tr><td>👁️ Auto-Watch (v1.23+)</td><td>Watches files & re-processes on change</td><td>Not available</td></tr>
<tr><td>🔌 Plugin System (v1.25+)</td><td>Extend with Python plugins — custom tabs, actions, and i18n</td><td>Not available</td></tr>
<tr><td>🚦 CI/CD Integration</td><td>GitHub Actions & GitLab CI — auto-generate PR context via <code>--git-base</code></td><td>Not available</td></tr>
<tr><td>🌳 Monorepo Support (v1.25+)</td><td>Lerna, NX, Turborepo, pnpm workspaces — cross-package imports, root config discovery</td><td>Not available</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Installation</h2>

<p><b>Prerequisites:</b> Python 3.10+, Git</p>

<pre>git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext
python -m venv venv
# Windows:
venv\Scripts\activate
# Linux/macOS:
source venv/bin/activate
pip install -r requirements.txt</pre>

<h3>PyPI (pip)</h3>
<pre>pip install codecontext-ai</pre>

<pre># Then launch:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Action</th><th>Command</th></tr></thead>
<tbody>
<tr><td>Install</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Search</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Update</td><td><code>yay -Syu</code></td></tr>
<tr><td>Remove</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>If <b>yay</b> is not installed:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternative: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUI Mode</h2>
<pre>python main.py</pre>

<h3>1. Interface Overview</h3>
<p>The window is split into three zones:</p>
<ul>
<li><b>Left sidebar (tabs)</b> — scan settings, filters, prompts, LLM config, themes</li>
<li><b>Center area</b> — folder list, file tree, token analytics</li>
<li><b>Top action bar</b> — Minify/No Comments/Skeleton toggles, output format, action buttons</li>
</ul>

<h3>2. Adding a Project</h3>
<table>
<thead><tr><th>Action</th><th>How</th></tr></thead>
<tbody>
<tr><td>Drag & drop</td><td>Just drag a project folder into the window</td></tr>
<tr><td>Browse dialog</td><td>Click "+ Папка ПК" on the <b>Sources</b> tab</td></tr>
<tr><td>GitHub repo</td><td>Click "+ GitHub / PR" — paste a repo or Pull Request URL</td></tr>
<tr><td>Save config</td><td>Click "💾 Save config" — creates <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>GitHub loading modes:</b></p>
<ul>
<li><b>Save permanently</b> — clones to a folder on your disk</li>
<li><b>Temporary</b> — clones to a temp folder (deleted on app close)</li>
</ul>

<h3>3. Scan Configuration</h3>

<h4>Sources Tab</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>Include only files changed in the last commit</td></tr>
<tr><td>☑ Respect .gitignore</td><td>Auto-exclude files from <code>.gitignore</code></td></tr>
<tr><td>🔍 Scan Files</td><td>Build the file tree with metadata</td></tr>
</tbody>
</table>

<h4>Filters Tab</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td><b>Extension presets</b></td><td>Quick switch between language sets (Python, Web, Golang, Rust, C#, etc.)</td></tr>
<tr><td><b>Extensions</b></td><td>Custom file extension whitelist</td></tr>
<tr><td><b>Ignored paths</b></td><td>Skip folders/files (node_modules, .git, build, dist, etc.)</td></tr>
<tr><td>☑ Include file tree</td><td>Prepends folder structure to the prompt</td></tr>
<tr><td>☑ Include dependency map</td><td>AST-based import analysis for Python/JS/TS</td></tr>
<tr><td>☑ Include Mermaid graph</td><td>Architecture diagram in Mermaid format</td></tr>
</tbody>
</table>

<p>💡 <b>Saving custom presets:</b> configure filters, click 💾, enter a name.</p>

<h4>Prompts Tab</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td><b>Prompt presets</b></td><td>Quick change of system prompt (Code Review, Bug Hunter, Refactoring, etc.)</td></tr>
<tr><td><b>System prompt</b></td><td>Custom prompt — sent to LLM as system context</td></tr>
<tr><td><b>🧩 Apply JSON patch</b></td><td>Paste LLM JSON response — preview diff and apply to disk</td></tr>
</tbody>
</table>

<p><b>Using JSON patches:</b></p>
<ol>
<li>Ask LLM for a JSON array: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Paste JSON, click <b>"Next"</b> → <b>Safety Diff Viewer</b> opens</li>
<li>Check/uncheck files, optionally click <b>"🤖 Check via LLM"</b></li>
<li>Click <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. Output Format Settings</h3>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>Trims leading/trailing whitespace on every line, removes blank lines — safe baseline compression for everyday use</td></tr>
<tr><td>☑ Aggressive</td><td>Extra minification pass — strips trailing whitespace aggressively on every line. Combine with Minify for maximum token savings when context is tight</td></tr>
<tr><td>☑ No Comments</td><td>Removes all comments from code</td></tr>
<tr><td>☑ No Secrets</td><td>Masks API keys, passwords, tokens</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Strips function bodies</b> — maximum token savings</td></tr>
<tr><td>☑ Dedup</td><td>Scans all files and excludes duplicates with identical content — eliminates redundant context from repeated files</td></tr>
<tr><td>☑ Checkpoints</td><td>Saves intermediate processing snapshots (before/after) to disk — useful for debugging pipeline stages or comparing outputs</td></tr>
<tr><td>☑ Auto-Watch</td><td>Watches project files for changes and automatically reprocesses — keeps your prompt up-to-date during active development</td></tr>
<tr><td>Format</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Jinja2 template picker</td></tr>
</tbody>
</table>

<p><b>Skeleton Mode:</b> removes function implementations (<code>def func_name(...):  # ... implementation ...</code>), preserving all classes — lets LLM understand massive projects with minimal tokens.</p>

<p><b>Minify vs Aggressive:</b> <b>Minify</b> strips leading/trailing whitespace and removes blank lines — safe for any codebase, reduces tokens without affecting readability. <b>Aggressive</b> adds an extra pass that eliminates trailing whitespace on every line for maximum compression. Combine both when you need to fit more code into a limited context window.</p>

<p><b>Dedup:</b> automatically detects files with identical content across your project and excludes duplicates from the output — prevents LLM from seeing the same code twice and wasting tokens.</p>

<p><b>Checkpoints:</b> saves intermediate results at each pipeline stage (before cleanup, after minification, etc.) to <code>checkpoints/</code> folder. Useful for debugging what each processing step does or comparing outputs side by side.</p>

<p><b>Auto-Watch:</b> monitors your project files for changes using the OS file watcher. When a file is saved, the pipeline automatically re-runs — ideal during active development when you need continuous prompt updates.</p>

<h3>5. Action Buttons</h3>
<table>
<thead><tr><th>Button</th><th>Action</th></tr></thead>
<tbody>
<tr><td>👀 Preview</td><td><b>Advanced Preview Dialog</b> — "Final Prompt" + "Before/After" tabs</td></tr>
<tr><td>📋 Copy to Clipboard</td><td>Copy result — paste into ChatGPT / Claude</td></tr>
<tr><td>🚀 Send to ChatGPT / Claude</td><td>Opens web chat and pastes context</td></tr>
<tr><td>💻 Open in Editor</td><td>Opens in VS Code / Cursor</td></tr>
<tr><td>💾 Save to File</td><td>Save result to disk</td></tr>
</tbody>
</table>

<h3>6. Advanced Preview Dialog</h3>
<p><b>"📝 Final Prompt" tab:</b> file list (left) + full text with highlighting (right). Copy All / Copy File.</p>
<p><b>"🔍 Before/After" tab:</b> colored diff between original and optimized. Counter: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM & OS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>Auto LLM patch verification before applying</td></tr>
<tr><td>URL / Key / Model</td><td>API endpoint (default OpenAI), key, model</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">OS Integration</th></tr></thead>
<tbody>
<tr><td>Install context menu</td><td>"Open with CodeContext AI" in right-click menu (Windows / Linux / macOS)</td></tr>
<tr><td>Add to PATH</td><td>Global <code>codecontext</code> CLI command</td></tr>
<tr><td>Editor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Themes</h3>
<ul>
<li><b>Theme:</b> Apple, Modern — <b>Mode:</b> light / dark</li>
<li>📂 Open themes folder / ➕ Import theme (.json)</li>
</ul>

<h3>9. 📊 Token Analytics</h3>
<p>Table: file path, tokens (tiktoken), compression, savings %, cost for model.</p>

<h3>10. 🎛️ UI Customization (v1.14+)</h3>
<p>Click <b>⚙</b> next to version — "Interface Settings (Premiere Pro style)" dialog. Toggle tabs (Sources, Filters, Prompts, LLM & OS, Themes) and action buttons (Preview, Clipboard, ChatGPT, Editor, File).</p>

<h3>11. Command Palette</h3>
<p><code>Ctrl+Shift+P</code> — mouse-free access to all actions.</p>

<h3>12. 🔌 Plugin System (v1.25+)</h3>
<p><b>CodeContext AI</b> supports a <b>Python plugin system</b> that lets you extend the app with custom functionality.</p>

<h4>📁 Plugin Structure</h4>
<pre>my_plugin/
├── manifest.json          # Plugin metadata
├── requirements.txt       # (Optional) pip dependencies
├── locales/
│   ├── en.json            # English translations
│   └── ru.json            # Russian translations
└── plugin.py              # Entry point</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Does something useful",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Example)</h4>
<pre>from src.services.plugin_manager import IPlugin

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, controller: MainController) -> None:
        # Add translations from locales/ folder (auto-loaded)
        # Register a sidebar tab
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("Hello from plugin!")
        )
        # Register an action button
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Plugin action clicked")
        )
        api.add_log("My Plugin initialized")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Security</h4>
<ul>
<li>Plugins get <b>full Python access</b> — only install from trusted sources</li>
<li>On first load, a security dialog asks your approval before enabling a plugin</li>
<li>If <code>requirements.txt</code> exists, you'll see a live pip install log before loading</li>
<li>Approved plugins are remembered in settings (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 Plugin API</h4>
<table>
<thead><tr><th>Property / Method</th><th>Description</th></tr></thead>
<tbody>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Add a tab to the left sidebar</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Add a button to the "Plugins 🔽" dropdown</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Write to the app log panel</td></tr>
</tbody>
</table>

<h4>⚙️ Visibility</h4>
<p>Plugin tabs and action buttons can be toggled via <b>⚙ UI Customization</b> — they appear alongside built-in tabs/actions with their own checkboxes.</p>

<hr>

<h2>💻 CLI Mode</h2>
<pre>python main.py --cli --path /path/to/project [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parameter</th><th>Type</th><th>Description</th><th>Example</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLI mode (no GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>Project path</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Extensions</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Ignored paths</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Enable minification</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Strip comments</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Mask secrets</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Skeleton mode</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Output file</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Print to stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Git changes only</td><td><code>--git</code></td></tr>
<tr><td><code>--git-base</code></td><td>str</td><td>Base branch for git diff in CI/CD</td><td><code>--git-base origin/main</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Respect .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>File tree</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaid graph</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Dependency map</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSON patch</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2 template</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Custom system prompt</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Examples</h3>
<pre># Minimal run
python main.py --cli --path ./myapp --stdout

# Full analysis with XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON patch
python main.py --cli --path ./myapp --patch llm_response.json

# Custom Jinja2 template
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid diagram
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Multiple paths
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml

# CI/CD — diff against base branch
python main.py --cli --path . --git --git-base origin/main --minify true --stdout</pre>

<hr>

<h2>🚦 CI/CD Integration</h2>

<p><b>CodeContext AI</b> can be integrated into your CI/CD pipelines to automatically generate a lean context of the files changed in a Pull/Merge Request and post it as a PR comment — no more manual copy-pasting for code review.</p>

<h3>How it works</h3>
<p>Normally <code>--git</code> runs <code>git diff HEAD</code>, which is useless in CI (HEAD is a merge commit).<br>
The <code>--git-base</code> flag tells CodeContext to diff against a real branch instead:</p>
<pre>git diff origin/main --name-only</pre>
<p>Only the files modified in the PR are scanned, minified, and assembled into a prompt.</p>

<h3>GitHub Actions</h3>
<p>Create <code>.github/workflows/codecontext-pr.yml</code>:</p>
<pre>name: 🧠 Generate PR Context

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
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: actions/setup-python@v5
        with:
          python-version: "3.11"
      - name: Install CodeContext AI
        run: pip install codecontext-ai
      - name: Generate PR Context
        run: |
          codecontext --cli --path . --git \
            --git-base "origin/${{ github.base_ref }}" \
            --format markdown --minify true \
            --no-comments true --stdout > pr_context.md
      - name: Comment on PR
        if: hashFiles('pr_context.md') != ''
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            let ctx = fs.readFileSync('pr_context.md', 'utf8').slice(0, 60000);
            await github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `&lt;details&gt;\n&lt;summary&gt;&lt;b&gt;CodeContext AI: PR Context&lt;/b&gt;&lt;/summary&gt;\n\n\`\`\`markdown\n${ctx}\n\`\`\`\n&lt;/details&gt;`
            });</pre>

<h3>GitLab CI</h3>
<p>Add to <code>.gitlab-ci.yml</code>. Requires a <code>GITLAB_API_TOKEN</code> variable with <code>api</code> scope.</p>
<pre>codecontext_pr_analysis:
  stage: test
  image: python:3.11-slim
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
  before_script:
    - apt-get update && apt-get install -y git curl jq
    - pip install codecontext-ai
    - git fetch origin $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
  script:
    - codecontext --cli --path . --git
        --git-base "origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME"
        --format markdown --minify true --no-comments true --stdout > mr_context.md
    - |
      CONTENT=$(cat mr_context.md | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | awk '{printf "%s\\n", $0}')
      JSON=$(jq -n --arg body "&lt;details&gt;&lt;summary&gt;&lt;b&gt;CodeContext AI: PR Context&lt;/b&gt;&lt;/summary&gt;\n\n\`\`\`markdown\n${CONTENT}\n\`\`\`\n&lt;/details&gt;" '{body: $body}')
      curl --request POST --header "PRIVATE-TOKEN: $GITLAB_API_TOKEN" \
           --header "Content-Type: application/json" --data "$JSON" \
           "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID/notes"</pre>

<p>See <a href="docs/CI_CD.md"><code>docs/CI_CD.md</code></a> for detailed setup instructions.</p>

<hr>

<h2>🏗️ Tech Stack</h2>
<table>
<thead><tr><th>Component</th><th>Technology</th></tr></thead>
<tbody>
<tr><td>Language</td><td>Python 3.10+</td></tr>
<tr><td>GUI Framework</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architecture</td><td>Clean Architecture</td></tr>
<tr><td>Tokenization</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Templating</td><td>jinja2 (11 built-in)</td></tr>
<tr><td>AST parsers</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribution</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation)</b> mode — indexing massive codebases using local vector DB (Chroma/FAISS).</li>
<li>🚫 <b>Deep .gitignore parsing</b> — support for nested <code>.gitignore</code> files & global <code>~/.gitignore</code>.</li>
<li>☁️ <b>Cloud Sync</b> — sync presets & configurations via GitHub Gists.</li>
<li>🌳 <b>Multi-root Workspaces</b> — improved monorepo support (Lerna, NX, Turborepo). ✅</li>
<li>🚀 <b>CI/CD Pipelines</b> — GitHub Actions & GitLab CI plugins for automated PR context generation.</li>
<li>🤖 <b>Direct OpenAI/Anthropic API integration</b> — complete the bridge from prompt generation to direct output.</li>
<li>🔌 Plugin system ✅</li>
</ul>

<hr>

<h2>👨‍💻 Team</h2>
<p><b>Developer:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs on GitHub</p>

<hr>

<h2>🤝 Contributing</h2>
<ol>
<li>Fork the repository</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Follow SOLID principles (see <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 License</h2>
<p>MIT. See <code>LICENSE</code> for details.</p>
