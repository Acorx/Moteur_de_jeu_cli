# Affichage et captures — M3

## Compilation

Le moteur reste headless par défaut : `cargo build --release --no-default-features`. La feature `display` n'ajoute aucune crate et utilise directement Win32/GDI sous Windows : `cargo build --release --features display`.

Sans la feature, `play` reste visible dans l'aide mais retourne une erreur explicite. Le cœur, le rasteriseur, PNG et les captures multi-vues restent disponibles sans affichage.

## Boucle et déterminisme

`play --path DIR [--max-ticks N]` conserve le tick fixe du monde. Le rafraîchissement de fenêtre est indépendant et la caméra n'appartient ni au snapshot ni au checksum. `--max-ticks` ferme automatiquement la boucle et facilite un smoke test manuel borné. Aucun test automatisé n'ouvre de fenêtre.

Touches : flèches (caméra), `+`/`-` (zoom borné 1–256), `R` (recentrer), espace (pause), `N` (un pas en pause), Échap (quitter). Ces commandes sont visuelles et ne mutent pas le monde. Toute future commande de gameplay devra être convertie en `InputEvent {tick, sequence, ...}` avant le système `input`.

## PNG

`capture --format png --output image.png` utilise un encodeur interne minimal : RGB 8 bits, filtre 0, DEFLATE non compressé, chunks IHDR/IDAT/IEND, CRC et Adler-32. Il n'écrit aucune métadonnée temporelle ; même image, mêmes octets. PPM P6 reste le format par défaut et les manifestes v1 restent compatibles.

## Multi-vues

`capture-multi --path DIR --views demo/views.json --output-dir target/views [--ticks N]` valide schéma, noms ASCII sûrs, collisions insensibles à la casse, dimensions, zoom et maximum de 64 vues. Le lot est entièrement rendu dans un dossier de staging puis publié par renommage. Le manifeste `aetherion.capture-multi/v1` trie les vues par nom et contient checksum monde/image, caméra, dimensions, entités visibles et chemins relatifs.

Le protocole agent accepte `capture.create` avec `format: "ppm"|"png"` et `capture.multi` avec `views`, `output_dir`, `dry_run` et `expected_revision`. Quotas, capacités et confinement M2 s'appliquent.

## Limite de validation

Le build et la logique Win32 peuvent être validés sans interaction, mais l'ouverture réelle dépend d'une session Windows graphique. En CI sans bureau, tester caméra/rendu séparément et ne pas lancer `play`.
