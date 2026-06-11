<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="../assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI 驱动的代码库分析与提示词准备工具**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.15.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 关于项目</h2>

<p><b>CodeContext AI</b> 是一款强大的桌面工具，用于准备您的代码库以配合大型语言模型（LLM）使用。它扫描项目文件夹，分析结构，构建依赖关系图，并生成单一、结构完美的提示词——针对 Token 消耗和架构清晰度进行了优化。</p>

<h3>❓ 为什么？</h3>
<p>在使用 AI 时，开发者会面临上下文窗口 Token 限制的问题——当代码被分部分复制时，LLM 会"丢失"架构连贯性。<b>CodeContext AI 解决了这个问题</b>：只需点击几下，即可将整个项目收集到一个结构化的提示词中，节省高达 80% 的 Token。</p>

<hr>

<h2>🚀 功能特性</h2>

<table>
<thead><tr><th>功能</th><th>CodeContext AI</th><th>手动操作</th></tr></thead>
<tbody>
<tr><td>🗜️ 压缩 + 骨架模式</td><td><b>高达 80%</b> Token 减少</td><td>手动复制粘贴</td></tr>
<tr><td>🧩 LLM 补丁器</td><td>预览和应用 JSON 补丁</td><td>不可用</td></tr>
<tr><td>✅ LLM 检查器</td><td>保存前自动验证代码</td><td>不可用</td></tr>
<tr><td>🔗 AST 依赖图</td><td>Python、JS/TS、Vue</td><td>仅文件列表</td></tr>
<tr><td>🖱️ 右键菜单</td><td>Windows / Linux</td><td>无</td></tr>
<tr><td>🎨 主题</td><td>Apple、Modern、自定义 JSON</td><td>固定 UI</td></tr>
<tr><td>⚙️ UI 自定义 (v1.14+)</td><td>Premiere Pro 风格</td><td>固定 UI</td></tr>
</tbody>
</table>

<hr>

<h2>📥 安装</h2>

<p><b>前提条件：</b>Python 3.10+、Git</p>

<pre>git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext
python -m venv venv
# Windows：
venv\Scripts\activate
# Linux/macOS：
source venv/bin/activate
pip install -r requirements.txt</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux（AUR）</h3>
<table>
<thead><tr><th>操作</th><th>命令</th></tr></thead>
<tbody>
<tr><td>安装</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>搜索</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>更新</td><td><code>yay -Syu</code></td></tr>
<tr><td>卸载</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>如果未安装 <b>yay</b>：</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>替代方案：<code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUI 模式</h2>
<pre>python main.py</pre>

<h3>1. 界面概览</h3>
<p>窗口分为三个区域：</p>
<ul>
<li><b>左侧边栏（选项卡）</b>——扫描设置、过滤器、提示词、LLM 配置、主题</li>
<li><b>中央区域</b>——文件夹列表、文件树、Token 分析</li>
<li><b>顶部操作栏</b>——压缩/无注释/骨架模式开关、输出格式、操作按钮</li>
</ul>

<h3>2. 添加项目</h3>
<table>
<thead><tr><th>操作</th><th>方法</th></tr></thead>
<tbody>
<tr><td>拖放</td><td>直接将项目文件夹拖入窗口</td></tr>
<tr><td>浏览对话框</td><td>在 <b>Sources</b> 选项卡上点击"+ Папка ПК"</td></tr>
<tr><td>GitHub 仓库</td><td>点击"+ GitHub / PR"——粘贴仓库或 Pull Request URL</td></tr>
<tr><td>保存配置</td><td>点击"💾 Save config"——创建 <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>GitHub 加载模式：</b></p>
<ul>
<li><b>永久保存</b>——克隆到磁盘上的文件夹</li>
<li><b>临时加载</b>——克隆到临时文件夹（关闭应用时删除）</li>
</ul>

<h3>3. 扫描配置</h3>

<h4>Sources 选项卡</h4>
<table>
<thead><tr><th>选项</th><th>描述</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>仅包含上次提交中更改的文件</td></tr>
<tr><td>☑ Respect .gitignore</td><td>自动排除 <code>.gitignore</code> 中的文件</td></tr>
<tr><td>🔍 Scan Files</td><td>构建带有元数据的文件树</td></tr>
</tbody>
</table>

