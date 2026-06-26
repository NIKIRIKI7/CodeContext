<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-संचालित कोडबेस विश्लेषण और प्रॉम्प्ट तैयारी उपकरण**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.28.0-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 परिचय</h2>

<p><b>CodeContext AI</b> आपके कोडबेस को बड़े भाषा मॉडल (LLM) के साथ काम करने के लिए तैयार करने हेतु एक शक्तिशाली डेस्कटॉप उपकरण है। यह प्रोजेक्ट फ़ोल्डरों को स्कैन करता है, संरचना का विश्लेषण करता है, निर्भरता ग्राफ बनाता है, और एक एकल, पूरी तरह से संरचित प्रॉम्प्ट उत्पन्न करता है — टोकन खपत और आर्किटेक्चरल स्पष्टता के लिए अनुकूलित।</p>

<h3>❓ क्यों?</h3>
<p>AI के साथ काम करते समय, डेवलपर्स को संदर्भ विंडो टोकन सीमाओं का सामना करना पड़ता है — कोड को भागों में कॉपी करने पर LLM आर्किटेक्चरल सुसंगतता "खो" देते हैं। <b>CodeContext AI इसे हल करता है</b>: कुछ क्लिक में अपने पूरे प्रोजेक्ट को एक संरचित प्रॉम्प्ट में इकट्ठा करें, टोकन पर 80% तक बचत करें।</p>

<hr>

<h2>🚀 विशेषताएं</h2>

<table>
<thead><tr><th>विशेषता</th><th>CodeContext AI</th><th>मैन्युअल</th></tr></thead>
<tbody>
<tr><td>🗜️ मिनिफाई + स्केलेटन</td><td><b>80% तक</b> टोकन कमी</td><td>मैन्युअल कॉपी-पेस्ट</td></tr>
<tr><td>🧩 LLM पैचर</td><td>JSON पैच का पूर्वावलोकन और आवेदन</td><td>उपलब्ध नहीं</td></tr>
<tr><td>✅ LLM चेकर</td><td>सहेजने से पहले कोड की स्वचालित जांच</td><td>उपलब्ध नहीं</td></tr>
<tr><td>🔗 AST निर्भरता ग्राफ</td><td>Python, JS/TS, Vue</td><td>केवल फ़ाइल सूचीकरण</td></tr>
<tr><td>🖱️ संदर्भ मेनू</td><td>Windows / Linux / macOS</td><td>कोई नहीं</td></tr>
<tr><td>🎨 थीम</td><td>Apple, Modern, कस्टम JSON</td><td>निश्चित UI</td></tr>
<tr><td>⚙️ UI अनुकूलन (v1.14+)</td><td>Premiere Pro-शैली</td><td>निश्चित UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 भाषाएँ, सिस्टम स्वचालित पतलग</td><td>एकल भाषा</td></tr>
<tr><td>♻️ डीडप्लिकेशन (v1.23+)</td><td>समान सामग्री वाली फ़ाइलों का पता लगाता और छोड़ता है</td><td>मैनुअल जाँच</td></tr>
<tr><td>⚡ आक्रमक मिनिफ़ाइ (v1.23+)</td><td>अतिरिक्त संपीड़न — प्रत्येक पंक्ति पर अंतिम रिक्तियों को हटाता है</td><td>मैनुअल हटाना</td></tr>
<tr><td>📌 चेकपॉइंट्स (v1.23+)</td><td>डिबगिंग के लिए पहले/बाद के स्नैपशॉट सहेजें</td><td>उपलब्ध नहीं</td></tr>
<tr><td>👁️ ऑटो-वॉच (v1.23+)</td><td>फ़ाइलों को देखता है और बदलाव पर पुनःसंसाधन करता है</td><td>उपलब्ध नहीं</td></tr>
<tr><td>🔌 प्लगइन सिस्टम (v1.25+)</td><td>Python प्लगइन्स से विस्तार करें — कस्टम टैब्स, कार्रवाइयाँ, और i18n</td><td>उपलब्ध नहीं</td></tr>
<tr><td>🚦 CI/CD एकीकरण</td><td>GitHub Actions और GitLab CI — <code>--git-base</code> के माध्यम से स्वचालित PR संदर्भ निर्माण</td><td>Not available</td></tr>
<tr><td>🌳 Monorepo Support (v1.25+)</td><td>Lerna, NX, Turborepo, pnpm workspaces — cross-package imports, root config discovery</td><td>Not available</td></tr>
</tbody>
</table>

