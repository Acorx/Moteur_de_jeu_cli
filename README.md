# Aetherion

Aetherion est un **socle de moteur de jeu CLI/agent-native**, headless, déterministe et inspectable. Le projet vise à long terme une qualité de production, une architecture modulaire et une extensibilité sûre vers certaines capacités d'un moteur généraliste, sans promettre de parité rapide. La sécurité par défaut, les formats versionnés et la rétrocompatibilité restent des contraintes structurantes ; physique, audio, réseau, scripting exécutable, éditeur complet et écosystème public sont des jalons futurs.

## Démarrage rapide

Prérequis : Rust stable et Cargo.

```console
cargo build --release
cargo run -- help
cargo run -- init --path ./demo
cargo run -- doctor --path ./demo
cargo run -- inspect --path ./demo
cargo run -- run --path ./demo --ticks 120 --json --telemetry ./demo/telemetry.json
cargo run -- capture --path ./demo --ticks 5 --output ./demo/capture.ppm
cargo run -- capture --path ./demo --ticks 5 --format png --channels color,depth,normals,segmentation --output ./demo/capture.png
cargo run -- capture-multi --path ./demo --views ./demo/views.json --output-dir ./demo/views-out
cargo run -- capture3d --scene ./demo/scene3d.json --animation walk --ticks 12 --output ./demo/capture3d.ppm --width 320 --height 240
cargo run --features gltf-import -- gltf-import --input ./assets/model.glb --output ./demo/imported-scene.json
cargo run --features render-gpu -- gpu-demo --scene ./demo/scene3d.json --width 1280 --height 720 --frames 120
cargo run --features display -- play --path ./demo --max-ticks 300
cargo run -- replay-create --path ./demo --ticks 10 --events ./demo/events.json --checkpoint-interval 4 --output ./demo/demo-v2.replay.json
cargo run -- replay-run --path ./demo --replay ./demo/demo.replay.json
cargo run -- diff --left ./demo/capture.ppm.json --right ./demo/capture.ppm.json
cargo run -- visual-diff --baseline ./demo/baseline.png --candidate ./demo/capture.png --max-channel-delta 2 --max-different-pixels 10 --max-different-percent-milli 50 --report ./demo/visual-diff.json
cargo run -- visual-diff3d --baseline-manifest ./demo/baseline.ppm.json --candidate-manifest ./demo/capture3d.ppm.json --report ./demo/visual-diff3d.json
cargo run -- certify-m4 --report ./docs/m4-certification.json
cargo run -- scenario-run --path ./demo --scenario ./demo/scenario-pass.json --report ./demo/scenario-report.json --audit ./demo/audit.jsonl
cargo run -- schema list
cargo run -- plugin validate ./plugins/example.plugin.json
cargo run -- plugin inspect ./plugins/example.plugin.json
cargo run -- plugin list ./plugins
cargo run -- plugin resolve --dir ./plugins --lockfile ./demo/aetherion.plugin-lock.json
cargo run -- plugin lock-check --dir ./plugins --lockfile ./demo/aetherion.plugin-lock.json
cargo run --features plugin-runtime -- plugin audit --manifest ./plugins/example.plugin.json --module ./plugins/example.wasm --report ./demo/plugin-audit.json
cargo run -- script-run --script ./demo/script.json --dry-run --report ./demo/script-report.json
cargo run -- bundle --path ./demo --output ./demo/game.bundle.zip
cargo run -- bundle-inspect --input ./demo/game.bundle.zip
cargo run -- agent --path ./demo --root . < ./demo/agent-exchange.jsonl
```

`plugin audit` nécessite la feature `plugin-runtime`. Il valide le manifeste, le module Wasm, l'export et les imports autorisés, puis publie `aetherion.plugin-audit/v1` avec les checksums FNV-1a, l'ABI, les capacités, les quotas et l'état sandbox réseau/WASI. Le rapport ne contient aucun chemin local. `signatures.status` vaut `not_implemented` tant que la tranche cryptographique n'est pas livrée.

Le scénario `demo/scenario-fail.json` illustre une assertion échouée (code 1). `scenario-run` accepte le schéma strict `aetherion.scenario/v1` : projet/empreinte optionnelle, tick final, événements de replay, assertions à un tick ou finales (`checksum`, position, vélocité, nombre et visibilité), et budgets d'entrée, sortie, ticks, événements et assertions. Le délai est seulement indicatif et n'influence jamais le verdict déterministe. Le rapport `aetherion.scenario-report/v1` est écrit atomiquement ; l'audit `aetherion.audit/v1` est JSON Lines append-only, sans horodatage ni secret, avec un `run_id` déterministe dérivé des empreintes projet/scénario.

