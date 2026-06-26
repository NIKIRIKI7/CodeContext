<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI駆動のコードベース分析＆プロンプト準備ツール**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.28.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟概要</h2>

<p><b>CodeContext AI</b>は、コードベースを大規模言語モデル（LLM）で使用するために準備するための強力なデスクトップツールです。プロジェクトフォルダをスキャンし、構造を分析し、依存関係グラフを構築し、トークン消費とアーキテクチャの明確さに最適化された単一の完全に構造化されたプロンプトを生成します。</p>

<h3>❓なぜ？</h3>
<p>AIを扱う際、開発者はコンテキストウィンドウのトークン制限に直面します — コードを部分的にコピーするとLLMはアーキテクチャの一貫性を「失います」。<b>CodeContext AIがこれを解決します</b>：数クリックでプロジェクト全体を1つの構造化されたプロンプトにまとめ、トークンを最大80%節約できます。</p>

<hr>

<h2>🚀機能</h2>

<table>
<thead><tr><th>機能</th><th>CodeContext AI</th><th>手動</th></tr></thead>
<tbody>
<tr><td>🗜️ 最小化＋スケルトン</td><td><b>最大80%</b>のトークン削減</td><td>手動コピーペースト</td></tr>
<tr><td>🧩 LLM Patcher</td><td>JSONパッチのプレビュー＆適用</td><td>利用不可</td></tr>
<tr><td>✅ LLM Checker</td><td>保存前にコードを自動検証</td><td>利用不可</td></tr>
<tr><td>🔗 AST依存関係グラフ</td><td>Python、JS/TS、Vue</td><td>ファイル一覧のみ</td></tr>
<tr><td>🖱️ コンテキストメニュー</td><td>Windows / Linux / macOS</td><td>なし</td></tr>
<tr><td>🎨 テーマ</td><td>Apple、Modern、カスタムJSON</td><td>固定UI</td></tr>
<tr><td>⚙️ UIカスタマイズ（v1.14+）</td><td>Premiere Pro風</td><td>固定UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15言語、システム自動検出</td><td>単一言語</td></tr>
<tr><td>♻️ 重複削除 (v1.23+)</td><td>同じ内容のファイルを検出してスキップ</td><td>手動チェック</td></tr>
<tr><td>⚡ 強制的最小化 (v1.23+)</td><td>追加圧縮 — 各行の末尾の空白を削除</td><td>手動削除</td></tr>
<tr><td>📌 チェックポイント (v1.23+)</td><td>デバッグ用のスナップショットを事前/事後保存</td><td>利用不可</td></tr>
<tr><td>👁️ 自動監視 (v1.23+)</td><td>ファイルを監視し、変更時に再処理</td><td>利用不可</td></tr>
<tr><td>🔌 プラグインシステム (v1.25+)</td><td>Pythonプラグインで拡張 — カスタムタブ、アクション、i18n</td><td>利用不可</td></tr>
<tr><td>🚦 CI/CD 統合</td><td>GitHub Actions と GitLab CI — <code>--git-base</code> による PR コンテキストの自動生成</td><td>Not available</td></tr>
<tr><td>🌳 Monorepo Support (v1.25+)</td><td>Lerna, NX, Turborepo, pnpm workspaces — cross-package imports, root config discovery</td><td>Not available</td></tr>
</tbody>
</table>

<hr>

<h2>📥インストール</h2>

<p><b>前提条件：</b>Python 3.10+、Git</p>

<pre>git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext
python -m venv venv
# Windows：
venv\Scripts\activate
# Linux/macOS：
source venv/bin/activate
pip install -r requirements.txt</pre>

<h3>PyPI (pip)</h3>
<pre>pip install codecontext-ai</pre>

<pre># 起動：
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux（AUR）</h3>
<table>
<thead><tr><th>操作</th><th>コマンド</th></tr></thead>
<tbody>
<tr><td>インストール</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>検索</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>更新</td><td><code>yay -Syu</code></td></tr>
<tr><td>削除</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p><b>yay</b>がインストールされていない場合：</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>代替：<code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUIモード</h2>
<pre>python main.py</pre>

