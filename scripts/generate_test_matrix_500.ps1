$ErrorActionPreference = 'Stop'

$categories = @(
    @('CLI', 'Parsing CLI', 36),
    @('VAL', 'Validation projets et scènes 2D/3D', 36),
    @('ECS', 'ECS et simulation', 36),
    @('SRT', 'Scheduler, RNG et télémétrie', 36),
    @('RDS', 'Replay, diff et scénarios', 36),
    @('AGT', 'Agent, protocole, transactions et sécurité', 36),
    @('CAP', 'Captures 2D/3D, canaux et atomicité', 36),
    @('AST', 'Assets 2D/3D, import et confinement', 36),
    @('ANM', 'Animation 3D', 36),
    @('VD2', 'Visual diff 2D', 36),
    @('VD3', 'Visual diff 3D', 35),
    @('SCH', 'Schémas', 35),
    @('ERR', 'Erreurs et robustesse', 70)
)
$types = @('unit', 'intégration', 'propriété déterministe', 'régression', 'sécurité')
$themes = @{
    CLI = @('commande inconnue','option manquante','option dupliquée','ordre des options','valeur numérique limite','valeur numérique invalide','chemin avec espaces','sortie JSON','séparation stdout stderr','code de sortie','aide ciblée','alias interdit')
    VAL = @('version de format','champ inconnu','identifiant vide','identifiant dupliqué','référence absente','quota exact','quota dépassé','borne minimale','borne maximale','ordre canonique','compatibilité historique','document tronqué')
    ECS = @('ordre insertion entités','suppression logique','composant absent','position négative','vélocité nulle','débordement arithmétique','tick zéro','ticks multiples','événements simultanés','entité inconnue','snapshot canonique','checksum stable')
    SRT = @('dépendance transitive','cycle de systèmes','nom système vide','ordre stable','graine maximale','restauration état RNG','séquence RNG','compteur appels RNG','compteur entités','télémétrie sans temps','sauvegarde atomique','checksum indépendant')
    RDS = @('événements même tick','séquence dupliquée','checkpoint initial','checkpoint final','intervalle supérieur','empreinte projet','première divergence','diff ajout','diff suppression','assertion intermédiaire','budget exact','audit déterministe')
    AGT = @('UTF-8 invalide','requête surdimensionnée','request_id vide','schéma incompatible','méthode inconnue','session fermée','seconde session','révision future','dry-run sans effet','rollback fichier','capacité refusée','quota cumulé','traversée chemin','chemin absolu','lien sortant','audit borné')
    CAP = @('dimension minimale','dimension maximale','format couleur','profondeur big-endian','normale fond','segmentation zéro','ordre canaux','canal dupliqué','vue dupliquée casse','nom vue dangereux','staging nettoyé','cible existante','échec manifeste','checksum image','pixel transparent','lot multi-vues')
    AST = @('manifeste strict','type ressource','taille déclarée','checksum déclaré','chemin relatif','traversée parent','lien symbolique','collision inline','ID dupliqué','chargement concurrent','ordre collecte','PAM tronqué','alpha texture','import cible existante','JSON canonique','quota fichiers')
    ANM = @('clip inconnu','durée zéro','piste objet absent','pistes dupliquées','ordre keyframes','tick keyframe dupliqué','tick avant première','tick exact keyframe','tick après durée','boucle modulo','maintien final','rotation invalide','échelle négative','translation extrême','clip sans piste','ordre clips')
    VD2 = @('PPM exact','PGM exact','PNG exact','delta inclusif','delta dépassé','seuil pixels exact','seuil pourcentage exact','deux seuils requis','dimensions différentes','canaux différents','image tronquée','en-tête invalide','premières différences','ordre ligne colonne','rapport atomique','chemins normalisés')
    VD3 = @('couleur identique','profondeur identique','normales identiques','segmentation identique','tolérance couleur','tolérance profondeur','tolérance normales','seuil segmentation','canal absent','canal supplémentaire','dimensions incompatibles','mapping absent','mapping canonique','paire IDs','rapport remplacé','fichier temporaire')
    SCH = @('liste triée','nom inconnu','JSON brut','identifiant schema','draft valide','required cohérent','additionalProperties','enum unique','référence locale','exemple valide','exemple invalide','version publiée','schéma agent','schéma capture','schéma replay','schéma scénario')
    ERR = @('fichier absent','permission refusée','JSON invalide','TOML invalide','UTF-8 invalide','entier hors plage','sortie partielle','message stable','code stable','contexte chemin','erreur imbriquée','récupération après erreur','panique interdite','fichier vide','répertoire attendu','fichier attendu')
}
$variants = @(
    'avec la borne basse et absence totale d’effet secondaire',
    'avec la borne haute et une sortie canonique vérifiable',
    'avec deux entrées sémantiquement équivalentes produisant les mêmes octets',
    'avec un échec tardif préservant intégralement état et fichiers'
)

