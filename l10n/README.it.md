<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**Strumento di analisi del codebase e preparazione di prompt basato sull'IA**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.25.1-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 Sul progetto</h2>

<p><b>CodeContext AI</b> è un potente strumento desktop per preparare il vostro codebase a lavorare con i Grandi Modelli Linguistici (LLM). Scansiona le cartelle del progetto, analizza la struttura, costruisce grafici delle dipendenze e genera un unico prompt perfettamente strutturato — ottimizzato per il consumo di token e la chiarezza architetturale.</p>

<h3>❓ Perché?</h3>
<p>Quando si lavora con l'IA, gli sviluppatori si scontrano con i limiti della finestra di contesto — gli LLM « perdono » la coerenza architetturale quando il codice viene copiato in parti. <b>CodeContext AI risolve questo problema</b>: raccogliete l'intero progetto in un prompt strutturato in pochi clic, risparmiando fino all'80 % dei token.</p>

<hr>

<h2>🚀 Funzionalità</h2>

<table>
<thead><tr><th>Funzionalità</th><th>CodeContext AI</th><th>Manuale</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify + Skeleton</td><td><b>Fino all'80 %</b> di riduzione dei token</td><td>Copia e incolla manuale</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Anteprima e applicazione di patch JSON</td><td>Non disponibile</td></tr>
<tr><td>✅ LLM Checker</td><td>Verifica automatica del codice prima del salvataggio</td><td>Non disponibile</td></tr>
<tr><td>🔗 Grafo delle dipendenze AST</td><td>Python, JS/TS, Vue</td><td>Solo elenco file</td></tr>
<tr><td>🖱️ Menu contestuale</td><td>Windows / Linux / macOS</td><td>Nessuno</td></tr>
<tr><td>🎨 Temi</td><td>Apple, Modern, JSON personalizzato</td><td>Interfaccia fissa</td></tr>
<tr><td>⚙️ Personalizzazione interfaccia (v1.14+)</td><td>Stile Premiere Pro</td><td>Interfaccia fissa</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 lingue, rilevamento automatico</td><td>Lingua singola</td></tr>
<tr><td>♻️ Deduplicazione (v1.23+)</td><td>Rileva e salta file con contenuto identico</td><td>Controllo manuale</td></tr>
<tr><td>⚡ Minificazione aggressiva (v1.23+)</td><td>Compressione extra — elimina spazi finali su ogni riga</td><td>Eliminazione manuale</td></tr>
<tr><td>📌 Checkpoint (v1.23+)</td><td>Salva istantanee prima/dopo per debug</td><td>Non disponibile</td></tr>
<tr><td>👁️ Auto-sorveglianza (v1.23+)</td><td>Monitora file e rielabora al cambiamento</td><td>Non disponibile</td></tr>
<tr><td>🔌 Sistema plugin (v1.25+)</td><td>Estendi con plugin Python — schede, azioni e i18n personalizzate</td><td>Non disponibile</td></tr>
<tr><td>🚦 Integrazione CI/CD</td><td>GitHub Actions e GitLab CI — generazione automatica del contesto PR tramite <code>--git-base</code></td><td>Not available</td></tr>
<tr><td>🌳 Monorepo Support (v1.25+)</td><td>Lerna, NX, Turborepo, pnpm workspaces — cross-package imports, root config discovery</td><td>Not available</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Installazione</h2>

<p><b>Prerequisiti:</b> Python 3.10+, Git</p>

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

<pre># Poi avvia:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Azione</th><th>Comando</th></tr></thead>
<tbody>
<tr><td>Installare</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Cercare</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Aggiornare</td><td><code>yay -Syu</code></td></tr>
<tr><td>Rimuovere</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Se <b>yay</b> non è installato:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternativa: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 Modalità GUI</h2>
<pre>python main.py</pre>

<h3>1. Panoramica dell'interfaccia</h3>
<p>La finestra è divisa in tre zone:</p>
<ul>
<li><b>Barra laterale sinistra (schede)</b> — impostazioni di scansione, filtri, prompt, configurazione LLM, temi</li>
<li><b>Area centrale</b> — elenco cartelle, albero dei file, analisi dei token</li>
<li><b>Barra delle azioni superiore</b> — interruttori Minify/No Comments/Skeleton, formato di output, pulsanti azione</li>
</ul>