<h4>Filters 选项卡</h4>
<table>
<thead><tr><th>选项</th><th>描述</th></tr></thead>
<tbody>
<tr><td><b>扩展名预设</b></td><td>在语言集之间快速切换（Python、Web、Golang、Rust、C# 等）</td></tr>
<tr><td><b>扩展名</b></td><td>自定义文件扩展名白名单</td></tr>
<tr><td><b>忽略路径</b></td><td>跳过文件夹/文件（node_modules、.git、build、dist 等）</td></tr>
<tr><td>☑ Include file tree</td><td>在提示词开头添加文件夹结构</td></tr>
<tr><td>☑ Include dependency map</td><td>基于 AST 的导入分析（Python/JS/TS）</td></tr>
<tr><td>☑ Include Mermaid graph</td><td>Mermaid 格式的架构图</td></tr>
</tbody>
</table>

<p>💡 <b>保存自定义预设：</b>配置过滤器，点击 💾，输入名称。</p>

<h4>Prompts 选项卡</h4>
<table>
<thead><tr><th>选项</th><th>描述</th></tr></thead>
<tbody>
<tr><td><b>提示词预设</b></td><td>快速更改系统提示词（Code Review、Bug Hunter、Refactoring 等）</td></tr>
<tr><td><b>系统提示词</b></td><td>自定义提示词——作为系统上下文发送给 LLM</td></tr>
<tr><td><b>🧩 应用 JSON 补丁</b></td><td>粘贴 LLM JSON 响应——预览差异并应用到磁盘</td></tr>
</tbody>
</table>

<p><b>使用 JSON 补丁：</b></p>
<ol>
<li>要求 LLM 返回 JSON 数组：<code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>粘贴 JSON，点击 <b>"Next"</b> → <b>Safety Diff Viewer</b> 打开</li>
<li>勾选/取消勾选文件，可选点击 <b>"🤖 Check via LLM"</b></li>
<li>点击 <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. 输出格式设置</h3>
<table>
<thead><tr><th>选项</th><th>描述</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>去除空白字符和空行</td></tr>
<tr><td>☑ No Comments</td><td>删除所有注释</td></tr>
<tr><td>☑ No Secrets</td><td>隐藏 API 密钥、密码、令牌</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>删除函数体</b>——最大化 Token 节省</td></tr>
<tr><td>格式</td><td>Markdown、XML、Plain、JSONL Chunks、Custom（Jinja2）</td></tr>
<tr><td>📁 模板</td><td>Jinja2 模板选择器</td></tr>
</tbody>
</table>

<p><b>骨架模式：</b>删除函数实现（<code>def func_name(...):  # ... 实现 ...</code>），保留所有类——让 LLM 用最少的 Token 理解大型项目。</p>

<h3>5. 操作按钮</h3>
<table>
<thead><tr><th>按钮</th><th>操作</th></tr></thead>
<tbody>
<tr><td>👀 预览</td><td><b>高级预览对话框</b>——"最终提示词"+"前后对比"选项卡</td></tr>
<tr><td>📋 复制到剪贴板</td><td>复制结果——粘贴到 ChatGPT / Claude</td></tr>
<tr><td>🚀 发送到 ChatGPT / Claude</td><td>打开网页聊天并粘贴上下文</td></tr>
<tr><td>💻 在编辑器中打开</td><td>在 VS Code / Cursor 中打开</td></tr>
<tr><td>💾 保存到文件</td><td>将结果保存到磁盘</td></tr>
</tbody>
</table>

<h3>6. 高级预览对话框</h3>
<p><b>"📝 最终提示词"选项卡：</b>文件列表（左侧）+ 完整文本（右侧，带高亮）。全部复制 / 复制单个文件。</p>
<p><b>"🔍 前后对比"选项卡：</b>原始与优化版本之间的彩色差异。计数器：<code>Before: 1500 → After: 300 (80%)</code>。</p>

