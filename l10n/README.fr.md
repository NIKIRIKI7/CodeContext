<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**Outil d'analyse de codebase et de préparation de prompts, piloté par IA**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.27.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 À propos</h2>

<p><b>CodeContext AI</b> est un outil de bureau puissant pour préparer votre codebase à travailler avec les grands modèles de langage (LLM). Il scanne les dossiers du projet, analyse la structure, construit des graphes de dépendances et génère un prompt unique et parfaitement structuré — optimisé pour la consommation de tokens et la clarté architecturale.</p>

<h3>❓ Pourquoi ?</h3>
<p>Quand on travaille avec l'IA, les développeurs se heurtent aux limites de la fenêtre de contexte — les LLM « perdent » la cohérence architecturale quand le code est copié en morceaux. <b>CodeContext AI résout ce problème</b> : rassemblez l'intégralité de votre projet en un prompt structuré en quelques clics, économisant jusqu'à 80 % des tokens.</p>

<hr>

<h2>🚀 Fonctionnalités</h2>

<table>
<thead><tr><th>Fonctionnalité</th><th>CodeContext AI</th><th>Manuel</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify + Skeleton</td><td><b>Jusqu'à 80 %</b> de réduction de tokens</td><td>Copier-coller manuel</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Aperçu et application de correctifs JSON</td><td>Indisponible</td></tr>
<tr><td>✅ LLM Checker</td><td>Vérification automatique du code avant écriture</td><td>Indisponible</td></tr>
<tr><td>🔗 Graphe de dépendances AST</td><td>Python, JS/TS, Vue</td><td>Liste de fichiers uniquement</td></tr>
<tr><td>🖱️ Menu contextuel</td><td>Windows / Linux / macOS</td><td>None</td></tr>
<tr><td>🎨 Thèmes</td><td>Apple, Modern, JSON personnalisé</td><td>UI fixe</td></tr>
<tr><td>⚙️ Personnalisation UI (v1.14+)</td><td>Style Premiere Pro</td><td>UI fixe</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 langues, détection automatique</td><td>Langue unique</td></tr>
<tr><td>♻️ Déduplication (v1.23+)</td><td>Détecte et ignore les fichiers au contenu identique</td><td>Vérification manuelle</td></tr>
<tr><td>⚡ Minification agressive (v1.23+)</td><td>Compression supplémentaire — supprime les espaces de fin</td><td>Suppression manuelle</td></tr>
<tr><td>📌 Points de contrôle (v1.23+)</td><td>Instantanés avant/après pour le débogage</td><td>Indisponible</td></tr>
<tr><td>👁️ Surveillance auto (v1.23+)</td><td>Surveille les fichiers et retraite en cas de modification</td><td>Indisponible</td></tr>
<tr><td>🔌 Système de plugins (v1.25+)</td><td>Extension avec des plugins Python — onglets, actions et i18n personnalisés</td><td>Indisponible</td></tr>
<tr><td>🚦 Intégration CI/CD</td><td>GitHub Actions & GitLab CI — génération automatique du contexte PR via <code>--git-base</code></td><td>Not available</td></tr>
<tr><td>🌳 Monorepo Support (v1.25+)</td><td>Lerna, NX, Turborepo, pnpm workspaces — cross-package imports, root config discovery</td><td>Not available</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Installation</h2>

<p><b>Prérequis :</b> Python 3.10+, Git</p>

<pre>git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext
python -m venv venv
# Windows :
venv\Scripts\activate
# Linux/macOS :
source venv/bin/activate
pip install -r requirements.txt</pre>

<h3>PyPI (pip)</h3>
<pre>pip install codecontext-ai</pre>

<pre># Puis lancer :
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Action</th><th>Commande</th></tr></thead>
<tbody>
<tr><td>Installer</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Rechercher</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Mettre à jour</td><td><code>yay -Syu</code></td></tr>
<tr><td>Supprimer</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Si <b>yay</b> n'est pas installé :</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternative : <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 Mode GUI</h2>
<pre>python main.py</pre>

<h3>1. Aperçu de l'interface</h3>
<p>La fenêtre est divisée en trois zones :</p>
<ul>
<li><b>Barre latérale gauche (onglets)</b> — paramètres de scan, filtres, prompts, config LLM, thèmes</li>
<li><b>Zone centrale</b> — liste des dossiers, arborescence des fichiers, analytique des tokens</li>
<li><b>Barre d'actions supérieure</b> — bascules Minify/No Comments/Skeleton, format de sortie, boutons d'action</li>
</ul>

