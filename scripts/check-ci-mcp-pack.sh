#!/usr/bin/env bash
# ============================================================================
# demeter-mcp-pack — check-ci.sh
# ============================================================================
# Rejoue en local ce que fait .github/workflows/release.yml avant de tagger
# une release : compilation des 5 serveurs pour VOTRE plateforme (la CI en
# couvre 2 : linux-x64 et windows-x64 — voir la limite ci-dessous), puis
# assemblage des archives + checksums.txt via scripts/package.sh, avec des
# vérifications supplémentaires que la CI elle-même ne fait pas (JSON des
# manifests, cohérence checksums.txt/archives).
#
#   chmod +x scripts/check-ci.sh
#   ./scripts/check-ci.sh
#
# ⚠️ Limite : la CI GitHub build sur une matrice ubuntu-latest/windows-latest
# (voir release.yml). Ce script ne peut tester QUE la plateforme sur laquelle
# vous le lancez (pas de cross-compilation Windows depuis Linux ici) — un
# succès local ne garantit donc pas que le build Windows de la CI passera
# aussi, seulement que rien n'est cassé côté code/manifests/scripts. En cas
# de doute sur le nouveau code, utilisez "workflow_dispatch" sur GitHub pour
# tester la matrice complète sans publier de release.
# ============================================================================
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FAILED=0
step() { echo -e "\n▶ $1"; }
ok()   { echo "  ✅ $1"; }
fail() { echo "  ❌ $1"; FAILED=1; }

# ── Détection de la plateforme locale (mêmes clés que la matrice CI) ───────
case "$(uname -s)" in
  Linux)  PLATFORM_KEY="linux-x64";   TARGET="x86_64-unknown-linux-gnu" ;;
  Darwin)
    if [[ "$(uname -m)" == "arm64" ]]; then
      PLATFORM_KEY="macos-arm64"; TARGET="aarch64-apple-darwin"
    else
      PLATFORM_KEY="macos-x64";   TARGET="x86_64-apple-darwin"
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*) PLATFORM_KEY="windows-x64"; TARGET="x86_64-pc-windows-msvc" ;;
  *) echo "Plateforme non reconnue ($(uname -s))"; exit 1 ;;
esac
echo "Plateforme locale détectée : $PLATFORM_KEY ($TARGET)"
if [[ "$PLATFORM_KEY" != "linux-x64" && "$PLATFORM_KEY" != "windows-x64" ]]; then
  echo "  ℹ️  Cette plateforme n'est pas dans la matrice CI (linux-x64, windows-x64) —"
  echo "     utile pour vérifier que le code compile, mais ne remplace pas la CI."
fi

# ══════════════════════════════════════════════════════════════════════════
# 1. Build des 5 serveurs pour la plateforme locale (scripts/build-all.sh)
# ══════════════════════════════════════════════════════════════════════════

step "scripts/build-all.sh $PLATFORM_KEY $TARGET"
if bash scripts/build-all.sh "$PLATFORM_KEY" "$TARGET"; then
  ok "Les 5 serveurs compilent pour $PLATFORM_KEY"
else
  fail "Le build a échoué pour au moins un serveur (voir ci-dessus)"
fi

# ══════════════════════════════════════════════════════════════════════════
# 2. Validation des manifest.json (structure attendue par Demeter)
# ══════════════════════════════════════════════════════════════════════════

step "Validation des manifest.json"
MANIFEST_OK=true
for m in mcp-servers/*/manifest.json; do
  if [[ ! -f "$m" ]]; then
    fail "$m manquant"
    MANIFEST_OK=false
    continue
  fi
  if ! python3 -c "
import json, sys
d = json.load(open('$m'))
for k in ('id', 'name', 'bin'):
    assert k in d, f'clé \'{k}\' manquante'
assert isinstance(d['bin'], dict) and d['bin'], '\"bin\" doit être un objet non vide'
" 2>/tmp/manifest_err; then
    fail "$m invalide : $(cat /tmp/manifest_err | tail -1)"
    MANIFEST_OK=false
  fi
done
$MANIFEST_OK && ok "Les 5 manifest.json sont présents et structurellement valides"

# ══════════════════════════════════════════════════════════════════════════
# 3. Assemblage des archives (scripts/package.sh)
# ══════════════════════════════════════════════════════════════════════════

step "scripts/package.sh"
if bash scripts/package.sh > /tmp/package_output 2>&1; then
  ok "Archives et checksums.txt générés dans dist/"
else
  fail "package.sh a échoué :"
  cat /tmp/package_output
fi

# ══════════════════════════════════════════════════════════════════════════
# 4. Cohérence checksums.txt ↔ archives (régression du bug préfixe "./")
# ══════════════════════════════════════════════════════════════════════════

step "Cohérence dist/checksums.txt"
if [[ -f dist/checksums.txt ]]; then
  CHECKSUM_OK=true

  if grep -qE '  \./' dist/checksums.txt; then
    fail "checksums.txt contient encore un préfixe './' — régression du bug déjà corrigé"
    CHECKSUM_OK=false
  fi

  for zip in dist/*.zip; do
    name="$(basename "$zip")"
    if ! grep -qE "  ${name}\$" dist/checksums.txt; then
      fail "Aucune entrée pour '$name' dans checksums.txt"
      CHECKSUM_OK=false
      continue
    fi
    expected="$(grep -E "  ${name}\$" dist/checksums.txt | awk '{print $1}')"
    if command -v sha256sum >/dev/null 2>&1; then
      actual="$(sha256sum "$zip" | awk '{print $1}')"
    else
      actual="$(shasum -a 256 "$zip" | awk '{print $1}')"
    fi
    if [[ "$expected" != "$actual" ]]; then
      fail "Checksum incorrect pour $name (attendu $expected, obtenu $actual)"
      CHECKSUM_OK=false
    fi
  done

  $CHECKSUM_OK && ok "checksums.txt cohérent avec les archives générées"
else
  fail "dist/checksums.txt introuvable — package.sh a-t-il vraiment tourné ?"
fi

# ── Résumé ───────────────────────────────────────────────────────────────────
echo
if [[ "$FAILED" -eq 0 ]]; then
  echo "✅ Tout est bon pour $PLATFORM_KEY. Rappel : seule cette plateforme a été testée localement —"
  echo "   la CI GitHub couvre aussi $([[ "$PLATFORM_KEY" == "linux-x64" ]] && echo "windows-x64" || echo "linux-x64")."
  exit 0
else
  echo "❌ Au moins une vérification a échoué — corrigez avant de tagger une release."
  exit 1
fi
