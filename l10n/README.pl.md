<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**Narzędzie do analizy kodu źródłowego i przygotowywania promptów z wykorzystaniem AI**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.25.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 O programie</h2>

<p><b>CodeContext AI</b> to potężne narzędzie desktopowe do przygotowywania bazy kodu do pracy z dużymi modelami językowymi (LLM). Skanuje foldery projektu, analizuje strukturę, buduje grafy zależności i generuje pojedynczy, doskonale ustrukturyzowany prompt — zoptymalizowany pod kątem zużycia tokenów i przejrzystości architektonicznej.</p>

<h3>❓ Dlaczego?</h3>
<p>Podczas pracy z AI deweloperzy napotykają ograniczenia okna kontekstowego — LLM "tracą" spójność architektoniczną, gdy kod jest kopiowany w częściach. <b>CodeContext AI rozwiązuje to</b>: zbierz cały projekt w jeden ustrukturyzowany prompt w kilku kliknięciach, oszczędzając do 80% na tokenach.</p>

<hr>

<h2>🚀 Funkcje</h2>

<table>
<thead><tr><th>Funkcja</th><th>CodeContext AI</th><th>Ręcznie</th></tr></thead>
<tbody>
<tr><td>🗜️ Minifikacja + Szkielet</td><td><b>Do 80%</b> redukcji tokenów</td><td>Ręczne kopiuj-wklej</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Podgląd i stosowanie łat JSON</td><td>Niedostępne</td></tr>
<tr><td>✅ LLM Checker</td><td>Automatyczna weryfikacja kodu przed zapisaniem</td><td>Niedostępne</td></tr>
<tr><td>🔗 Graf zależności AST</td><td>Python, JS/TS, Vue</td><td>Tylko lista plików</td></tr>
<tr><td>🖱️ Menu kontekstowe</td><td>Windows / Linux</td><td>Brak</td></tr>
<tr><td>🎨 Motywy</td><td>Apple, Modern, niestandardowy JSON</td><td>Stały interfejs</td></tr>
<tr><td>⚙️ Dostosowanie interfejsu (v1.14+)</td><td>Styl Premiere Pro</td><td>Stały interfejs</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 języków, automatyczne wykrywanie</td><td>Jeden język</td></tr>
<tr><td>♻️ Deduplikacja (v1.23+)</td><td>Wykrywa i pomija pliki o identycznej zawartości</td><td>Ręczne sprawdzanie</td></tr>
<tr><td>⚡ Agresywna minifikacja (v1.23+)</td><td>Dodatkowa kompresja — usuwa końcowe spacje w każdej linii</td><td>Ręczne usuwanie</td></tr>
<tr><td>📌 Punkty kontrolne (v1.23+)</td><td>Zapisuje migawki przed/po do debugowania</td><td>Niedostępne</td></tr>
<tr><td>👁️ Auto-nadzór (v1.23+)</td><td>Śledzi pliki i przetwarza ponownie przy zmianie</td><td>Niedostępne</td></tr>
<tr><td>🔌 System wtyczek (v1.25+)</td><td>Rozszerz przez pluginy Python — niestandardowe zakładki, akcje i i18n</td><td>Niedostępne</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Instalacja</h2>

<p><b>Wymagania:</b> Python 3.10+, Git</p>

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

<pre># Następnie uruchom:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Akcja</th><th>Polecenie</th></tr></thead>
<tbody>
<tr><td>Instaluj</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Szukaj</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Aktualizuj</td><td><code>yay -Syu</code></td></tr>
<tr><td>Usuń</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Jeśli <b>yay</b> nie jest zainstalowany:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternatywa: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 Tryb GUI</h2>
<pre>python main.py</pre>

<h3>1. Przegląd interfejsu</h3>
<p>Okno podzielone jest na trzy strefy:</p>
<ul>
<li><b>Lewy panel boczny (zakładki)</b> — ustawienia skanowania, filtry, prompty, konfiguracja LLM, motywy</li>
<li><b>Obszar centralny</b> — lista folderów, drzewo plików, analiza tokenów</li>
<li><b>Górny pasek akcji</b> — przełączniki Minifikuj/Brak komentarzy/Szkielet, format wyjścia, przyciski akcji</li>
</ul>

