<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**Herramienta de análisis de código fuente y preparación de prompts impulsada por IA**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.25.1-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 Acerca de</h2>

<p><b>CodeContext AI</b> es una potente herramienta de escritorio para preparar su base de código para trabajar con Grandes Modelos de Lenguaje (LLM). Escanea carpetas de proyectos, analiza la estructura, construye gráficos de dependencias y genera un único prompt perfectamente estructurado — optimizado para el consumo de tokens y la claridad arquitectónica.</p>

<h3>❓ ¿Por qué?</h3>
<p>Al trabajar con IA, los desarrolladores se enfrentan a los límites de la ventana de contexto — los LLM « pierden » la coherencia arquitectónica cuando el código se copia en partes. <b>CodeContext AI resuelve esto</b>: recopile todo su proyecto en un prompt estructurado con unos pocos clics, ahorrando hasta un 80 % de tokens.</p>

<hr>

<h2>🚀 Funcionalidades</h2>

<table>
<thead><tr><th>Funcionalidad</th><th>CodeContext AI</th><th>Manual</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify + Skeleton</td><td><b>Hasta un 80 %</b> de reducción de tokens</td><td>Copiar y pegar manual</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Previsualizar y aplicar parches JSON</td><td>No disponible</td></tr>
<tr><td>✅ LLM Checker</td><td>Verificación automática del código antes de guardar</td><td>No disponible</td></tr>
<tr><td>🔗 Grafo de dependencias AST</td><td>Python, JS/TS, Vue</td><td>Solo listado de archivos</td></tr>
<tr><td>🖱️ Menú contextual</td><td>Windows / Linux / macOS</td><td>Ninguno</td></tr>
<tr><td>🎨 Temas</td><td>Apple, Modern, JSON personalizado</td><td>IU fija</td></tr>
<tr><td>⚙️ Personalización de IU (v1.14+)</td><td>Estilo Premiere Pro</td><td>IU fija</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 idiomas, detección automática</td><td>Idioma único</td></tr>
<tr><td>♻️ Deduplicación (v1.23+)</td><td>Detecta y omite archivos con contenido idéntico</td><td>Comprobación manual</td></tr>
<tr><td>⚡ Minificación agresiva (v1.23+)</td><td>Compresión extra — elimina espacios finales en cada línea</td><td>Eliminación manual</td></tr>
<tr><td>📌 Puntos de control (v1.23+)</td><td>Guarda instantáneas antes/después para depuración</td><td>No disponible</td></tr>
<tr><td>👁️ Vigilancia auto (v1.23+)</td><td>Vigila archivos y reprocesa al cambiar</td><td>No disponible</td></tr>
<tr><td>🔌 Sistema de plugins (v1.25+)</td><td>Extiende con plugins Python — pestañas, acciones e i18n personalizadas</td><td>No disponible</td></tr>
<tr><td>🚦 Integración CI/CD</td><td>GitHub Actions y GitLab CI — generación automática de contexto PR mediante <code>--git-base</code></td><td>Not available</td></tr>
<tr><td>🌳 Monorepo Support (v1.25+)</td><td>Lerna, NX, Turborepo, pnpm workspaces — cross-package imports, root config discovery</td><td>Not available</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Instalación</h2>

<p><b>Requisitos:</b> Python 3.10+, Git</p>

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

<pre># Luego ejecutar:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Acción</th><th>Comando</th></tr></thead>
<tbody>
<tr><td>Instalar</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Buscar</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Actualizar</td><td><code>yay -Syu</code></td></tr>
<tr><td>Eliminar</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Si <b>yay</b> no está instalado:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternativa: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 Modo GUI</h2>
<pre>python main.py</pre>

<h3>1. Vista general de la interfaz</h3>
<p>La ventana está dividida en tres zonas:</p>
<ul>
<li><b>Barra lateral izquierda (pestañas)</b> — ajustes de escaneo, filtros, prompts, configuración LLM, temas</li>
<li><b>Área central</b> — lista de carpetas, árbol de archivos, analítica de tokens</li>
<li><b>Barra de acciones superior</b> — interruptores Minify/No Comments/Skeleton, formato de salida, botones de acción</li>
</ul>