<h3>2. Ajouter un projet</h3>
<table>
<thead><tr><th>Action</th><th>Comment</th></tr></thead>
<tbody>
<tr><td>Glisser-déposer</td><td>Glissez un dossier projet dans la fenêtre</td></tr>
<tr><td>Dialogue de navigation</td><td>Cliquez « + Папка ПК » sur l'onglet <b>Sources</b></td></tr>
<tr><td>Dépôt GitHub</td><td>Cliquez « + GitHub / PR » — collez une URL de dépôt ou de Pull Request</td></tr>
<tr><td>Sauvegarder la config</td><td>Cliquez « 💾 Save config » — crée <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>Modes de chargement GitHub :</b></p>
<ul>
<li><b>Sauvegarder définitivement</b> — clone dans un dossier sur votre disque</li>
<li><b>Temporaire</b> — clone dans un dossier temporaire (supprimé à la fermeture de l'app)</li>
</ul>

<h3>3. Configuration du scan</h3>

<h4>Onglet Sources</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>Inclure seulement les fichiers modifiés dans le dernier commit</td></tr>
<tr><td>☑ Respect .gitignore</td><td>Exclure automatiquement les fichiers de <code>.gitignore</code></td></tr>
<tr><td>🔍 Scan Files</td><td>Construire l'arborescence avec métadonnées</td></tr>
</tbody>
</table>

<h4>Onglet Filtres</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td><b>Préréglages d'extensions</b></td><td>Changement rapide entre ensembles de langages (Python, Web, Golang, Rust, C#, etc.)</td></tr>
<tr><td><b>Extensions</b></td><td>Liste blanche d'extensions personnalisées</td></tr>
<tr><td><b>Chemins ignorés</b></td><td>Ignorer dossiers/fichiers (node_modules, .git, build, dist, etc.)</td></tr>
<tr><td>☑ Inclure l'arborescence</td><td>Ajoute la structure des dossiers au prompt</td></tr>
<tr><td>☑ Inclure la carte de dépendances</td><td>Analyse AST des imports (Python/JS/TS)</td></tr>
<tr><td>☑ Inclure le graphe Mermaid</td><td>Diagramme d'architecture au format Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>Sauvegarder des préréglages :</b> configurez les filtres, cliquez 💾, entrez un nom.</p>

<h4>Onglet Prompts</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td><b>Préréglages de prompts</b></td><td>Changement rapide du prompt système (Code Review, Bug Hunter, Refactoring, etc.)</td></tr>
<tr><td><b>Prompt système</b></td><td>Prompt personnalisé — envoyé au LLM comme contexte système</td></tr>
<tr><td><b>🧩 Appliquer un correctif JSON</b></td><td>Collez la réponse JSON du LLM — prévisualisez les diffs et appliquez au disque</td></tr>
</tbody>
</table>

<p><b>Utilisation des correctifs JSON :</b></p>
<ol>
<li>Demandez au LLM un tableau JSON : <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Collez le JSON, cliquez <b>"Next"</b> → le <b>Safety Diff Viewer</b> s'ouvre</li>
<li>Cochez/décochez les fichiers, cliquez éventuellement <b>"🤖 Check via LLM"</b></li>
<li>Cliquez <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. Paramètres de format de sortie</h3>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>Supprime les espaces et lignes vides</td></tr>
<tr><td>☑ Agressif</td><td>Minification agressive — supprime toutes les lignes vides</td></tr>
<tr><td>☑ No Comments</td><td>Supprime tous les commentaires</td></tr>
<tr><td>☑ No Secrets</td><td>Masque les clés API, mots de passe, tokens</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Supprime les corps de fonctions</b> — économie maximale de tokens</td></tr>
<tr><td>☑ Dedup</td><td>Supprime les fichiers en double au contenu identique</td></tr>
<tr><td>☑ Checkpoints</td><td>Sauvegarde les points de contrôle intermédiaires</td></tr>
<tr><td>☑ Auto-Watch</td><td>Retraite automatique en cas de modification des fichiers</td></tr>
<tr><td>Format</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Sélecteur de template Jinja2</td></tr>
</tbody>
</table>

<p><b>Mode Skeleton :</b> supprime l'implémentation des fonctions (<code>def func_name(...):  # ... implémentation ...</code>), préserve toutes les classes — permet au LLM de comprendre des projets massifs avec un minimum de tokens.</p>

<p><b>Minify vs Agressif :</b> <b>Minify</b> supprime les espaces de début/fin de ligne et les lignes vides — sans risque pour toute base de code, réduit les tokens sans affecter la lisibilité. <b>Agressif</b> ajoute une passe supplémentaire qui élimine les espaces de fin sur chaque ligne pour une compression maximale. Combinez les deux lorsque vous devez faire tenir plus de code dans une fenêtre de contexte limitée.</p>

<p><b>Dedup :</b> détecte automatiquement les fichiers au contenu identique dans votre projet et exclut les doublons de la sortie — empêche le LLM de voir deux fois le même code et de gaspiller des tokens.</p>

<p><b>Checkpoints :</b> sauvegarde les résultats intermédiaires à chaque étape du pipeline (avant le nettoyage, après la minification, etc.) dans le dossier <code>checkpoints/</code>. Utile pour déboguer ce que chaque étape de traitement fait ou comparer les sorties côte à côte.</p>

<p><b>Auto-Watch :</b> surveille les fichiers de votre projet via l'observateur de fichiers du système. Lorsqu'un fichier est sauvegardé, le pipeline s'exécute automatiquement à nouveau — idéal pendant le développement actif lorsque vous avez besoin de mises à jour continues du prompt.</p>

<h3>5. Boutons d'action</h3>
<table>
<thead><tr><th>Bouton</th><th>Action</th></tr></thead>
<tbody>
<tr><td>👀 Preview</td><td><b>Advanced Preview Dialog</b> — onglets « Final Prompt » + « Before/After »</td></tr>
<tr><td>📋 Copy to Clipboard</td><td>Copie le résultat — collez dans ChatGPT / Claude</td></tr>
<tr><td>🚀 Send to ChatGPT / Claude</td><td>Ouvre le chat web et colle le contexte</td></tr>
<tr><td>💻 Open in Editor</td><td>Ouvre dans VS Code / Cursor</td></tr>
<tr><td>💾 Save to File</td><td>Sauvegarde le résultat sur le disque</td></tr>
</tbody>
</table>

<h3>6. Advanced Preview Dialog</h3>
<p><b>Onglet « 📝 Final Prompt » :</b> liste des fichiers (gauche) + texte complet avec coloration (droite). Copy All / Copy File.</p>
<p><b>Onglet « 🔍 Before/After » :</b> diff coloré entre l'original et l'optimisé. Compteur : <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM & OS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>Vérification automatique du correctif LLM avant application</td></tr>
<tr><td>URL / Key / Model</td><td>Point d'accès API (OpenAI par défaut), clé, modèle</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">Intégration OS</th></tr></thead>
<tbody>
<tr><td>Installer le menu contextuel</td><td>« Open with CodeContext AI » dans le menu contextuel</td></tr>
<tr><td>Ajouter au PATH</td><td>Commande CLI globale <code>codecontext</code></td></tr>
<tr><td>Éditeur</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Thèmes</h3>
<ul>
<li><b>Thème :</b> Apple, Modern — <b>Mode :</b> clair / sombre</li>
<li>📂 Ouvrir le dossier des thèmes / ➕ Importer un thème (.json)</li>
</ul>

<h3>9. 📊 Analytique des tokens</h3>
<p>Tableau : chemin du fichier, tokens (tiktoken), compression, économie %, coût pour le modèle.</p>

<h3>10. 🎛️ Personnalisation UI (v1.14+)</h3>
<p>Cliquez <b>⚙</b> à côté de la version — dialogue « Interface Settings (Premiere Pro style) ». Activez/désactivez les onglets (Sources, Filters, Prompts, LLM & OS, Themes) et les boutons d'action (Preview, Clipboard, ChatGPT, Editor, File).</p>

<h3>11. Palette de commandes</h3>
<p><code>Ctrl+Shift+P</code> — accès sans souris à toutes les actions.</p>

<h3>12. 🔌 Système de plugins (v1.25+)</h3>
<p><b>CodeContext AI</b> prend en charge un <b>système de plugins Python</b> qui vous permet d'étendre l'application avec des fonctionnalités personnalisées.</p>

<h4>📁 Structure du plugin</h4>
<pre>my_plugin/
├── manifest.json          # Métadonnées du plugin
├── requirements.txt       # (Optionnel) Dépendances pip
├── locales/
│   ├── en.json            # Traductions anglaises
│   └── ru.json            # Traductions russes
└── plugin.py              # Point d'entrée</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Fait quelque chose d'utile",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Exemple)</h4>
<pre>from src.services.plugin_manager import IPlugin

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, controller: MainController) -> None:
        # Les traductions du dossier locales/ sont chargées automatiquement
        # Enregistrer un onglet dans la barre latérale
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("Bonjour du plugin!")
        )
        # Enregistrer un bouton d'action
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Clic sur le plugin")
        )
        api.add_log("Mon plugin initialisé")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Sécurité</h4>