<h3>2. Dodawanie projektu</h3>
<table>
<thead><tr><th>Akcja</th><th>Jak</th></tr></thead>
<tbody>
<tr><td>Przeciągnij i upuść</td><td>Po prostu przeciągnij folder projektu do okna</td></tr>
<tr><td>Okno przeglądania</td><td>Kliknij "+ Folder PC" na zakładce <b>Źródła</b></td></tr>
<tr><td>Repozytorium GitHub</td><td>Kliknij "+ GitHub / PR" — wklej URL repozytorium lub Pull Requesta</td></tr>
<tr><td>Zapisz konfigurację</td><td>Kliknij "💾 Zapisz konfigurację" — tworzy <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>Tryby ładowania GitHub:</b></p>
<ul>
<li><b>Zapisz trwale</b> — klonuje do folderu na dysku</li>
<li><b>Tymczasowy</b> — klonuje do tymczasowego folderu (usuwany po zamknięciu aplikacji)</li>
</ul>

<h3>3. Konfiguracja skanowania</h3>

<h4>Zakładka Źródła</h4>
<table>
<thead><tr><th>Opcja</th><th>Opis</th></tr></thead>
<tbody>
<tr><td>☑ Tylko zmiany Git</td><td>Uwzględnij tylko pliki zmienione w ostatnim commicie</td></tr>
<tr><td>☑ Szanuj .gitignore</td><td>Automatycznie wykluczaj pliki z <code>.gitignore</code></td></tr>
<tr><td>🔍 Skanuj pliki</td><td>Zbuduj drzewo plików z metadanymi</td></tr>
</tbody>
</table>