<h3>2. Añadir un proyecto</h3>
<table>
<thead><tr><th>Acción</th><th>Cómo</th></tr></thead>
<tbody>
<tr><td>Arrastrar y soltar</td><td>Simplemente arrastre una carpeta de proyecto a la ventana</td></tr>
<tr><td>Diálogo de exploración</td><td>Haga clic en "+ Папка ПК" en la pestaña <b>Sources</b></td></tr>
<tr><td>Repositorio GitHub</td><td>Haga clic en "+ GitHub / PR" — pegue una URL de repositorio o Pull Request</td></tr>
<tr><td>Guardar configuración</td><td>Haga clic en "💾 Save config" — crea <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>Modos de carga GitHub:</b></p>
<ul>
<li><b>Guardar permanentemente</b> — clona en una carpeta de su disco</li>
<li><b>Temporal</b> — clona en una carpeta temporal (se elimina al cerrar la app)</li>
</ul>

<h3>3. Configuración de escaneo</h3>

<h4>Pestaña Sources</h4>
<table>
<thead><tr><th>Opción</th><th>Descripción</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>Incluir solo archivos modificados en el último commit</td></tr>
<tr><td>☑ Respect .gitignore</td><td>Excluir automáticamente archivos de <code>.gitignore</code></td></tr>
<tr><td>🔍 Scan Files</td><td>Construir el árbol de archivos con metadatos</td></tr>
</tbody>
</table>

