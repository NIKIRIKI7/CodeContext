<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="../assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**Ferramenta de análise de código-fonte e preparação de prompts com IA**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.21.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 Sobre</h2>

<p><b>CodeContext AI</b> é uma ferramenta desktop poderosa para preparar sua base de código para trabalhar com Grandes Modelos de Linguagem (LLMs). Ela escaneia pastas de projeto, analisa a estrutura, constrói gráficos de dependência e gera um único prompt perfeitamente estruturado — otimizado para consumo de tokens e clareza arquitetural.</p>

<h3>❓ Por quê?</h3>
<p>Ao trabalhar com IA, desenvolvedores enfrentam limites de janela de contexto — LLMs "perdem" coerência arquitetural quando o código é copiado em partes. <b>CodeContext AI resolve isso</b>: colete todo o seu projeto em um prompt estruturado em poucos cliques, economizando até 80% em tokens.</p>

<hr>

<h2>🚀 Funcionalidades</h2>

<table>
<thead><tr><th>Funcionalidade</th><th>CodeContext AI</th><th>Manual</th></tr></thead>
<tbody>
<tr><td>🗜️ Minificar + Esqueleto</td><td><b>Até 80%</b> de redução de tokens</td><td>Cópia manual</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Visualizar & aplicar patches JSON</td><td>Não disponível</td></tr>
<tr><td>✅ LLM Checker</td><td>Verificar código automaticamente antes de salvar</td><td>Não disponível</td></tr>
<tr><td>🔗 Grafo de dependência AST</td><td>Python, JS/TS, Vue</td><td>Listagem de arquivos apenas</td></tr>
<tr><td>🖱️ Menu de contexto</td><td>Windows / Linux</td><td>Nenhum</td></tr>
<tr><td>🎨 Temas</td><td>Apple, Modern, JSON personalizado</td><td>Interface fixa</td></tr>
<tr><td>⚙️ Personalização de UI (v1.14+)</td><td>Estilo Premiere Pro</td><td>Interface fixa</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 languages, system auto-detect</td><td>Single language</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Instalação</h2>

<p><b>Pré-requisitos:</b> Python 3.10+, Git</p>

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
<thead><tr><th>Ação</th><th>Comando</th></tr></thead>
<tbody>
<tr><td>Instalar</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Pesquisar</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Atualizar</td><td><code>yay -Syu</code></td></tr>
<tr><td>Remover</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Se <b>yay</b> não estiver instalado:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternativa: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 Modo GUI</h2>
<pre>python main.py</pre>

<h3>1. Visão Geral da Interface</h3>
<p>A janela é dividida em três zonas:</p>
<ul>
<li><b>Barra lateral esquerda (abas)</b> — configurações de varredura, filtros, prompts, configuração de LLM, temas</li>
<li><b>Área central</b> — lista de pastas, árvore de arquivos, análise de tokens</li>
<li><b>Barra de ação superior</b> — alternâncias Minificar/Sem Comentários/Esqueleto, formato de saída, botões de ação</li>
</ul>

<h3>2. Adicionando um Projeto</h3>
<table>
<thead><tr><th>Ação</th><th>Como</th></tr></thead>
<tbody>
<tr><td>Arrastar & soltar</td><td>Basta arrastar uma pasta de projeto para a janela</td></tr>
<tr><td>Diálogo de navegação</td><td>Clique em "+ Pasta PC" na aba <b>Origens</b></td></tr>
<tr><td>Repositório GitHub</td><td>Clique em "+ GitHub / PR" — cole a URL de um repositório ou Pull Request</td></tr>
<tr><td>Salvar configuração</td><td>Clique em "💾 Salvar config" — cria <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>Modos de carregamento do GitHub:</b></p>
<ul>
<li><b>Salvar permanentemente</b> — clona para uma pasta no seu disco</li>
<li><b>Temporário</b> — clona para uma pasta temporária (excluída ao fechar o app)</li>
</ul>

<h3>3. Configuração de Varredura</h3>

