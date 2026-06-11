<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="../assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI 기반 코드베이스 분석 및 프롬프트 준비 도구**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.18.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 소개</h2>

<p><b>CodeContext AI</b>는 코드베이스를 대규모 언어 모델(LLM)과 함께 사용할 수 있도록 준비하는 강력한 데스크톱 도구입니다. 프로젝트 폴더를 스캔하고, 구조를 분석하며, 종속성 그래프를 구축하고, 토큰 소비와 아키텍처 명확성에 최적화된 단일의 완벽하게 구조화된 프롬프트를 생성합니다.</p>

<h3>❓ 왜?</h3>
<p>AI로 작업할 때 개발자는 컨텍스트 윈도우 토큰 제한에 직면합니다 — 코드를 부분적으로 복사하면 LLM이 아키텍처 일관성을 "잃습니다". <b>CodeContext AI가 이를 해결합니다</b>: 몇 번의 클릭으로 전체 프로젝트를 하나의 구조화된 프롬프트로 수집하여 최대 80%의 토큰을 절약하세요.</p>

<hr>

<h2>🚀 기능</h2>

<table>
<thead><tr><th>기능</th><th>CodeContext AI</th><th>수동</th></tr></thead>
<tbody>
<tr><td>🗜️ 최소화 + 스켈레톤</td><td><b>최대 80%</b> 토큰 감소</td><td>수동 복사-붙여넣기</td></tr>
<tr><td>🧩 LLM Patcher</td><td>JSON 패치 미리보기 및 적용</td><td>사용 불가</td></tr>
<tr><td>✅ LLM Checker</td><td>저장 전 코드 자동 검증</td><td>사용 불가</td></tr>
<tr><td>🔗 AST 종속성 그래프</td><td>Python, JS/TS, Vue</td><td>파일 목록만</td></tr>
<tr><td>🖱️ 컨텍스트 메뉴</td><td>Windows / Linux</td><td>없음</td></tr>
<tr><td>🎨 테마</td><td>Apple, Modern, 사용자 지정 JSON</td><td>고정 UI</td></tr>
<tr><td>⚙️ UI 사용자 지정 (v1.14+)</td><td>Premiere Pro 스타일</td><td>고정 UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 languages, system auto-detect</td><td>Single language</td></tr>
</tbody>
</table>

<hr>

<h2>📥 설치</h2>

<p><b>사전 요구 사항:</b> Python 3.10+, Git</p>

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
<thead><tr><th>작업</th><th>명령어</th></tr></thead>
<tbody>
<tr><td>설치</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>검색</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>업데이트</td><td><code>yay -Syu</code></td></tr>
<tr><td>제거</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p><b>yay</b>가 설치되지 않은 경우:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>대체: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUI 모드</h2>
<pre>python main.py</pre>

<h3>1. 인터페이스 개요</h3>
<p>창은 세 영역으로 나뉩니다:</p>
<ul>
<li><b>왼쪽 사이드바 (탭)</b> — 스캔 설정, 필터, 프롬프트, LLM 설정, 테마</li>
<li><b>중앙 영역</b> — 폴더 목록, 파일 트리, 토큰 분석</li>
<li><b>상단 작업 표시줄</b> — 최소화/주석 없음/스켈레톤 토글, 출력 형식, 작업 버튼</li>
</ul>

<h3>2. 프로젝트 추가</h3>
<table>
<thead><tr><th>작업</th><th>방법</th></tr></thead>
<tbody>
<tr><td>드래그 & 드롭</td><td>프로젝트 폴더를 창으로 끌어다 놓기만 하면 됩니다</td></tr>
<tr><td>찾아보기 대화상자</td><td><b>소스</b> 탭에서 "+ PC 폴더" 클릭</td></tr>
<tr><td>GitHub 저장소</td><td>"+ GitHub / PR" 클릭 — 저장소 또는 풀 리퀘스트 URL 붙여넣기</td></tr>
<tr><td>설정 저장</td><td>"💾 설정 저장" 클릭 — <code>.codecontextrc</code> 생성</td></tr>
</tbody>
</table>

<p><b>GitHub 로딩 모드:</b></p>
<ul>
<li><b>영구 저장</b> — 디스크의 폴더에 클론</li>
<li><b>임시</b> — 임시 폴더에 클론 (앱 종료 시 삭제)</li>
</ul>

<h3>3. 스캔 설정</h3>

