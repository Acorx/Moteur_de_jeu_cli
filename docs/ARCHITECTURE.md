# Architecture d'Aetherion

## Principes

Aetherion est un moteur CLI headless dont les sorties observables sont reproductibles. La simulation, les transformations, le rendu logiciel, les tolerances et les checksums utilisent des entiers, des ordres canoniques et des formats versionnes. Aucun flottant n'entre dans le chemin deterministe.

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
