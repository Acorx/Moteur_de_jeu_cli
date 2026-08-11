# Feuille de route

Les jalons ci-dessous décrivent une trajectoire, pas des fonctionnalités présentes.

## M0 — Socle CLI (terminé)

Format TOML v1, validation, entités position/vélocité, ticks bornés, `inspect`, snapshots JSON, tests et documentation.

## M0.1 — Capture 2D headless (terminé)

Rendu logiciel RGB déterministe, caméra et apparences rétrocompatibles, capture PPM P6, manifeste JSON agent-readable, scène de démonstration et tests pixels/intégration.

## M0.2 — Replay et diff (terminé)

Replay JSON v1, empreinte projet, événements horodatés et totalement ordonnés, quatre commandes d'entrée déterministes, checksums à chaque tick, détection JSON des divergences, diff structuré de snapshots/manifestes, fixture et tests unitaires/intégration.

## M0.3 — Scénarios agent-native et garde-fous (terminé)

Scénarios JSON v1, événements déterministes réutilisés, assertions par tick/finales (checksum, état d'entité, cardinalité, visibilité), budgets stricts, rapport JSON atomique, audit JSONL append-only, codes de sortie stables, fixtures pass/fail et tests de déterminisme/validation.

## M1 — Simulation reproductible (terminé)

- stockages ECS séparés et ordre canonique par `EntityId` ;
- ordonnanceur explicite validant ordre, noms et dépendances (`input` puis `movement`) ;
- SplitMix64 entier documenté, état restaurable et vecteurs golden ;
- replay v2 à checkpoints espacés/configurables, lecture et vérification automatiques des fixtures v1 ;
- télémétrie déterministe `aetherion.telemetry/v1` exportable par la CLI, compteurs par système et temps mural explicitement hors verdict ;
- snapshots/captures/scénarios/checksums historiques conservés et limites de ressources maintenues.

## M2 — Automatisation par agents (terminé)

- schémas JSON versionnés publiés et commandes `schema list/show` ;
- protocole JSONL local strict sur stdin/stdout, sessions isolées et erreurs stables ;
- inspection, ticks, entrées, captures et diff sémantique canonique ;
- transactions atomiques sur clone, staging des fichiers, rollback et dry-run ;
- contrôle optimiste par révision, capacités, confinement des chemins et quotas IO/travail ;

## M3 — Affichage optionnel (terminé)

- fenêtre interactive Windows 2D derrière la feature `display`, sans crate graphique dans le build headless ;
- caméra, pause/pas-à-pas et rendu découplés du tick déterministe ;
- encodeur PNG déterministe interne et captures multi-vues publiées atomiquement ;
- intégration agent `capture.create` PNG et `capture.multi` dry-run/transactionnel ;
- mode headless, commandes, snapshots et checksums M0–M2 conservés.

## M4 — Rendu étendu (en cours)

- terminé : captures 2D profondeur/normales/segmentation et comparaison visuelle automatisée avec tolérances entières ;
- sous-étape 3D actuelle terminée : scènes strictes `aetherion.scene3d/v1`, triangles historiques rétrocompatibles, meshes/matériaux/objets réutilisables, transformations entières, projection orthographique/z-buffer et commande headless `aetherion capture3d --scene FILE --output FILE [--width N] [--height N]` produisant un PPM et son manifeste atomiquement ;
- conventions actuelles : échelle `1000` = 1, ordre échelle → Rx → Ry → Rz → translation, rotations en millidegrés limitées aux multiples de `90000`, opacité entière `0..1000`, ordre canonique et quotas stricts (1 MiB, 10 000 meshes/matériaux, 100 000 objets/triangles, 300 000 sommets, 16 777 216 pixels) ;
- terminé : animation 3D déterministe par clips, pistes et keyframes entières, échantillonnage en escalier, boucle ou maintien final, sélection CLI par `--animation` et `--ticks` ;
- terminé (M4-F) : ressources 3D externes strictes mesh/material, confinement, quotas, taille/checksum, chargement concurrent avec collecte canonique, résolution `capture3d --assets` et import atomique `asset3d-import` ;
- terminé (M4-G) : canaux 3D couleur, profondeur PGM P5, normales PPM P6 et segmentation PPM P6, manifestes et publication atomique ;
- terminé (M4-H) : diff 3D intégré par manifestes, tolérances indépendantes, résumé déterministe des IDs/mappings de segmentation, rapport atomique et codes 0/1/2 ;
- restant : clôture et validation globale du jalon étendu. M4 reste donc en cours.

## M5 — Plugins et durcissement

- ABI/API versionnée ;
- plugins sandboxés par capacités, idéalement WebAssembly ;
- signatures, manifeste de permissions et registre de provenance ;
- limites CPU/mémoire/IO, politique réseau restrictive ;
- fuzzing des formats, revue de supply-chain et réponse aux vulnérabilités.

## Hors objectif immédiat

Éditeur visuel complet, rendu AAA, marketplace et compatibilité générale avec les grands moteurs. Chaque extension devra préserver le binaire headless, le déterminisme observable et les contrats agent-native.