<h4>소스 탭</h4>
<table>
<thead><tr><th>옵션</th><th>설명</th></tr></thead>
<tbody>
<tr><td>☑ Git 변경 사항만</td><td>마지막 커밋에서 변경된 파일만 포함</td></tr>
<tr><td>☑ .gitignore 준수</td><td><code>.gitignore</code>에서 파일 자동 제외</td></tr>
<tr><td>🔍 파일 스캔</td><td>메타데이터가 포함된 파일 트리 구축</td></tr>
</tbody>
</table>

<h4>필터 탭</h4>
<table>
<thead><tr><th>옵션</th><th>설명</th></tr></thead>
<tbody>
<tr><td><b>확장자 프리셋</b></td><td>언어 세트 간 빠른 전환 (Python, Web, Golang, Rust, C# 등)</td></tr>
<tr><td><b>확장자</b></td><td>사용자 지정 파일 확장자 허용 목록</td></tr>
<tr><td><b>무시할 경로</b></td><td>폴더/파일 건너뛰기 (node_modules, .git, build, dist 등)</td></tr>
<tr><td>☑ 파일 트리 포함</td><td>프롬프트 앞에 폴더 구조 추가</td></tr>
<tr><td>☑ 종속성 맵 포함</td><td>Python/JS/TS용 AST 기반 가져오기 분석</td></tr>
<tr><td>☑ Mermaid 그래프 포함</td><td>Mermaid 형식의 아키텍처 다이어그램</td></tr>
</tbody>
</table>

<p>💡 <b>사용자 지정 프리셋 저장:</b> 필터를 구성하고, 💾을 클릭하고, 이름을 입력하세요.</p>

<h4>프롬프트 탭</h4>
<table>
<thead><tr><th>옵션</th><th>설명</th></tr></thead>
<tbody>
<tr><td><b>프롬프트 프리셋</b></td><td>시스템 프롬프트 빠른 변경 (코드 리뷰, 버그 헌터, 리팩토링 등)</td></tr>
<tr><td><b>시스템 프롬프트</b></td><td>사용자 지정 프롬프트 — 시스템 컨텍스트로 LLM에 전송</td></tr>
<tr><td><b>🧩 JSON 패치 적용</b></td><td>LLM JSON 응답 붙여넣기 — diff 미리보기 및 디스크에 적용</td></tr>
</tbody>
</table>

<p><b>JSON 패치 사용:</b></p>
<ol>
<li>LLM에 JSON 배열 요청: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>JSON 붙여넣기, <b>"다음"</b> 클릭 → <b>안전 Diff 뷰어</b> 열림</li>
<li>파일 체크/언체크, 선택적으로 <b>"🤖 LLM으로 확인"</b> 클릭</li>
<li><b>"💾 선택 항목을 디스크에 저장"</b> 클릭</li>
</ol>

<h3>4. 출력 형식 설정</h3>
<table>
<thead><tr><th>옵션</th><th>설명</th></tr></thead>
<tbody>
<tr><td>☑ 최소화</td><td>공백과 빈 줄 제거</td></tr>
<tr><td>☑ 주석 없음</td><td>모든 주석 제거</td></tr>
<tr><td>☑ 비밀 정보 없음</td><td>API 키, 비밀번호, 토큰 마스킹</td></tr>
<tr><td>☑ 스켈레톤 ☠️</td><td><b>함수 본문 제거</b> — 최대 토큰 절약</td></tr>
<tr><td>형식</td><td>Markdown, XML, Plain, JSONL Chunks, 사용자 지정 (Jinja2)</td></tr>
<tr><td>📁 템플릿</td><td>Jinja2 템플릿 선택기</td></tr>
</tbody>
</table>

<p><b>스켈레톤 모드:</b> 함수 구현 제거 (<code>def func_name(...):  # ... implementation ...</code>), 모든 클래스 유지 — LLM이 최소 토큰으로 대규모 프로젝트를 이해할 수 있게 합니다.</p>

<h3>5. 작업 버튼</h3>
<table>
<thead><tr><th>버튼</th><th>작업</th></tr></thead>
<tbody>
<tr><td>👀 미리보기</td><td><b>고급 미리보기 대화상자</b> — "최종 프롬프트" + "비교" 탭</td></tr>
<tr><td>📋 클립보드에 복사</td><td>결과 복사 — ChatGPT / Claude에 붙여넣기</td></tr>
<tr><td>🚀 ChatGPT / Claude로 보내기</td><td>웹 채팅 열고 컨텍스트 붙여넣기</td></tr>
<tr><td>💻 편집기에서 열기</td><td>VS Code / Cursor에서 열기</td></tr>
<tr><td>💾 파일에 저장</td><td>결과를 디스크에 저장</td></tr>
</tbody>
</table>

<h3>6. 고급 미리보기 대화상자</h3>
<p><b>"📝 최종 프롬프트" 탭:</b> 파일 목록 (왼쪽) + 하이라이트가 있는 전체 텍스트 (오른쪽). 모두 복사 / 파일 복사.</p>
<p><b>"🔍 비교" 탭:</b> 원본과 최적화 간 색상 diff. 카운터: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM 및 OS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ 검증 활성화</td><td>적용 전 LLM 패치 자동 검증</td></tr>
<tr><td>URL / 키 / 모델</td><td>API 엔드포인트 (기본 OpenAI), 키, 모델</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">OS 통합</th></tr></thead>
<tbody>
<tr><td>컨텍스트 메뉴 설치</td><td>마우스 오른쪽 버튼 메뉴에 "CodeContext AI로 열기"</td></tr>
<tr><td>PATH에 추가</td><td>전역 <code>codecontext</code> CLI 명령어</td></tr>
<tr><td>편집기</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. 테마</h3>
<ul>
<li><b>테마:</b> Apple, Modern — <b>모드:</b> 라이트 / 다크</li>
<li>📂 테마 폴더 열기 / ➕ 테마 가져오기 (.json)</li>
</ul>

<h3>9. 📊 토큰 분석</h3>
<p>표: 파일 경로, 토큰 (tiktoken), 압축률, 절약 %, 모델 비용.</p>

<h3>10. 🎛️ UI 사용자 지정 (v1.14+)</h3>
<p>버전 옆 <b>⚙</b> 클릭 — "인터페이스 설정 (Premiere Pro 스타일)" 대화상자. 탭 (소스, 필터, 프롬프트, LLM 및 OS, 테마) 및 작업 버튼 (미리보기, 클립보드, ChatGPT, 편집기, 파일) 전환.</p>

<h3>11. 명령 팔레트</h3>
<p><code>Ctrl+Shift+P</code> — 마우스 없이 모든 작업에 접근.</p>

<hr>

<h2>💻 CLI 모드</h2>
<pre>python main.py --cli --path /path/to/project [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>매개변수</th><th>유형</th><th>설명</th><th>예시</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLI 모드 (GUI 없음)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>프로젝트 경로</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>확장자</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>무시할 경로</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>최소화 활성화</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>주석 제거</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>비밀 정보 마스킹</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>스켈레톤 모드</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>출력 파일</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>stdout으로 출력</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Git 변경 사항만</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>.gitignore 준수</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>파일 트리</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaid 그래프</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>종속성 맵</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSON 패치</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2 템플릿</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>사용자 지정 시스템 프롬프트</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>예시</h3>
<pre># 최소 실행
python main.py --cli --path ./myapp --stdout

# XML로 전체 분석
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON 패치
python main.py --cli --path ./myapp --patch llm_response.json

# 사용자 지정 Jinja2 템플릿
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid 다이어그램
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# 여러 경로
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ 기술 스택</h2>
<table>
<thead><tr><th>구성 요소</th><th>기술</th></tr></thead>
<tbody>
<tr><td>언어</td><td>Python 3.10+</td></tr>
<tr><td>GUI 프레임워크</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>아키텍처</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>토큰화</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>템플릿</td><td>jinja2 (11 내장)</td></tr>
<tr><td>AST 파서</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>배포</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ 로드맵</h2>
<ul>
<li>🍎 macOS Finder 컨텍스트 메뉴</li>
<li>🤖 OpenAI/Anthropic API 직접 통합</li>
<li>🏛️ 헥사고날 아키텍처 분석</li>
<li>🔌 플러그인 시스템</li>
<li>🌐 앱 내 i18n</li>
</ul>

<hr>

<h2>👨‍💻 팀</h2>
<p><b>개발자:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · GitHub에서 Issues & PRs</p>

<hr>

<h2>🤝 기여하기</h2>
<ol>
<li>저장소 포크</li>
<li>브랜치: <code>git checkout -b feature/AmazingFeature</code></li>
<li>커밋: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>푸시: <code>git push origin feature/AmazingFeature</code></li>
<li>풀 리퀘스트</li>
</ol>
<p>SOLID 원칙을 따르세요 (<code>../docs/ARCHITECTURE.md</code> 참조).</p>

<hr>

<h2>📄 라이선스</h2>
<p>MIT. 자세한 내용은 <code>../LICENSE</code>를 참조하세요.</p>