<h3>2. Aggiungere un progetto</h3>
<table>
<thead><tr><th>Azione</th><th>Come</th></tr></thead>
<tbody>
<tr><td>Trascinare e rilasciare</td><td>Trascinate una cartella progetto nella finestra</td></tr>
<tr><td>Dialogo di apertura</td><td>Cliccate "+ Папка ПК" sulla scheda <b>Sources</b></td></tr>
<tr><td>Repo GitHub</td><td>Cliccate "+ GitHub / PR" — incollate un URL di repo o Pull Request</td></tr>
<tr><td>Salvare configurazione</td><td>Cliccate "💾 Save config" — crea <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>Modalità di caricamento GitHub:</b></p>
<ul>
<li><b>Salvare permanentemente</b> — clona in una cartella sul disco</li>
<li><b>Temporanea</b> — clona in una cartella temporanea (eliminata alla chiusura dell'app)</li>
</ul>

<h3>3. Configurazione scansione</h3>

<h4>Scheda Sources</h4>
<table>
<thead><tr><th>Opzione</th><th>Descrizione</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>Includere solo i file modificati nell'ultimo commit</td></tr>
<tr><td>☑ Respect .gitignore</td><td>Escludere automaticamente i file da <code>.gitignore</code></td></tr>
<tr><td>🔍 Scan Files</td><td>Costruire l'albero dei file con metadati</td></tr>
</tbody>
</table>

<h4>Scheda Filters</h4>
<table>
<thead><tr><th>Opzione</th><th>Descrizione</th></tr></thead>
<tbody>
<tr><td><b>Preimpostazioni estensioni</b></td><td>Cambio rapido tra set di linguaggi (Python, Web, Golang, Rust, C#, ecc.)</td></tr>
<tr><td><b>Estensioni</b></td><td>Whitelist personalizzata di estensioni file</td></tr>
<tr><td><b>Percorsi ignorati</b></td><td>Saltare cartelle/file (node_modules, .git, build, dist, ecc.)</td></tr>
<tr><td>☑ Include file tree</td><td>Antepone la struttura delle cartelle al prompt</td></tr>
<tr><td>☑ Include dependency map</td><td>Analisi delle importazioni basata su AST per Python/JS/TS</td></tr>
<tr><td>☑ Include Mermaid graph</td><td>Diagramma architetturale in formato Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>Salvare preimpostazioni personalizzate:</b> configurate i filtri, cliccate 💾, inserite un nome.</p>

<h4>Scheda Prompts</h4>
<table>
<thead><tr><th>Opzione</th><th>Descrizione</th></tr></thead>
<tbody>
<tr><td><b>Preimpostazioni prompt</b></td><td>Cambio rapido del prompt di sistema (Code Review, Bug Hunter, Refactoring, ecc.)</td></tr>
<tr><td><b>Prompt di sistema</b></td><td>Prompt personalizzato — inviato al LLM come contesto di sistema</td></tr>
<tr><td><b>🧩 Applica patch JSON</b></td><td>Incollate la risposta JSON del LLM — visualizzate il diff e applicate al disco</td></tr>
</tbody>
</table>

<p><b>Utilizzo delle patch JSON:</b></p>
<ol>
<li>Chiedete al LLM un array JSON: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Incollate il JSON, cliccate <b>"Next"</b> → si apre il <b>Safety Diff Viewer</b></li>
<li>Selezionate/deselezionate i file, opzionalmente cliccate <b>"🤖 Check via LLM"</b></li>
<li>Cliccate <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. Impostazioni formato di output</h3>
<table>
<thead><tr><th>Opzione</th><th>Descrizione</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>Rimuove spazi bianchi e righe vuote</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — Compressione extra — elimina spazi finali su ogni riga</td></tr>
<tr><td>☑ No Comments</td><td>Rimuove tutti i commenti</td></tr>
<tr><td>☑ No Secrets</td><td>Maschera chiavi API, password, token</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Rimuove i corpi delle funzioni</b> — massimo risparmio di token</td></tr>
<tr><td>☑ Dedup</td><td>Rimuove i file duplicati con contenuto identico</td></tr>
<tr><td>☑ Checkpoints</td><td>Salva checkpoint intermedi dell'elaborazione</td></tr>
<tr><td>☑ Auto-Watch</td><td>Monitora file e rielabora al cambiamento</td></tr>
<tr><td>Formato</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Selettore template Jinja2</td></tr>
</tbody>
</table>

<p><b>Modalità Skeleton:</b> rimuove l'implementazione delle funzioni (<code>def func_name(...):  # ... implementazione ...</code>), preserva tutte le classi — permette al LLM di comprendere progetti enormi con token minimi.</p>


<p><b>Minify vs Aggressive:</b> <b>Minify</b> rimuove spazi iniziali/finali e righe vuote — sicuro per qualsiasi codebase, riduce i token senza compromettere la leggibilità. <b>Aggressive</b> aggiunge un passaggio extra che elimina gli spazi finali su ogni riga per la massima compressione. Combinate entrambi quando dovete inserire più codice in una finestra di contesto limitata.</p>

<p><b>Dedup:</b> rileva automaticamente i file con contenuto identico nel progetto ed esclude i duplicati dall'output — impedisce al LLM di vedere lo stesso codice due volte e sprecare token.</p>

<p><b>Checkpoints:</b> salva i risultati intermedi in ogni fase della pipeline (prima della pulizia, dopo la minificazione, ecc.) nella cartella <code>checkpoints/</code>. Utile per eseguire il debug di ogni fase di elaborazione o confrontare gli output affiancati.</p>

<p><b>Auto-Watch:</b> monitors your project files for changes using the OS file watcher. When a file is saved, the pipeline automatically re-runs — ideal during active development when you need continuous prompt updates.</p>
<h3>5. Pulsanti azione</h3>
<table>
<thead><tr><th>Pulsante</th><th>Azione</th></tr></thead>
<tbody>
<tr><td>👀 Preview</td><td><b>Advanced Preview Dialog</b> — schede "Final Prompt" + "Before/After"</td></tr>
<tr><td>📋 Copy to Clipboard</td><td>Copia il risultato — incollate in ChatGPT / Claude</td></tr>
<tr><td>🚀 Send to ChatGPT / Claude</td><td>Apre la chat web e incolla il contesto</td></tr>
<tr><td>💻 Open in Editor</td><td>Apre in VS Code / Cursor</td></tr>
<tr><td>💾 Save to File</td><td>Salva il risultato su disco</td></tr>
</tbody>
</table>

<h3>6. Advanced Preview Dialog</h3>
<p><b>Scheda "📝 Final Prompt":</b> elenco file (sinistra) + testo completo evidenziato (destra). Copy All / Copy File.</p>
<p><b>Scheda "🔍 Before/After":</b> diff colorato tra originale e ottimizzato. Contatore: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM e OS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>Verifica automatica della patch LLM prima dell'applicazione</td></tr>
<tr><td>URL / Key / Model</td><td>Endpoint API (OpenAI predefinito), chiave, modello</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">Integrazione con OS</th></tr></thead>
<tbody>
<tr><td>Installare menu contestuale</td><td>"Open with CodeContext AI" nel menu contestuale</td></tr>
<tr><td>Aggiungere al PATH</td><td>Comando CLI globale <code>codecontext</code></td></tr>
<tr><td>Editor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Temi</h3>
<ul>
<li><b>Tema:</b> Apple, Modern — <b>Modalità:</b> chiaro / scuro</li>
<li>📂 Aprire cartella temi / ➕ Importare tema (.json)</li>
</ul>

<h3>9. 📊 Analisi dei token</h3>
<p>Tabella: percorso file, token (tiktoken), compressione, risparmio %, costo per il modello.</p>

<h3>10. 🎛️ Personalizzazione interfaccia (v1.14+)</h3>
<p>Cliccate <b>⚙</b> accanto alla versione — dialogo "Interface Settings (Premiere Pro style)". Attivate/disattivate schede (Sources, Filters, Prompts, LLM & OS, Themes) e pulsanti azione (Preview, Clipboard, ChatGPT, Editor, File).</p>

<h3>11. Paletta comandi</h3>
<p><code>Ctrl+Shift+P</code> — accesso senza mouse a tutte le azioni.</p>

<h3>12. 🔌 Sistema plugin (v1.25+)</h3>
<p><b>CodeContext AI</b> supporta un <b>sistema di plugin Python</b> che permette di estendere l'app con funzionalità personalizzate.</p>

<h4>📁 Struttura del plugin</h4>
<pre>my_plugin/
├── manifest.json          # Metadati del plugin
├── requirements.txt       # (Opzionale) dipendenze pip
├── locales/
│   ├── en.json            # Traduzioni inglesi
│   └── ru.json            # Traduzioni russe
└── plugin.py              # Punto di ingresso</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Fa qualcosa di utile",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Esempio)</h4>
<pre>from src.api.plugin_api import IPlugin, PluginAPI

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        api.ui.register_sidebar_tab(
            "my_tab", "La Mia Scheda",
            lambda: QLabel("Ciao dal plugin!")
        )
        api.ui.register_action_button(
            "my_action", "La Mia Azione",
            lambda: api.add_log("Azione del plugin cliccata")
        )
        api.add_log("Plugin inizializzato")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Sicurezza</h4>
<ul>
<li>I plugin ottengono <b>accesso completo a Python</b> — installate solo da fonti attendibili</li>
<li>Al primo caricamento, un dialogo di sicurezza chiede la vostra approvazione prima di abilitare un plugin</li>
<li>Se <code>requirements.txt</code> esiste, vedrete un log live di pip install prima del caricamento</li>
<li>I plugin approvati vengono ricordati nelle impostazioni (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 API del Plugin</h4>
<table>
<thead><tr><th>Proprietà / Metodo</th><th>Descrizione</th></tr></thead>
<tbody>
<tr><td><code>api.store</code></td><td>Store Redux di sola lettura (accesso allo stato tramite <code>state.settings.xxx</code>)</td></tr>
<tr><td><code>api.dispatcher</code></td><td>Invia azioni (es. <code>UI_ADD_LOG</code>)</td></tr>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Aggiunge una scheda alla barra laterale sinistra</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Aggiunge un pulsante al menu a discesa "Plugin 🔽"</td></tr>
<tr><td><code>api.add_translations(lang, data)</code></td><td>Aggiunge traduzioni runtime (unite sopra quelle integrate)</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Scrive nel pannello dei log dell'app</td></tr>
</tbody>
</table>

<h4>⚙️ Visibilità</h4>
<p>Le schede e i pulsanti dei plugin possono essere attivati/disattivati tramite <b>⚙ Personalizzazione interfaccia</b> — appaiono accanto a schede/azioni integrate con le proprie caselle di controllo.</p>

<hr>

<h2>💻 Modalità CLI</h2>
<pre>python main.py --cli --path /percorso/al/progetto [opzioni]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parametro</th><th>Tipo</th><th>Descrizione</th><th>Esempio</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>Modalità CLI (senza GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>lista</td><td>Percorso del progetto</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Estensioni</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Percorsi ignorati</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Abilitare minificazione</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Rimuovere commenti</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Mascherare segreti</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Modalità scheletro</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>File di output</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Stampare su stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Solo modifiche Git</td><td><code>--git</code></td></tr>
<tr><td><code>--git-base</code></td><td>str</td><td>Branch base per git diff in CI/CD</td><td><code>--git-base origin/main</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Rispettare .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Albero dei file</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Grafo Mermaid</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Mappa delle dipendenze</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>Patch JSON LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Template Jinja2</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Prompt di sistema personalizzato</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Esempi</h3>
<pre># Esecuzione minima
python main.py --cli --path ./myapp --stdout

# Analisi completa con XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Diff Git
python main.py --cli --path ./myapp --git --gitignore --stdout

# Patch JSON LLM
python main.py --cli --path ./myapp --patch llm_response.json

# Template Jinja2 personalizzato
python main.py --cli --path ./myapp --template my.j2 --stdout

# Diagramma Mermaid
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Percorsi multipli
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml

# CI/CD — diff rispetto al branch base
python main.py --cli --path . --git --git-base origin/main --minify true --stdout</pre>

<hr>

<h2>🏗️ Stack tecnologico</h2>
<table>
<thead><tr><th>Componente</th><th>Tecnologia</th></tr></thead>
<tbody>
<tr><td>Linguaggio</td><td>Python 3.10+</td></tr>
<tr><td>Framework GUI</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architettura</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Tokenizzazione</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Template</td><td>jinja2 (11 integrati)</td></tr>
<tr><td>Parser AST</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribuzione</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation)</b> — indicizzazione di codebase massive con DB vettoriale locale (Chroma/FAISS).</li>
<li>🚫 <b>Analisi approfondita del .gitignore</b> — supporto per file <code>.gitignore</code> annidati e <code>~/.gitignore</code> globale.</li>
<li>☁️ <b>Sincronizzazione cloud</b> — sincronizza le tue impostazioni tramite GitHub Gists.</li>
<li>🌳 <b>Workspace multi-radice</b> — supporto migliorato per monorepo (Lerna, NX, Turborepo).</li>
<li>🚀 <b>Pipeline CI/CD</b> — plugin GitHub Actions e GitLab CI per la generazione automatica del contesto PR.</li>
<li>🤖 <b>Integrazione diretta OpenAI/Anthropic API</b> — ponte completo dalla generazione del prompt all'output diretto.</li>
<li>🔌 Sistema di plugin ✅</li>
</ul>

<hr>

<h2>👨‍💻 Team</h2>
<p><b>Sviluppatore:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs su GitHub</p>

<hr>

<h2>🤝 Contribuire</h2>
<ol>
<li>Fork del repository</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Seguire i principi SOLID (vedere <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Licenza</h2>
<p>MIT. Vedere <code>LICENSE</code> per i dettagli.</p>
