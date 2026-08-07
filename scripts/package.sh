#!/usr/bin/env bash
# ============================================================================
# demeter-mcp-pack — package.sh
# ============================================================================
# À exécuter une fois que mcp-servers/mcp-*/bin/ contient les binaires de
# TOUTES les plateformes (après avoir fait tourner build-all.sh pour chacune,
# typiquement téléchargés depuis les artifacts des jobs CI — voir release.yml).
#
# Produit dans dist/ :
#   - mcp-pack-<version>.zip           (les 5 serveurs, toutes plateformes)
#   - mcp-judilibre-<version>.zip      (un seul serveur, toutes plateformes)
#   - mcp-grist-<version>.zip
#   - mcp-legifrance-<version>.zip
#   - mcp-system-<version>.zip
#   - checksums.txt                    (SHA-256 de chaque archive)
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(cat "$ROOT/VERSION")"
DIST="$ROOT/dist"

rm -rf "$DIST"
mkdir -p "$DIST"

echo "📦 Assemblage du pack v$VERSION"

# Pack combiné : les 5 serveurs
(
  cd "$ROOT/mcp-servers"
  zip -rq "$DIST/mcp-pack-${VERSION}.zip" mcp-judilibre mcp-grist mcp-legifrance mcp-system mcp-image \
    -x "*/.gitkeep"
)

# Archives individuelles, pour un usage hors Demeter (ex. Claude Desktop)
for name in mcp-judilibre mcp-grist mcp-legifrance mcp-system mcp-image; do
  (
    cd "$ROOT/mcp-servers"
    zip -rq "$DIST/${name}-${VERSION}.zip" "$name" -x "*/.gitkeep"
  )
done

# Checksums
(
  cd "$DIST"
  sha256sum -- *.zip > checksums.txt 2>/dev/null || shasum -a 256 -- *.zip > checksums.txt
)

echo ""
echo "✅ Archives prêtes dans $DIST :"
ls -la "$DIST"
echo ""
cat "$DIST/checksums.txt"
