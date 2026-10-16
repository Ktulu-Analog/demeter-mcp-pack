# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Le format s'inspire de [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet suit [Semantic Versioning](https://semver.org/lang/fr/).

---

## [0.3.3] — 2026-10-16

`mcp-legifrance` 0.2.2. Corrections de paramètres déclarés dans le schéma des outils mais ignorés par le code, repérées en comparant chaque schéma au code de son outil.

### Corrigé

- `legifrance_lister_codes` : `filtre` et `en_vigueur_seulement` étaient toujours ignorés. Ce correctif figurait dans la 0.3.2 mais n'y était pas appliqué ; il l'est désormais. Les paramètres sont transmis à `/list/code` (`codeName`, `states`), la liste est paginée par pages de 100 (maximum de l'API — l'ancien `pageSize` de 200 le dépassait) et lue avec les bons champs (`titre`, `cid`, `etat`).
- `legifrance_derniers_jo` : `taille` était ignoré (toujours 10 JO) ; il est lu et borné entre 1 et 2 499.
- `legifrance_dates_sans_jo` : `annee` était ignoré ; il filtre désormais les dates retournées.
- `legifrance_convention_cont`, `legifrance_convention_article`, `legifrance_convention_section` : le paramètre `date`, déclaré mais jamais utilisé (aucun équivalent dans l'API), est retiré du schéma.

### Modifié

- Annotations des outils réduites à `readOnlyHint: true` (au lieu de quatre indices) : environ 1 500 jetons de moins par requête quand les 63 outils sont exposés au LLM, sans perte d'information pour un outil en lecture seule.

### Vérifié

- Harnais de conformité (63 outils, requêtes validées contre le swagger) : 0 écart ; 11 tests unitaires ; build sans avertissement.

---

## [0.3.2] — 2026-10-15

Audit de conformité de `mcp-legifrance` (v0.2.0) contre le swagger Légifrance : **67 des 68 opérations** sont désormais couvertes (la 68ᵉ, `/consult/code/tableMatieres`, est dépréciée et remplacée par `/consult/legi/tableMatieres`). Chaque outil a été exécuté contre un faux serveur validant requêtes et schémas du swagger : 0 écart.

### Corrigé — requêtes invalides (HTTP 400/404)

- Chemins inexistants : `getArticleVersions` → `/consult/getArticleByCid` ; `getArticlesWithSameNum` → `/consult/sameNumArticle` (corps `date`, `articleCid`, `textCid`, `articleNum`) ; `getCnilByOldId` / `getJuriByOldId` → `getCnilWithAncienId` / `getJuriWithAncienId` (champ `ancienId`) ; `hasLinksServicePublic` → `/consult/hasServicePublicLinksArticle` (corps = tableau de CID) ; `/consult/jorf/article` → `/consult/getJoWithNor` ; `/consult/jorf/section` → `/consult/jorfPart` ; `/consult/legi/tableMatieresByOldId` → `/consult/getCodeWithAncienId` ; `/list/kali` → `/list/conventions`.
- `legifrance_jorf` envoyait `id` au lieu de `textCid` (requis). Idem `jorf_part`.
- `kaliContIdcc` : champ `id` (et non `idcc`). `boccsAndTexts` : `idcc` scalaire. `bodmr` / `docsAdmins` : `years` = tableau de chaînes, pas de pagination pour docsAdmins.
- Recherche `/search` : filtres en `valeurs` (tableau) ; `NOM_CODE` / `TEXT_NOM_CODE` (nom du code) au lieu du `CODE_ID` inexistant ; `DATE_VERSION` en `singleDate` ; `sort` toujours renseigné et traduit par fond (`DATE_DESC` → `SIGNATURE_DATE_DESC` pour JORF/LODA/KALI/CIRC…) ; fonds validés contre l'enum (`JADE` → `CETAT`, `DOSSIER_LEGISLATIF` refusé avec message clair) ; `legifrance_jurisprudence_plan_classement` appelle enfin `/consult/getJuriPlanClassement` (il faisait une recherche JADE).
- `legifrance_legi_part` appelait `getSectionByCid` ; il appelle `/consult/legiPart`. `legifrance_loi_decret` utilise `/consult/lawDecree` (repli `legiPart`, `JORFTEXT` → `/consult/jorf`).
- `legifrance_consulter_code` : `/consult/legi/tableMatieres` (le point d'entrée précédent est déprécié). `legifrance_lister_codes` : `filtre` et `en_vigueur_seulement` étaient ignorés ; pagination ≤ 100.
- `legifrance_article_par_numero` ignorait `date` : il renvoie maintenant la version en vigueur à cette date. Numéros normalisés (`L. 1234-1` → `L1234-1`).

### Corrigé — lecture des réponses (résultats vides ou faux)

- **`strip_html` corrompait tous les accents** (`é` → `Ã©`) ; **panics** `&texte[..N]` sur caractère multi-octet (arrêt du serveur en stdio). Troncatures désormais sûres.
- Champs lus conformes au swagger : `chronolegiExists`, `lstDateDisabled`, `lstYearDisabled`, `liensSP`, `listSection`, `dossiersLegislatifs`, `newTexts`/`oldTexts`, `liensCitePar`/`liensCite`, `tables[]`, `pdfMetadatas`, `text` / `acco` / `circulaire` / `dossierLegislatif` / `debat`, `versions` (map) de la chronologie, `results` (map) de `/suggest`.
- `legifrance_historique_texte` : `POST /chrono/textCid` (le `GET` indique seulement l'existence de versions).
- `legifrance_loi_decret` / `jorf` / `legi_part` retournent désormais le texte des articles (budget ≈ 40 000 caractères), plus seulement les titres de sections.

### Ajouté

- `legifrance_ping` (5 endpoints `/ping`). Résolution dynamique de tout code via `/list/code` (plus seulement 16 codes). Paramètres de recherche `page`, `statut`, `type_champ`, `type_recherche`, `proximite`. `legifrance_code_complet` accepte `section_cid`.
- MCP : négociation de `protocolVersion`, annotations `readOnlyHint`, réponse 202 aux notifications HTTP, 405 sur `GET /mcp`, contrôle `Origin`, isolation des panics par appel.
- Tests unitaires (11). Variables `LEGIFRANCE_API_URL` / `LEGIFRANCE_OAUTH_URL` (proxy, tests).

### Modifié

- **Écoute HTTP par défaut sur `127.0.0.1`** (et non `0.0.0.0`) : le serveur n'a pas d'authentification et consomme le quota PISTE. Sans effet en mode stdio (manifest). `MCP_HOST=0.0.0.0` pour restaurer l'ancien comportement.

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