<ul>
<li>Les plugins ont <b>un accès complet à Python</b> — installez uniquement depuis des sources fiables</li>
<li>Au premier chargement, une boîte de dialogue de sécurité demande votre approbation avant d'activer un plugin</li>
<li>Si <code>requirements.txt</code> existe, vous verrez un journal d'installation pip en direct avant le chargement</li>
<li>Les plugins approuvés sont mémorisés dans les paramètres (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 API Plugin</h4>
<table>
<thead><tr><th>Propriété / Méthode</th><th>Description</th></tr></thead>
<tbody>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Ajouter un onglet à la barre latérale gauche</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Ajouter un bouton au menu déroulant « Plugins 🔽 »</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Écrire dans le panneau de journal de l'application</td></tr>
</tbody>
</table>

<h4>⚙️ Visibilité</h4>
<p>Les onglets et boutons des plugins peuvent être activés/désactivés via <b>⚙ Personnalisation UI</b> — ils apparaissent aux côtés des onglets/boutons intégrés avec leurs propres cases à cocher.</p>

<hr>

<h2>💻 Mode CLI</h2>
<pre>python main.py --cli --path /chemin/vers/projet [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Paramètre</th><th>Type</th><th>Description</th><th>Exemple</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>Mode CLI (sans GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>liste</td><td>Chemin du projet</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Extensions</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Chemins ignorés</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Activer la minification</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Supprimer les commentaires</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Masquer les secrets</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Mode squelette</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Fichier de sortie</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Afficher sur stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Modifications Git uniquement</td><td><code>--git</code></td></tr>
<tr><td><code>--git-base</code></td><td>str</td><td>Branche de base pour git diff en CI/CD</td><td><code>--git-base origin/main</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Respecter .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Arborescence des fichiers</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Graphe Mermaid</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Carte de dépendances</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>Correctif JSON LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Template Jinja2</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Prompt système personnalisé</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Exemples</h3>
<pre># Exécution minimale
python main.py --cli --path ./myapp --stdout

# Analyse complète avec XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Diff Git
python main.py --cli --path ./myapp --git --gitignore --stdout

# Correctif JSON LLM
python main.py --cli --path ./myapp --patch llm_response.json

# Template Jinja2 personnalisé
python main.py --cli --path ./myapp --template my.j2 --stdout

# Diagramme Mermaid
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Chemins multiples
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml

# CI/CD — diff par rapport à la branche de base
python main.py --cli --path . --git --git-base origin/main --minify true --stdout</pre>

<hr>

<h2>🏗️ Stack technique</h2>
<table>
<thead><tr><th>Composant</th><th>Technologie</th></tr></thead>
<tbody>
<tr><td>Langage</td><td>Python 3.10+</td></tr>
<tr><td>Framework GUI</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architecture</td><td>Clean Architecture</td></tr>
<tr><td>Tokenisation</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Templating</td><td>jinja2 (11 modèles intégrés)</td></tr>
<tr><td>Analyseurs AST</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribution</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation)</b> — indexation de codebases massives via une base vectorielle locale (Chroma/FAISS).</li>
<li>🚫 <b>Parsing approfondi du .gitignore</b> — prise en charge des fichiers <code>.gitignore</code> imbriqués et du <code>~/.gitignore</code> global.</li>
<li>☁️ <b>Synchronisation cloud</b> — synchronisez vos préréglages via GitHub Gists.</li>
<li>🌳 <b>Espaces de travail multi-racines</b> — prise en charge améliorée des monorepos (Lerna, NX, Turborepo).</li>
<li>🚀 <b>Pipelines CI/CD</b> — plugins GitHub Actions et GitLab CI pour la génération automatique de contexte PR.</li>
<li>🤖 <b>Intégration directe OpenAI/Anthropic API</b> — pont complet de la génération de prompt à la sortie directe.</li>
<li>🔌 Système de plugins ✅</li>
</ul>

<hr>

<h2>👨‍💻 Équipe</h2>
<p><b>Développeur :</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs sur GitHub</p>

<hr>

<h2>🤝 Contribuer</h2>
<ol>
<li>Forkez le dépôt</li>
<li>Branche : <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit : <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push : <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Suivez les principes SOLID (voir <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Licence</h2>
<p>MIT. Voir <code>LICENSE</code> pour les détails.</p>
