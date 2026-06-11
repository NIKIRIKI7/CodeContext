# Установка CodeContext из AUR

Пакет в AUR: [codecontext-ai](https://aur.archlinux.org/packages/codecontext-ai)

## Установка

### yay

```bash
yay -S codecontext-ai
```

### paru

```bash
paru -S codecontext-ai
```

### Вручную

```bash
git clone https://aur.archlinux.org/codecontext-ai.git
cd codecontext-ai
makepkg -si
```

## Обновление

```bash
yay -Syu
```
или только этот пакет:
```bash
yay -S codecontext-ai
```

## Удаление

```bash
sudo pacman -Rns codecontext-ai
```

---

# Обновление AUR-пакета (для мейнтейнера)

## Быстрый способ (рекомендуется)

Версия задаётся **только** в `VERSION.txt`. Остальные файлы обновляются вручную.

```powershell
# 1. Обновить версию
Set-Content VERSION.txt -Value "1.19.0"

# 2. Обновить pkgver в aur_build/PKGBUILD и aur_build/.SRCINFO
# 3. Закоммитить
git add VERSION.txt aur_build/
git commit -m "Bump version: 1.14.0 → 1.19.0"
git push
```

Убедись, что тег `v1.19.0` существует на GitHub.

## Пошагово

### 1. Обновить VERSION.txt

```bash
echo "1.19.0" > VERSION.txt
```

### 2. Обновить AUR-сборки

Вручную изменить `pkgver` в:
- `aur_build/PKGBUILD`
- `aur_build/.SRCINFO`
- `aur_build/codecontext-ai/PKGBUILD`
- `aur_build/codecontext-ai/.SRCINFO`

### 3. Закоммитить и запушить в AUR

```bash
cd aur_build/codecontext-ai
git add PKGBUILD .SRCINFO
git commit -m "update to v1.19.0"
git push
cd ../..
```

### 4. Если нужен bumpversion (альтернатива п.1)

```bash
bumpversion patch  # или minor, или major
# Затем вручную обновить AUR-файлы
```

### 5. Проверить

Открой https://aur.archlinux.org/packages/codecontext-ai