<h4>Aba Origens</h4>
<table>
<thead><tr><th>Opção</th><th>Descrição</th></tr></thead>
<tbody>
<tr><td>☑ Apenas Mudanças Git</td><td>Incluir apenas arquivos alterados no último commit</td></tr>
<tr><td>☑ Respeitar .gitignore</td><td>Excluir automaticamente arquivos do <code>.gitignore</code></td></tr>
<tr><td>🔍 Escanear Arquivos</td><td>Construir a árvore de arquivos com metadados</td></tr>
</tbody>
</table>

<h4>Aba Filtros</h4>
<table>
<thead><tr><th>Opção</th><th>Descrição</th></tr></thead>
<tbody>
<tr><td><b>Predefinições de extensão</b></td><td>Alternância rápida entre conjuntos de linguagem (Python, Web, Golang, Rust, C#, etc.)</td></tr>
<tr><td><b>Extensões</b></td><td>Lista de permissões de extensões de arquivo personalizadas</td></tr>
<tr><td><b>Caminhos ignorados</b></td><td>Pular pastas/arquivos (node_modules, .git, build, dist, etc.)</td></tr>
<tr><td>☑ Incluir árvore de arquivos</td><td>Antepõe a estrutura de pastas ao prompt</td></tr>
<tr><td>☑ Incluir mapa de dependências</td><td>Análise de importação baseada em AST para Python/JS/TS</td></tr>
<tr><td>☑ Incluir gráfico Mermaid</td><td>Diagrama de arquitetura no formato Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>Salvando predefinições personalizadas:</b> configure os filtros, clique em 💾, digite um nome.</p>

<h4>Aba Prompts</h4>
<table>
<thead><tr><th>Opção</th><th>Descrição</th></tr></thead>
<tbody>
<tr><td><b>Predefinições de prompt</b></td><td>Mudança rápida de prompt de sistema (Revisão de Código, Caçador de Bugs, Refatoração, etc.)</td></tr>
<tr><td><b>Prompt de sistema</b></td><td>Prompt personalizado — enviado ao LLM como contexto de sistema</td></tr>
<tr><td><b>🧩 Aplicar patch JSON</b></td><td>Colar resposta JSON do LLM — visualizar diff e aplicar ao disco</td></tr>
</tbody>
</table>

<p><b>Usando patches JSON:</b></p>
<ol>
<li>Peça ao LLM um array JSON: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Cole o JSON, clique em <b>"Avançar"</b> → <b>Visualizador de Diff de Segurança</b> abre</li>
<li>Marque/desmarque arquivos, opcionalmente clique em <b>"🤖 Verificar via LLM"</b></li>
<li>Clique em <b>"💾 Salvar selecionados no disco"</b></li>
</ol>

<h3>4. Configurações de Formato de Saída</h3>
<table>
<thead><tr><th>Opção</th><th>Descrição</th></tr></thead>
<tbody>
<tr><td>☑ Minificar</td><td>Remove espaços em branco e linhas em branco</td></tr>
<tr><td>☑ Sem Comentários</td><td>Remove todos os comentários</td></tr>
<tr><td>☑ Sem Segredos</td><td>Mascara chaves de API, senhas, tokens</td></tr>
<tr><td>☑ Esqueleto ☠️</td><td><b>Remove corpos de funções</b> — economia máxima de tokens</td></tr>
<tr><td>Formato</td><td>Markdown, XML, Plain, JSONL Chunks, Personalizado (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Seletor de template Jinja2</td></tr>
</tbody>
</table>

<p><b>Modo Esqueleto:</b> remove implementações de funções (<code>def func_name(...):  # ... implementation ...</code>), preservando todas as classes — permite que o LLM entenda projetos massivos com tokens mínimos.</p>

<h3>5. Botões de Ação</h3>
<table>
<thead><tr><th>Botão</th><th>Ação</th></tr></thead>
<tbody>
<tr><td>👀 Visualizar</td><td><b>Diálogo de Visualização Avançada</b> — abas "Prompt Final" + "Antes/Depois"</td></tr>
<tr><td>📋 Copiar para Área de Transferência</td><td>Copiar resultado — colar no ChatGPT / Claude</td></tr>
<tr><td>🚀 Enviar para ChatGPT / Claude</td><td>Abre chat web e cola o contexto</td></tr>
<tr><td>💻 Abrir no Editor</td><td>Abre no VS Code / Cursor</td></tr>
<tr><td>💾 Salvar em Arquivo</td><td>Salvar resultado no disco</td></tr>
</tbody>
</table>

<h3>6. Diálogo de Visualização Avançada</h3>
<p><b>Aba "📝 Prompt Final":</b> lista de arquivos (esquerda) + texto completo com realce (direita). Copiar Tudo / Copiar Arquivo.</p>
<p><b>Aba "🔍 Antes/Depois":</b> diff colorido entre original e otimizado. Contador: <code>Antes: 1500 → Depois: 300 (80%)</code>.</p>

<h3>7. LLM e SO</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Ativar verificação</td><td>Verificação automática de patch LLM antes de aplicar</td></tr>
<tr><td>URL / Chave / Modelo</td><td>Endpoint da API (padrão OpenAI), chave, modelo</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">Integração com SO</th></tr></thead>
<tbody>
<tr><td>Instalar menu de contexto</td><td>"Abrir com CodeContext AI" no menu de clique direito</td></tr>
<tr><td>Adicionar ao PATH</td><td>Comando CLI global <code>codecontext</code></td></tr>
<tr><td>Editor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Temas</h3>
<ul>
<li><b>Tema:</b> Apple, Modern — <b>Modo:</b> claro / escuro</li>
<li>📂 Abrir pasta de temas / ➕ Importar tema (.json)</li>
</ul>

<h3>9. 📊 Análise de Tokens</h3>
<p>Tabela: caminho do arquivo, tokens (tiktoken), compressão, % de economia, custo por modelo.</p>

<h3>10. 🎛️ Personalização de UI (v1.14+)</h3>
<p>Clique em <b>⚙</b> ao lado da versão — diálogo "Configurações de Interface (estilo Premiere Pro)". Alterne abas (Origens, Filtros, Prompts, LLM & SO, Temas) e botões de ação (Visualizar, Área de Transferência, ChatGPT, Editor, Arquivo).</p>

<h3>11. Paleta de Comandos</h3>
<p><code>Ctrl+Shift+P</code> — acesso a todas as ações sem usar o mouse.</p>

<hr>

<h2>💻 Modo CLI</h2>
<pre>python main.py --cli --path /caminho/para/projeto [opções]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parâmetro</th><th>Tipo</th><th>Descrição</th><th>Exemplo</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>Modo CLI (sem GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>Caminho do projeto</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Extensões</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Caminhos ignorados</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Habilitar minificação</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Remover comentários</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Mascarar segredos</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Modo esqueleto</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Arquivo de saída</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Imprimir no stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Apenas mudanças Git</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Respeitar .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Árvore de arquivos</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Gráfico Mermaid</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Mapa de dependências</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>Patch JSON LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Template Jinja2</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Prompt de sistema personalizado</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Exemplos</h3>
<pre># Execução mínima
python main.py --cli --path ./myapp --stdout

# Análise completa com XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# Patch JSON LLM
python main.py --cli --path ./myapp --patch llm_response.json

# Template Jinja2 personalizado
python main.py --cli --path ./myapp --template my.j2 --stdout

# Diagrama Mermaid
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Múltiplos caminhos
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Stack Tecnológica</h2>
<table>
<thead><tr><th>Componente</th><th>Tecnologia</th></tr></thead>
<tbody>
<tr><td>Linguagem</td><td>Python 3.10+</td></tr>
<tr><td>Framework GUI</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Arquitetura</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Tokenização</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Templating</td><td>jinja2 (11 embutidos)</td></tr>
<tr><td>Parsers AST</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribuição</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>🍎 Menu de contexto do macOS Finder</li>
<li>🤖 Integração direta com API OpenAI/Anthropic</li>
<li>🏛️ Análise de Arquitetura Hexagonal</li>
<li>🔌 Sistema de plugins</li>
<li>🌐 i18n no aplicativo</li>
</ul>

<hr>

<h2>👨‍💻 Equipe</h2>
<p><b>Desenvolvedor:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs no GitHub</p>

<hr>

<h2>🤝 Contribuindo</h2>
<ol>
<li>Fork do repositório</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Siga os princípios SOLID (veja <code>../docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Licença</h2>
<p>MIT. Veja <code>../LICENSE</code> para detalhes.</p>
