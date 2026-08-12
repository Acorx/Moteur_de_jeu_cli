# Benchmark GPU M11.4

Ce dossier décrit les paliers de charge du renderer GPU sans les confondre avec
les sorties déterministes de la simulation. Les scènes sont générées dans un
répertoire de travail par `generate_scenes.py` afin d'éviter de versionner des
fichiers JSON multi-mégaoctets.

## Générer les paliers

```console
python benchmarks/gpu/generate_scenes.py --output-dir .tmp/aetherion-gpu-bench
```

Le générateur publie trois dossiers :

- `triangles-1k` : 1 000 triangles ;
- `triangles-10k` : 10 000 triangles ;
- `triangles-100k` : 100 000 triangles.

Chaque dossier contient `scene.json`, `assets.json`, `mesh.json` et
`material.json`. Le mesh est une grille de triangles indépendants afin de
mesurer une charge connue sans dépendre d'un importeur externe. Le manifeste
calcule le checksum FNV-1a attendu par `assets3d`.

## Mesurer

```console
cargo run --release --features render-gpu -- gpu-benchmark \
  --scene .tmp/aetherion-gpu-bench/triangles-1k/scene.json \
  --assets .tmp/aetherion-gpu-bench/triangles-1k/assets.json \
  --width 1280 --height 720 --frames 240
```

Le rapport publié sur stdout suit `aetherion.gpu-benchmark/v1` :

- `frames_rendered`, `triangles`, `width`, `height` : charge effectivement
  présentée ;
- `elapsed_ms` : temps mural de la boucle de fenêtre ;
- `fps_milli` : FPS moyen en milli-FPS, calculé à partir des frames présentées ;
- `adapter` : nom de l'adaptateur wgpu sélectionné.

Ces mesures dépendent du système, du pilote, du backend et de la politique de
présentation. Elles ne doivent jamais être utilisées dans les checksums,
replays, scénarios, lockstep ou captures CPU. Pour comparer deux machines,
conserver la version du commit, le palier, la résolution, le nombre de frames,
le backend graphique et le rapport complet.
