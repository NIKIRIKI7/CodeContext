<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="../assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**أداة تحليل قاعدة الأكواد وإعداد الاستعلامات المدعومة بالذكاء الاصطناعي**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.18.1-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 حول المشروع</h2>

<p><b>CodeContext AI</b> هي أداة سطح مكتب قوية لإعداد قاعدة الأكواد الخاصة بك للعمل مع نماذج اللغة الكبيرة (LLM). تقوم بمسح مجلدات المشروع، وتحليل الهيكل، وبناء رسوم بيانية للتبعيات، وإنشاء استعلام واحد منظم بشكل مثالي — محسّن لاستهلاك الرموز (tokens) والوضوح المعماري.</p>

<h3>❓ لماذا؟</h3>
<p>عند العمل مع الذكاء الاصطناعي، يواجه المطورون حدود نافذة السياق — حيث « تفقد » نماذج LLM التماسك المعماري عندما يتم نسخ الكود في أجزاء. <b>CodeContext AI يحل هذه المشكلة</b>: اجمع مشروعك بالكامل في استعلام منظم بنقرات قليلة، مع توفير يصل إلى 80 % من الرموز.</p>

<hr>

<h2>🚀 الميزات</h2>

<table>
<thead><tr><th>الميزة</th><th>CodeContext AI</th><th>يدوياً</th></tr></thead>
<tbody>
<tr><td>🗜️ تصغير + هيكل عظمي</td><td><b>توفير يصل إلى 80 %</b> من الرموز</td><td>نسخ ولصق يدوي</td></tr>
<tr><td>🧩 مصحح LLM</td><td>معاينة وتطبيق تصحيحات JSON</td><td>غير متاح</td></tr>
<tr><td>✅ مدقق LLM</td><td>تحقق تلقائي من الكود قبل الحفظ</td><td>غير متاح</td></tr>
<tr><td>🔗 رسم بياني للتبعيات AST</td><td>Python, JS/TS, Vue</td><td>قائمة ملفات فقط</td></tr>
<tr><td>🖱️ قائمة سياقية</td><td>Windows / Linux</td><td>لا يوجد</td></tr>
<tr><td>🎨 سمات</td><td>Apple, Modern, JSON مخصص</td><td>واجهة ثابتة</td></tr>
<tr><td>⚙️ تخصيص الواجهة (v1.14+)</td><td>على نمط Premiere Pro</td><td>واجهة ثابتة</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 languages, system auto-detect</td><td>Single language</td></tr>
</tbody>
</table>

<hr>

<h2>📥 التثبيت</h2>

<p><b>المتطلبات:</b> Python 3.10+، Git</p>

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
<thead><tr><th>الإجراء</th><th>الأمر</th></tr></thead>
<tbody>
<tr><td>تثبيت</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>بحث</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>تحديث</td><td><code>yay -Syu</code></td></tr>
<tr><td>إزالة</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>إذا لم يكن <b>yay</b> مثبتاً:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>بديل: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 وضع الواجهة الرسومية (GUI)</h2>
<pre>python main.py</pre>

<h3>1. نظرة عامة على الواجهة</h3>
<p>تنقسم النافذة إلى ثلاث مناطق:</p>
<ul>
<li><b>الشريط الجانبي الأيسر (علامات التبويب)</b> — إعدادات المسح، المرشحات، الاستعلامات، تكوين LLM، السمات</li>
<li><b>المنطقة المركزية</b> — قائمة المجلدات، شجرة الملفات، تحليل الرموز</li>
<li><b>شريط الإجراءات العلوي</b> — مفاتيح التصغير/بدون تعليقات/هيكل عظمي، تنسيق الإخراج، أزرار الإجراءات</li>
</ul>

<h3>2. إضافة مشروع</h3>
<table>
<thead><tr><th>الإجراء</th><th>كيف</th></tr></thead>
<tbody>
<tr><td>سحب وإفلات</td><td>ما عليك سوى سحب مجلد المشروع إلى النافذة</td></tr>
<tr><td>مربع حوار التصفح</td><td>انقر على "+ Папка ПК" في علامة التبويب <b>Sources</b></td></tr>
<tr><td>مستودع GitHub</td><td>انقر على "+ GitHub / PR" — الصق رابط المستودع أو طلب السحب</td></tr>
<tr><td>حفظ الإعدادات</td><td>انقر على "💾 Save config" — ينشئ <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>أوضاع تحميل GitHub:</b></p>
<ul>
<li><b>حفظ دائم</b> — يستنسخ إلى مجلد على القرص الخاص بك</li>
<li><b>مؤقت</b> — يستنسخ إلى مجلد مؤقت (يُحذف عند إغلاق التطبيق)</li>
</ul>

<h3>3. إعدادات المسح</h3>

