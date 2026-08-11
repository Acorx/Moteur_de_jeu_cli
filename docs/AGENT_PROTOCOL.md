# Protocole local d'agents — M2

## Transport et lancement

```console
aetherion agent --path ./demo --root . [--policy policy.json] [--audit audit.jsonl] < requests.jsonl
```

Le transport est exclusivement JSON Lines sur stdin/stdout : une requête et une réponse par ligne UTF-8. Aucun socket n'est créé. stdout ne contient que les réponses ; EOF termine proprement. Une ligne JSON invalide produit une erreur et la boucle continue.

## Enveloppes

Requête : `{"schema":"aetherion.agent-request/v1","request_id":"r1","method":"handshake","params":{}}`.
Réponse réussie : `{"schema":"aetherion.agent-response/v1","request_id":"r1","status":"ok","result":{...}}`.
Réponse échouée : même schéma, `status:error`, puis `error:{code,message,retryable,details?}`. Les champs inconnus sont refusés, y compris dans les paramètres typés.

## Méthodes

- `handshake {}` : version, méthodes, capacités, plafonds et absence de réseau.
- `session.create {}` / `session.close {session_id}` : un monde isolé, une session maximum.
- `world.inspect {session_id}` : snapshot, révision et checksum sans mutation.
- `world.step` et `world.run` : `{session_id,ticks,events?,expected_revision?}`.
- `input.apply` : `{session_id,events,expected_revision?}` ; applique les commandes déterministes existantes sans tick.
- `capture.create` : `{session_id,path,format?:"ppm"|"png",expected_revision?}` ; image et manifeste sous la racine.
- `capture.multi` : `{session_id,output_dir,views,dry_run?,expected_revision?}` ; lot multi-vues validé et publié atomiquement, ou effets prévus sans fichier en dry-run.
- `state.diff`/`snapshot.diff` : `{session_id,snapshot}` ; opérations canoniques `add|remove|replace`, champ, ancienne/nouvelle valeur.
- `transaction.execute` : `{session_id,operations,dry_run?,expected_revision?}`. Opérations : `world.step`, `input.apply`, `capture.create`, `capture.multi` (chaque vue compte dans `max_captures`).

`dry_run` exécute sur clone, retourne diff/checksums/effets prévus et ne change ni monde, ni révision, ni fichiers. Un commit publie le monde entier puis les captures préparées dans `.aetherion-staging`; une erreur nettoie le staging et renvoie `transaction_aborted`. Les cibles existantes sont refusées afin de ne jamais exposer un remplacement partiel.

## Erreurs stables

`invalid_request`, `incompatible_version`, `method_not_found`, `session_not_found`, `capability_denied`, `quota_exceeded`, `stale_revision`, `transaction_aborted`. Une erreur locale n'arrête pas le protocole.

## Capacités, quotas et confinement

La politique `aetherion.capability-policy/v1` contrôle `project_read`, `world_mutate`, `capture`, `file_write` et les maxima ligne/opérations/ticks/événements/captures/sortie/audit. Plafonds absolus : ligne 4 MiB, 1024 opérations, 1 000 000 ticks, 10 000 événements, 64 captures, sortie/audit 16 MiB. Les valeurs par défaut plus strictes sont publiées par `handshake`.

Tout chemin écrit est résolu sous `--root`. Les composants `..`, chemins absolus hors racine et ancêtres/liens canonicalisés hors racine sont rejetés. Aetherion ne lance ni processus, ni script, ni requête réseau. Sur les plateformes où les reparse points évoluent concurremment, la protection portable reste soumise aux garanties du système de fichiers ; employer une racine privée non modifiable par un tiers.

## Concurrence et observabilité

Chaque mutation réussie incrémente `revision`. `expected_revision` active le contrôle optimiste et produit `stale_revision` sans effet. Les résultats transactionnels contiennent révisions/checksums avant/après, ticks, opérations, événements, fichiers et diff. L'audit optionnel est borné, append-only, sans secret ni temps mural : request_id, méthode, verdict et compteurs uniquement. Aucun temps mural n'entre dans les checksums.

## Schémas et exemple

`aetherion schema list` puis `aetherion schema show agent-request` écrivent du JSON brut. Les schémas sont dans `schemas/`. `demo/agent-exchange.jsonl` montre handshake, session, dry-run, commit, inspection, révision obsolète et rollback. Une politique de refus se trouve dans `demo/agent-policy-readonly.json`.