<hr>

<h2>📥 स्थापना</h2>

<p><b>पूर्वापेक्षाएं:</b> Python 3.10+, Git</p>

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

<pre># फिर चलाएँ:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>कार्रवाई</th><th>कमांड</th></tr></thead>
<tbody>
<tr><td>इंस्टॉल करें</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>खोजें</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>अपडेट करें</td><td><code>yay -Syu</code></td></tr>
<tr><td>हटाएं</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>यदि <b>yay</b> स्थापित नहीं है:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>वैकल्पिक: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUI मोड</h2>
<pre>python main.py</pre>

<h3>1. इंटरफ़ेस अवलोकन</h3>
<p>विंडो तीन क्षेत्रों में विभाजित है:</p>
<ul>
<li><b>बायां साइडबार (टैब)</b> — स्कैन सेटिंग्स, फ़िल्टर, प्रॉम्प्ट, LLM कॉन्फ़िग, थीम</li>
<li><b>केंद्र क्षेत्र</b> — फ़ोल्डर सूची, फ़ाइल ट्री, टोकन विश्लेषण</li>
<li><b>शीर्ष क्रिया पट्टी</b> — मिनिफाई/कोई टिप्पणी नहीं/स्केलेटन टॉगल, आउटपुट प्रारूप, क्रिया बटन</li>
</ul>

<h3>2. प्रोजेक्ट जोड़ना</h3>
<table>
<thead><tr><th>कार्रवाई</th><th>कैसे</th></tr></thead>
<tbody>
<tr><td>ड्रैग और ड्रॉप</td><td>बस एक प्रोजेक्ट फ़ोल्डर को विंडो में खींचें</td></tr>
<tr><td>ब्राउज़ संवाद</td><td><b>स्रोत</b> टैब पर "+ PC फ़ोल्डर" पर क्लिक करें</td></tr>
<tr><td>GitHub रेपो</td><td>"+ GitHub / PR" पर क्लिक करें — रेपो या पुल रिक्वेस्ट URL पेस्ट करें</td></tr>
<tr><td>कॉन्फ़िग सहेजें</td><td>"💾 कॉन्फ़िग सहेजें" पर क्लिक करें — <code>.codecontextrc</code> बनाता है</td></tr>
</tbody>
</table>

<p><b>GitHub लोडिंग मोड:</b></p>
<ul>
<li><b>स्थायी रूप से सहेजें</b> — आपकी डिस्क पर एक फ़ोल्डर में क्लोन करता है</li>
<li><b>अस्थायी</b> — एक अस्थायी फ़ोल्डर में क्लोन करता है (ऐप बंद होने पर हटा दिया जाता है)</li>
</ul>

<h3>3. स्कैन कॉन्फ़िगरेशन</h3>

<h4>स्रोत टैब</h4>
<table>
<thead><tr><th>विकल्प</th><th>विवरण</th></tr></thead>
<tbody>
<tr><td>☑ केवल Git परिवर्तन</td><td>अंतिम कमिट में बदली गई फ़ाइलें ही शामिल करें</td></tr>
<tr><td>☑ .gitignore का सम्मान करें</td><td><code>.gitignore</code> से फ़ाइलों को स्वचालित रूप से बाहर करें</td></tr>
<tr><td>🔍 फ़ाइलें स्कैन करें</td><td>मेटाडेटा के साथ फ़ाइल ट्री बनाएं</td></tr>
</tbody>
</table>