<h3>1. インターフェース概要</h3>
<p>ウィンドウは3つのゾーンに分かれています：</p>
<ul>
<li><b>左サイドバー（タブ）</b> — スキャン設定、フィルター、プロンプト、LLM設定、テーマ</li>
<li><b>中央エリア</b> — フォルダ一覧、ファイルツリー、トークン分析</li>
<li><b>上部アクションバー</b> — 最小化/コメントなし/スケルトンの切り替え、出力形式、アクションボタン</li>
</ul>

<h3>2. プロジェクトの追加</h3>
<table>
<thead><tr><th>操作</th><th>方法</th></tr></thead>
<tbody>
<tr><td>ドラッグ＆ドロップ</td><td>プロジェクトフォルダをウィンドウにドラッグするだけ</td></tr>
<tr><td>参照ダイアログ</td><td><b>ソース</b>タブの「+ PCフォルダ」をクリック</td></tr>
<tr><td>GitHubリポジトリ</td><td>「+ GitHub / PR」をクリック — リポジトリまたはプルリクエストのURLを貼り付け</td></tr>
<tr><td>設定を保存</td><td>「💾 設定を保存」をクリック — <code>.codecontextrc</code>を作成</td></tr>
</tbody>
</table>

<p><b>GitHub読み込みモード：</b></p>
<ul>
<li><b>永続的に保存</b> — ディスク上のフォルダにクローン</li>
<li><b>一時的</b> — 一時フォルダにクローン（アプリ終了時に削除）</li>
</ul>

<h3>3. スキャン設定</h3>

<h4>ソースタブ</h4>
<table>
<thead><tr><th>オプション</th><th>説明</th></tr></thead>
<tbody>
<tr><td>☑ Git変更のみ</td><td>最後のコミットで変更されたファイルのみを含める</td></tr>
<tr><td>☑ .gitignoreを尊重</td><td><code>.gitignore</code>からファイルを自動除外</td></tr>
<tr><td>🔍 ファイルをスキャン</td><td>メタデータ付きのファイルツリーを構築</td></tr>
</tbody>
</table>

<h4>フィルタタブ</h4>
<table>
<thead><tr><th>オプション</th><th>説明</th></tr></thead>
<tbody>
<tr><td><b>拡張子プリセット</b></td><td>言語セット間のクイックスイッチ（Python、Web、Golang、Rust、C#など）</td></tr>
<tr><td><b>拡張子</b></td><td>カスタムファイル拡張子のホワイトリスト</td></tr>
<tr><td><b>無視するパス</b></td><td>フォルダ/ファイルをスキップ（node_modules、.git、build、distなど）</td></tr>
<tr><td>☑ ファイルツリーを含める</td><td>プロンプトの前にフォルダ構造を追加</td></tr>
<tr><td>☑ 依存関係マップを含める</td><td>Python/JS/TSのASTベースのインポート分析</td></tr>
<tr><td>☑ Mermaidグラフを含める</td><td>Mermaid形式のアーキテクチャ図</td></tr>
</tbody>
</table>

<p>💡 <b>カスタムプリセットの保存：</b>フィルタを設定し、💾をクリック、名前を入力します。</p>

<h4>プロンプトタブ</h4>
<table>
<thead><tr><th>オプション</th><th>説明</th></tr></thead>
<tbody>
<tr><td><b>プロンプトプリセット</b></td><td>システムプロンプトのクイック変更（コードレビュー、バグハンター、リファクタリングなど）</td></tr>
<tr><td><b>システムプロンプト</b></td><td>カスタムプロンプト — システムコンテキストとしてLLMに送信</td></tr>
<tr><td><b>🧩 JSONパッチを適用</b></td><td>LLMのJSONレスポンスを貼り付け — diffをプレビューしてディスクに適用</td></tr>
</tbody>
</table>

