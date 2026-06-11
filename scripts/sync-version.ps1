param(
  [string]$VERSION = "",
  [switch]$CommitAur
)

$ROOT = Resolve-Path "$PSScriptRoot\.."
$VERSION_TXT = "$ROOT\VERSION.txt"

if (-not $VERSION) {
  $VERSION = (Get-Content $VERSION_TXT).Trim()
}

Write-Host "=== Syncing version: $VERSION ==="

# aur_build/PKGBUILD (local copy) + AUR git repo
$PKGBUILDS = @(
  "$ROOT\aur_build\PKGBUILD",
  "$ROOT\aur_build\codecontext-ai\PKGBUILD"
)

foreach ($f in $PKGBUILDS) {
  if (Test-Path $f) {
    $content = Get-Content $f -Raw
    $content = $content -replace '(?<=\n)pkgver=\d+\.\d+\.\d+', "`npkgver=$VERSION"
    Set-Content -NoNewline -Path $f -Value $content
    Write-Host "  Updated: $f"
  }
}

# .SRCINFO files
$SRCINFOS = @(
  "$ROOT\aur_build\.SRCINFO",
  "$ROOT\aur_build\codecontext-ai\.SRCINFO"
)

foreach ($f in $SRCINFOS) {
  if (Test-Path $f) {
    $content = Get-Content $f -Raw
    $content = $content -replace '(?<=\n\s)pkgver = \d+\.\d+\.\d+', "`n`tpkgver = $VERSION"
    $content = $content -replace '(?<=tags/)v\S+\.tar\.gz', "v$VERSION.tar.gz"
    $content = $content -replace '(?<=codecontext-ai-)\d+\.\d+\.\d+(?=\.tar\.gz)', $VERSION
    Set-Content -NoNewline -Path $f -Value $content
    Write-Host "  Updated: $f"
  }
}

# aur_build/codecontext-ai/.SRCINFO also needs source URL update
Write-Host ""
Write-Host "=== Done. Version $VERSION synced to all build files ==="

if ($CommitAur) {
  Push-Location "$ROOT\aur_build\codecontext-ai"
  git add PKGBUILD .SRCINFO
  git commit -m "update to v$VERSION"
  git push
  Pop-Location
  Write-Host "  Pushed to AUR"
}
