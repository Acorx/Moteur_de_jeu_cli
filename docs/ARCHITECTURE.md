# Architecture d'Aetherion

## Principes

Aetherion est un moteur CLI headless dont les sorties observables sont reproductibles. La simulation, les transformations, le rendu logiciel, les tolerances et les checksums utilisent des entiers, des ordres canoniques et des formats versionnes. Aucun flottant n'entre dans le chemin deterministe.

## Runtime WebAssembly M5-C0

Le runtime `plugin_runtime` est compile uniquement avec la feature Cargo `plugin-runtime` et repose sur l'interpreteur `wasmi 0.32.3`. Le build par defaut ne depend donc pas de WebAssembly. C0 instancie les modules avec un `Linker` vide, sans import hote ni WASI, puis appelle un export `aetherion_main` de signature `() -> i32`. Les erreurs de lecture, compilation, instanciation, demarrage, export et trap sont converties en prefixes stables `plugin_runtime_*`.

Cette couche ne lit encore aucun manifeste ni capacite. C1 applique deja `fuel` via `Config::consume_fuel`/`Store::set_fuel` et la memoire via `StoreLimitsBuilder`, avec classification deterministe des depassements. Les quotas sont encore fournis directement par `RuntimeLimits`; leur projection depuis `PluginManifest` et l'API hote arrive en C2. IO, reseau, CLI, rapports et provenance sont ensuite bornes par C3-C6.

## Audit de provenance M5-C6

`plugin_audit` reste derriere `plugin-runtime` et compose les validations du manifeste et du runtime sans instancier le module. Il calcule les checksums FNV-1a des octets exacts du manifeste et du module, puis publie `aetherion.plugin-audit/v1` avec l'ABI, les capacités, les quotas, `wasmi 0.32.3` et l'interdiction explicite du réseau/WASI. La publication est atomique et ne contient aucun chemin local. Le statut `verified` signifie que la structure, l'export et les imports sont vérifiés; `signatures.status` reste `not_implemented` tant que la confiance cryptographique n'est pas implémentée.

## Fondations du module de physique 2D M7-A

Le format projet et les scenes acceptent un `collider` optionnel par entite. Il declare des demi-tailles entieres strictement positives, une masse en milli-unites, une restitution comprise entre `0` et `1000`, et `is_static`. L'absence de collider preserve les projets existants.

L'ECS stocke les colliders dans une `BTreeMap<EntityId, Collider>`, independamment des positions et velocites. L'ordonnanceur expose l'ordre canonique `input`, `movement`, `physics` et rejette les ordres incomplets, dupliques ou incompatibles. Le systeme `physics` visite les colliders dans l'ordre des `EntityId`, detecte les paires AABB par force brute, choisit l'axe de penetration minimale (X en cas d'egalite), separe les corps avec des corrections entieres et applique une reponse de vitesse a restitution milli-unitaire. Les corps statiques ne bougent jamais.

Les colliders sont exposes dans les snapshots lorsqu'ils existent. La telemetrie ajoute `collisions_resolved` et `entities_modified` pour `physics`. Les checksums des projets historiques sans collider restent inchanges, car le champ optionnel est absent de leur serialisation.

## Module d'animation 3D

Le module `render3d` etend `aetherion.scene3d/v1` avec des clips optionnels. Un clip possede un identifiant, une duree entiere en ticks, un mode boucle ou non boucle et des pistes ciblant chacune un objet 3D. Les keyframes sont strictement ordonnees et portent un transform entier : echelle au millieme, rotations en millidegres limitees aux quarts de tour et translation entiere.

La commande headless selectionne une pose avec `aetherion capture3d --scene scene.json --animation <id> --ticks <n> --output capture.ppm`. Le manifeste `aetherion.capture3d/v1` ajoute l'identifiant et le tick demandes; sans animation, ces champs sont omis pour preserver le comportement historique.

### Echantillonnage en escalier

Pour chaque piste, la derniere keyframe dont le tick est inferieur ou egal au tick echantillonne fournit la pose complete. Il n'existe aucune interpolation. Ce choix rend le resultat exact, inspectable et compatible avec les rotations discretes.

### Boucle et maintien final

Pour un clip boucle, le tick effectif vaut `tick % duration_ticks`. Pour un clip non boucle, il est borne a `duration_ticks`; la derniere pose reste maintenue pour tous les ticks ulterieurs.

### Validation et determinisme

Les identifiants, references, doublons, durees, ordre des keyframes, rotations et quotas sont valides avant rendu. Une animation inconnue ou des arguments invalides retournent le code 2. La publication du PPM et du manifeste est atomique, sans sortie partielle.

Toutes les operations restent entieres : modulo et bornage des ticks, transformations, projection orthographique, z-buffer et opacite. L'absence de flottants evite les divergences d'arrondi entre plateformes.

## Ressources 3D externes M4-F

Le module dedie `assets3d` lit le manifeste strict `aetherion.assets3d/v1`. Les entrees mesh/material declarent identifiant, chemin relatif, type, taille et checksum FNV-1a 64 bits. Les documents stricts `aetherion.mesh3d/v1` et `aetherion.material3d/v1` enveloppent les types publics de `render3d`.

Les chemins sont confines au dossier canonique du manifeste, les quotas sont bornes et taille/checksum sont verifies avant decodage. Les fichiers sont charges par threads, puis collectes dans une `BTreeMap`; la fusion avec les ressources inline est donc canonique et rejette toute collision. `capture3d --assets FILE` resout ces ressources avant animation et rendu, sans modifier le comportement historique en l'absence de l'option. `asset3d-import` valide et republie atomiquement le JSON canonique.