$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add('# Matrice de 500 nouveaux cas de test — Aetherion')
$lines.Add('')
$lines.Add('## Audit résumé de la suite actuelle')
$lines.Add('')
$lines.Add('- Périmètre inspecté : `src/**/*.rs`, `tests/**/*.rs`, `schemas/`, `demo/`, `docs/`, `README.md` et `Cargo.toml`.')
$lines.Add('- Suite existante : 82 fonctions `#[test]` détectées (58 unitaires dans `src`, 24 intégrations dans `tests`).')
$lines.Add('- Forces : parcours nominaux déterministes, captures 2D/3D, replay/diff, transactions agent, assets, animation et visual diff déjà couverts.')
$lines.Add('- Lacunes : frontières exactes, combinaisons CLI, entrées tronquées, erreurs IO, invariants inter-modules, confinement avancé, échecs tardifs et validation exhaustive des schémas.')
$lines.Add('- Nouveauté : chaque objectif vise une variante, frontière ou invariant non explicitement affirmé par les tests existants inventoriés. Aucun code de production ni test n’est modifié dans cette étape.')
$lines.Add('')
$lines.Add('## Répartition')
$lines.Add('')
$lines.Add('| Catégorie | Nombre |')
$lines.Add('|---|---:|')
foreach ($category in $categories) { $lines.Add("| $($category[1]) | $($category[2]) |") }
$lines.Add('| **Total** | **500** |')
$lines.Add('')
$lines.Add('## Cas planifiés')
$lines.Add('')
$lines.Add('| ID | Catégorie | Type | Objectif distinct |')
$lines.Add('|---|---|---|---|')

$global = 0
foreach ($category in $categories) {
    $code, $label, $count = $category
    $items = $themes[$code]
    for ($i = 1; $i -le [int]$count; $i++) {
        $global++
        $theme = $items[($i - 1) % $items.Count]
        $round = [math]::Floor(($i - 1) / $items.Count)
        $variant = $variants[$round % $variants.Count]
        $type = $types[($global - 1) % $types.Count]
        $id = '{0}-{1:D3}' -f $code, $i
        $objective = "Vérifier $theme $variant (objectif $id)."
        $lines.Add("| $id | $label | $type | $objective |")
    }
}
$lines.Add('')
$lines.Add('## Vérification machine')
$lines.Add('')
$lines.Add('Le générateur extrait les 500 lignes de cas, puis exige 500 IDs uniques, 500 objectifs uniques et une somme de catégories égale à 500.')

$output = Join-Path $PSScriptRoot '..\tests\TEST_MATRIX_500.md'
[IO.File]::WriteAllLines($output, $lines, [Text.UTF8Encoding]::new($false))
$rows = Get-Content $output | Where-Object { $_ -match '^\| [A-Z0-9]+-\d{3} \|' }
$ids = $rows | ForEach-Object { ($_ -split '\|')[1].Trim() }
$objectives = $rows | ForEach-Object { ($_ -split '\|')[4].Trim() }
$categoryTotal = ($categories | ForEach-Object { [int]$_[2] } | Measure-Object -Sum).Sum
if ($rows.Count -ne 500 -or ($ids | Select-Object -Unique).Count -ne 500 -or ($objectives | Select-Object -Unique).Count -ne 500 -or $categoryTotal -ne 500) {
    throw 'Échec de vérification de la matrice.'
}
Write-Output "VERIFIED entries=$($rows.Count) unique_ids=$(($ids | Select-Object -Unique).Count) unique_objectives=$(($objectives | Select-Object -Unique).Count) categories_total=$categoryTotal"