<h3>7. LLM 与操作系统</h3>
<table>
<thead><tr><th colspan="2">LLM 检查器</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>应用前自动进行 LLM 补丁验证</td></tr>
<tr><td>URL / Key / Model</td><td>API 端点（默认为 OpenAI）、密钥、模型</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">操作系统集成</th></tr></thead>
<tbody>
<tr><td>安装右键菜单</td><td>右键菜单中的"Open with CodeContext AI"</td></tr>
<tr><td>添加到 PATH</td><td>全局 <code>codecontext</code> CLI 命令</td></tr>
<tr><td>编辑器</td><td><code>code</code>、<code>cursor</code>、<code>idea</code>、<code>vim</code></td></tr>
</tbody>
</table>

<h3>8. 主题</h3>
<ul>
<li><b>主题：</b>Apple、Modern — <b>模式：</b>浅色 / 深色</li>
<li>📂 打开主题文件夹 / ➕ 导入主题 (.json)</li>
</ul>

<h3>9. 📊 Token 分析</h3>
<p>表格：文件路径、Token 数（tiktoken）、压缩率、节省百分比、模型成本。</p>

<h3>10. 🎛️ UI 自定义 (v1.14+)</h3>
<p>点击版本号旁的 <b>⚙</b>——"Interface Settings (Premiere Pro style)"对话框。开关选项卡（Sources、Filters、Prompts、LLM & OS、Themes）和操作按钮（Preview、Clipboard、ChatGPT、Editor、File）。</p>

<h3>11. 命令面板</h3>
<p><code>Ctrl+Shift+P</code>——无需鼠标即可访问所有操作。</p>

<hr>

<h2>💻 CLI 模式</h2>
<pre>python main.py --cli --path /项目/路径 [选项]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>参数</th><th>类型</th><th>描述</th><th>示例</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>标志</td><td>CLI 模式（无 GUI）</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>列表</td><td>项目路径</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>字符串</td><td>文件扩展名</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>字符串</td><td>忽略的路径</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>枚举</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>枚举</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>标志</td><td>启用压缩</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>标志</td><td>删除注释</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>标志</td><td>隐藏机密</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>标志</td><td>骨架模式</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>字符串</td><td>输出文件</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>标志</td><td>输出到控制台</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>标志</td><td>仅 Git 更改</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>标志</td><td>遵守 .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>标志</td><td>文件树</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>标志</td><td>Mermaid 图</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>标志</td><td>依赖关系图</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>字符串</td><td>LLM JSON 补丁</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>字符串</td><td>Jinja2 模板</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>字符串</td><td>自定义系统提示词</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>示例</h3>
<pre># 最小化运行
python main.py --cli --path ./myapp --stdout

# 完整 XML 分析
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git 差异
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON 补丁
python main.py --cli --path ./myapp --patch llm_response.json

# 自定义 Jinja2 模板
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid 图表
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# 多个路径
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ 技术栈</h2>
<table>
<thead><tr><th>组件</th><th>技术</th></tr></thead>
<tbody>
<tr><td>语言</td><td>Python 3.10+</td></tr>
<tr><td>GUI 框架</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>架构</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Token 化</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>模板引擎</td><td>jinja2（11 个内置模板）</td></tr>
<tr><td>AST 解析器</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>分发</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ 路线图</h2>
<ul>
<li>🍎 macOS Finder 右键菜单</li>
<li>🤖 直接 OpenAI/Anthropic API 集成</li>
<li>🏛️ 六边形架构分析</li>
<li>🔌 插件系统</li>
<li>🌐 应用内 i18n</li>
</ul>

<hr>

<h2>👨‍💻 团队</h2>
<p><b>开发者：</b>mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs 提交至 GitHub</p>

<hr>

<h2>🤝 贡献指南</h2>
<ol>
<li>Fork 本仓库</li>
<li>分支：<code>git checkout -b feature/AmazingFeature</code></li>
<li>提交：<code>git commit -m 'Add AmazingFeature'</code></li>
<li>推送：<code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>遵循 SOLID 原则（参见 <code>docs/ARCHITECTURE.md</code>）。</p>

<hr>

<h2>📄 许可证</h2>
<p>MIT。详情请参阅 <code>LICENSE</code>。</p>