<h4>علامة التبويب Sources</h4>
<table>
<thead><tr><th>خيار</th><th>وصف</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>تضمين الملفات المعدلة في آخر commit فقط</td></tr>
<tr><td>☑ Respect .gitignore</td><td>استبعاد الملفات من <code>.gitignore</code> تلقائياً</td></tr>
<tr><td>🔍 Scan Files</td><td>بناء شجرة الملفات مع البيانات الوصفية</td></tr>
</tbody>
</table>

<h4>علامة التبويب Filters</h4>
<table>
<thead><tr><th>خيار</th><th>وصف</th></tr></thead>
<tbody>
<tr><td><b>إعدادات مسبقة للإضافات</b></td><td>تبديل سريع بين مجموعات اللغات (Python, Web, Golang, Rust, C#، إلخ)</td></tr>
<tr><td><b>الإضافات</b></td><td>قائمة بيضاء مخصصة لإضافات الملفات</td></tr>
<tr><td><b>المسارات المتجاهلة</b></td><td>تخطي المجلدات/الملفات (node_modules, .git, build, dist، إلخ)</td></tr>
<tr><td>☑ Include file tree</td><td>يسبق هيكل المجلدات للاستعلام</td></tr>
<tr><td>☑ Include dependency map</td><td>تحليل الاستيراد المستند إلى AST لـ Python/JS/TS</td></tr>
<tr><td>☑ Include Mermaid graph</td><td>مخطط معماري بتنسيق Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>حفظ الإعدادات المسبقة المخصصة:</b> قم بتكوين المرشحات، انقر على 💾، أدخل اسماً.</p>

<h4>علامة التبويب Prompts</h4>
<table>
<thead><tr><th>خيار</th><th>وصف</th></tr></thead>
<tbody>
<tr><td><b>إعدادات مسبقة للاستعلامات</b></td><td>تغيير سريع لاستعلام النظام (Code Review, Bug Hunter, Refactoring، إلخ)</td></tr>
<tr><td><b>استعلام النظام</b></td><td>استعلام مخصص — يُرسل إلى LLM كسياق للنظام</td></tr>
<tr><td><b>🧩 تطبيق تصحيح JSON</b></td><td>الصق استجابة JSON من LLM — اعرض الفروق وطبقها على القرص</td></tr>
</tbody>
</table>

<p><b>استخدام تصحيحات JSON:</b></p>
<ol>
<li>اطلب من LLM مصفوفة JSON: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>الصق JSON، انقر على <b>"Next"</b> → يفتح <b>Safety Diff Viewer</b></li>
<li>حدد/ألغ تحديد الملفات، اختيارياً انقر على <b>"🤖 Check via LLM"</b></li>
<li>انقر على <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. إعدادات تنسيق الإخراج</h3>
<table>
<thead><tr><th>خيار</th><th>وصف</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>يزيل المسافات البيضاء والأسطر الفارغة</td></tr>
<tr><td>☑ No Comments</td><td>يزيل جميع التعليقات</td></tr>
<tr><td>☑ No Secrets</td><td>يخفي مفاتيح API وكلمات المرور والرموز</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>يزيل أجسام الدوال</b> — أقصى توفير للرموز</td></tr>
<tr><td>التنسيق</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 قالب</td><td>محدد قالب Jinja2</td></tr>
</tbody>
</table>

<p><b>وضع الهيكل العظمي:</b> يزيل تطبيقات الدوال (<code>def func_name(...):  # ... تطبيق ...</code>)، مع الحفاظ على جميع الفئات — يسمح لـ LLM بفهم المشاريع الضخمة بأقل عدد من الرموز.</p>

<h3>5. أزرار الإجراءات</h3>
<table>
<thead><tr><th>الزر</th><th>الإجراء</th></tr></thead>
<tbody>
<tr><td>👀 معاينة</td><td><b>حوار المعاينة المتقدم</b> — علامتا تبويب "الاستعلام النهائي" + "قبل/بعد"</td></tr>
<tr><td>📋 نسخ إلى الحافظة</td><td>نسخ النتيجة — الصق في ChatGPT / Claude</td></tr>
<tr><td>🚀 إرسال إلى ChatGPT / Claude</td><td>يفتح الدردشة على الويب ويلصق السياق</td></tr>
<tr><td>💻 فتح في المحرر</td><td>يفتح في VS Code / Cursor</td></tr>
<tr><td>💾 حفظ إلى ملف</td><td>حفظ النتيجة على القرص</td></tr>
</tbody>
</table>

<h3>6. حوار المعاينة المتقدم</h3>
<p><b>علامة التبويب "📝 الاستعلام النهائي":</b> قائمة الملفات (يسار) + النص الكامل مع الإبراز (يمين). نسخ الكل / نسخ الملف.</p>
<p><b>علامة التبويب "🔍 قبل/بعد":</b> فرق ملون بين الأصل والنسخة المحسنة. العداد: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM ونظام التشغيل</h3>
<table>
<thead><tr><th colspan="2">مدقق LLM</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>تحقق تلقائي من تصحيح LLM قبل التطبيق</td></tr>
<tr><td>URL / Key / Model</td><td>نقطة نهاية API (OpenAI افتراضياً)، المفتاح، النموذج</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">التكامل مع نظام التشغيل</th></tr></thead>
<tbody>
<tr><td>تثبيت القائمة السياقية</td><td>"Open with CodeContext AI" في قائمة النقر بزر الماوس الأيمن</td></tr>
<tr><td>إضافة إلى PATH</td><td>أمر CLI عام <code>codecontext</code></td></tr>
<tr><td>المحرر</td><td><code>code</code>، <code>cursor</code>، <code>idea</code>، <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. السمات</h3>
<ul>
<li><b>السمة:</b> Apple، Modern — <b>الوضع:</b> فاتح / داكن</li>
<li>📂 فتح مجلد السمات / ➕ استيراد سمة (.json)</li>
</ul>

<h3>9. 📊 تحليل الرموز</h3>
<p>جدول: مسار الملف، الرموز (tiktoken)، الضغط، نسبة التوفير، التكلفة للنموذج.</p>

<h3>10. 🎛️ تخصيص الواجهة (v1.14+)</h3>
<p>انقر على <b>⚙</b> بجانب الإصدار — حوار "Interface Settings (Premiere Pro style)". قم بتشغيل/إيقاف علامات التبويب (Sources, Filters, Prompts, LLM & OS, Themes) وأزرار الإجراءات (Preview, Clipboard, ChatGPT, Editor, File).</p>

<h3>11. لوحة الأوامر</h3>
<p><code>Ctrl+Shift+P</code> — وصول بدون ماوس إلى جميع الإجراءات.</p>

<hr>

<h2>💻 وضع CLI</h2>
<pre>python main.py --cli --path /مسار/المشروع [خيارات]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>المعامل</th><th>النوع</th><th>الوصف</th><th>مثال</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>علامة</td><td>وضع CLI (بدون واجهة رسومية)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>قائمة</td><td>مسار المشروع</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>نص</td><td>إضافات الملفات</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>نص</td><td>المسارات المتجاهلة</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>تعداد</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>تعداد</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>علامة</td><td>تفعيل التصغير</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>علامة</td><td>إزالة التعليقات</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>علامة</td><td>إخفاء الأسرار</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>علامة</td><td>وضع الهيكل العظمي</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>نص</td><td>ملف الإخراج</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>علامة</td><td>طباعة إلى stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>علامة</td><td>تغييرات Git فقط</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>علامة</td><td>احترام .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>علامة</td><td>شجرة الملفات</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>علامة</td><td>رسم Mermaid البياني</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>علامة</td><td>خريطة التبعيات</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>نص</td><td>تصحيح JSON من LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>نص</td><td>قالب Jinja2</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>نص</td><td>استعلام نظام مخصص</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>أمثلة</h3>
<pre># تشغيل بسيط
python main.py --cli --path ./myapp --stdout

# تحليل كامل مع XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# فرق Git
python main.py --cli --path ./myapp --git --gitignore --stdout

# تصحيح JSON من LLM
python main.py --cli --path ./myapp --patch llm_response.json

# قالب Jinja2 مخصص
python main.py --cli --path ./myapp --template my.j2 --stdout

# رسم Mermaid البياني
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# مسارات متعددة
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ التقنيات المستخدمة</h2>
<table>
<thead><tr><th>المكون</th><th>التقنية</th></tr></thead>
<tbody>
<tr><td>اللغة</td><td>Python 3.10+</td></tr>
<tr><td>إطار الواجهة</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>الهندسة المعمارية</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>تقسيم الرموز</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>القوالب</td><td>jinja2 (11 قالباً مدمجاً)</td></tr>
<tr><td>محللات AST</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>التوزيع</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ خارطة الطريق</h2>
<ul>
<li>🍎 قائمة سياقية لنظام macOS Finder</li>
<li>🤖 تكامل مباشر مع OpenAI/Anthropic API</li>
<li>🏛️ تحليل الهندسة السداسية</li>
<li>🔌 نظام إضافات</li>
<li>🌐 تدويل داخل التطبيق</li>
</ul>

<hr>

<h2>👨‍💻 الفريق</h2>
<p><b>المطور:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · المشكلات وطلبات السحب على GitHub</p>

<hr>

<h2>🤝 المساهمة</h2>
<ol>
<li>انسخ (Fork) المستودع</li>
<li>الفرع: <code>git checkout -b feature/AmazingFeature</code></li>
<li>الالتزام: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>الدفع: <code>git push origin feature/AmazingFeature</code></li>
<li>طلب سحب (Pull Request)</li>
</ol>
<p>اتبع مبادئ SOLID (انظر <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 الترخيص</h2>
<p>MIT. انظر <code>LICENSE</code> للتفاصيل.</p>
