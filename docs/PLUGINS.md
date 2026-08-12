# Plugins M5 — runtime WebAssembly sécurisé

## État présent

Aetherion accepte des **manifestes de plugins** et fournit un runtime WebAssembly optionnel derrière `plugin-runtime`. Les commandes `plugin run` et `plugin audit` sont disponibles avec cette feature; les rapports et les schémas restent versionnés et déterministes.

Le format strict `aetherion.plugin/v1` est publié par `aetherion schema show plugin`. Un manifeste est limité à 64 Kio. Un catalogue contient au plus 256 fichiers réguliers nommés `*.plugin.json`; les liens symboliques sont refusés. Le résultat est trié par identifiant, indépendamment de l'ordre du système de fichiers.

## M5-C0 — Runtime WebAssembly optionnel

La feature Cargo `plugin-runtime` active `wasmi 0.32.3` sans modifier le build par défaut :

```console
cargo check
cargo test --features plugin-runtime plugin_runtime::tests
```

Le module `src/plugin_runtime.rs` charge un binaire WebAssembly, l'instancie avec un linker vide — aucun import hôte ni WASI — puis appelle l'export `aetherion_main` de signature `() -> i32`. Cette phase ne branche encore aucun manifeste ni capacité. C1 applique toutefois les quotas `RuntimeLimits { fuel, memory_bytes }` : `Config::consume_fuel(true)` et `Store::set_fuel` bornent l'exécution, tandis que `StoreLimitsBuilder` limite chaque mémoire linéaire et force un trap lors d'une croissance refusée. Les dépassements produisent `plugin_runtime_fuel_exhausted` ou `plugin_runtime_memory_limit`.

La consommation de fuel est publiée dans `ExecutionResult` et deux exécutions du même module avec les mêmes limites donnent le même résultat. C2 ajoute `execute_bytes_with_manifest` et `execute_file_with_manifest` : les quotas `fuel` et `memory_bytes` sont alors lus depuis le manifeste validé.

## M5-C2 — API hôte versionnée et capacités explicites

Le module d'import Wasm est strictement `aetherion_v1`. Il est versionné indépendamment de l'export d'entrée `aetherion_main: () -> i32`. Un import est accepté uniquement si son nom est connu et si la capacité correspondante est déclarée dans le manifeste. Les imports WASI, sockets, réseau, HTTP, TCP, UDP ou DNS sont refusés avant l'instanciation avec `plugin_runtime_network_denied`; un autre module système ou inconnu produit `plugin_runtime_import_denied`. Une capacité absente produit `plugin_runtime_capability_denied`.

Imports actuellement publiés :

| Capacité | Imports | Contrat |
| --- | --- | --- |
| `simulation_read` | `simulation_tick`, `simulation_checksum`, `simulation_entity_count`, `simulation_entity_field(index, field)` | lecture d'une copie canonique du monde; champs `0=id`, `1..4=position X/Y et vélocité X/Y` |
| `scene_read` | `scene_entity_count`, `scene_asset_count` | lecture des cardinalités de la scène validée |
| `asset_read` | `asset_count`, `asset_size(index)`, `asset_read_byte(index, offset)` | lecture par index d'assets explicitement sélectionnés; aucun chemin n'est transmis au plugin |
| `telemetry_write` | `telemetry_write(key, value)`, `telemetry_len` | ajout dans un tampon mémoire borné à 1024 enregistrements; aucune écriture disque |

Les fonctions de lecture renvoient `-1` pour une vue absente ou un index/champ invalide. Les entités et assets sont triés canoniquement. `HostContext::from_world` copie les données nécessaires; `with_scene` revalide la scène; `with_assets_from_manager` réutilise le confinement, la taille et le checksum du gestionnaire d'assets. Ces choix empêchent toute mutation du `World` et toute traversée de chemin depuis Wasm. Le rapport Rust `ExecutionReport` sépare le résultat d'exécution du tampon de télémétrie.