Codes stables : `0` succès, `1` diff/assertions échouées, `2` usage/validation, `3` divergence replay ou budget dépassé.

Sous Windows, le binaire release se trouve dans `target/release/aetherion.exe`.

## MVP actuel

- M2 : protocole local JSONL strict, session isolée, transactions/dry-run, révisions, capacités, quotas, confinement et staging atomique ;
- M5-C0/C1/C2/C3/C4/C5/C6 : runtime WebAssembly optionnel derrière `plugin-runtime`, fuel/mémoire/IO/fichiers bornés, interdiction réseau explicite, commande `plugin run` avec dry-run et rapport atomique, audit de provenance avec checksums module/manifeste, API hôte `aetherion_v1` activée uniquement par capacités, sans WASI ni imports système implicites ;
- schémas JSON versionnés exposés par `schema list/show` et diff sémantique canonique ;
- dix-sept sous-commandes, dont `agent`, `schema`, `scene` et le prototype `capture3d`, en conservant les commandes historiques ;
- scénarios JSON v1, assertions détaillées, budgets stricts, rapport machine-readable atomique et audit JSONL append-only ;
- stockage ECS réel et séparé (`EntityMetadata`, `Position`, `Velocity`, `Appearance`, `Collider` optionnel) fondé sur des `BTreeMap`, avec itération canonique par `EntityId` ;
- ordonnanceur explicite et inspectable, ordre stable `input` puis `movement` puis `physics`, avec rejet des systèmes inconnus, doublons et dépendances impossibles ;
- PRNG SplitMix64 entier, reproductible sur toutes les plateformes, initialisé par `simulation.seed`, état sérialisable/restaurable et API opt-in qui ne change pas les scènes historiques ;
- télémétrie JSON `aetherion.telemetry/v1` via `run --telemetry FILE` : tick/checksum, ordre des systèmes et compteurs (ticks, entités visitées/modifiées, événements, appels PRNG, collisions résolues) ; aucun temps mural n'entre dans ce format, les checksums ou les verdicts ;
- replay JSON v2 avec `checkpoint_interval`, tick 0 et tick final obligatoires ; `--checkpoint-interval 1` conserve un checksum par tick et les replays v1 restent lus et vérifiés sans migration manuelle ;
- entrées déterministes `set_velocity`, `impulse`, `translate`, `stop`, appliquées par le système `input` avant le système `movement` ;
- diff JSON structuré par tick, entité et chemin de champ, utilisable aussi pour les manifestes de capture ;
- rendu 2D logiciel déterministe, sans GPU, vers PPM P6 ;
- premier pipeline 3D GPU temps réel optionnel (`render-gpu`) avec fenêtre wgpu/winit, depth buffer, caméra orthographique et chargement des scènes `Scene3d` ;
- manifeste JSON adjacent avec caméra, dimensions, tick, checksums et entités visibles ;
- projet déclaratif `aetherion.toml`, lisible et facilement modifiable ;
- monde minimal de type entité/composants : identité, position 2D, vélocité 2D ;
- simulation à pas fixe utilisant des entiers, ordre canonique par identifiant ;
- exécution obligatoirement bornée par `--ticks` (10 par défaut) ;
- snapshots JSON versionnés (`aetherion.snapshot/v1`) ;
- M7-A : collider 2D optionnel validé, détection AABB canonique, séparation entière, réponse de vitesse à restitution milli-unitaire, corps statiques, snapshot inspectable et télémétrie du système `physics` ;
- validation stricte et tests unitaires/intégration.

`inspect` décrit l'état initial sans le modifier. `run --json` décrit l'état final. Les sorties JSON vont sur stdout ; les erreurs vont sur stderr avec un code non nul.

## Format du projet

```toml
[project]
name = "mon-jeu"
format_version = 1

[simulation]
tick_rate = 60
seed = 1

[[entities]]
id = 1
name = "player"
position = { x = 0, y = 0 }
velocity = { x = 1, y = 0 }
collider = { half_width = 1, half_height = 1, mass_milli = 1000, restitution_milli = 1000, is_static = false }
```