<h4>फ़िल्टर टैब</h4>
<table>
<thead><tr><th>विकल्प</th><th>विवरण</th></tr></thead>
<tbody>
<tr><td><b>एक्सटेंशन प्रीसेट</b></td><td>भाषा सेट के बीच त्वरित स्विच (Python, Web, Golang, Rust, C# आदि)</td></tr>
<tr><td><b>एक्सटेंशन</b></td><td>कस्टम फ़ाइल एक्सटेंशन श्वेतसूची</td></tr>
<tr><td><b>अनदेखा किए गए पथ</b></td><td>फ़ोल्डर/फ़ाइलें छोड़ें (node_modules, .git, build, dist आदि)</td></tr>
<tr><td>☑ फ़ाइल ट्री शामिल करें</td><td>प्रॉम्प्ट से पहले फ़ोल्डर संरचना जोड़ता है</td></tr>
<tr><td>☑ निर्भरता मानचित्र शामिल करें</td><td>Python/JS/TS के लिए AST-आधारित आयात विश्लेषण</td></tr>
<tr><td>☑ Mermaid ग्राफ़ शामिल करें</td><td>Mermaid प्रारूप में आर्किटेक्चर आरेख</td></tr>
</tbody>
</table>

<p>💡 <b>कस्टम प्रीसेट सहेजना:</b> फ़िल्टर कॉन्फ़िगर करें, 💾 पर क्लिक करें, एक नाम दर्ज करें।</p>

<h4>प्रॉम्प्ट टैब</h4>
<table>
<thead><tr><th>विकल्प</th><th>विवरण</th></tr></thead>
<tbody>
<tr><td><b>प्रॉम्प्ट प्रीसेट</b></td><td>सिस्टम प्रॉम्प्ट का त्वरित परिवर्तन (कोड समीक्षा, बग हंटर, रिफैक्टरिंग आदि)</td></tr>
<tr><td><b>सिस्टम प्रॉम्प्ट</b></td><td>कस्टम प्रॉम्प्ट — सिस्टम संदर्भ के रूप में LLM को भेजा जाता है</td></tr>
<tr><td><b>🧩 JSON पैच लागू करें</b></td><td>LLM JSON प्रतिक्रिया पेस्ट करें — diff का पूर्वावलोकन करें और डिस्क पर लागू करें</td></tr>
</tbody>
</table>

<p><b>JSON पैच का उपयोग:</b></p>
<ol>
<li>LLM से JSON ऐरे का अनुरोध करें: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>JSON पेस्ट करें, <b>"अगला"</b> पर क्लिक करें → <b>सुरक्षा Diff व्यूअर</b> खुलता है</li>
<li>फ़ाइलों को चेक/अनचेक करें, वैकल्पिक रूप से <b>"🤖 LLM के माध्यम से जांचें"</b> पर क्लिक करें</li>
<li><b>"💾 चयनित को डिस्क पर सहेजें"</b> पर क्लिक करें</li>
</ol>

<h3>4. आउटपुट प्रारूप सेटिंग्स</h3>
<table>
<thead><tr><th>विकल्प</th><th>विवरण</th></tr></thead>
<tbody>
<tr><td>☑ मिनिफाई</td><td>व्हाइटस्पेस और खाली पंक्तियां हटाता है</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — अतिरिक्त संपीड़न — प्रत्येक पंक्ति पर अंतिम रिक्तियों को हटाता है</td></tr>
<tr><td>☑ कोई टिप्पणी नहीं</td><td>सभी टिप्पणियां हटाता है</td></tr>
<tr><td>☑ कोई रहस्य नहीं</td><td>API कुंजी, पासवर्ड, टोकन मास्क करता है</td></tr>
<tr><td>☑ स्केलेटन ☠️</td><td><b>फ़ंक्शन बॉडी हटाता है</b> — अधिकतम टोकन बचत</td></tr>
<tr><td>☑ Dedup</td><td>समान सामग्री वाली डुप्लिकेट फ़ाइलों को हटाता है</td></tr>
<tr><td>☑ Checkpoints</td><td>मध्यवर्ती प्रसंस्करण चेकपॉइंट सहेजता है</td></tr>
<tr><td>☑ Auto-Watch</td><td>फ़ाइलों को देखता है और बदलाव पर पुनःसंसाधन करता है</td></tr>
<tr><td>प्रारूप</td><td>Markdown, XML, Plain, JSONL Chunks, कस्टम (Jinja2)</td></tr>
<tr><td>📁 टेम्पलेट</td><td>Jinja2 टेम्पलेट चयनकर्ता</td></tr>
</tbody>
</table>

<p><b>स्केलेटन मोड:</b> फ़ंक्शन कार्यान्वयन हटाता है (<code>def func_name(...):  # ... implementation ...</code>), सभी कक्षाओं को संरक्षित करता है — LLM को न्यूनतम टोकन के साथ बड़े प्रोजेक्ट समझने देता है।</p>


<p><b>Minify बनाम Aggressive:</b> <b>Minify</b> आगे/पीछे की व्हाइटस्पेस हटाता है और खाली पंक्तियाँ हटाता है — किसी भी कोडबेस के लिए सुरक्षित, पठनीयता को प्रभावित किए बिना टोकन कम करता है। <b>Aggressive</b> अधिकतम संपीड़न के लिए प्रत्येक पंक्ति पर अंतिम रिक्तियों को हटाने वाला एक अतिरिक्त पास जोड़ता है। जब आप सीमित संदर्भ विंडो में अधिक कोड फ़िट करना चाहते हैं तो दोनों को मिलाएँ।</p>

<p><b>Dedup:</b> आपके प्रोजेक्ट में समान सामग्री वाली फ़ाइलों का स्वचालित रूप से पता लगाता है और डुप्लिकेट को आउटपुट से बाहर करता है — LLM को एक ही कोड दो बार देखने और टोकन बर्बाद करने से रोकता है।</p>

<p><b>Checkpoints:</b> प्रत्येक पाइपलाइन चरण (सफाई से पहले, मिनिफिकेशन के बाद, आदि) पर मध्यवर्ती परिणामों को <code>checkpoints/</code> फ़ोल्डर में सहेजता है। प्रत्येक प्रसंस्करण चरण के डिबगिंग या साइड बाय साइड आउटपुट की तुलना करने के लिए उपयोगी।</p>

<p><b>Auto-Watch:</b> OS फ़ाइल वॉचर का उपयोग करके प्रोजेक्ट फ़ाइलों में बदलाव की निगरानी करता है। जब कोई फ़ाइल सहेजी जाती है, तो पाइपलाइन स्वचालित रूप से पुनः चलती है — सक्रिय विकास के दौरान आदर्श जब आपको निरंतर प्रॉम्प्ट अपडेट की आवश्यकता होती है।</p>
<h3>5. क्रिया बटन</h3>
<table>
<thead><tr><th>बटन</th><th>क्रिया</th></tr></thead>
<tbody>
<tr><td>👀 पूर्वावलोकन</td><td><b>उन्नत पूर्वावलोकन संवाद</b> — "अंतिम प्रॉम्प्ट" + "पहले/बाद" टैब</td></tr>
<tr><td>📋 क्लिपबोर्ड पर कॉपी करें</td><td>परिणाम कॉपी करें — ChatGPT / Claude में पेस्ट करें</td></tr>
<tr><td>🚀 ChatGPT / Claude को भेजें</td><td>वेब चैट खोलता है और संदर्भ पेस्ट करता है</td></tr>
<tr><td>💻 संपादक में खोलें</td><td>VS Code / Cursor में खोलता है</td></tr>
<tr><td>💾 फ़ाइल में सहेजें</td><td>परिणाम डिस्क पर सहेजें</td></tr>
</tbody>
</table>

<h3>6. उन्नत पूर्वावलोकन संवाद</h3>
<p><b>"📝 अंतिम प्रॉम्प्ट" टैब:</b> फ़ाइल सूची (बाएं) + हाइलाइटिंग के साथ पूर्ण टेक्स्ट (दाएं)। सभी कॉपी करें / फ़ाइल कॉपी करें।</p>
<p><b>"🔍 पहले/बाद" टैब:</b> मूल और अनुकूलित के बीच रंगीन diff। काउंटर: <code>Before: 1500 → After: 300 (80%)</code>।</p>

<h3>7. LLM और OS</h3>
<table>
<thead><tr><th colspan="2">LLM चेकर</th></tr></thead>
<tbody>
<tr><td>☑ सत्यापन सक्षम करें</td><td>लागू करने से पहले स्वचालित LLM पैच सत्यापन</td></tr>
<tr><td>URL / कुंजी / मॉडल</td><td>API एंडपॉइंट (डिफ़ॉल्ट OpenAI), कुंजी, मॉडल</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">OS एकीकरण</th></tr></thead>
<tbody>
<tr><td>संदर्भ मेनू स्थापित करें</td><td>राइट-क्लिक मेनू में "CodeContext AI के साथ खोलें"</td></tr>
<tr><td>PATH में जोड़ें</td><td>वैश्विक <code>codecontext</code> CLI कमांड</td></tr>
<tr><td>संपादक</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. थीम</h3>
<ul>
<li><b>थीम:</b> Apple, Modern — <b>मोड:</b> लाइट / डार्क</li>
<li>📂 थीम फ़ोल्डर खोलें / ➕ थीम आयात करें (.json)</li>
</ul>

<h3>9. 📊 टोकन विश्लेषण</h3>
<p>तालिका: फ़ाइल पथ, टोकन (tiktoken), संपीड़न, बचत %, मॉडल की लागत।</p>

<h3>10. 🎛️ UI अनुकूलन (v1.14+)</h3>
<p>संस्करण के बगल में <b>⚙</b> पर क्लिक करें — "इंटरफ़ेस सेटिंग्स (Premiere Pro शैली)" संवाद। टैब (स्रोत, फ़िल्टर, प्रॉम्प्ट, LLM और OS, थीम) और क्रिया बटन (पूर्वावलोकन, क्लिपबोर्ड, ChatGPT, संपादक, फ़ाइल) टॉगल करें।</p>

<h3>11. कमांड पैलेट</h3>
<p><code>Ctrl+Shift+P</code> — माउस-मुक्त सभी क्रियाओं तक पहुंच।</p>

<h3>12. 🔌 प्लगइन सिस्टम (v1.25+)</h3>
<p><b>CodeContext AI</b> एक <b>Python प्लगइन प्रणाली</b> का समर्थन करता है जो आपको कस्टम कार्यक्षमता के साथ ऐप को विस्तारित करने देता है।</p>

<h4>📁 प्लगइन संरचना</h4>
<pre>my_plugin/
├── manifest.json          # प्लगइन मेटाडेटा
├── requirements.txt       # (वैकल्पिक) pip निर्भरताएँ
├── locales/
│   ├── en.json            # अंग्रेज़ी अनुवाद
│   └── ru.json            # रूसी अनुवाद
└── plugin.py              # प्रवेश बिंदु</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "कुछ उपयोगी करता है",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (उदाहरण)</h4>
<pre>from src.services.plugin_manager import IPlugin

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, controller: MainController) -> None:
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("प्लगइन से नमस्ते!")
        )
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("प्लगइन क्रिया क्लिक की गई")
        )
        api.add_log("मेरा प्लगइन आरंभ किया गया")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 सुरक्षा</h4>