<h4>Zakładka Filtry</h4>
<table>
<thead><tr><th>Opcja</th><th>Opis</th></tr></thead>
<tbody>
<tr><td><b>Presety rozszerzeń</b></td><td>Szybkie przełączanie między zestawami języków (Python, Web, Golang, Rust, C#, itp.)</td></tr>
<tr><td><b>Rozszerzenia</b></td><td>Niestandardowa biała lista rozszerzeń plików</td></tr>
<tr><td><b>Ignorowane ścieżki</b></td><td>Pomiń foldery/pliki (node_modules, .git, build, dist, itp.)</td></tr>
<tr><td>☑ Dołącz drzewo plików</td><td>Dodaje strukturę folderów przed promptem</td></tr>
<tr><td>☑ Dołącz mapę zależności</td><td>Analiza importów oparta na AST dla Python/JS/TS</td></tr>
<tr><td>☑ Dołącz graf Mermaid</td><td>Diagram architektury w formacie Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>Zapisywanie niestandardowych presetów:</b> skonfiguruj filtry, kliknij 💾, wprowadź nazwę.</p>

<h4>Zakładka Prompty</h4>
<table>
<thead><tr><th>Opcja</th><th>Opis</th></tr></thead>
<tbody>
<tr><td><b>Presety promptów</b></td><td>Szybka zmiana promptu systemowego (Code Review, Bug Hunter, Refaktoryzacja, itp.)</td></tr>
<tr><td><b>Prompt systemowy</b></td><td>Niestandardowy prompt — wysyłany do LLM jako kontekst systemowy</td></tr>
<tr><td><b>🧩 Zastosuj łatkę JSON</b></td><td>Wklej odpowiedź JSON z LLM — podgląd diff i zastosuj na dysku</td></tr>
</tbody>
</table>

<p><b>Używanie łat JSON:</b></p>
<ol>
<li>Poproś LLM o tablicę JSON: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Wklej JSON, kliknij <b>"Dalej"</b> → otwiera się <b>Bezpieczny podgląd diff</b></li>
<li>Zaznacz/odznacz pliki, opcjonalnie kliknij <b>"🤖 Sprawdź przez LLM"</b></li>
<li>Kliknij <b>"💾 Zapisz zaznaczone na dysku"</b></li>
</ol>

<h3>4. Ustawienia formatu wyjścia</h3>
<table>
<thead><tr><th>Opcja</th><th>Opis</th></tr></thead>
<tbody>
<tr><td>☑ Minifikuj</td><td>Usuwa białe znaki i puste linie</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — Dodatkowa kompresja — usuwa końcowe spacje w każdej linii</td></tr>
<tr><td>☑ Bez komentarzy</td><td>Usuwa wszystkie komentarze</td></tr>
<tr><td>☑ Bez sekretów</td><td>Maskuje klucze API, hasła, tokeny</td></tr>
<tr><td>☑ Szkielet ☠️</td><td><b>Usuwa ciała funkcji</b> — maksymalne oszczędności tokenów</td></tr>
<tr><td>☑ Dedup</td><td>Usuwa zduplikowane pliki o identycznej zawartości</td></tr>
<tr><td>☑ Checkpoints</td><td>Zapisuje pośrednie punkty kontrolne przetwarzania</td></tr>
<tr><td>☑ Auto-Watch</td><td>Śledzi pliki i przetwarza ponownie przy zmianie</td></tr>
<tr><td>Format</td><td>Markdown, XML, Plain, JSONL Chunks, Niestandardowy (Jinja2)</td></tr>
<tr><td>📁 szablon</td><td>Selektor szablonów Jinja2</td></tr>
</tbody>
</table>

<p><b>Tryb szkieletowy:</b> usuwa implementacje funkcji (<code>def func_name(...):  # ... implementation ...</code>), zachowując wszystkie klasy — pozwala LLM zrozumieć duże projekty przy minimalnej liczbie tokenów.</p>


<p><b>Minify vs Aggressive:</b> <b>Minify</b> usuwa początkowe/końcowe białe znaki i puste linie — bezpieczne dla każdego projektu, redukuje tokeny bez wpływu na czytelność. <b>Aggressive</b> dodaje dodatkowy przebieg, który eliminuje końcowe białe znaki w każdej linii dla maksymalnej kompresji. Łącz oba, gdy potrzebujesz zmieścić więcej kodu w ograniczonym oknie kontekstowym.</p>

<p><b>Dedup:</b> automatycznie wykrywa pliki o identycznej zawartości w projekcie i wyklucza duplikaty z wyniku — zapobiega wielokrotnemu wysyłaniu tego samego kodu do LLM i marnowaniu tokenów.</p>

<p><b>Checkpoints:</b> zapisuje pośrednie wyniki na każdym etapie przetwarzania (przed czyszczeniem, po minifikacji itd.) w folderze <code>checkpoints/</code>. Przydatne do debugowania i porównywania wyników poszczególnych etapów.</p>

<p><b>Auto-Watch:</b> monitoruje pliki projektu pod kątem zmian za pomocą systemowego obserwatora plików. Gdy plik zostanie zapisany, pipeline automatycznie uruchamia się ponownie — idealne podczas aktywnego rozwoju, gdy potrzebujesz ciągłych aktualizacji promptu.</p>
<h3>5. Przyciski akcji</h3>
<table>
<thead><tr><th>Przycisk</th><th>Akcja</th></tr></thead>
<tbody>
<tr><td>👀 Podgląd</td><td><b>Zaawansowane okno podglądu</b> — zakładki "Ostateczny prompt" + "Przed/Po"</td></tr>
<tr><td>📋 Kopiuj do schowka</td><td>Skopiuj wynik — wklej do ChatGPT / Claude</td></tr>
<tr><td>🚀 Wyślij do ChatGPT / Claude</td><td>Otwiera czat internetowy i wkleja kontekst</td></tr>
<tr><td>💻 Otwórz w edytorze</td><td>Otwiera w VS Code / Cursor</td></tr>
<tr><td>💾 Zapisz do pliku</td><td>Zapisz wynik na dysku</td></tr>
</tbody>
</table>

<h3>6. Zaawansowane okno podglądu</h3>
<p><b>Zakładka "📝 Ostateczny prompt":</b> lista plików (lewo) + pełny tekst z podświetlaniem (prawo). Kopiuj wszystko / Kopiuj plik.</p>
<p><b>Zakładka "🔍 Przed/Po":</b> kolorowy diff między oryginałem a optymalizacją. Licznik: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM i OS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Włącz weryfikację</td><td>Automatyczna weryfikacja łatki LLM przed zastosowaniem</td></tr>
<tr><td>URL / Klucz / Model</td><td>Endpoint API (domyślnie OpenAI), klucz, model</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">Integracja z OS</th></tr></thead>
<tbody>
<tr><td>Zainstaluj menu kontekstowe</td><td>"Otwórz za pomocą CodeContext AI" w menu prawym przyciskiem myszy</td></tr>
<tr><td>Dodaj do PATH</td><td>Globalne polecenie CLI <code>codecontext</code></td></tr>
<tr><td>Edytor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Motywy</h3>
<ul>
<li><b>Motyw:</b> Apple, Modern — <b>Tryb:</b> jasny / ciemny</li>
<li>📂 Otwórz folder motywów / ➕ Importuj motyw (.json)</li>
</ul>

<h3>9. 📊 Analiza tokenów</h3>
<p>Tabela: ścieżka pliku, tokeny (tiktoken), kompresja, % oszczędności, koszt dla modelu.</p>

<h3>10. 🎛️ Dostosowanie interfejsu (v1.14+)</h3>
<p>Kliknij <b>⚙</b> obok wersji — okno "Ustawienia interfejsu (styl Premiere Pro)". Przełączaj zakładki (Źródła, Filtry, Prompty, LLM i OS, Motywy) i przyciski akcji (Podgląd, Schowek, ChatGPT, Edytor, Plik).</p>

<h3>11. Paleta poleceń</h3>
<p><code>Ctrl+Shift+P</code> — dostęp do wszystkich akcji bez użycia myszy.</p>

<h3>12. 🔌 System wtyczek (v1.25+)</h3>
<p><b>CodeContext AI</b> obsługuje <b>system wtyczek Python</b>, który pozwala rozszerzyć aplikację o niestandardowe funkcje.</p>

<h4>📁 Struktura wtyczki</h4>
<pre>my_plugin/
├── manifest.json          # Metadane wtyczki
├── requirements.txt       # (Opcjonalnie) zależności pip
├── locales/
│   ├── en.json            # Tłumaczenia angielskie
│   └── ru.json            # Tłumaczenia rosyjskie
└── plugin.py              # Punkt wejścia</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Robi coś użytecznego",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Przykład)</h4>
<pre>from src.api.plugin_api import IPlugin, PluginAPI

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        # Tłumaczenia z folderu locales/ są ładowane automatycznie
        # Rejestrujemy zakładkę w panelu bocznym
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("Witaj z wtyczki!")
        )
        # Rejestrujemy przycisk akcji
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Kliknięcie wtyczki")
        )
        api.add_log("Moja wtyczka zainicjalizowana")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Bezpieczeństwo</h4>