Les sections `[render]`, `[render.camera]`, le champ `appearance` et le champ `collider` des entités sont optionnels : les anciens projets gardent les valeurs par défaut (160×120, caméra centrée, rectangles verts 2×2) et restent sans simulation physique active. Un collider déclare des demi-tailles positives, une masse en milli-unités, une restitution de `0` à `1000` et un éventuel `is_static`. Le format PPM P6 est volontairement retenu pour éviter une dépendance d'encodage ; le manifeste adjacent porte le suffixe `.ppm.json`.

Les coordonnées sont des entiers signés : ce choix évite les divergences usuelles des flottants. SplitMix64 utilise exclusivement `wrapping_add`, XOR, décalages et multiplications modulo 2^64 (constantes `9e3779b97f4a7c15`, `bf58476d1ce4e5b9`, `94d049bb133111eb`) ; son état est l'unique mot `u64`. La graine initialise cet état. Les scènes historiques n'appellent pas le PRNG par défaut, donc leurs snapshots, captures et checksums restent inchangés.

## Scènes et captures 3D

Le schéma strict `aetherion.scene3d/v1` accepte toujours les `triangles` historiques et ajoute des ressources réutilisables `meshes`, `materials` et `objects`. Les transformations entières sont appliquées dans l'ordre **échelle → rotation X → rotation Y → rotation Z → translation**. Une échelle de `1000` vaut 1 ; les rotations sont exprimées en millidegrés et limitées aux multiples de `90000`. L'opacité matériau est un entier de `0` à `1000`, composé de façon déterministe avec le fond.

Limites actuelles : scène 1 MiB, 10 000 meshes, 10 000 matériaux, 100 000 objets, 100 000 triangles développés, 300 000 sommets et 16 777 216 pixels. Les identifiants et références sont validés strictement, les indices hors limites sont rejetés et le rendu suit un ordre canonique. `capture3d` publie atomiquement un PPM et un manifeste adjacent ; une validation échouée retourne le code 2 sans sortie partielle.

Le prototype temps réel s'utilise avec `cargo run --features render-gpu -- gpu-demo --scene FILE [--assets FILE] [--frames N]`. Avec `--frames`, la fenêtre se ferme automatiquement et publie le rapport `aetherion.gpu-demo/v1`, ce qui permet un smoke test borné ; sans cette option, elle reste ouverte jusqu'à fermeture utilisateur. `gpu-benchmark --scene FILE --frames N` publie séparément `aetherion.gpu-benchmark/v1` avec le temps mural, le FPS moyen en milli-FPS, l'adaptateur utilisé, le nombre d'objets visibles/rejetés et les draw calls. Le backend applique un culling AABB orthographique aux objets avant upload GPU. Ces valeurs servent au suivi de performance uniquement : elles ne sont ni des sorties déterministes ni des entrées de simulation. Il partage la validation et la résolution d'assets de `capture3d`, mais les captures reproductibles restent produites par le chemin CPU.

Les meshes peuvent désormais conserver des `normals` optionnelles quantifiées à `1_000_000` et des `uvs` optionnelles quantifiées à `1_000_000`. L'absence de ces tableaux reste valide pour les scènes historiques ; le CPU conserve son rendu inchangé et le GPU utilise alors les normales plates calculées par face.

Les matériaux acceptent `base_color_texture`, un ID d'asset externe. Les manifests `aetherion.assets3d/v1` acceptent les entrées `texture` dont les octets sont PNG ou JPEG, avec les mêmes contrôles de chemin, taille et checksum. `capture3d` charge et valide ces assets mais garde volontairement la couleur plate ; `gpu-demo` et `gpu-benchmark` décodent les textures référencées, bornent leurs dimensions à 4096 et les dessinent par lots de matériau avec sampler filtrant.

Les commandes `capture3d`, `gpu-demo` et `gpu-benchmark` acceptent `--cache-dir DIR`. Le cache est hors du format de scène, indexé par type d'asset, checksum, version de format et version d'importeur. Le fichier source est toujours relu et vérifié avant utilisation ; un cache absent, corrompu ou non inscriptible est ignoré sans modifier le résultat. Il s'agit d'une optimisation de présentation, jamais d'une entrée de simulation.

