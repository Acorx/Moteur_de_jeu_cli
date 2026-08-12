# Feuille de route

## Vision et règles de décision

Aetherion vise un moteur CLI/agent-native de classe production, modulaire et extensible. La convergence vers les capacités d'un moteur généraliste est une trajectoire longue, pas une promesse de parité. Chaque incrément doit rester headless-first, déterministe et borné, sûr par défaut, observable par formats machine-readable versionnés, et rétrocompatible avec les projets, snapshots, replays et commandes publiés.

Les jalons ci-dessous décrivent une trajectoire, pas des fonctionnalités présentes. Une fonction n'est considérée présente que lorsqu'elle est documentée comme terminée, couverte par des tests et exposée par la CLI ou l'API publique.

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

## M4 — Rendu étendu (terminé)

- captures 2D profondeur/normales/segmentation, comparaison visuelle à tolérances entières et défaut couleur historique préservé ;
- scènes 3D strictes, transformations et animations entières, projection/z-buffer logiciel, ressources externes confinées et chargées canoniquement ;
- canaux 3D couleur/profondeur/normales/segmentation, publication atomique et diff 3D intégré avec rapport déterministe ;
- clôture certifiée par `cargo run -- certify-m4 --report docs/m4-certification.json` : rapport `aetherion.m4-certification/v1` stable, schéma publié, test d’intégration de reproductibilité/atomicité et matrice M0–M4 verte ;
- contrats conservés : codes 0/1/2/3, formats historiques, ordre canonique, quotas et absence de dépendance lourde.

## M5 — Plateforme de plugins sécurisée (premières tranches présentes)

**M5-A présent et couvert :** manifeste JSON strict `aetherion.plugin/v1`, ABI hôte `1.1`, politique de compatibilité mineure testée sur les hôtes `1.0` et `1.1`, identifiant et version validés, capacités déclaratives limitées, quotas mémoire/fuel/IO/fichiers, refus des doublons et ABI incompatibles, chargement déterministe d'un catalogue (maximum 256 manifestes) et commandes `plugin validate`, `plugin inspect`, `plugin list`.

**M5-B présent et couvert :** `plugin resolve` publie atomiquement le lockfile versionné `aetherion.plugin-lock/v1`, canoniquement trié par identifiant, avec chemins relatifs, ABI, capacités, version et checksum FNV-1a. `plugin lock-check` recalcule ces données et retourne le code 1 avec rapport JSON en cas de divergence. Le test d'intégration `tests/plugin_lock.rs` vérifie la stabilité, l'acceptation et la détection d'une modification de checksum.

Cette tranche reste **manifest-only** pour les commandes de catalogue : l'exécution passe désormais par `plugin run`, tandis que l'audit de provenance passe par `plugin audit`.

**M5-C0 présent et couvert :** la feature optionnelle `plugin-runtime` embarque `wasmi 0.32.3`, instancie un module avec un linker vide et appelle `aetherion_main: () -> i32`. Le build par défaut reste sans runtime WebAssembly; les modules invalides, exports absents et traps produisent des erreurs `plugin_runtime_*` stables.

**M5-C1 présent et couvert :** `RuntimeLimits` applique réellement `fuel` et `memory_bytes`, mesure le fuel consommé et distingue les dépassements fuel/mémoire. Les tests vérifient la reproductibilité, la boucle bornée et `memory.grow` refusé.

**M5-C2 présent et couvert :** le module d'import versionné `aetherion_v1` expose uniquement les fonctions autorisées par `asset_read`, `scene_read`, `simulation_read` et `telemetry_write`. Les vues de monde/scène/assets sont des copies validées, les imports non hôte et les capacités absentes sont refusés avant instanciation, et la télémétrie reste dans un tampon mémoire borné. Les tests couvrent lecture, absence de capacité, lecture seule, confinement logique des assets, import non hôte et déterminisme.

**M5-C3 présent et couvert :** les quotas `io_read_bytes` et `files` sont appliqués aux imports d'assets avant et pendant l'exécution. Les dépassements produisent `plugin_runtime_io_read_quota` ou `plugin_runtime_files_quota`; `ExecutionReport.io` publie les compteurs déterministes. Aucun import d'écriture n'est exposé, donc `io_write_bytes` reste nul par construction. Les tests couvrent quota autorisé, dépassement de lecture et dépassement de fichiers.

**M5-C4 présent et couvert :** le runtime inspecte chaque import avant instanciation et refuse explicitement WASI, sockets, réseau, HTTP, TCP, UDP et DNS avec `plugin_runtime_network_denied`. Aucun linker, import ou capability réseau n'est enregistré; les tests couvrent modules WASI, `wasi:io`, socket et réseau explicites.

