# Bundles déterministes

`bundle --path DIR --output FILE` crée une archive ZIP stockée (sans compression) de façon déterministe: fichiers triés par chemin, dates et champs variables absents. La première entrée est `aetherion.bundle.json`, un manifeste `aetherion.bundle/v1` avec tailles et checksums FNV-1a. L'écriture de l'archive est atomique.

`bundle-inspect --input FILE` lit le central directory et affiche en JSON les entrées, leurs tailles et checksums.