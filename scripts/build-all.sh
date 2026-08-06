#!/usr/bin/env bash
# ============================================================================
# demeter-mcp-pack — build-all.sh
# ============================================================================
# Compile les 5 serveurs MCP officiels pour UNE plateforme donnée, et range
# les binaires produits directement dans mcp-servers/mcp-*/bin/, avec le nom
# attendu par les manifest.json (voir platform_key() côté Demeter).
#
# Usage :
#   scripts/build-all.sh <platform-key> <rust-target-triple>
#
# Exemples (voir aussi .github/workflows/release.yml pour la matrice complète) :
#   scripts/build-all.sh linux-x64    x86_64-unknown-linux-gnu
#   scripts/build-all.sh macos-x64    x86_64-apple-darwin
#   scripts/build-all.sh macos-arm64  aarch64-apple-darwin
#   scripts/build-all.sh windows-x64  x86_64-pc-windows-msvc
# ============================================================================
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <platform-key> <rust-target-triple>" >&2
  exit 1
fi

PLATFORM_KEY="$1"
TARGET="$2"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# name_du_dossier_pack:name_du_dossier_source:nom_du_binaire_cargo
SERVERS=(
  "mcp-judilibre:mcp-judilibre-rs:mcp-judilibre"
  "mcp-grist:mcp-grist-rs:mcp-grist"
  "mcp-legifrance:mcp-legifrance-rs:mcp-legifrance"
  "mcp-system:mcp-system-rs:mcp-system"
  "mcp-image:mcp-image-rs:mcp-image"
)

echo "Ajout de la cible Rust $TARGET (si absente)…"
rustup target add "$TARGET" >/dev/null 2>&1 || true

EXT=""
[[ "$TARGET" == *windows* ]] && EXT=".exe"

for entry in "${SERVERS[@]}"; do
  IFS=":" read -r pack_name src_dir bin_name <<< "$entry"

  echo ""
  echo "══ $pack_name ($src_dir) → $PLATFORM_KEY ══"
  (
    cd "$ROOT/sources/$src_dir"
    cargo build --release --target "$TARGET"
  )

  mkdir -p "$ROOT/mcp-servers/$pack_name/bin"
  src_bin="$ROOT/sources/$src_dir/target/$TARGET/release/${bin_name}${EXT}"
  dst_bin="$ROOT/mcp-servers/$pack_name/bin/${bin_name}-${PLATFORM_KEY}${EXT}"

  cp "$src_bin" "$dst_bin"
  chmod +x "$dst_bin" 2>/dev/null || true
  echo "  → $dst_bin"
done

echo ""
echo "✅ Build terminé pour $PLATFORM_KEY ($TARGET)"
