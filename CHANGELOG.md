# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Le format s'inspire de [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet suit [Semantic Versioning](https://semver.org/lang/fr/).

---

## [0.3.1] — 2026-08-07

### Corrigé

- **`manifest.json` manquants** — `mcp-servers/mcp-judilibre/`, `mcp-grist/`, `mcp-legifrance/`, `mcp-system/` et `mcp-image/` ne contenaient aucun `manifest.json`, alors que la structure documentée dans le README (et attendue par le client MCP stdio de Demeter) en prévoit un par serveur. Résultat côté Demeter : les binaires étaient bien téléchargés et installés, mais aucun serveur n'était détecté (absence de `manifest.json` à découvrir). Les 5 manifests ont été ajoutés, avec les chemins de binaires (`linux-x64`, `windows-x64`), l'argument `--transport stdio` et les paramètres de configuration (clés d'API, URLs) attendus par chaque serveur.
- **`checksums.txt` avec un préfixe `./` sur chaque nom de fichier** — `scripts/package.sh` générait les checksums via `sha256sum ./*.zip`, ce qui préfixait chaque entrée par `./` (ex. `./mcp-pack-0.3.0.zip`). Le client MCP de Demeter, qui compare des noms de fichiers sans ce préfixe, échouait alors avec `Aucune entrée pour '<archive>.zip' dans checksums.txt`. Corrigé en `sha256sum -- *.zip` (le fix a aussi été appliqué côté Demeter, qui ignore désormais un éventuel préfixe `./` par robustesse, quelle que soit la version du pack).

### Modifié

- **Identifiants d'API rendus optionnels** dans les manifests `mcp-judilibre`, `mcp-grist` et `mcp-legifrance` (`required: false` au lieu de `true`) — ces trois binaires démarrent déjà normalement sans leurs identifiants respectifs, avec un simple avertissement en log et une erreur renvoyée par les outils concernés le cas échéant. Les marquer `required` dans le manifest aurait pu laisser penser à un binaire corrompu ou mal installé en cas d'oubli, alors qu'il s'agit d'un fonctionnement attendu.

---

## [0.3.0] et versions antérieures

Historique non tracé, versions privées jusqu'à 0.3.0.
La 0.3.0 correspond à l'état du pack tel que publié avant ce correctif : 5 serveurs (`mcp-judilibre`, `mcp-grist`, `mcp-legifrance`, `mcp-system`, `mcp-image`), transport HTTP Streamable et stdio, build multi-plateforme (`linux-x64`, `windows-x64`) et publication automatisée par tag Git.