<h4>Pestaña Filters</h4>
<table>
<thead><tr><th>Opción</th><th>Descripción</th></tr></thead>
<tbody>
<tr><td><b>Preajustes de extensiones</b></td><td>Cambio rápido entre conjuntos de idiomas (Python, Web, Golang, Rust, C#, etc.)</td></tr>
<tr><td><b>Extensiones</b></td><td>Lista blanca de extensiones de archivo personalizadas</td></tr>
<tr><td><b>Rutas ignoradas</b></td><td>Omitir carpetas/archivos (node_modules, .git, build, dist, etc.)</td></tr>
<tr><td>☑ Include file tree</td><td>Antepone la estructura de carpetas al prompt</td></tr>
<tr><td>☑ Include dependency map</td><td>Análisis de importaciones basado en AST para Python/JS/TS</td></tr>
<tr><td>☑ Include Mermaid graph</td><td>Diagrama de arquitectura en formato Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>Guardar preajustes personalizados:</b> configure filtros, haga clic en 💾, introduzca un nombre.</p>

<h4>Pestaña Prompts</h4>
<table>
<thead><tr><th>Opción</th><th>Descripción</th></tr></thead>
<tbody>
<tr><td><b>Preajustes de prompts</b></td><td>Cambio rápido del prompt del sistema (Code Review, Bug Hunter, Refactoring, etc.)</td></tr>
<tr><td><b>System prompt</b></td><td>Prompt personalizado — enviado al LLM como contexto del sistema</td></tr>
<tr><td><b>🧩 Aplicar parche JSON</b></td><td>Pegue la respuesta JSON del LLM — previsualice el diff y aplíquelo al disco</td></tr>
</tbody>
</table>

<p><b>Uso de parches JSON:</b></p>
<ol>
<li>Solicite al LLM un array JSON: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Pegue el JSON, haga clic en <b>"Next"</b> → se abre el <b>Safety Diff Viewer</b></li>
<li>Marque/desmarque archivos, opcionalmente haga clic en <b>"🤖 Check via LLM"</b></li>
<li>Haga clic en <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. Ajustes de formato de salida</h3>
<table>
<thead><tr><th>Opción</th><th>Descripción</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>Elimina espacios en blanco y líneas vacías</td></tr>
<tr><td>☑ Aggressive</td><td>Minificación agresiva — elimina todas las líneas en blanco</td></tr>
<tr><td>☑ No Comments</td><td>Elimina todos los comentarios</td></tr>
<tr><td>☑ No Secrets</td><td>Enmascara claves API, contraseñas, tokens</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Elimina cuerpos de funciones</b> — máximo ahorro de tokens</td></tr>
<tr><td>☑ Dedup</td><td>Elimina archivos duplicados con contenido idéntico</td></tr>
<tr><td>☑ Checkpoints</td><td>Guarda puntos de control intermedios del procesamiento</td></tr>
<tr><td>☑ Auto-Watch</td><td>Reprocesa automáticamente al cambiar archivos</td></tr>
<tr><td>Formato</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Selector de plantilla Jinja2</td></tr>
</tbody>
</table>

<p><b>Modo Skeleton:</b> elimina implementaciones de funciones (<code>def func_name(...):  # ... implementación ...</code>), preserva todas las clases — permite al LLM entender proyectos masivos con tokens mínimos.</p>

<p><b>Minify vs Aggressive:</b> <b>Minify</b> elimina espacios al inicio/final y líneas en blanco — seguro para cualquier código, reduce tokens sin afectar la legibilidad. <b>Aggressive</b> añade una pasada extra que elimina espacios finales en cada línea para máxima compresión. Combine ambas cuando necesite encajar más código en una ventana de contexto limitada.</p>

<p><b>Dedup:</b> detecta automáticamente archivos con contenido idéntico en su proyecto y excluye duplicados de la salida — evita que el LLM vea el mismo código dos veces y desperdicie tokens.</p>

<p><b>Checkpoints:</b> guarda resultados intermedios en cada etapa del pipeline (antes de la limpieza, después de la minificación, etc.) en la carpeta <code>checkpoints/</code>. Útil para depurar lo que hace cada paso o comparar salidas lado a lado.</p>

<p><b>Auto-Watch:</b> monitorea los archivos de su proyecto en busca de cambios usando el vigilante de archivos del SO. Cuando se guarda un archivo, el pipeline se re-ejecuta automáticamente — ideal durante el desarrollo activo cuando necesita actualizaciones continuas del prompt.</p>

<h3>5. Botones de acción</h3>
<table>
<thead><tr><th>Botón</th><th>Acción</th></tr></thead>
<tbody>
<tr><td>👀 Preview</td><td><b>Advanced Preview Dialog</b> — pestañas "Final Prompt" + "Before/After"</td></tr>
<tr><td>📋 Copy to Clipboard</td><td>Copiar resultado — pegar en ChatGPT / Claude</td></tr>
<tr><td>🚀 Send to ChatGPT / Claude</td><td>Abre el chat web y pega el contexto</td></tr>
<tr><td>💻 Open in Editor</td><td>Abre en VS Code / Cursor</td></tr>
<tr><td>💾 Save to File</td><td>Guardar resultado en disco</td></tr>
</tbody>
</table>

<h3>6. Advanced Preview Dialog</h3>
<p><b>Pestaña "📝 Final Prompt":</b> lista de archivos (izquierda) + texto completo resaltado (derecha). Copy All / Copy File.</p>
<p><b>Pestaña "🔍 Before/After":</b> diff coloreado entre original y optimizado. Contador: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM y SO</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>Verificación automática del parche LLM antes de aplicar</td></tr>
<tr><td>URL / Key / Model</td><td>Endpoint API (OpenAI por defecto), clave, modelo</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">Integración con SO</th></tr></thead>
<tbody>
<tr><td>Instalar menú contextual</td><td>"Open with CodeContext AI" en el menú contextual</td></tr>
<tr><td>Añadir al PATH</td><td>Comando CLI global <code>codecontext</code></td></tr>
<tr><td>Editor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Temas</h3>
<ul>
<li><b>Tema:</b> Apple, Modern — <b>Modo:</b> claro / oscuro</li>
<li>📂 Abrir carpeta de temas / ➕ Importar tema (.json)</li>
</ul>

<h3>9. 📊 Analítica de tokens</h3>
<p>Tabla: ruta del archivo, tokens (tiktoken), compresión, ahorro %, costo para el modelo.</p>

<h3>10. 🎛️ Personalización de IU (v1.14+)</h3>
<p>Haga clic en <b>⚙</b> junto a la versión — diálogo "Interface Settings (Premiere Pro style)". Active/desactive pestañas (Sources, Filters, Prompts, LLM & OS, Themes) y botones de acción (Preview, Clipboard, ChatGPT, Editor, File).</p>

<h3>11. Paleta de comandos</h3>
<p><code>Ctrl+Shift+P</code> — acceso sin ratón a todas las acciones.</p>

<h3>12. 🔌 Sistema de plugins (v1.25+)</h3>
<p><b>CodeContext AI</b> soporta un <b>sistema de plugins Python</b> que permite extender la aplicación con funcionalidad personalizada.</p>

<h4>📁 Estructura del plugin</h4>
<pre>my_plugin/
├── manifest.json          # Metadatos del plugin
├── requirements.txt       # (Opcional) dependencias pip
├── locales/
│   ├── en.json            # Traducciones al inglés
│   └── ru.json            # Traducciones al ruso
└── plugin.py              # Punto de entrada</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Hace algo útil",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Ejemplo)</h4>
<pre>from src.api.plugin_api import IPlugin, PluginAPI

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("¡Hola desde el plugin!")
        )
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Acción del plugin clickeada")
        )
        api.add_log("Mi Plugin inicializado")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Seguridad</h4>