<p><b>JSONパッチの使用：</b></p>
<ol>
<li>LLMにJSON配列をリクエスト：<code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>JSONを貼り付け、<b>"次へ"</b>をクリック → <b>セーフティDiffビューア</b>が開く</li>
<li>ファイルをチェック/アンチェック、オプションで<b>"🤖 LLMで確認"</b>をクリック</li>
<li><b>"💾 選択したものをディスクに保存"</b>をクリック</li>
</ol>

<h3>4. 出力形式設定</h3>
<table>
<thead><tr><th>オプション</th><th>説明</th></tr></thead>
<tbody>
<tr><td>☑ 最小化</td><td>空白と空行を削除</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — 追加圧縮 — 各行の末尾の空白を削除</td></tr>
<tr><td>☑ コメントなし</td><td>すべてのコメントを削除</td></tr>
<tr><td>☑ シークレットなし</td><td>APIキー、パスワード、トークンをマスク</td></tr>
<tr><td>☑ スケルトン ☠️</td><td><b>関数本体を削除</b> — 最大のトークン節約</td></tr>
<tr><td>☑ Dedup</td><td>同じ内容の重複ファイルを削除します</td></tr>
<tr><td>☑ Checkpoints</td><td>処理途中のチェックポイントを保存します</td></tr>
<tr><td>☑ Auto-Watch</td><td>ファイルを監視し、変更時に再処理</td></tr>
<tr><td>形式</td><td>Markdown、XML、Plain、JSONL Chunks、カスタム（Jinja2）</td></tr>
<tr><td>📁 テンプレート</td><td>Jinja2テンプレートピッカー</td></tr>
</tbody>
</table>

<p><b>スケルトンモード：</b>関数の実装を削除（<code>def func_name(...):  # ... implementation ...</code>）、すべてのクラスを保持 — LLMが最小トークンで大規模プロジェクトを理解できるようにします。</p>


<p><b>Minify vs Aggressive:</b> <b>Minify</b>は前後の空白を削除し空行を取り除きます — どのコードベースでも安全で、可読性に影響を与えずトークンを削減します。<b>Aggressive</b>はさらに各行の末尾の空白を除去して最大限圧縮します。限られたコンテキストウィンドウにより多くのコードを含める必要がある場合に両方を組み合わせてください。</p>

<p><b>Dedup:</b> プロジェクト内で同じ内容のファイルを自動検出し、出力から重複を除外します — LLMが同じコードを2回見てトークンを無駄にするのを防ぎます。</p>

<p><b>Checkpoints:</b> 各パイプラインステージ（クリーンアップ前、ミニファイ後など）の中間結果を<code>checkpoints/</code>フォルダに保存します。各処理ステップの動作をデバッグしたり、出力を並べて比較するのに便利です。</p>

<p><b>Auto-Watch:</b> OSファイルウォッチャーを使用してプロジェクトファイルの変更を監視します。ファイルが保存されるとパイプラインが自動的に再実行され、継続的なプロンプト更新が必要なアクティブ開発中に最適です。</p>
<h3>5. アクションボタン</h3>
<table>
<thead><tr><th>ボタン</th><th>アクション</th></tr></thead>
<tbody>
<tr><td>👀 プレビュー</td><td><b>高度なプレビューダイアログ</b> — 「最終プロンプト」+「比較」タブ</td></tr>
<tr><td>📋 クリップボードにコピー</td><td>結果をコピー — ChatGPT / Claudeに貼り付け</td></tr>
<tr><td>🚀 ChatGPT / Claudeに送信</td><td>Webチャットを開いてコンテキストを貼り付け</td></tr>
<tr><td>💻 エディタで開く</td><td>VS Code / Cursorで開く</td></tr>
<tr><td>💾 ファイルに保存</td><td>結果をディスクに保存</td></tr>
</tbody>
</table>