Le déterminisme est testé par les mêmes modules, manifestes et contextes; aucune horloge, thread, réseau ou accès implicite au système n'est exposé.

## M5-C3 — Quotas IO et fichiers

Les quotas `io_read_bytes`, `io_write_bytes` et `files` sont lus depuis le manifeste validé par `execute_bytes_with_manifest` et `execute_file_with_manifest`.

- chaque appel valide à `asset_read_byte` consomme un octet de `io_read_bytes`; une répétition est comptée à nouveau ;
- les assets copiés dans `HostContext` sont triés et comptés comme fichiers sélectionnés avant l'instanciation; un dépassement produit `plugin_runtime_files_quota` ;
- un dépassement de lecture interrompt immédiatement l'appel Wasm avec `plugin_runtime_io_read_quota` ;
- `ExecutionReport.io` publie `read_bytes`, `write_bytes` et `files` de manière déterministe ;
- `write_bytes` reste toujours nul : aucune capacité ni aucun import d'écriture n'existe en C3, même si le manifeste réserve un quota d'écriture ;
- le chargement du module Wasm par l'hôte et la préparation préalable des assets ne sont pas des imports plugin et ne sont pas débités du compteur d'appels hôte ;
- aucun chemin arbitraire, répertoire ou descripteur n'est transmis au plugin.

Les limites maximales restent celles du manifeste : lecture/écriture `0..64 MiB` et `0..1024` fichiers. Les erreurs de quota sont déterministes et testées avec le même module, manifeste et contexte.

## M5-C4 — Interdiction réseau

Le runtime n'enregistre aucun linker réseau et ne fournit aucune capacité réseau dans `Capability`. Avant toute instanciation, chaque import est inspecté; les marqueurs de modules ou fonctions `wasi`, `socket`, `network`, `tcp`, `udp`, `http`, `https` et `dns` sont refusés avec `plugin_runtime_network_denied`. Cette règle couvre notamment `wasi_snapshot_preview1`, `wasi:io`, `env/socket_open` et les modules réseau explicites.

L'interdiction est structurelle : elle ne dépend pas d'un quota, d'un chemin de fichier, du contenu du manifeste ou de l'absence d'appel effectif. Une extension réseau nécessiterait un nouveau contrat d'ABI et une nouvelle capacité versionnée; elle n'est pas implicite dans M5.

## M5-C5 — Commande `plugin run`

Avec `--features plugin-runtime`, un plugin peut être lancé depuis la CLI :

```console
aetherion plugin run \
  --manifest plugins/example.plugin.json \
  --module plugins/example.wasm \
  --report plugin-report.json
```

Options :

- `--manifest FILE` et `--module FILE` sont obligatoires ;
- `--export NAME` sélectionne l'export, `aetherion_main` par défaut ;
- `--path DIR` expose une copie de lecture du projet ;
- `--scene ID` expose une scène validée et nécessite `--path` ;
- `--assets FILE` sélectionne un manifeste d'assets confiné ;
- `--dry-run` valide le manifeste, compile le module, vérifie l’export/imports et les références sans instancier ni appeler le plugin ;
- `--report FILE` publie `aetherion.plugin-run-report/v1` atomiquement.

Le rapport ne contient aucun chemin local. En mode dry-run, son statut est `planned`; en exécution, il est `executed` et contient le code retour, le fuel, les compteurs IO et la télémétrie. Sans la feature Cargo, la commande est reconnue mais retourne `plugin_runtime_feature_disabled` sans charger de module.

## M5-C6 — Audit de provenance et corpus de frontières

`plugin audit --manifest FILE --module FILE` valide le manifeste, lit le module sans l'exécuter, vérifie l'export demandé et réutilise le contrôle des imports/capacités du runtime. La commande publie `aetherion.plugin-audit/v1` avec :

- le checksum FNV-1a du manifeste et du module Wasm exacts, sans chemin local ;
- l'identité/version du plugin, l'ABI hôte `1.1`, les capacités triées et les quotas ;
- le moteur `wasmi 0.32.3`, avec `network: false` et `wasi: false` ;
- `signatures.status: "not_implemented"`, contrat explicite avant l'ajout des signatures cryptographiques.