## Canaux de capture 3D M4-G

`capture3d --channels color,depth,normals,segmentation` produit trois fichiers auxiliaires adjacents au PPM couleur. La profondeur est un PGM P5 u16 big-endian (`*.depth.pgm`) : le fond vaut 65535 et une surface contient sa profondeur entiere bornee a 0..65535. Les normales sont un PPM P6 RGB8 (`*.normals.ppm`) : le fond vaut noir et les surfaces utilisent la convention orthographique deterministe `[128,128,255]`. La segmentation est un PPM P6 RGB8 (`*.segmentation.ppm`) contenant `triangle_id + 1` sur 24 bits big-endian, zero etant reserve au fond.

Le manifeste liste les canaux dans l'ordre fixe profondeur, normales, segmentation et fournit une table de segmentation triee par identifiant de triangle. La couleur seule conserve le manifeste historique sans champs optionnels. Couleur, manifeste et auxiliaires sont ecrits dans un staging unique puis publies comme un lot avec rollback en cas d'echec, afin qu'aucune capture partielle ne reste visible.

## Comparaison visuelle 3D M4-H

Le module `visual_diff3d` orchestre `visual_diff` a partir de deux manifestes `aetherion.capture3d/v1`. Le flux est : validation stricte des manifestes et dimensions, resolution de la couleur adjacente et des fichiers declares, verification de la presence et de l'encodage de chaque canal, comparaison en ordre canonique par nom, puis construction du rapport `aetherion.visual-diff3d/v1`.

Couleur, normales et segmentation sont decodees comme PPM P6 RGB8; la profondeur comme PGM P5 u16 big-endian. Les tolerances couleur/profondeur/normales sont independantes. La segmentation est comparee pixel par pixel et ses differences sont agregees dans une `BTreeMap` par paire `(baseline_id, candidate_id)`, puis enrichies avec les mappings de primitives (`triangle_id`, `source`, `rank`). Le rapport est serialise de facon deterministe et publie atomiquement. Un depassement retourne 1 avec le JSON sur stdout; un manifeste/canal absent ou incompatible retourne 2.

## Pipeline GPU temps reel M11

Le pipeline temps reel est optionnel et n'est pas une extension du chemin deterministe. La feature Cargo `render-gpu` active `wgpu 0.19`, `winit 0.29`, `glam` et les utilitaires de buffers. Le build par defaut reste sans dependance GPU et `gpu-demo` retourne `render_gpu_feature_disabled` lorsqu'il est compile sans cette feature.

`gpu-demo` charge une `Scene3d` validee, resout optionnellement le meme manifeste `--assets` que `capture3d`, puis construit un snapshot de presentation immuable. Les triangles sont convertis explicitement des coordonnees entieres vers des sommets `f32`; aucune donnee GPU ne revient vers `World` ou la simulation. La camera conserve actuellement la semantique orthographique `pixels_per_unit` du format historique.

Le backend initialise une surface, choisit un format sRGB et le mode FIFO lorsque le pilote le propose, puis rend un pipeline de triangles colores avec depth buffer `Depth24Plus`. Les sommets portent maintenant une normale de face derivee de la geometrie entiere avant conversion; le fragment shader applique un eclairage directionnel fixe avec une composante ambiante, sans pretendre a une illumination deterministe cross-device. Les erreurs de surface sont classees (`Lost`, `Outdated`, `Timeout`, `OutOfMemory`) et la boucle redimensionne la surface sans interrompre la simulation. `--frames N` borne la boucle a un million de frames maximum et publie `aetherion.gpu-demo/v1` avec dimensions, nombre de frames effectivement presentees et triangles. Ce rendu n'est pas une preuve de determinisme et ne remplace pas les captures CPU.

La decision complete est documentee dans [`docs/adr/0001-frontiere-rendu-gpu.md`](adr/0001-frontiere-rendu-gpu.md). Les prochaines extensions doivent conserver cette frontiere avant d'ajouter textures, glTF, animation GPU, culling ou physique de presentation.

## Import glTF M12.1

`gltf3d` est active par la feature optionnelle `gltf-import`; `render-gpu` l'active automatiquement. `gltf-import --input FILE --output FILE` lit les fichiers glTF et GLB via le loader glTF, selectionne la scene par defaut (ou la premiere scene), parcourt les noeuds dans l'ordre du document et convertit chaque primitive `TRIANGLES` en `Mesh3d`/`Object3d` du format `aetherion.scene3d/v1`.

Les matrices locales sont composees avec les matrices parentes puis cuites dans les sommets. Les positions flottantes sont quantifiees au millieme avec un arrondi symetrique et controle de finitude/overflow. Les facteurs de couleur PBR deviennent des `Material3d` RGB8/opacity milli-unitaire. Les IDs produits sont derives des index glTF, donc stables independamment des noms optionnels. Les textures, skins, morph targets et animations sont explicitement hors de cette tranche et les modes primitifs autres que triangles sont refuses.

Le fichier principal est limite a 16 MiB, les buffers charges a 64 MiB et le nombre d'images declarees a 4096. L'importeur utilise `Gltf::open` et `import_buffers` directement : les images ne sont jamais decodees ni lues tant que les textures sont hors contrat. La sortie JSON est validee par le meme contrat que `capture3d`, serialisee avec `serde_json::to_vec_pretty`, terminee par LF et publiee atomiquement sans ecraser une sortie existante. Le rapport `aetherion.gltf-import/v1` expose uniquement des compteurs et la version d'echelle, sans chemin local.