<h3>6. 高度なプレビューダイアログ</h3>
<p><b>「📝 最終プロンプト」タブ：</b>ファイル一覧（左）+ ハイライト付き全文（右）。すべてコピー / ファイルをコピー。</p>
<p><b>「🔍 比較」タブ：</b>オリジナルと最適化の間の色付きdiff。カウンター：<code>Before: 1500 → After: 300 (80%)</code>。</p>

<h3>7. LLMとOS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ 検証を有効化</td><td>適用前にLLMパッチを自動検証</td></tr>
<tr><td>URL / キー / モデル</td><td>APIエンドポイント（デフォルトOpenAI）、キー、モデル</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">OS統合</th></tr></thead>
<tbody>
<tr><td>コンテキストメニューをインストール</td><td>右クリックメニューに「CodeContext AIで開く」</td></tr>
<tr><td>PATHに追加</td><td>グローバルな<code>codecontext</code> CLIコマンド</td></tr>
<tr><td>エディタ</td><td><code>code</code>、<code>cursor</code>、<code>idea</code>、<code>vim</code></td></tr>
</tbody>
</table>

<h3>8. テーマ</h3>
<ul>
<li><b>テーマ：</b>Apple、Modern — <b>モード：</b>ライト / ダーク</li>
<li>📂 テーマフォルダを開く / ➕ テーマをインポート（.json）</li>
</ul>

<h3>9. 📊 トークン分析</h3>
<p>テーブル：ファイルパス、トークン（tiktoken）、圧縮率、節約%、モデルのコスト。</p>

<h3>10. 🎛️ UIカスタマイズ（v1.14+）</h3>
<p>バージョンの横にある<b>⚙</b>をクリック — 「インターフェース設定（Premiere Pro風）」ダイアログ。タブ（ソース、フィルタ、プロンプト、LLMとOS、テーマ）とアクションボタン（プレビュー、クリップボード、ChatGPT、エディタ、ファイル）を切り替え。</p>

<h3>11. コマンドパレット</h3>
<p><code>Ctrl+Shift+P</code> — マウスを使わずにすべてのアクションにアクセス。</p>

<h3>12. 🔌 プラグインシステム (v1.25+)</h3>
<p><b>CodeContext AI</b>は<b>Pythonプラグインシステム</b>をサポートしており、カスタム機能でアプリを拡張できます。</p>

<h4>📁 プラグイン構造</h4>
<pre>my_plugin/
├── manifest.json          # プラグインメタデータ
├── requirements.txt       # （オプション）pip依存関係
├── locales/
│   ├── en.json            # 英語翻訳
│   └── ru.json            # ロシア語翻訳
└── plugin.py              # エントリポイント</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "何か便利なことをします",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py（例）</h4>
<pre>from src.services.plugin_manager import IPlugin

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, controller: MainController) -> None:
        # locales/フォルダの翻訳は自動的に読み込まれます
        # サイドバーにタブを登録
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("プラグインからのこんにちは！")
        )
        # アクションボタンを登録
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("プラグインアクションがクリックされました")
        )
        api.add_log("マイプラグインが初期化されました")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 セキュリティ</h4>
<ul>
<li>プラグインは<b>Pythonへの完全アクセス</b>を持ちます — 信頼できるソースからのみインストールしてください</li>
<li>初回ロード時、セキュリティダイアログでプラグインを有効にする前に承認を求めます</li>
<li><code>requirements.txt</code>が存在する場合、ロード前にpipインストールログが表示されます</li>
<li>承認済みプラグインは設定に記憶されます（<code>approved_plugins</code>）</li>
</ul>

<h4>🛠 Plugin API</h4>
<table>
<thead><tr><th>プロパティ / メソッド</th><th>説明</th></tr></thead>
<tbody>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>左サイドバーにタブを追加</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>「プラグイン 🔽」ドロップダウンにボタンを追加</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>アプリログパネルに書き込み</td></tr>
</tbody>
</table>

<h4>⚙️ 表示設定</h4>
<p>プラグインタブとアクションボタンは<b>⚙ UIカスタマイズ</b>からオン/オフを切り替えられます — 組み込みのタブ/アクションと一緒に独自のチェックボックスが表示されます。</p>

