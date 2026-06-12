<div align="center">

[🇷🇺 Русский](README.ru.md) · [🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI destekli kod tabanı analizi ve prompt hazırlama aracı**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.25.1-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 Hakkında</h2>

<p><b>CodeContext AI</b>, kod tabanınızı Büyük Dil Modelleri (LLM) ile çalışmaya hazırlamak için güçlü bir masaüstü aracıdır. Proje klasörlerini tarar, yapıyı analiz eder, bağımlılık grafikleri oluşturur ve token tüketimi ve mimari netlik için optimize edilmiş, mükemmel şekilde yapılandırılmış tek bir prompt üretir.</p>

<h3>❓ Neden?</h3>
<p>AI ile çalışırken, geliştiriciler bağlam penceresi token sınırlarıyla karşılaşır — kod parçalar halinde kopyalandığında LLM'ler mimari tutarlılığı "kaybeder". <b>CodeContext AI bunu çözer</b>: tüm projenizi birkaç tıklamayla yapılandırılmış tek bir prompTta toplayın, tokenlerden %80'e varan oranda tasarruf edin.</p>

<hr>

<h2>🚀 Özellikler</h2>

<table>
<thead><tr><th>Özellik</th><th>CodeContext AI</th><th>Manuel</th></tr></thead>
<tbody>
<tr><td>🗜️ Küçültme + İskelet</td><td><b>%80'e varan</b> token azaltma</td><td>Manuel kopyala-yapıştır</td></tr>
<tr><td>🧩 LLM Patcher</td><td>JSON yamalarını önizle ve uygula</td><td>Mevcut değil</td></tr>
<tr><td>✅ LLM Checker</td><td>Kaydetmeden önce kodu otomatik doğrula</td><td>Mevcut değil</td></tr>
<tr><td>🔗 AST bağımlılık grafiği</td><td>Python, JS/TS, Vue</td><td>Yalnızca dosya listeleme</td></tr>
<tr><td>🖱️ Bağlam menüsü</td><td>Windows / Linux</td><td>Yok</td></tr>
<tr><td>🎨 Temalar</td><td>Apple, Modern, özel JSON</td><td>Sabit arayüz</td></tr>
<tr><td>⚙️ Arayüz özelleştirme (v1.14+)</td><td>Premiere Pro tarzı</td><td>Sabit arayüz</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 dil, sistem otomatik algılama</td><td>Tek dil</td></tr>
<tr><td>♻️ Tekilleştirme (v1.23+)</td><td>Aynı içeriğe sahip dosyaları algılar ve atlar</td><td>Manuel kontrol</td></tr>
<tr><td>⚡ Agresif küçültme (v1.23+)</td><td>Ek sıkıştırma — her satırdaki sondaki boşlukları kaldırır</td><td>Manuel silme</td></tr>
<tr><td>📌 Kontrol noktaları (v1.23+)</td><td>Hata ayıklama için önce/sonra anlık görüntüleri kaydeder</td><td>Mevcut değil</td></tr>
<tr><td>👁️ Otomatik izleme (v1.23+)</td><td>Dosyaları izler ve değişiklikte yeniden işler</td><td>Mevcut değil</td></tr>
<tr><td>🔌 Eklenti sistemi (v1.25+)</td><td>Python eklentileriyle genişletin — özel sekmeler, eylemler ve i18n</td><td>Mevcut değil</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Kurulum</h2>

<p><b>Ön koşullar:</b> Python 3.10+, Git</p>

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

<pre># Ardından başlatın:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Eylem</th><th>Komut</th></tr></thead>
<tbody>
<tr><td>Kur</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Ara</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Güncelle</td><td><code>yay -Syu</code></td></tr>
<tr><td>Kaldır</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p><b>yay</b> kurulu değilse:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Alternatif: <code>paru -S codecontext-ai</code></p>

<hr>

<h2>💻 GUI Modu</h2>
<pre>python main.py</pre>

<h3>1. Arayüze Genel Bakış</h3>
<p>Pencere üç bölgeye ayrılmıştır:</p>
<ul>
<li><b>Sol kenar çubuğu (sekmeler)</b> — tarama ayarları, filtreler, promptlar, LLM yapılandırması, temalar</li>
<li><b>Merkez alan</b> — klasör listesi, dosya ağacı, token analitiği</li>
<li><b>Üst eylem çubuğu</b> — Küçült/Yorum Yok/İskelet geçişleri, çıktı formatı, eylem düğmeleri</li>
</ul>

<h3>2. Proje Ekleme</h3>
<table>
<thead><tr><th>Eylem</th><th>Nasıl</th></tr></thead>
<tbody>
<tr><td>Sürükle & bırak</td><td>Bir proje klasörünü pencereye sürükleyin</td></tr>
<tr><td>Gözat iletişimi</td><td><b>Kaynaklar</b> sekmesinde "+ PC Klasörü"ne tıklayın</td></tr>
<tr><td>GitHub deposu</td><td>"+ GitHub / PR"a tıklayın — bir depo veya Pull Request URL'si yapıştırın</td></tr>
<tr><td>Yapılandırmayı kaydet</td><td>"💾 Yapılandırmayı kaydet"e tıklayın — <code>.codecontextrc</code> oluşturur</td></tr>
</tbody>
</table>

<p><b>GitHub yükleme modları:</b></p>
<ul>
<li><b>Kalıcı olarak kaydet</b> — diskinizdeki bir klasöre klonlar</li>
<li><b>Geçici</b> — geçici bir klasöre klonlar (uygulama kapanınca silinir)</li>
</ul>

<h3>3. Tarama Yapılandırması</h3>

<h4>Kaynaklar Sekmesi</h4>
<table>
<thead><tr><th>Seçenek</th><th>Açıklama</th></tr></thead>
<tbody>
<tr><td>☑ Yalnızca Git Değişiklikleri</td><td>Yalnızca son commit'te değiştirilen dosyaları dahil et</td></tr>
<tr><td>☑ .gitignore'a saygı göster</td><td>Dosyaları <code>.gitignore</code>'dan otomatik olarak hariç tut</td></tr>
<tr><td>🔍 Dosyaları Tara</td><td>Meta verilerle dosya ağacı oluştur</td></tr>
</tbody>
</table>

<h4>Filtreler Sekmesi</h4>
<table>
<thead><tr><th>Seçenek</th><th>Açıklama</th></tr></thead>
<tbody>
<tr><td><b>Uzantı ön ayarları</b></td><td>Dil kümeleri arasında hızlı geçiş (Python, Web, Golang, Rust, C#, vb.)</td></tr>
<tr><td><b>Uzantılar</b></td><td>Özel dosya uzantısı beyaz listesi</td></tr>
<tr><td><b>Yoksayılan yollar</b></td><td>Klasörleri/dosyaları atla (node_modules, .git, build, dist, vb.)</td></tr>
<tr><td>☑ Dosya ağacını dahil et</td><td>Prompttan önce klasör yapısını ekler</td></tr>
<tr><td>☑ Bağımlılık haritasını dahil et</td><td>Python/JS/TS için AST tabanlı içe aktarma analizi</td></tr>
<tr><td>☑ Mermaid grafiğini dahil et</td><td>Mermaid formatında mimari diyagram</td></tr>
</tbody>
</table>

<p>💡 <b>Özel ön ayarları kaydetme:</b> filtreleri yapılandırın, 💾'a tıklayın, bir ad girin.</p>

<h4>Promptlar Sekmesi</h4>
<table>
<thead><tr><th>Seçenek</th><th>Açıklama</th></tr></thead>
<tbody>
<tr><td><b>Prompt ön ayarları</b></td><td>Sistem promptunda hızlı değişiklik (Kod İnceleme, Hata Avcısı, Refaktoring, vb.)</td></tr>
<tr><td><b>Sistem promptu</b></td><td>Özel prompt — sistem bağlamı olarak LLM'ye gönderilir</td></tr>
<tr><td><b>🧩 JSON yaması uygula</b></td><td>LLM JSON yanıtını yapıştır — diff'i önizle ve diske uygula</td></tr>
</tbody>
</table>

<p><b>JSON yamalarını kullanma:</b></p>
<ol>
<li>LLM'den bir JSON dizisi isteyin: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>JSON'u yapıştırın, <b>"İleri"</b>ye tıklayın → <b>Güvenli Diff Görüntüleyici</b> açılır</li>
<li>Dosyaları işaretleyin/işaretini kaldırın, isteğe bağlı olarak <b>"🤖 LLM ile kontrol et"</b>e tıklayın</li>
<li><b>"💾 Seçilenleri diske kaydet"</b>e tıklayın</li>
</ol>

<h3>4. Çıktı Formatı Ayarları</h3>
<table>
<thead><tr><th>Seçenek</th><th>Açıklama</th></tr></thead>
<tbody>
<tr><td>☑ Küçült</td><td>Boşlukları ve boş satırları kaldırır</td></tr>
<tr><td>☑ Aggressive</td><td>Aggressive minification — Ek sıkıştırma — her satırdaki sondaki boşlukları kaldırır</td></tr>
<tr><td>☑ Yorum Yok</td><td>Tüm yorumları kaldırır</td></tr>
<tr><td>☑ Sır Yok</td><td>API anahtarlarını, şifreleri, tokenları maskeler</td></tr>
<tr><td>☑ İskelet ☠️</td><td><b>Fonksiyon gövdelerini kaldırır</b> — maksimum token tasarrufu</td></tr>
<tr><td>☑ Dedup</td><td>Aynı içeriğe sahip yinelenen dosyaları kaldırır</td></tr>
<tr><td>☑ Checkpoints</td><td>Ara işleme kontrol noktalarını kaydeder</td></tr>
<tr><td>☑ Auto-Watch</td><td>Dosyaları izler ve değişiklikte yeniden işler</td></tr>
<tr><td>Format</td><td>Markdown, XML, Plain, JSONL Chunks, Özel (Jinja2)</td></tr>
<tr><td>📁 şablon</td><td>Jinja2 şablon seçici</td></tr>
</tbody>
</table>

<p><b>İskelet Modu:</b> fonksiyon uygulamalarını kaldırır (<code>def func_name(...):  # ... implementation ...</code>), tüm sınıfları korur — LLM'nin minimum token ile büyük projeleri anlamasını sağlar.</p>


<p><b>Küçültme vs Agresif:</b> <b>Küçültme</b>, baştaki/sondaki boşlukları ve boş satırları kaldırır — her kod tabanı için güvenlidir, okunabilirliği etkilemeden tokenları azaltır. <b>Agresif</b>, maksimum sıkıştırma için her satırdaki sondaki boşlukları ortadan kaldıran ek bir geçiş ekler. Sınırlı bir bağlam penceresine daha fazla kod sığdırmanız gerektiğinde ikisini birleştirin.</p>

<p><b>Tekilleştirme:</b> projeniz genelinde aynı içeriğe sahip dosyaları otomatik olarak algılar ve yinelenenleri çıktıdan hariç tutar — LLM'nin aynı kodu iki kez görmesini ve token israfını önler.</p>

<p><b>Kontrol Noktaları:</b> her işlem hattı aşamasında (temizlemeden önce, küçültmeden sonra vb.) ara sonuçları <code>checkpoints/</code> klasörüne kaydeder. Her işlem adımının ne yaptığını hata ayıklamak veya çıktıları yan yana karşılaştırmak için kullanışlıdır.</p>

<p><b>Auto-Watch:</b> monitors your project files for changes using the OS file watcher. When a file is saved, the pipeline automatically re-runs — ideal during active development when you need continuous prompt updates.</p>
<h3>5. Eylem Düğmeleri</h3>
<table>
<thead><tr><th>Düğme</th><th>Eylem</th></tr></thead>
<tbody>
<tr><td>👀 Önizleme</td><td><b>Gelişmiş Önizleme İletişimi</b> — "Son Prompt" + "Önce/Sonra" sekmeleri</td></tr>
<tr><td>📋 Panoya Kopyala</td><td>Sonucu kopyala — ChatGPT / Claude'a yapıştır</td></tr>
<tr><td>🚀 ChatGPT / Claude'a Gönder</td><td>Web sohbetini açar ve bağlamı yapıştırır</td></tr>
<tr><td>💻 Düzenleyicide Aç</td><td>VS Code / Cursor'da açar</td></tr>
<tr><td>💾 Dosyaya Kaydet</td><td>Sonucu diske kaydet</td></tr>
</tbody>
</table>

<h3>6. Gelişmiş Önizleme İletişimi</h3>
<p><b>"📝 Son Prompt" sekmesi:</b> dosya listesi (sol) + vurgulamalı tam metin (sağ). Tümünü Kopyala / Dosyayı Kopyala.</p>
<p><b>"🔍 Önce/Sonra" sekmesi:</b> orijinal ve optimize edilmiş arasında renkli diff. Sayaç: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM ve İS</h3>
<table>
<thead><tr><th colspan="2">LLM Denetleyici</th></tr></thead>
<tbody>
<tr><td>☑ Doğrulamayı etkinleştir</td><td>Uygulamadan önce otomatik LLM yaması doğrulaması</td></tr>
<tr><td>URL / Anahtar / Model</td><td>API uç noktası (varsayılan OpenAI), anahtar, model</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">İS Entegrasyonu</th></tr></thead>
<tbody>
<tr><td>Bağlam menüsünü yükle</td><td>Sağ tıklama menüsünde "CodeContext AI ile Aç"</td></tr>
<tr><td>PATH'e ekle</td><td>Global <code>codecontext</code> CLI komutu</td></tr>
<tr><td>Düzenleyici</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Temalar</h3>
<ul>
<li><b>Tema:</b> Apple, Modern — <b>Mod:</b> aydınlık / karanlık</li>
<li>📂 Tema klasörünü aç / ➕ Temayı içe aktar (.json)</li>
</ul>

<h3>9. 📊 Token Analitiği</h3>
<p>Tablo: dosya yolu, tokenlar (tiktoken), sıkıştırma, tasarruf %, model için maliyet.</p>

<h3>10. 🎛️ Arayüz Özelleştirme (v1.14+)</h3>
<p>Sürümün yanındaki <b>⚙</b> düğmesine tıklayın — "Arayüz Ayarları (Premiere Pro stili)" iletişimi. Sekmeleri (Kaynaklar, Filtreler, Promptlar, LLM ve İS, Temalar) ve eylem düğmelerini (Önizleme, Pano, ChatGPT, Düzenleyici, Dosya) açıp kapatın.</p>

<h3>11. Komut Paleti</h3>
<p><code>Ctrl+Shift+P</code> — fare kullanmadan tüm eylemlere erişim.</p>

<h3>12. 🔌 Eklenti sistemi (v1.25+)</h3>
<p><b>CodeContext AI</b>, uygulamayı özel işlevlerle genişletmenize olanak tanıyan bir <b>Python eklenti sistemini</b> destekler.</p>

<h4>📁 Eklenti Yapısı</h4>
<pre>my_plugin/
├── manifest.json          # Eklenti meta verileri
├── requirements.txt       # (İsteğe bağlı) pip bağımlılıkları
├── locales/
│   ├── en.json            # İngilizce çeviriler
│   └── ru.json            # Rusça çeviriler
└── plugin.py              # Giriş noktası</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Yararlı bir şey yapar",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Örnek)</h4>
<pre>from src.api.plugin_api import IPlugin, PluginAPI

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        # locales/ klasöründeki çeviriler otomatik olarak yüklenir
        # Kenar çubuğuna sekme ekle
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("Eklentiden merhaba!")
        )
        # Eylem düğmesi ekle
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Eklenti eylemi tıklandı")
        )
        api.add_log("Eklentim başlatıldı")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Güvenlik</h4>
<ul>
<li>Eklentiler <b>tam Python erişimine</b> sahiptir — yalnızca güvenilir kaynaklardan yükleyin</li>
<li>İlk yüklemede, bir güvenlik iletişim kutusu eklentiyi etkinleştirmeden önce onayınızı ister</li>
<li><code>requirements.txt</code> varsa, yüklemeden önce canlı pip kurulum günlüğünü görürsünüz</li>
<li>Onaylanan eklentiler ayarlarda saklanır (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 Eklenti API'si</h4>
<table>
<thead><tr><th>Özellik / Metod</th><th>Açıklama</th></tr></thead>
<tbody>
<tr><td><code>api.store</code></td><td>Salt okunur Redux deposu (durum erişimi: <code>state.settings.xxx</code>)</td></tr>
<tr><td><code>api.dispatcher</code></td><td>Eylem gönderme (ör. <code>UI_ADD_LOG</code>)</td></tr>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Sol kenar çubuğuna sekme ekler</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>"Eklentiler 🔽" menüsüne düğme ekler</td></tr>
<tr><td><code>api.add_translations(lang, data)</code></td><td>Çalışma zamanı çevirileri ekler (yerleşiklerin üzerine eklenir)</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Uygulama günlük paneline yazar</td></tr>
</tbody>
</table>

<h4>⚙️ Görünürlük</h4>
<p>Eklenti sekmeleri ve eylem düğmeleri <b>⚙ Arayüz Özelleştirme</b> üzerinden açılıp kapatılabilir — yerleşik sekmeler/eylemlerin yanında kendi onay kutularıyla görünürler.</p>

<hr>

<h2>💻 CLI Modu</h2>
<pre>python main.py --cli --path /path/to/project [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parametre</th><th>Tür</th><th>Açıklama</th><th>Örnek</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLI modu (GUI yok)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>Proje yolu</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Uzantılar</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Yoksayılan yollar</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Küçültmeyi etkinleştir</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Yorumları kaldır</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Sırları maskele</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>İskelet modu</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Çıktı dosyası</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>stdout'a yazdır</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Yalnızca Git değişiklikleri</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>.gitignore'a saygı göster</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>Dosya ağacı</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaid grafiği</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Bağımlılık haritası</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSON yaması</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2 şablonu</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Özel sistem promptu</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Örnekler</h3>
<pre># Minimum çalıştırma
python main.py --cli --path ./myapp --stdout

# XML ile tam analiz
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON yaması
python main.py --cli --path ./myapp --patch llm_response.json

# Özel Jinja2 şablonu
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid diyagramı
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Birden çok yol
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Teknoloji Yığını</h2>
<table>
<thead><tr><th>Bileşen</th><th>Teknoloji</th></tr></thead>
<tbody>
<tr><td>Dil</td><td>Python 3.10+</td></tr>
<tr><td>GUI Çerçevesi</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Mimari</td><td>Clean Architecture + Redux-benzeri</td></tr>
<tr><td>Tokenizasyon</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Şablonlama</td><td>jinja2 (11 yerleşik)</td></tr>
<tr><td>AST ayrıştırıcıları</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Dağıtım</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>🗺️ Yol Haritası</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation) modu</b> — yerel vektör DB (Chroma/FAISS) ile büyük kod tabanlarını indeksleme.</li>
<li>🚫 <b>Derin .gitignore ayrıştırma</b> — iç içe <code>.gitignore</code> dosyaları ve global <code>~/.gitignore</code> desteği.</li>
<li>☁️ <b>Bulut senkronizasyonu</b> — GitHub Gists üzerinden ön ayarları ve yapılandırmaları senkronize edin.</li>
<li>🌳 <b>Çoklu kök çalışma alanları</b> — monorepo (Lerna, NX, Turborepo) desteği iyileştirildi.</li>
<li>🚀 <b>CI/CD boru hatları</b> — otomatik PR bağlamı oluşturma için GitHub Actions ve GitLab CI eklentileri.</li>
<li>🤖 <b>Doğrudan OpenAI/Anthropic API entegrasyonu</b> — prompt oluşturmadan doğrudan çıktıya tam köprü.</li>
<li>🍎 macOS Finder bağlam menüsü</li>
<li>🔌 Eklenti sistemi ✅</li>
</ul>

<hr>

<h2>👨‍💻 Ekip</h2>
<p><b>Geliştirici:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · GitHub'da Issues ve PR'ler</p>

<hr>

<h2>🤝 Katkıda Bulunma</h2>
<ol>
<li>Depoyu fork edin</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>SOLID prensiplerini izleyin (<code>../docs/ARCHITECTURE.md</code> dosyasına bakın).</p>

<hr>

<h2>📄 Lisans</h2>
<p>MIT. Ayrıntılar için <code>../LICENSE</code> dosyasına bakın.</p>