<ul>
<li>Los plugins obtienen <b>acceso completo a Python</b> — instale solo desde fuentes confiables</li>
<li>En la primera carga, un diálogo de seguridad solicita su aprobación antes de habilitar un plugin</li>
<li>Si existe <code>requirements.txt</code>, verá un registro en vivo de pip install antes de cargar</li>
<li>Los plugins aprobados se recuerdan en la configuración (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 Plugin API</h4>
<table>
<thead><tr><th>Propiedad / Método</th><th>Descripción</th></tr></thead>
<tbody>
<tr><td><code>api.store</code></td><td>Redux store de solo lectura (acceso a estado vía <code>state.settings.xxx</code>)</td></tr>
<tr><td><code>api.dispatcher</code></td><td>Despachar acciones (ej. <code>UI_ADD_LOG</code>)</td></tr>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Añadir una pestaña a la barra lateral izquierda</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Añadir un botón al menú desplegable "Plugins 🔽"</td></tr>
<tr><td><code>api.add_translations(lang, data)</code></td><td>Añadir traducciones en tiempo de ejecución (fusionadas sobre las incorporadas)</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Escribir en el panel de registro de la aplicación</td></tr>
</tbody>
</table>

<h4>⚙️ Visibilidad</h4>
<p>Las pestañas y botones de acción de los plugins se pueden activar/desactivar mediante <b>⚙ Personalización de IU</b> — aparecen junto a las pestañas/acciones incorporadas con sus propias casillas de verificación.</p>

<hr>

<h2>💻 Modo CLI</h2>
<pre>python main.py --cli --path /ruta/al/proyecto [opciones]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parámetro</th><th>Tipo</th><th>Descripción</th><th>Ejemplo</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>Modo CLI (sin GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>lista</td><td>Ruta del proyecto</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Extensiones</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Rutas ignoradas</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Activar minificación</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Eliminar comentarios</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Enmascarar secretos</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Modo esqueleto</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Archivo de salida</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Imprimir en stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Solo cambios Git</td><td><code>--git</code></td></tr>
<tr><td><code>--git-base</code></td><td>str</td><td>Rama base para git diff en CI/CD</td><td><code>--git-base origin/main</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Respetar .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Árbol de archivos</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Grafo Mermaid</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Mapa de dependencias</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>Parche JSON LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Plantilla Jinja2</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Prompt del sistema personalizado</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Ejemplos</h3>
<pre># Ejecución mínima
python main.py --cli --path ./myapp --stdout

# Análisis completo con XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Diff Git
python main.py --cli --path ./myapp --git --gitignore --stdout

# Parche JSON LLM
python main.py --cli --path ./myapp --patch llm_response.json

# Plantilla Jinja2 personalizada
python main.py --cli --path ./myapp --template my.j2 --stdout

# Diagrama Mermaid
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Rutas múltiples
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml

# CI/CD — diff contra la rama base
python main.py --cli --path . --git --git-base origin/main --minify true --stdout</pre>

<hr>

<h2>🏗️ Stack tecnológico</h2>
<table>
<thead><tr><th>Componente</th><th>Tecnología</th></tr></thead>
<tbody>
<tr><td>Lenguaje</td><td>Python 3.10+</td></tr>
<tr><td>Framework GUI</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Arquitectura</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Tokenización</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Plantillas</td><td>jinja2 (11 incorporadas)</td></tr>
<tr><td>Analizadores AST</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribución</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Hoja de ruta</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation)</b> — indexación de bases de código masivas con base de datos vectorial local (Chroma/FAISS).</li>
<li>🚫 <b>Análisis profundo de .gitignore</b> — soporte para archivos <code>.gitignore</code> anidados y <code>~/.gitignore</code> global.</li>
<li>☁️ <b>Sincronización en la nube</b> — sincronice sus ajustes a través de GitHub Gists.</li>
<li>🌳 <b>Espacios de trabajo multi-raíz</b> — soporte mejorado para monorepos (Lerna, NX, Turborepo).</li>
<li>🚀 <b>Tuberías CI/CD</b> — plugins para GitHub Actions y GitLab CI para generación automática de contexto en PRs.</li>
<li>🤖 <b>Integración directa con OpenAI/Anthropic API</b> — complete el puente desde la generación de prompts hasta la salida directa.</li>
<li>🔌 Sistema de plugins ✅</li>
</ul>

<hr>

<h2>👨‍💻 Equipo</h2>
<p><b>Desarrollador:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs en GitHub</p>

<hr>

<h2>🤝 Contribuir</h2>
<ol>
<li>Fork del repositorio</li>
<li>Rama: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Siga los principios SOLID (ver <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Licencia</h2>
<p>MIT. Consulte <code>LICENSE</code> para más detalles.</p>