<ul>
<li>Wtyczki otrzymują <b>pełny dostęp do Pythona</b> — instaluj tylko z zaufanych źródeł</li>
<li>Przy pierwszym ładowaniu pojawi się okno dialogowe bezpieczeństwa z prośbą o zatwierdzenie</li>
<li>Jeśli istnieje <code>requirements.txt</code>, przed załadowaniem zobaczysz log na żywo z pip install</li>
<li>Zatwierdzone wtyczki są zapamiętywane w ustawieniach (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 API wtyczek</h4>
<table>
<thead><tr><th>Właściwość / Metoda</th><th>Opis</th></tr></thead>
<tbody>
<tr><td><code>api.store</code></td><td>Redux-store (tylko do odczytu, dostęp: <code>state.settings.xxx</code>)</td></tr>
<tr><td><code>api.dispatcher</code></td><td>Wysyłanie akcji (np. <code>UI_ADD_LOG</code>)</td></tr>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Dodaje zakładkę do lewego panelu bocznego</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Dodaje przycisk do menu rozwijanego "Wtyczki 🔽"</td></tr>
<tr><td><code>api.add_translations(lang, data)</code></td><td>Dodaje tłumaczenia w czasie wykonania (scalane z wbudowanymi)</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Zapisuje do panelu logów aplikacji</td></tr>
</tbody>
</table>

<h4>⚙️ Widoczność</h4>
<p>Zakładki i przyciski wtyczek można włączać/wyłączać poprzez <b>⚙ Dostosowanie interfejsu</b> — pojawiają się obok wbudowanych zakładek/przycisków z własnymi polami wyboru.</p>

<hr>

<h2>💻 Tryb CLI</h2>
<pre>python main.py --cli --path /ścieżka/do/projektu [opcje]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parametr</th><th>Typ</th><th>Opis</th><th>Przykład</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>Tryb CLI (bez GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>Ścieżka projektu</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Rozszerzenia</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Ignorowane ścieżki</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Włącz minifikację</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Usuń komentarze</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Maskuj sekrety</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Tryb szkieletowy</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Plik wyjściowy</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Wypisz na stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Tylko zmiany Git</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Szanuj .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Drzewo plików</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Graf Mermaid</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Mapa zależności</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>Łatka JSON LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Szablon Jinja2</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Niestandardowy prompt systemowy</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Przykłady</h3>
<pre># Minimalne uruchomienie
python main.py --cli --path ./myapp --stdout

# Pełna analiza z XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# Łatka JSON LLM
python main.py --cli --path ./myapp --patch llm_response.json

# Niestandardowy szablon Jinja2
python main.py --cli --path ./myapp --template my.j2 --stdout

# Diagram Mermaid
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Wiele ścieżek
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Stos technologiczny</h2>
<table>
<thead><tr><th>Komponent</th><th>Technologia</th></tr></thead>
<tbody>
<tr><td>Język</td><td>Python 3.10+</td></tr>
<tr><td>Framework GUI</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architektura</td><td>Clean Architecture + Redux-podobna</td></tr>
<tr><td>Tokenizacja</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Szablony</td><td>jinja2 (11 wbudowanych)</td></tr>
<tr><td>Parsery AST</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Dystrybucja</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Plan rozwoju</h2>
<ul>
<li>📚 <b>Tryb RAG (Retrieval-Augmented Generation)</b> — indeksowanie ogromnych baz kodu przy użyciu lokalnej bazy wektorowej (Chroma/FAISS).</li>
<li>🚫 <b>Dogłębne parsowanie .gitignore</b> — obsługa zagnieżdżonych plików <code>.gitignore</code> i globalnego <code>~/.gitignore</code>.</li>
<li>☁️ <b>Synchronizacja w chmurze</b> — synchronizuj ustawienia przez GitHub Gists.</li>
<li>🌳 <b>Obszary robocze z wieloma korzeniami</b> — ulepszona obsługa monorepozytoriów (Lerna, NX, Turborepo).</li>
<li>🚀 <b>Potoki CI/CD</b> — wtyczki GitHub Actions i GitLab CI do automatycznego generowania kontekstu PR.</li>
<li>🤖 <b>Bezpośrednia integracja z API OpenAI/Anthropic</b> — pełny most od generowania promptu do bezpośredniego wyniku.</li>
<li>🍎 Menu kontekstowe macOS Finder</li>
<li>🔌 System wtyczek ✅</li>
</ul>

<hr>

<h2>👨‍💻 Zespół</h2>
<p><b>Deweloper:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues i PR na GitHubie</p>

<hr>

<h2>🤝 Wkład</h2>
<ol>
<li>Sforkuj repozytorium</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Postępuj zgodnie z zasadami SOLID (zobacz <code>../docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Licencja</h2>
<p>MIT. Szczegóły w <code>../LICENSE</code>.</p>
