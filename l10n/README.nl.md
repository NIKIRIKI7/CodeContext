<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-gestuurde codebase-analyse & prompt-voorbereidingstool**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.25.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 Over</h2>

<p><b>CodeContext AI</b> is een krachtige desktop tool om uw codebase voor te bereiden voor gebruik met grote taalmodellen (LLM's). Het scant projectmappen, analyseert structuur, bouwt afhankelijkheidsgrafieken en genereert een enkele, perfect gestructureerde prompt — geoptimaliseerd voor tokenverbruik en architecturale duidelijkheid.</p>

<h3>❓ Waarom?</h3>
<p>Bij het werken met AI worden ontwikkelaars geconfronteerd met contextvenster-tokenlimieten — LLM's "verliezen" architecturale samenhang wanneer code in delen wordt gekopieerd. <b>CodeContext AI lost dit op</b>: verzamel uw hele project in een paar klikken in één gestructureerde prompt en bespaar tot 80% op tokens.</p>

<hr>

<h2>🚀 Functies</h2>

<table>
<thead><tr><th>Functie</th><th>CodeContext AI</th><th>Handmatig</th></tr></thead>
<tbody>
<tr><td>🗜️ Minificatie + Skelet</td><td><b>Tot 80%</b> tokenreductie</td><td>Handmatig kopiëren-plakken</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Bekijk & pas JSON-patches toe</td><td>Niet beschikbaar</td></tr>
<tr><td>✅ LLM Checker</td><td>Code automatisch verifiëren voor opslaan</td><td>Niet beschikbaar</td></tr>
<tr><td>🔗 AST-afhankelijkheidsgraaf</td><td>Python, JS/TS, Vue</td><td>Alleen bestandslijst</td></tr>
<tr><td>🖱️ Contextmenu</td><td>Windows / Linux</td><td>Geen</td></tr>
<tr><td>🎨 Thema's</td><td>Apple, Modern, aangepaste JSON</td><td>Vaste UI</td></tr>
<tr><td>⚙️ UI-aanpassing (v1.14+)</td><td>Premiere Pro-stijl</td><td>Vaste UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 talen, automatische detectie</td><td>Enkele taal</td></tr>
<tr><td>♻️ Deduplicatie (v1.23+)</td><td>Detecteert en slaat bestanden met identieke inhoud over</td><td>Handmatige controle</td></tr>
<tr><td>⚡ Agressieve minificatie (v1.23+)</td><td>Extra compressie — verwijdert volgspaties op elke regel</td><td>Handmatig verwijderen</td></tr>
<tr><td>📌 Checkpoints (v1.23+)</td><td>Bewaar voor/na-opnamen voor foutopsporing</td><td>Niet beschikbaar</td></tr>
<tr><td>👁️ Auto-bewaking (v1.23+)</td><td>Bewaakt bestanden en verwerkt opnieuw bij wijziging</td><td>Niet beschikbaar</td></tr>
<tr><td>🔌 Plugin-systeem (v1.25+)</td><td>Uitbreiden met Python-plugins — aangepaste tabs, acties en i18n</td><td>Niet beschikbaar</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Installatie</h2>

<p><b>Vereisten:</b> Python 3.10+, Git</p>

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

<pre># Starten:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Actie</th><th>Commando</th></tr></thead>
<tbody>
<tr><td>Installeren</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Zoeken</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Bijwerken</td><td><code>yay -Syu</code></td></tr>
<tr><td>Verwijderen</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Als <b>yay</b> niet is geïnstalleerd:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternatief: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUI-modus</h2>
<pre>python main.py</pre>

<h3>1. Interface-overzicht</h3>
<p>Het venster is verdeeld in drie zones:</p>
<ul>
<li><b>Linker zijbalk (tabbladen)</b> — scaninstellingen, filters, prompts, LLM-configuratie, thema's</li>
<li><b>Centraal gebied</b> — mappenlijst, bestandsboom, tokenanalyses</li>
<li><b>Bovenste actiebalk</b> — Minificatie/Geen opmerkingen/Skelet-schakelaars, uitvoerformaat, actieknoppen</li>
</ul>

<h3>2. Een project toevoegen</h3>
<table>
<thead><tr><th>Actie</th><th>Hoe</th></tr></thead>
<tbody>
<tr><td>Slepen & neerzetten</td><td>Sleep een projectmap naar het venster</td></tr>
<tr><td>Bladerdialoog</td><td>Klik op "+ PC-map" op het tabblad <b>Bronnen</b></td></tr>
<tr><td>GitHub-repo</td><td>Klik op "+ GitHub / PR" — plak een repo- of Pull Request-URL</td></tr>
<tr><td>Config opslaan</td><td>Klik op "💾 Config opslaan" — maakt <code>.codecontextrc</code> aan</td></tr>
</tbody>
</table>

<p><b>GitHub-laadmodi:</b></p>
<ul>
<li><b>Permanent opslaan</b> — kloont naar een map op uw schijf</li>
<li><b>Tijdelijk</b> — kloont naar een tijdelijke map (verwijderd bij sluiten app)</li>
</ul>

<h3>3. Scanconfiguratie</h3>

<h4>Bronnen-tabblad</h4>
<table>
<thead><tr><th>Optie</th><th>Beschrijving</th></tr></thead>
<tbody>
<tr><td>☑ Alleen Git-wijzigingen</td><td>Alleen bestanden opnemen die in de laatste commit zijn gewijzigd</td></tr>
<tr><td>☑ .gitignore respecteren</td><td>Bestanden automatisch uitsluiten van <code>.gitignore</code></td></tr>
<tr><td>🔍 Bestanden scannen</td><td>Bestandsboom met metadata bouwen</td></tr>
</tbody>
</table>

<h4>Filters-tabblad</h4>
<table>
<thead><tr><th>Optie</th><th>Beschrijving</th></tr></thead>
<tbody>
<tr><td><b>Extensie-voorinstellingen</b></td><td>Snel schakelen tussen taalsets (Python, Web, Golang, Rust, C#, etc.)</td></tr>
<tr><td><b>Extensies</b></td><td>Aangepaste bestandsextensie-witte lijst</td></tr>
<tr><td><b>Genegeerde paden</b></td><td>Mappen/bestanden overslaan (node_modules, .git, build, dist, etc.)</td></tr>
<tr><td>☑ Bestandsboom opnemen</td><td>Voegt mapstructuur toe vóór de prompt</td></tr>
<tr><td>☑ Afhankelijkheidskaart opnemen</td><td>AST-gebaseerde importanalyse voor Python/JS/TS</td></tr>
<tr><td>☑ Mermaid-graaf opnemen</td><td>Architectuurdiagram in Mermaid-formaat</td></tr>
</tbody>
</table>

<p>💡 <b>Aangepaste voorinstellingen opslaan:</b> configureer filters, klik op 💾, voer een naam in.</p>

<h4>Prompts-tabblad</h4>
<table>
<thead><tr><th>Optie</th><th>Beschrijving</th></tr></thead>
<tbody>
<tr><td><b>Prompt-voorinstellingen</b></td><td>Snel wijzigen van systeemprompt (Code Review, Bug Hunter, Refactoring, etc.)</td></tr>
<tr><td><b>Systeemprompt</b></td><td>Aangepaste prompt — verzonden naar LLM als systeemcontext</td></tr>
<tr><td><b>🧩 JSON-patch toepassen</b></td><td>Plak LLM JSON-reactie — bekijk diff en pas toe op schijf</td></tr>
</tbody>
</table>

<p><b>JSON-patches gebruiken:</b></p>
<ol>
<li>Vraag LLM om een JSON-array: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Plak JSON, klik op <b>"Volgende"</b> → <b>Veilige Diff-viewer</b> opent</li>
<li>Vink bestanden aan/uit, klik optioneel op <b>"🤖 Controleren via LLM"</b></li>
<li>Klik op <b>"💾 Geselecteerde opslaan op schijf"</b></li>
</ol>

<h3>4. Uitvoerformaat-instellingen</h3>
<table>
<thead><tr><th>Optie</th><th>Beschrijving</th></tr></thead>
<tbody>
<tr><td>☑ Minificatie</td><td>Verwijdert witruimte en lege regels</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — Extra compressie — verwijdert volgspaties op elke regel</td></tr>
<tr><td>☑ Geen opmerkingen</td><td>Verwijdert alle opmerkingen</td></tr>
<tr><td>☑ Geen geheimen</td><td>Maskeert API-sleutels, wachtwoorden, tokens</td></tr>
<tr><td>☑ Skelet ☠️</td><td><b>Verwijdert functielichamen</b> — maximale tokenbesparing</td></tr>
<tr><td>☑ Dedup</td><td>Verwijdert dubbele bestanden met identieke inhoud</td></tr>
<tr><td>☑ Checkpoints</td><td>Slaat tussentijdse verwerkingscheckpoints op</td></tr>
<tr><td>☑ Auto-Watch</td><td>Bewaakt bestanden en verwerkt opnieuw bij wijziging</td></tr>
<tr><td>Formaat</td><td>Markdown, XML, Plain, JSONL Chunks, Aangepast (Jinja2)</td></tr>
<tr><td>📁 sjabloon</td><td>Jinja2-sjabloonkiezer</td></tr>
</tbody>
</table>

<p><b>Skeletmodus:</b> verwijdert functie-implementaties (<code>def func_name(...):  # ... implementation ...</code>), behoudt alle klassen — laat LLM grote projecten begrijpen met minimale tokens.</p>


<p><b>Minify vs Agressief:</b> <b>Minify</b> verwijdert voorloop-/achterloopspaties en lege regels — veilig voor elke codebase, vermindert tokens zonder de leesbaarheid te beïnvloeden. <b>Agressief</b> voegt een extra pas toe die volgspaties op elke regel verwijdert voor maximale compressie. Combineer beide wanneer u meer code in een beperkt contextvenster wilt passen.</p>

<p><b>Dedup:</b> detecteert automatisch bestanden met identieke inhoud in uw project en sluit duplicaten uit van de uitvoer — voorkomt dat de LLM dezelfde code twee keer ziet en tokens verspilt.</p>

<p><b>Checkpoints:</b> slaat tussentijdse resultaten op bij elke pijplijnfase (vóór opschoning, na minificatie, etc.) in de map <code>checkpoints/</code>. Nuttig voor het debuggen van wat elke verwerkingsstap doet of voor het vergelijken van uitvoeren naast elkaar.</p>

<p><b>Auto-Watch:</b> monitors your project files for changes using the OS file watcher. When a file is saved, the pipeline automatically re-runs — ideal during active development when you need continuous prompt updates.</p>
<h3>5. Actieknoppen</h3>
<table>
<thead><tr><th>Knop</th><th>Actie</th></tr></thead>
<tbody>
<tr><td>👀 Voorbeeld</td><td><b>Geavanceerd voorbeelddialoog</b> — "Definitieve prompt" + "Voor/Na"-tabbladen</td></tr>
<tr><td>📋 Kopiëren naar klembord</td><td>Resultaat kopiëren — plakken in ChatGPT / Claude</td></tr>
<tr><td>🚀 Verzenden naar ChatGPT / Claude</td><td>Opent webchat en plakt context</td></tr>
<tr><td>💻 Openen in editor</td><td>Opent in VS Code / Cursor</td></tr>
<tr><td>💾 Opslaan naar bestand</td><td>Resultaat opslaan op schijf</td></tr>
</tbody>
</table>

<h3>6. Geavanceerd voorbeelddialoog</h3>
<p><b>"📝 Definitieve prompt" tabblad:</b> bestandslijst (links) + volledige tekst met markering (rechts). Alles kopiëren / Bestand kopiëren.</p>
<p><b>"🔍 Voor/Na" tabblad:</b> gekleurde diff tussen origineel en geoptimaliseerd. Teller: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM & Besturingssysteem</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Verificatie inschakelen</td><td>Automatische LLM-patchverificatie voor toepassen</td></tr>
<tr><td>URL / Sleutel / Model</td><td>API-eindpunt (standaard OpenAI), sleutel, model</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">OS-integratie</th></tr></thead>
<tbody>
<tr><td>Contextmenu installeren</td><td>"Openen met CodeContext AI" in rechtsklikmenu</td></tr>
<tr><td>Toevoegen aan PATH</td><td>Globale <code>codecontext</code> CLI-opdracht</td></tr>
<tr><td>Editor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Thema's</h3>
<ul>
<li><b>Thema:</b> Apple, Modern — <b>Modus:</b> licht / donker</li>
<li>📂 Themamap openen / ➕ Thema importeren (.json)</li>
</ul>

<h3>9. 📊 Tokenanalyses</h3>
<p>Tabel: bestandspad, tokens (tiktoken), compressie, besparing %, kosten voor model.</p>

<h3>10. 🎛️ UI-aanpassing (v1.14+)</h3>
<p>Klik op <b>⚙</b> naast versie — "Interface-instellingen (Premiere Pro-stijl)" dialoog. Schakel tabbladen (Bronnen, Filters, Prompts, LLM & OS, Thema's) en actieknoppen (Voorbeeld, Klembord, ChatGPT, Editor, Bestand) in/uit.</p>

<h3>11. Commandopalet</h3>
<p><code>Ctrl+Shift+P</code> — muisvrije toegang tot alle acties.</p>

<h3>12. 🔌 Plugin-systeem (v1.25+)</h3>
<p><b>CodeContext AI</b> ondersteunt een <b>Python-plugin-systeem</b> waarmee u de app kunt uitbreiden met aangepaste functionaliteit.</p>

<h4>📁 Pluginstructuur</h4>
<pre>my_plugin/
├── manifest.json          # Plugin-metadata
├── requirements.txt       # (Optioneel) pip-afhankelijkheden
├── locales/
│   ├── en.json            # Engelse vertalingen
│   └── ru.json            # Russische vertalingen
└── plugin.py              # Toegangspunt</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Doet iets nuttigs",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Voorbeeld)</h4>
<pre>from src.api.plugin_api import IPlugin, PluginAPI

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("Hallo van plugin!")
        )
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Plugin-actie geklikt")
        )
        api.add_log("Mijn plugin geïnitialiseerd")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Beveiliging</h4>
<ul>
<li>Plugins krijgen <b>volledige Python-toegang</b> — installeer alleen uit vertrouwde bronnen</li>
<li>Bij de eerste keer laden vraagt een beveiligingsdialoog om uw goedkeuring voordat een plugin wordt ingeschakeld</li>
<li>Als <code>requirements.txt</code> bestaat, ziet u een live pip-installatielog voordat het laden begint</li>
<li>Goedgekeurde plugins worden onthouden in instellingen (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 Plugin-API</h4>
<table>
<thead><tr><th>Eigenschap / Methode</th><th>Beschrijving</th></tr></thead>
<tbody>
<tr><td><code>api.store</code></td><td>Alleen-lezen Redux-winkel (statustoegang via <code>state.settings.xxx</code>)</td></tr>
<tr><td><code>api.dispatcher</code></td><td>Acties verzenden (bijv. <code>UI_ADD_LOG</code>)</td></tr>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Voeg een tabblad toe aan de linker zijbalk</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Voeg een knop toe aan het "Plugins 🔽"-dropdownmenu</td></tr>
<tr><td><code>api.add_translations(lang, data)</code></td><td>Voeg runtime-vertalingen toe (samengevoegd bovenop ingebouwde)</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Schrijf naar het app-logpaneel</td></tr>
</tbody>
</table>

<h4>⚙️ Zichtbaarheid</h4>
<p>Plugin-tabbladen en actieknoppen kunnen worden in- of uitgeschakeld via <b>⚙ UI-aanpassing</b> — ze verschijnen naast ingebouwde tabbladen/acties met hun eigen selectievakjes.</p>

<hr>

<h2>💻 CLI-modus</h2>
<pre>python main.py --cli --path /pad/naar/project [opties]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parameter</th><th>Type</th><th>Beschrijving</th><th>Voorbeeld</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLI-modus (geen GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>Projectpad</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Extensies</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Genegeerde paden</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Minificatie inschakelen</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Opmerkingen verwijderen</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Geheimen maskeren</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Skeletmodus</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Uitvoerbestand</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Afdrukken naar stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Alleen Git-wijzigingen</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>.gitignore respecteren</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Bestandsboom</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaid-graaf</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Afhankelijkheidskaart</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSON-patch</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2-sjabloon</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Aangepaste systeemprompt</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Voorbeelden</h3>
<pre># Minimale uitvoering
python main.py --cli --path ./myapp --stdout

# Volledige analyse met XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON-patch
python main.py --cli --path ./myapp --patch llm_response.json

# Aangepaste Jinja2-sjabloon
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid-diagram
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Meerdere paden
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Technologiestack</h2>
<table>
<thead><tr><th>Component</th><th>Technologie</th></tr></thead>
<tbody>
<tr><td>Taal</td><td>Python 3.10+</td></tr>
<tr><td>GUI-framework</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architectuur</td><td>Clean Architecture + Redux-achtig</td></tr>
<tr><td>Tokenisatie</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Templating</td><td>jinja2 (11 ingebouwd)</td></tr>
<tr><td>AST-parsers</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distributie</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation) modus</b> — indexeren van grote codebases met lokale vector-DB (Chroma/FAISS).</li>
<li>🚫 <b>Diepgaand .gitignore-parsen</b> — ondersteuning voor geneste <code>.gitignore</code>-bestanden en globale <code>~/.gitignore</code>.</li>
<li>☁️ <b>Cloudsynchronisatie</b> — synchroniseer uw voorinstellingen via GitHub Gists.</li>
<li>🌳 <b>Multi-root-werkruimten</b> — verbeterde ondersteuning voor monorepo's (Lerna, NX, Turborepo).</li>
<li>🚀 <b>CI/CD-pijplijnen</b> — GitHub Actions- en GitLab CI-plugins voor geautomatiseerde PR-contextgeneratie.</li>
<li>🤖 <b>Directe OpenAI/Anthropic API-integratie</b> — volledige brug van promptgeneratie naar directe uitvoer.</li>
<li>🍎 macOS Finder-contextmenu</li>
<li>🔌 Pluginsysteem ✅</li>
</ul>

<hr>

<h2>👨‍💻 Team</h2>
<p><b>Ontwikkelaar:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PR's op GitHub</p>

<hr>

<h2>🤝 Bijdragen</h2>
<ol>
<li>Fork de repository</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Volg SOLID-principes (zie <code>../docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Licentie</h2>
<p>MIT. Zie <code>../LICENSE</code> voor details.</p>