<hr>

<h2>💻 CLIモード</h2>
<pre>python main.py --cli --path /path/to/project [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>パラメータ</th><th>型</th><th>説明</th><th>例</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLIモード（GUIなし）</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>プロジェクトパス</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>拡張子</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>無視するパス</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>最小化を有効にする</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>コメントを削除</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>シークレットをマスク</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>スケルトンモード</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>出力ファイル</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>stdoutに出力</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Git変更のみ</td><td><code>--git</code></td></tr>
<tr><td><code>--git-base</code></td><td>str</td><td>CI/CD で git diff のベースブランチ</td><td><code>--git-base origin/main</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>.gitignoreを尊重</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>ファイルツリー</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaidグラフ</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>依存関係マップ</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSONパッチ</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2テンプレート</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>カスタムシステムプロンプト</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>例</h3>
<pre># 最小実行
python main.py --cli --path ./myapp --stdout

# XMLによる完全分析
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSONパッチ
python main.py --cli --path ./myapp --patch llm_response.json

# カスタムJinja2テンプレート
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid図
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# 複数パス
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml

# CI/CD — ベースブランチとの差分
python main.py --cli --path . --git --git-base origin/main --minify true --stdout</pre>

<hr>

<h2>🏗️ 技術スタック</h2>
<table>
<thead><tr><th>コンポーネント</th><th>技術</th></tr></thead>
<tbody>
<tr><td>言語</td><td>Python 3.10+</td></tr>
<tr><td>GUIフレームワーク</td><td>PySide6（Qt 6）</td></tr>
<tr><td>アーキテクチャ</td><td>Clean Architecture</td></tr>
<tr><td>トークン化</td><td>tiktoken（OpenAI）</td></tr>
<tr><td>テンプレーティング</td><td>jinja2（11組み込み）</td></tr>
<tr><td>ASTパーサー</td><td>ast（Python）、tree-sitter（JS/TS/Go/Rust）</td></tr>
<tr><td>配布</td><td>PyInstaller、AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ ロードマップ</h2>
<ul>
<li>📚 <b>RAG（検索拡張生成）モード</b> — ローカルベクトルDB（Chroma/FAISS）を使用した大規模コードベースのインデックス化。</li>
<li>🚫 <b>詳細な.gitignore解析</b> — ネストされた<code>.gitignore</code>ファイルとグローバルな<code>~/.gitignore</code>のサポート。</li>
<li>☁️ <b>クラウド同期</b> — GitHub Gistsを介したプリセットと設定の同期。</li>
<li>🌳 <b>マルチルートワークスペース</b> — モノレポ（Lerna、NX、Turborepo）のサポート向上。</li>
<li>🚀 <b>CI/CDパイプライン</b> — PRコンテキストを自動生成するGitHub ActionsおよびGitLab CIプラグイン。</li>
<li>🤖 <b>OpenAI/Anthropic API直接統合</b> — プロンプト生成から直接出力への完全なブリッジ。</li>
<li>🔌 プラグインシステム ✅</li>
</ul>

<hr>

<h2>👨‍💻 チーム</h2>
<p><b>開発者：</b>mcniki · <a href="https://vk.com/gor_niki">VK：gor_niki</a> · Issues & PRs on GitHub</p>

<hr>

<h2>🤝 コントリビューション</h2>
<ol>
<li>リポジトリをフォーク</li>
<li>ブランチ：<code>git checkout -b feature/AmazingFeature</code></li>
<li>コミット：<code>git commit -m 'Add AmazingFeature'</code></li>
<li>プッシュ：<code>git push origin feature/AmazingFeature</code></li>
<li>プルリクエスト</li>
</ol>
<p>SOLID原則に従ってください（<code>../docs/ARCHITECTURE.md</code>参照）。</p>

<hr>

<h2>📄 ライセンス</h2>
<p>MIT。詳細は<code>../LICENSE</code>を参照してください。</p>