```console
aetherion plugin audit --manifest plugins/example.plugin.json --module plugins/example.wasm --report plugin-audit.json
aetherion plugin audit --manifest plugins/example.plugin.json --module plugins/example.wasm --export custom_entry --report plugin-audit.json
```

L'audit ne modifie pas `aetherion.plugin-lock/v1` : le lockfile M5-B conserve son contrat manifest-only, tandis que le rapport C6 porte la provenance du module associé. Les tests d'intégration gardent des golden reports pour le dry-run, l'exécution, la télémétrie/IO et l'audit, ainsi qu'un corpus borné d'entrées invalides et de quotas.

## Commandes

```console
aetherion plugin validate plugins/example.plugin.json
aetherion plugin inspect plugins/example.plugin.json
aetherion plugin run --manifest plugins/example.plugin.json --module plugins/example.wasm --dry-run
aetherion plugin audit --manifest plugins/example.plugin.json --module plugins/example.wasm
aetherion plugin list plugins
```

`validate` retourne un rapport JSON et le code 0, ou une erreur de validation et le code 2. `inspect` retourne le manifeste validé avec ses capacités en ordre canonique. `list` valide tous les manifestes du dossier, rejette les identifiants dupliqués et émet un catalogue JSON déterministe. `audit` retourne le rapport de provenance et le code 0 uniquement après validation complète du module; sans `plugin-runtime`, `run` et `audit` retournent `plugin_runtime_feature_disabled`.

## Lockfile M5-B

```console
aetherion plugin resolve --dir plugins --lockfile aetherion.plugin-lock.json
aetherion plugin lock-check --dir plugins --lockfile aetherion.plugin-lock.json
```

`resolve` publie atomiquement un lockfile `aetherion.plugin-lock/v1`, trié par identifiant. Chaque entrée conserve le chemin relatif, la version ABI, les capacités, la version et le checksum FNV-1a exact du manifeste. `lock-check` recalcule ces données; toute divergence retourne le code 1 avec un rapport JSON. Sans `--lockfile`, le nom par défaut est `aetherion.plugin-lock.json`.

## Exemple

```json
{
  "schema": "aetherion.plugin/v1",
  "id": "org.example.telemetry",
  "version": "1.0.0",
  "abi": {
    "major": 1,
    "minimum_host_minor": 0
  },
  "capabilities": ["simulation_read", "telemetry_write"],
  "quotas": {
    "memory_bytes": 16777216,
    "fuel": 1000000,
    "io_read_bytes": 1048576,
    "io_write_bytes": 1048576,
    "files": 16
  }
}
```

L'ABI hôte actuelle est `1.1`. Le `major` doit être identique et `minimum_host_minor` ne peut dépasser celui de l'hôte. Cette politique est testée sur deux versions mineures : un plugin exigeant `1.0` est accepté par un hôte `1.0` et `1.1`, tandis qu'un plugin exigeant `1.1` est refusé par `1.0` et accepté par `1.1`. Un changement de `major` reste toujours incompatible. Les versions de plugins utilisent exactement trois composantes numériques sans suffixe.

Le contrat est implémenté par `plugin::validate_against_host` et couvert par les tests unitaires de `src/plugin.rs` ainsi que par le test CLI `tests/plugin_abi_compat.rs`. Le rapport de validation publie l'ABI courante et la politique de compatibilité.

Capacités disponibles : `asset_read`, `scene_read`, `simulation_read`, `telemetry_write`. Depuis C2, elles activent uniquement les imports correspondants du module `aetherion_v1`; elles n'accordent jamais un accès général au système.

Bornes : mémoire 1–256 Mio, fuel 1–1 000 000 000, lecture et écriture chacune 0–64 Mio, fichiers 0–1024. C2 applique les limites mémoire/fuel et le tampon de télémétrie; C3 applique les quotas IO/fichiers aux imports d'assets.