**M5-C5 présent et couvert :** `plugin run --manifest FILE --module FILE` exécute un module borné avec `--export`, vues optionnelles de projet/scène/assets, `--dry-run` et rapport atomique `aetherion.plugin-run-report/v1`. Le build sans `plugin-runtime` conserve une erreur stable `plugin_runtime_feature_disabled`; les rapports n'embarquent aucun chemin machine.

**M5-C6 présent et couvert :** `plugin audit --manifest FILE --module FILE` valide sans exécution le manifeste, le module, l'export et les imports autorisés, puis publie atomiquement `aetherion.plugin-audit/v1`. Le rapport contient les checksums FNV-1a exacts du manifeste et du module, l'identité/version, l'ABI `1.1`, les capacités triées, les quotas et la provenance du runtime `wasmi 0.32.3` avec réseau/WASI désactivés. `signatures.status` est explicitement `not_implemented` jusqu'à la tranche cryptographique dédiée. Des golden tests couvrent dry-run, exécution, télémétrie/IO et audit; un corpus borné couvre les modules vides/invalides, exports/signatures, imports inconnus/réseau, capacités, doublons et quotas frontières.

**Étapes futures mesurables :**

1. C7 : signatures cryptographiques hors ligne, vérification de clés de confiance, révocation et SBOM sans modifier les contrats C5/C6.

## M6 — Scripting déterministe (tranches A/B présentes)

**M6-A présent et couvert :** `script-run` interprète un format JSON strict `aetherion.script/v1`, avec substitutions `{{variable}}`, commandes bornées sans shell ni processus externe (`true`, `false`, `echo`, `noop`), dry-run et budgets de commandes/ticks. Les dépassements retournent le code 3.

**M6-B présent et couvert :** les politiques `stop`/`continue`, les échecs au code 1 et le rapport atomique versionné `aetherion.script-report/v1` sont couverts par `tests/script_run.rs`.

Ce n'est pas une VM ni un langage de scripting général : aucun script ne participe encore à la simulation, aux snapshots ou aux replays. Une VM sandboxée avec API versionnée, budgets d'instructions/mémoire et absence d'horloge/IO/réseau implicite reste future.

## M7 — Scène, physique, audio et rendu (premières fondations présentes)

**M7-A présent et couvert :** composant `collider` optionnel dans les projets/scènes, stockage ECS dédié, détection AABB canonique, séparation entière, réponse de vitesse à restitution milli-unitaire, corps statiques et télémétrie (`collisions_resolved`, `entities_modified`) dans le système `physics`. Les projets sans collider conservent leur comportement de simulation historique.

- scène : hiérarchie et préfabs versionnés avec migration et tests golden ;
- physique : détection/résolution 2D à pas fixe, collisions reproductibles et corpus de conformance ;
- audio : graphe offline/headless testable avant sortie temps réel optionnelle ;
- rendu : matériaux, éclairage et pipelines plus riches derrière interfaces stables, sans faire dépendre la simulation du GPU.

Chaque sous-système doit publier ses quotas, formats et tests de déterminisme. Il n'existe aujourd'hui ni physique avancée/3D ni audio généralistes, ni rendu AAA.

## M11 — Premier pipeline GPU temps réel (tranche 1 présente)

**Présent et couvert :** feature Cargo `render-gpu` optionnelle avec `wgpu 0.19`, `winit 0.29`, `glam`, `bytemuck` et `pollster`. La commande `gpu-demo --scene FILE [--assets FILE] [--width N] [--height N] [--frames N]` charge une `Scene3d` validée, résout les assets existants et ouvre une fenêtre temps réel. `--frames` borne l'exécution et publie le rapport versionné `aetherion.gpu-demo/v1`. La commande `gpu-benchmark --scene FILE --frames N` réutilise cette boucle et publie `aetherion.gpu-benchmark/v1` avec adaptateur, temps mural et FPS millième.

Le pipeline sélectionne l'adaptateur compatible, configure la surface en sRGB avec présentation FIFO lorsque disponible, crée un pipeline de triangles colorés, consomme les normales/UV optionnels des meshes avec fallback flat-shading, applique un culling AABB orthographique des objets et un éclairage directionnel/ambiant simple. Il utilise une caméra orthographique dérivée de `Camera3d`, un depth buffer `Depth24Plus`, et gère redimensionnement/perte de surface/épuisement mémoire. Les sommets GPU sont des copies `f32` d'un snapshot de scène ; le renderer ne possède aucun accès mutable à la simulation.

