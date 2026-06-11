<div align="center">

[🇷🇺 Русский](l10n/README.ru.md) · [🇫🇷 Français](l10n/README.fr.md) · [🇩🇪 Deutsch](l10n/README.de.md) · [🇨🇳 中文](l10n/README.zh.md) · [🇪🇸 Español](l10n/README.es.md) · [🇮🇹 Italiano](l10n/README.it.md) · [🇸🇦 العربية](l10n/README.ar.md) · [🇧🇷 Português](l10n/README.pt.md) · [🇯🇵 日本語](l10n/README.ja.md) · [🇰🇷 한국어](l10n/README.ko.md) · [🇮🇳 हिन्दी](l10n/README.hi.md) · [🇹🇷 Türkçe](l10n/README.tr.md) · [🇳🇱 Nederlands](l10n/README.nl.md) · [🇵🇱 Polski](l10n/README.pl.md)

<br>

# CodeContext AI

<img src="assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-powered codebase analysis & prompt preparation tool**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.23.1-blue?style=flat-square)](VERSION.txt)
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
<tr><td>🗜️ Minify + Skeleton</td><td><b>Up to 80%</b> token reduction</td><td>Manual copy-paste</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Preview & apply JSON patches</td><td>Not available</td></tr>
<tr><td>✅ LLM Checker</td><td>Auto-verify code before saving</td><td>Not available</td></tr>
<tr><td>🔗 AST dependency graph</td><td>Python, JS/TS, Vue</td><td>File listing only</td></tr>
<tr><td>🖱️ Context menu</td><td>Windows / Linux</td><td>None</td></tr>
<tr><td>🎨 Themes</td><td>Apple, Modern, custom JSON</td><td>Fixed UI</td></tr>
<tr><td>⚙️ UI customization (v1.14+)</td><td>Premiere Pro-style</td><td>Fixed UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 languages, system auto-detect</td><td>Single language</td></tr>
<tr><td>♻️ Dedup (v1.23+)</td><td>Auto-remove duplicate files</td><td>Manual check</td></tr>
<tr><td>⚡ Aggressive minify (v1.23+)</td><td>Strips all blank lines</td><td>Manual delete</td></tr>
<tr><td>📌 Checkpoints (v1.23+)</td><td>Save intermediate results</td><td>Not available</td></tr>
<tr><td>👁️ Auto-Watch (v1.23+)</td><td>Auto-reprocess on file changes</td><td>Not available</td></tr>
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
<tr><td>☑ Minify</td><td>Strips whitespace and blank lines</td></tr>
<tr><td>☑ No Comments</td><td>Removes all comments</td></tr>
<tr><td>☑ No Secrets</td><td>Masks API keys, passwords, tokens</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Strips function bodies</b> — maximum token savings</td></tr>
<tr><td>☑ Dedup</td><td>Removes duplicate files with identical content</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — strips all blank lines</td></tr>
<tr><td>☑ Checkpoints</td><td>Saves intermediate processing checkpoints</td></tr>
<tr><td>☑ Auto-Watch</td><td>Auto-reprocess on file changes</td></tr>
<tr><td>Format</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Jinja2 template picker</td></tr>
</tbody>
</table>

<p><b>Skeleton Mode:</b> removes function implementations (<code>def func_name(...):  # ... implementation ...</code>), preserving all classes — lets LLM understand massive projects with minimal tokens.</p>

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
<tr><td>Install context menu</td><td>"Open with CodeContext AI" in right-click menu</td></tr>
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
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Tech Stack</h2>
<table>
<thead><tr><th>Component</th><th>Technology</th></tr></thead>
<tbody>
<tr><td>Language</td><td>Python 3.10+</td></tr>
<tr><td>GUI Framework</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architecture</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Tokenization</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Templating</td><td>jinja2 (11 built-in)</td></tr>
<tr><td>AST parsers</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribution</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>🍎 macOS Finder context menu</li>
<li>🤖 Direct OpenAI/Anthropic API integration</li>
<li>🏛️ Hexagonal Architecture analysis</li>
<li>🔌 Plugin system</li>
<li>🌐 In-app i18n</li>
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