<ul>
<li>प्लगइन्स को <b>पूर्ण Python पहुँच</b> मिलती है — केवल विश्वसनीय स्रोतों से स्थापित करें</li>
<li>पहली बार लोड करने पर, एक सुरक्षा संवाद प्लगइन सक्षम करने से पहले आपकी अनुमति माँगता है</li>
<li>यदि <code>requirements.txt</code> मौजूद है, तो आप लोड करने से पहले एक लाइव pip install लॉग देखेंगे</li>
<li>अनुमोदित प्लगइन्स सेटिंग्स में याद रखे जाते हैं (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 Plugin API</h4>
<table>
<thead><tr><th>प्रॉपर्टी / मेथड</th><th>विवरण</th></tr></thead>
<tbody>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>बाएँ साइडबार में टैब जोड़ें</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>"प्लगइन्स 🔽" ड्रॉपडाउन में बटन जोड़ें</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>ऐप लॉग पैनल में लिखें</td></tr>
</tbody>
</table>

<h4>⚙️ दृश्यता</h4>
<p>प्लगइन टैब और क्रिया बटन को <b>⚙ UI अनुकूलन</b> के माध्यम से टॉगल किया जा सकता है — वे अपने स्वयं के चेकबॉक्स के साथ अंतर्निहित टैब/क्रियाओं के साथ दिखाई देते हैं।</p>

<hr>

<h2>💻 CLI मोड</h2>
<pre>python main.py --cli --path /path/to/project [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>पैरामीटर</th><th>प्रकार</th><th>विवरण</th><th>उदाहरण</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLI मोड (GUI नहीं)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>प्रोजेक्ट पथ</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>एक्सटेंशन</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>अनदेखा किए गए पथ</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>मिनिफिकेशन सक्षम करें</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>टिप्पणियां हटाएं</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>रहस्य मास्क करें</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>स्केलेटन मोड</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>आउटपुट फ़ाइल</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>stdout पर प्रिंट करें</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>केवल Git परिवर्तन</td><td><code>--git</code></td></tr>
<tr><td><code>--git-base</code></td><td>str</td><td>CI/CD में git diff के लिए आधार शाखा</td><td><code>--git-base origin/main</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>.gitignore का सम्मान करें</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>फ़ाइल ट्री</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaid ग्राफ़</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>निर्भरता मानचित्र</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSON पैच</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2 टेम्पलेट</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>कस्टम सिस्टम प्रॉम्प्ट</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>उदाहरण</h3>
<pre># न्यूनतम रन
python main.py --cli --path ./myapp --stdout

# XML के साथ पूर्ण विश्लेषण
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON पैच
python main.py --cli --path ./myapp --patch llm_response.json

# कस्टम Jinja2 टेम्पलेट
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid आरेख
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# एकाधिक पथ
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml

# CI/CD — आधार शाखा के साथ अंतर
python main.py --cli --path . --git --git-base origin/main --minify true --stdout</pre>

<hr>

<h2>🏗️ तकनीकी स्टैक</h2>
<table>
<thead><tr><th>घटक</th><th>तकनीक</th></tr></thead>
<tbody>
<tr><td>भाषा</td><td>Python 3.10+</td></tr>
<tr><td>GUI फ्रेमवर्क</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>आर्किटेक्चर</td><td>Clean Architecture</td></tr>
<tr><td>टोकनीकरण</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>टेम्प्लेटिंग</td><td>jinja2 (11 अंतर्निहित)</td></tr>
<tr><td>AST पार्सर</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>वितरण</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ रोडमैप</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation) मोड</b> — स्थानीय वेक्टर DB (Chroma/FAISS) का उपयोग करके विशाल कोडबेस की इंडेक्सिंग।</li>
<li>🚫 <b>गहन .gitignore पार्सिंग</b> — नेस्टेड <code>.gitignore</code> फ़ाइलों और वैश्विक <code>~/.gitignore</code> के लिए समर्थन।</li>
<li>☁️ <b>क्लाउड सिंक</b> — GitHub Gists के माध्यम से प्रीसेट और कॉन्फ़िगरेशन सिंक करें।</li>
<li>🌳 <b>मल्टी-रूट वर्कस्पेस</b> — मोनोरेपो (Lerna, NX, Turborepo) के लिए बेहतर समर्थन।</li>
<li>🚀 <b>CI/CD पाइपलाइन</b> — स्वचालित PR संदर्भ निर्माण के लिए GitHub Actions और GitLab CI प्लगइन।</li>
<li>🤖 <b>प्रत्यक्ष OpenAI/Anthropic API एकीकरण</b> — प्रॉम्प्ट जनरेशन से सीधे आउटपुट तक पूर्ण ब्रिज।</li>
<li>🍎 macOS फ़ाइंडर संदर्भ मेनू</li>
<li>🔌 प्लगिन सिस्टम ✅</li>
</ul>

<hr>

<h2>👨‍💻 टीम</h2>
<p><b>डेवलपर:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · GitHub पर Issues और PRs</p>

<hr>

<h2>🤝 योगदान</h2>
<ol>
<li>रिपॉजिटरी को फोर्क करें</li>
<li>ब्रांच: <code>git checkout -b feature/AmazingFeature</code></li>
<li>कमिट: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>पुश: <code>git push origin feature/AmazingFeature</code></li>
<li>पुल रिक्वेस्ट</li>
</ol>
<p>SOLID सिद्धांतों का पालन करें (<code>../docs/ARCHITECTURE.md</code> देखें)।</p>

<hr>

<h2>📄 लाइसेंस</h2>
<p>MIT. विवरण के लिए <code>../LICENSE</code> देखें।</p>