Le chemin headless par défaut reste compilable et exécutable sans dépendances GPU. Le rendu GPU n'est pas déterministe bit-à-bit et ne remplace ni `capture3d`, ni les captures CPU, ni les visual diffs. L'ADR [`docs/adr/0001-frontiere-rendu-gpu.md`](adr/0001-frontiere-rendu-gpu.md) fixe cette frontière.

**Critères restant pour clôturer M11 :** paliers de géométrie versionnés, mesure GPU timestamp/temps CPU lorsque les fonctionnalités du pilote le permettent, capture de référence contrôlée et validation sur Windows/Linux/macOS avec au moins un backend logiciel CI. La commande et le rapport de benchmark sont présents ; la validation multi-adaptateurs reste à exécuter dans CI et sur matériel représentatif.

## M12 — Assets glTF (tranche 1 présente)

**Présent et couvert :** feature `gltf-import` optionnelle, commande `gltf-import --input FILE --output FILE`, support glTF/GLB, selection de la scene par defaut, traversal hierarchique des noeuds, composition des transformations, import des positions et indices de primitives triangulees, conversion des couleurs PBR de base et publication canonique `aetherion.scene3d/v1`.

Les quotas d'entree (16 MiB), buffers (64 MiB), images declarees (4096), validation des nombres, overflow de quantification, primitives non triangulees, indices hors limites et sorties existantes sont refuses avec des erreurs stables. Les images ne sont pas decodees dans cette tranche. Les textures, skins, morph targets et animations glTF restent explicitement futures. `render-gpu` active automatiquement cette tranche afin qu'un fichier importe puisse etre affiche par `gpu-demo`.

**M12.3 présente :** les meshes conservent des normales optionnelles quantifiées à `1_000_000` et des UV optionnels quantifiés à `1_000_000`. Les longueurs sont vérifiées, les normales nulles sont refusées, les scènes sans attributs restent compatibles et le chemin CPU historique n'est pas modifié.

**M12.4 présente :** `Material3d` accepte un ID `base_color_texture`, les manifests acceptent les assets binaires PNG/JPEG, et le backend GPU décode les textures référencées avec quotas 4096×4096/16 777 216 pixels. Le renderer crée un bind group par texture et regroupe les sommets en lots de matériau ; le CPU ignore les textures et reste l'oracle déterministe.

**M12.5 présente :** `--cache-dir` pour `capture3d`, `gpu-demo` et `gpu-benchmark`. Le cache est versionné par type, checksum, format et importeur, publié atomiquement et toujours précédé d'une vérification du fichier source. Il est strictement hors des données déterministes et les erreurs de cache déclenchent un repli transparent.

**Prochaines tranches M12 :** chargement asynchrone, import des textures glTF et matériaux textures plus riches avec quotas.

## M15 — Préparation GPU et visibilité

**M15.1 tranche présente :** le backend GPU calcule une AABB agrégée par objet après transformation entière, teste son intersection avec le volume orthographique et n'upload pas les objets entièrement hors champ. Les rapports `gpu-demo` et `gpu-benchmark` publient `objects`, `culled_objects`, `triangles` effectivement préparés et `draw_calls`. Cette tranche ne prétend pas encore fournir de l'instancing : la géométrie locale et l'instance buffer seront introduits séparément.

**Prochaines tranches M15 :** géométrie locale réutilisable, instance buffers, réduction des draw calls, puis LOD et culling GPU.

## M8 — Tooling, build et packaging (futur)

CI multi-plateforme, artefacts signés, SBOM, vérification des schémas/migrations, profils de build headless minimaux, cache déterministe et installateur/versionnement documentés. Cible mesurable : installation propre, exécution des smoke tests et désinstallation automatisées sur chaque plateforme supportée.

## M9 — Réseau (futur)

Commencer par protocoles explicites et simulation réseau testable (latence/perte), puis réplication autoritaire et rollback si les invariants déterministes sont démontrés. Chiffrement, authentification, quotas et résistance aux entrées hostiles sont des prérequis. Aucun multijoueur n'est présent.

## M10 — Éditeur optionnel et écosystème (futur)

L'éditeur restera un client optionnel des mêmes contrats CLI/API, jamais une dépendance du runtime headless. Avant tout registre public : format de paquet stable, provenance/signatures, compatibilité résolue hors ligne, revue et révocation. Exemples, templates et SDK agents versionnés précéderont une éventuelle place de marché.

## Hors objectif immédiat

Parité rapide avec les moteurs généralistes, éditeur visuel complet, rendu AAA et marketplace. Chaque extension devra préserver le binaire headless, le déterminisme observable, la sécurité par défaut et les contrats agent-native.