`gltf-import` convertit un fichier glTF ou GLB vers `aetherion.scene3d/v1`. Les positions sont quantifiées au millième, les normales sont transformées par l'inverse-transposée puis quantifiées à `1_000_000`, les UV du canal 0 sont quantifiés à `1_000_000`, les transformations de nœuds sont composées puis cuites dans les vertices, les primitives non triangulées sont refusées et les matériaux utilisent leur couleur PBR de base. Les textures glTF, skins et animations ne sont pas encore importés. La sortie est canonique et publiée atomiquement ; la commande nécessite `--features gltf-import`.

### Ressources 3D externes M4-F

`capture3d --assets FILE` charge le manifeste strict `aetherion.assets3d/v1`. Chaque entrée contient `id`, `path`, `type` (`mesh`, `material` ou `texture`), `size` et `checksum` FNV-1a 64 bits. Les fichiers `mesh` et `material` sont des documents JSON stricts `aetherion.mesh3d/v1` et `aetherion.material3d/v1`; une texture est un fichier PNG/JPEG binaire. Les chemins sont relatifs au manifeste et confinés à son dossier ; traversals, liens sortants, tailles/checksums incorrects, quotas et doublons sont rejetés. Le chargement est concurrent, puis collecté dans une `BTreeMap` et fusionné en ordre canonique. Une collision avec une ressource inline est une erreur ; les objets existants peuvent référencer les IDs externes. Les textures sont validées mais ignorées par le renderer CPU et consommées uniquement par `render-gpu`.

`asset3d-import --input FILE --type mesh|material --output FILE` valide le schéma strict et republie atomiquement un JSON canonique. La sortie doit ne pas exister.

### Comparaison intégrée 3D M4-H

Syntaxe : `aetherion visual-diff3d --baseline-manifest FILE --candidate-manifest FILE --report FILE [--color-max-channel-delta N] [--color-max-different-pixels N] [--color-max-different-percent-milli N] [--depth-max-channel-delta N] [--depth-max-different-pixels N] [--depth-max-different-percent-milli N] [--normals-max-channel-delta N] [--normals-max-different-pixels N] [--normals-max-different-percent-milli N] [--segmentation-max-different-pixels N]`.

La commande lit deux manifestes `aetherion.capture3d/v1`, retrouve la couleur adjacente et compare tous les canaux déclarés. Les captures M4-G utilisent réellement PPM P6 RGB8 pour la couleur, les normales (`*.normals.ppm`) et la segmentation (`*.segmentation.ppm`), et PGM P5 u16 big-endian pour la profondeur (`*.depth.pgm`) ; il ne s'agit pas de PNG. Les normales codent le fond en noir et les surfaces en `[128,128,255]`. La segmentation code `triangle_id + 1` en RGB 24 bits big-endian, zéro étant le fond ; le rapport résume chaque paire d'IDs différente avec son nombre de pixels et, si disponible, les mappings `triangle_id`, `source` et `rank`.

Toutes les tolérances valent zéro par défaut. Couleur, profondeur et normales disposent chacune des trois seuils entiers de `visual-diff`; la segmentation accepte un nombre maximal de pixels différents. `--report` est obligatoire et remplace atomiquement le rapport par le même JSON déterministe que stdout. Codes : `0` tous les canaux sont dans les tolérances, `1` au moins un canal les dépasse, `2` usage, manifeste/canal manquant, image invalide ou formats/dimensions incompatibles.

```console
aetherion visual-diff3d --baseline-manifest baseline.ppm.json --candidate-manifest capture.ppm.json --report diff3d.json
aetherion visual-diff3d --baseline-manifest baseline.ppm.json --candidate-manifest capture.ppm.json --color-max-channel-delta 2 --depth-max-channel-delta 4 --normals-max-different-pixels 2 --segmentation-max-different-pixels 1 --report diff3d.json
```

## Certification M4

La commande `cargo run -- certify-m4 --report docs/m4-certification.json` exécute une matrice autonome et bornée couvrant le défaut historique de capture, les canaux 2D, le diff 2D, les assets externes 3D, les canaux 3D et le diff 3D. Elle publie atomiquement le rapport versionné `aetherion.m4-certification/v1`, également émis sur stdout. Les preuves sont des checksums FNV-1a stables, sans chemin temporaire ni temps mural. Le code est `0` si toutes les vérifications passent et `2` pour une erreur de validation ou de publication ; les codes historiques `1` et `3` restent réservés aux verdicts de diff/assertion et divergences/budgets.

