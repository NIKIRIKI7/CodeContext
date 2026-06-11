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

Эти инструкции для тех, кто поддерживает пакет `codecontext-ai` в AUR.

## Подготовка

```bash
cd aur_build/codecontext-ai
git pull
```

## 1. Обновить версию в PKGBUILD

Измени `pkgver` на новую версию (например, `1.15.0`) и при необходимости `pkgrel` (сбрось в `1` при новом `pkgver`, увеличь при исправлении сборки):

```bash
# Например: vim PKGBUILD
# pkgver=1.15.0
# pkgrel=1
```

Убедись, что тег `v<новая-версия>` существует на GitHub:

```bash
git ls-remote --tags https://github.com/NIKIRIKI7/CodeContext.git | grep v1.15.0
```

## 2. Обновить .SRCINFO

Сгенерируй новый `.SRCINFO` из PKGBUILD:

```bash
makepkg --printsrcinfo > .SRCINFO
```

Если нет `makepkg` (например, на Windows), отредактируй `.SRCINFO` вручную: обнови `pkgver`, `pkgrel` и `source` (URL с тегом).

## 3. Закоммитить и запушить

```bash
git add PKGBUILD .SRCINFO
git commit -m "update to v1.15.0"
git push
```

## 4. Проверить

Открой https://aur.archlinux.org/packages/codecontext-ai — должна отображаться новая версия.
