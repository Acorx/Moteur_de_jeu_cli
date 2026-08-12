# ADR-0001 — Frontière simulation/rendu GPU

- **Statut** : accepté
- **Date** : 2026-08-12
- **Portée** : M11, premier pipeline 3D temps réel

## Contexte

Aetherion possède déjà une simulation entière déterministe et un renderer 3D
CPU/offline utilisé pour les captures, les tests et les visual diffs. Le moteur
doit maintenant afficher une scène 3D dans une fenêtre sans rendre le GPU
responsable de la reproductibilité de la simulation.

La reproductibilité bit-à-bit d'un pipeline GPU n'est pas une hypothèse
portable : les pilotes, formats, implémentations de profondeur et unités de
calcul varient selon le matériel. La simulation ne doit donc jamais dépendre
d'un résultat de rendu.

## Décision

1. Le chemin déterministe reste headless et conserve le renderer CPU actuel.
2. Le chemin temps réel est optionnel, derrière la feature Cargo
   `render-gpu`.
3. `wgpu 0.19` fournit l'abstraction GPU cross-platform et `winit 0.29`
   fournit la fenêtre et la boucle d'événements.
4. `glam` est limité aux mathématiques de présentation (`f32`). Les types
   entiers de `Scene3d` restent la source canonique des données de scène.
5. Le renderer GPU reçoit une scène validée et immuable. Il peut construire
   des buffers et des matrices de caméra, mais ne peut pas modifier un
   `World` ni écrire dans la simulation.
6. `gpu-demo` réutilise `Scene3d`, `render3d::expanded_triangles` et le
   manifeste d'assets existant. Le mode actuel est orthographique, avec
   depth buffer `Depth24Plus`, pipeline de triangles colorés et cadence
   `Fifo` lorsqu'elle est disponible.
7. L'absence de feature GPU produit une erreur explicite et ne tire aucune
   dépendance graphique dans le build headless.

## Conséquences

### Positives

- Le premier vertical slice GPU est testable sans migration des formats.
- Les captures CPU, replays, checksums et scénarios historiques restent
  indépendants du matériel graphique.
- Les assets inline et externes disposent d'un seul chemin de validation.
- Le futur pipeline glTF, matériaux, animation et culling peut être ajouté
  derrière des interfaces de rendu sans modifier la simulation.

### Contraintes

- `gpu-demo` n'est pas une capture déterministe et ne doit pas être utilisé
  comme oracle de visual diff.
- La présence d'un adaptateur GPU compatible est une précondition d'exécution.
- Le mode GPU ne couvre pas encore les textures, l'éclairage, l'animation
  temps réel ni la physique 3D.
- Les versions de `wgpu`, `winit` et du shader font partie de la surface de
  compatibilité de la feature et devront être verrouillées par CI.

## Rejet d'alternatives

- **Remplacer le renderer CPU par le GPU** : rejeté, car cela détruirait les
  propriétés headless et la certification visuelle existantes.
- **Faire entrer des flottants dans la simulation** : rejeté, car cela
  compromettrait lockstep, replay et futures validations réseau.
- **Rendre `wgpu` obligatoire** : rejeté, car les agents et CI doivent pouvoir
  compiler et exécuter Aetherion sans fenêtre ni matériel GPU.