Le rapport de référence est [`docs/m4-certification.json`](docs/m4-certification.json) et son schéma est exposé par `schema show m4-certification`.

## Développement

```console
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo check --features plugin-runtime
cargo test --features plugin-runtime plugin_runtime::tests
```

### Note sur la toolchain Windows testée

Le poste utilisé possède Rust GNU et MSVC, mais ni `dlltool.exe` ni les Build Tools C++ de Visual Studio ; son `link.exe` dans le `PATH` appartient à Git et n'est pas le linker MSVC. Les commandes ont donc été validées avec `rust-lld.exe` livré par Rust GNU :

```bat
set "RUSTFLAGS=-C linker=%USERPROFILE%\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\rust-lld.exe"
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Sur une installation Rust standard disposant de MinGW (`dlltool`) ou des Build Tools C++ MSVC, les commandes simples suffisent.

Voir [`docs/AGENT_PROTOCOL.md`](docs/AGENT_PROTOCOL.md), [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) et [`docs/ROADMAP.md`](docs/ROADMAP.md).


## Canaux de capture M4-C

`capture` et `capture-multi` acceptent `--channels color,depth,normals,segmentation`. `color` est obligatoire; les canaux inconnus, vides ou dupliqués sont rejetés. Sans option, seule la couleur est produite et ses octets restent identiques aux versions précédentes.

Pour `capture --output capture.png`, les fichiers auxiliaires adjacents sont `capture.depth.pgm` (PGM P5, 16 bits non signés big-endian), `capture.normals.png` (RGB8, normale constante `[128,128,255]`) et `capture.segmentation.png` (RGB8 contenant l'identifiant 24 bits big-endian; zéro est le fond). Le manifeste optionnel `channels` donne nom, fichier, encodage, checksum et dimensions; `segmentation_mapping` associe chaque valeur non nulle à l'EntityId et au nom. La profondeur est un rang entier déterministe suivant l'ordre `(z, EntityId)`. Les texels alpha zéro ne modifient aucun canal auxiliaire.

Chaque vue JSON peut aussi définir `"channels":"color,depth,normals,segmentation"`; sinon elle hérite de l'option CLI. La capture simple publie l'ensemble couleur/auxiliaires/manifeste via staging avec rollback, et la capture multi-vues publie son dossier complet par renommage atomique.


## Comparaison visuelle M4-D

Syntaxe : `aetherion visual-diff --baseline FILE --candidate FILE [--max-channel-delta N] [--max-different-pixels N] [--max-different-percent-milli N] [--report FILE]`.

La comparaison accepte PPM P6 RGB8, PGM P5 profondeur u16 big-endian et le sous-ensemble PNG RGB8 déterministe produit par Aetherion (deflate stocké, filtre 0). Utilisez la commande une fois par canal : image couleur, `.depth.pgm`, `.normals.png` ou `.segmentation.png`. Les dimensions, le nombre de canaux et la profondeur de bits doivent correspondre.

Toutes les tolérances sont entières et valent zéro par défaut. Un pixel diffère si au moins un échantillon dépasse `max_channel_delta`. Le verdict passe seulement si `different_pixels <= max_different_pixels` **et** si le pourcentage est inférieur ou égal à `max_different_percent_milli`, exprimé en millièmes de pourcent (`1000` = 1 %, `100000` = 100 %). Aucun flottant n'est utilisé.

Le rapport JSON `aetherion.visual-diff/v1` contient les chemins normalisés, le format, les dimensions, les tolérances, les compteurs, le delta maximal, la somme d'erreur, les moyennes/pourcentages rationnels et au plus 100 premières différences en ordre ligne-colonne. `--report` écrit atomiquement le même JSON que stdout. Codes : `0` dans les tolérances, `1` différence hors tolérances, `2` usage, image invalide ou formats incompatibles.

```console
aetherion visual-diff --baseline baseline.png --candidate color.png
aetherion visual-diff --baseline baseline.depth.pgm --candidate capture.depth.pgm --max-channel-delta 4
aetherion visual-diff --baseline baseline.normals.png --candidate capture.normals.png --max-different-pixels 2 --max-different-percent-milli 10
aetherion visual-diff --baseline baseline.segmentation.png --candidate capture.segmentation.png --report segmentation-diff.json
```
