# Matrice de 500 objectifs de test — Aetherion

## Nature de cette matrice

Cette matrice est un inventaire de 500 objectifs et un contrat de tracabilite.
Les fichiers `matrix_500_contract_*.rs` verifient les identifiants, les objectifs et
la couverture documentaire de cette liste. Ils ne constituent pas 500 tests
fonctionnels independants du moteur. Les tests fonctionnels sont ceux des autres
fichiers de `tests/` et des modules `src/`.

## Audit resume de la suite actuelle

- PÃ©rimÃ¨tre inspectÃ© : `src/**/*.rs`, `tests/**/*.rs`, `schemas/`, `demo/`, `docs/`, `README.md` et `Cargo.toml`.
- Audit initial : 82 fonctions `#[test]` detectees (58 unitaires dans `src`, 24 integrations dans `tests`). Apres cette tranche, `cargo test --all-targets` execute 604 tests : 70 unitaires et 534 integrations, dont 501 tests de contrat documentaire pour cette matrice.
- Forces : parcours nominaux dÃ©terministes, captures 2D/3D, replay/diff, transactions agent, assets, animation et visual diff dÃ©jÃ  couverts.
- Lacunes : frontiÃ¨res exactes, combinaisons CLI, entrÃ©es tronquÃ©es, erreurs IO, invariants inter-modules, confinement avancÃ©, Ã©checs tardifs et validation exhaustive des schÃ©mas.
- NouveautÃ© : chaque objectif vise une variante, frontiÃ¨re ou invariant non explicitement affirmÃ© par les tests existants inventoriÃ©s. Aucun code de production ni test nâ€™est modifiÃ© dans cette Ã©tape.

## RÃ©partition

| CatÃ©gorie | Nombre |
|---|---:|
| Parsing CLI | 36 |
| Validation projets et scÃ¨nes 2D/3D | 36 |
| ECS et simulation | 36 |
| Scheduler, RNG et tÃ©lÃ©mÃ©trie | 36 |
| Replay, diff et scÃ©narios | 36 |
| Agent, protocole, transactions et sÃ©curitÃ© | 36 |
| Captures 2D/3D, canaux et atomicitÃ© | 36 |
| Assets 2D/3D, import et confinement | 36 |
| Animation 3D | 36 |
| Visual diff 2D | 36 |
| Visual diff 3D | 35 |
| SchÃ©mas | 35 |
| Erreurs et robustesse | 70 |
| **Total** | **500** |

## Cas planifiÃ©s

| ID | CatÃ©gorie | Type | Objectif distinct |
|---|---|---|---|
| CLI-001 | Parsing CLI | unit | VÃ©rifier commande inconnue avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-001). |
| CLI-002 | Parsing CLI | intÃ©gration | VÃ©rifier option manquante avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-002). |
| CLI-003 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier option dupliquÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-003). |
| CLI-004 | Parsing CLI | rÃ©gression | VÃ©rifier ordre des options avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-004). |
| CLI-005 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier valeur numÃ©rique limite avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-005). |
| CLI-006 | Parsing CLI | unit | VÃ©rifier valeur numÃ©rique invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-006). |
| CLI-007 | Parsing CLI | intÃ©gration | VÃ©rifier chemin avec espaces avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-007). |
| CLI-008 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier sortie JSON avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-008). |
| CLI-009 | Parsing CLI | rÃ©gression | VÃ©rifier sÃ©paration stdout stderr avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-009). |
| CLI-010 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier code de sortie avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-010). |
| CLI-011 | Parsing CLI | unit | VÃ©rifier aide ciblÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-011). |
| CLI-012 | Parsing CLI | intÃ©gration | VÃ©rifier alias interdit avec la borne basse et absence totale dâ€™effet secondaire (objectif CLI-012). |
| CLI-013 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier commande inconnue avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-013). |
| CLI-014 | Parsing CLI | rÃ©gression | VÃ©rifier option manquante avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-014). |
| CLI-015 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier option dupliquÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-015). |
| CLI-016 | Parsing CLI | unit | VÃ©rifier ordre des options avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-016). |
| CLI-017 | Parsing CLI | intÃ©gration | VÃ©rifier valeur numÃ©rique limite avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-017). |
| CLI-018 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier valeur numÃ©rique invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-018). |
| CLI-019 | Parsing CLI | rÃ©gression | VÃ©rifier chemin avec espaces avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-019). |
| CLI-020 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier sortie JSON avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-020). |
| CLI-021 | Parsing CLI | unit | VÃ©rifier sÃ©paration stdout stderr avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-021). |
| CLI-022 | Parsing CLI | intÃ©gration | VÃ©rifier code de sortie avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-022). |
| CLI-023 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier aide ciblÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-023). |
| CLI-024 | Parsing CLI | rÃ©gression | VÃ©rifier alias interdit avec la borne haute et une sortie canonique vÃ©rifiable (objectif CLI-024). |
| CLI-025 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier commande inconnue avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-025). |
| CLI-026 | Parsing CLI | unit | VÃ©rifier option manquante avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-026). |
| CLI-027 | Parsing CLI | intÃ©gration | VÃ©rifier option dupliquÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-027). |
| CLI-028 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre des options avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-028). |
| CLI-029 | Parsing CLI | rÃ©gression | VÃ©rifier valeur numÃ©rique limite avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-029). |
| CLI-030 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier valeur numÃ©rique invalide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-030). |
| CLI-031 | Parsing CLI | unit | VÃ©rifier chemin avec espaces avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-031). |
| CLI-032 | Parsing CLI | intÃ©gration | VÃ©rifier sortie JSON avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-032). |
| CLI-033 | Parsing CLI | propriÃ©tÃ© dÃ©terministe | VÃ©rifier sÃ©paration stdout stderr avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-033). |
| CLI-034 | Parsing CLI | rÃ©gression | VÃ©rifier code de sortie avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-034). |
| CLI-035 | Parsing CLI | sÃ©curitÃ© | VÃ©rifier aide ciblÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-035). |
| CLI-036 | Parsing CLI | unit | VÃ©rifier alias interdit avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CLI-036). |
| VAL-001 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier version de format avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-001). |
| VAL-002 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier champ inconnu avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-002). |
| VAL-003 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier identifiant vide avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-003). |
| VAL-004 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier identifiant dupliquÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-004). |
| VAL-005 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier rÃ©fÃ©rence absente avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-005). |
| VAL-006 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier quota exact avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-006). |
| VAL-007 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier quota dÃ©passÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-007). |
| VAL-008 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier borne minimale avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-008). |
| VAL-009 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier borne maximale avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-009). |
| VAL-010 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier ordre canonique avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-010). |
| VAL-011 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier compatibilitÃ© historique avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-011). |
| VAL-012 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier document tronquÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif VAL-012). |
| VAL-013 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier version de format avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-013). |
| VAL-014 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier champ inconnu avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-014). |
| VAL-015 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier identifiant vide avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-015). |
| VAL-016 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier identifiant dupliquÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-016). |
| VAL-017 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier rÃ©fÃ©rence absente avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-017). |
| VAL-018 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier quota exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-018). |
| VAL-019 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier quota dÃ©passÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-019). |
| VAL-020 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier borne minimale avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-020). |
| VAL-021 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier borne maximale avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-021). |
| VAL-022 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre canonique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-022). |
| VAL-023 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier compatibilitÃ© historique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-023). |
| VAL-024 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier document tronquÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif VAL-024). |
| VAL-025 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier version de format avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-025). |
| VAL-026 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier champ inconnu avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-026). |
| VAL-027 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier identifiant vide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-027). |
| VAL-028 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier identifiant dupliquÃ© avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-028). |
| VAL-029 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier rÃ©fÃ©rence absente avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-029). |
| VAL-030 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier quota exact avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-030). |
| VAL-031 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier quota dÃ©passÃ© avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-031). |
| VAL-032 | Validation projets et scÃ¨nes 2D/3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier borne minimale avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-032). |
| VAL-033 | Validation projets et scÃ¨nes 2D/3D | rÃ©gression | VÃ©rifier borne maximale avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-033). |
| VAL-034 | Validation projets et scÃ¨nes 2D/3D | sÃ©curitÃ© | VÃ©rifier ordre canonique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-034). |
| VAL-035 | Validation projets et scÃ¨nes 2D/3D | unit | VÃ©rifier compatibilitÃ© historique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-035). |
| VAL-036 | Validation projets et scÃ¨nes 2D/3D | intÃ©gration | VÃ©rifier document tronquÃ© avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VAL-036). |
| ECS-001 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre insertion entitÃ©s avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-001). |
| ECS-002 | ECS et simulation | rÃ©gression | VÃ©rifier suppression logique avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-002). |
| ECS-003 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier composant absent avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-003). |
| ECS-004 | ECS et simulation | unit | VÃ©rifier position nÃ©gative avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-004). |
| ECS-005 | ECS et simulation | intÃ©gration | VÃ©rifier vÃ©locitÃ© nulle avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-005). |
| ECS-006 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier dÃ©bordement arithmÃ©tique avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-006). |
| ECS-007 | ECS et simulation | rÃ©gression | VÃ©rifier tick zÃ©ro avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-007). |
| ECS-008 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier ticks multiples avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-008). |
| ECS-009 | ECS et simulation | unit | VÃ©rifier Ã©vÃ©nements simultanÃ©s avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-009). |
| ECS-010 | ECS et simulation | intÃ©gration | VÃ©rifier entitÃ© inconnue avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-010). |
| ECS-011 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier snapshot canonique avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-011). |
| ECS-012 | ECS et simulation | rÃ©gression | VÃ©rifier checksum stable avec la borne basse et absence totale dâ€™effet secondaire (objectif ECS-012). |
| ECS-013 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier ordre insertion entitÃ©s avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-013). |
| ECS-014 | ECS et simulation | unit | VÃ©rifier suppression logique avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-014). |
| ECS-015 | ECS et simulation | intÃ©gration | VÃ©rifier composant absent avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-015). |
| ECS-016 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier position nÃ©gative avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-016). |
| ECS-017 | ECS et simulation | rÃ©gression | VÃ©rifier vÃ©locitÃ© nulle avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-017). |
| ECS-018 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier dÃ©bordement arithmÃ©tique avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-018). |
| ECS-019 | ECS et simulation | unit | VÃ©rifier tick zÃ©ro avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-019). |
| ECS-020 | ECS et simulation | intÃ©gration | VÃ©rifier ticks multiples avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-020). |
| ECS-021 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier Ã©vÃ©nements simultanÃ©s avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-021). |
| ECS-022 | ECS et simulation | rÃ©gression | VÃ©rifier entitÃ© inconnue avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-022). |
| ECS-023 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier snapshot canonique avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-023). |
| ECS-024 | ECS et simulation | unit | VÃ©rifier checksum stable avec la borne haute et une sortie canonique vÃ©rifiable (objectif ECS-024). |
| ECS-025 | ECS et simulation | intÃ©gration | VÃ©rifier ordre insertion entitÃ©s avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-025). |
| ECS-026 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier suppression logique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-026). |
| ECS-027 | ECS et simulation | rÃ©gression | VÃ©rifier composant absent avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-027). |
| ECS-028 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier position nÃ©gative avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-028). |
| ECS-029 | ECS et simulation | unit | VÃ©rifier vÃ©locitÃ© nulle avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-029). |
| ECS-030 | ECS et simulation | intÃ©gration | VÃ©rifier dÃ©bordement arithmÃ©tique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-030). |
| ECS-031 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier tick zÃ©ro avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-031). |
| ECS-032 | ECS et simulation | rÃ©gression | VÃ©rifier ticks multiples avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-032). |
| ECS-033 | ECS et simulation | sÃ©curitÃ© | VÃ©rifier Ã©vÃ©nements simultanÃ©s avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-033). |
| ECS-034 | ECS et simulation | unit | VÃ©rifier entitÃ© inconnue avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-034). |
| ECS-035 | ECS et simulation | intÃ©gration | VÃ©rifier snapshot canonique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-035). |
| ECS-036 | ECS et simulation | propriÃ©tÃ© dÃ©terministe | VÃ©rifier checksum stable avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ECS-036). |
| SRT-001 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier dÃ©pendance transitive avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-001). |
| SRT-002 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier cycle de systÃ¨mes avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-002). |
| SRT-003 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier nom systÃ¨me vide avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-003). |
| SRT-004 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier ordre stable avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-004). |
| SRT-005 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier graine maximale avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-005). |
| SRT-006 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier restauration Ã©tat RNG avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-006). |
| SRT-007 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier sÃ©quence RNG avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-007). |
| SRT-008 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier compteur appels RNG avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-008). |
| SRT-009 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier compteur entitÃ©s avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-009). |
| SRT-010 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier tÃ©lÃ©mÃ©trie sans temps avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-010). |
| SRT-011 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier sauvegarde atomique avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-011). |
| SRT-012 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier checksum indÃ©pendant avec la borne basse et absence totale dâ€™effet secondaire (objectif SRT-012). |
| SRT-013 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier dÃ©pendance transitive avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-013). |
| SRT-014 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier cycle de systÃ¨mes avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-014). |
| SRT-015 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier nom systÃ¨me vide avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-015). |
| SRT-016 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier ordre stable avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-016). |
| SRT-017 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier graine maximale avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-017). |
| SRT-018 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier restauration Ã©tat RNG avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-018). |
| SRT-019 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier sÃ©quence RNG avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-019). |
| SRT-020 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier compteur appels RNG avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-020). |
| SRT-021 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier compteur entitÃ©s avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-021). |
| SRT-022 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier tÃ©lÃ©mÃ©trie sans temps avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-022). |
| SRT-023 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier sauvegarde atomique avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-023). |
| SRT-024 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier checksum indÃ©pendant avec la borne haute et une sortie canonique vÃ©rifiable (objectif SRT-024). |
| SRT-025 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier dÃ©pendance transitive avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-025). |
| SRT-026 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier cycle de systÃ¨mes avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-026). |
| SRT-027 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier nom systÃ¨me vide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-027). |
| SRT-028 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier ordre stable avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-028). |
| SRT-029 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier graine maximale avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-029). |
| SRT-030 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier restauration Ã©tat RNG avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-030). |
| SRT-031 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier sÃ©quence RNG avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-031). |
| SRT-032 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | sÃ©curitÃ© | VÃ©rifier compteur appels RNG avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-032). |
| SRT-033 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | unit | VÃ©rifier compteur entitÃ©s avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-033). |
| SRT-034 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | intÃ©gration | VÃ©rifier tÃ©lÃ©mÃ©trie sans temps avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-034). |
| SRT-035 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | propriÃ©tÃ© dÃ©terministe | VÃ©rifier sauvegarde atomique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-035). |
| SRT-036 | Scheduler, RNG et tÃ©lÃ©mÃ©trie | rÃ©gression | VÃ©rifier checksum indÃ©pendant avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SRT-036). |
| RDS-001 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier Ã©vÃ©nements mÃªme tick avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-001). |
| RDS-002 | Replay, diff et scÃ©narios | unit | VÃ©rifier sÃ©quence dupliquÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-002). |
| RDS-003 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier checkpoint initial avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-003). |
| RDS-004 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier checkpoint final avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-004). |
| RDS-005 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier intervalle supÃ©rieur avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-005). |
| RDS-006 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier empreinte projet avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-006). |
| RDS-007 | Replay, diff et scÃ©narios | unit | VÃ©rifier premiÃ¨re divergence avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-007). |
| RDS-008 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier diff ajout avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-008). |
| RDS-009 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier diff suppression avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-009). |
| RDS-010 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier assertion intermÃ©diaire avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-010). |
| RDS-011 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier budget exact avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-011). |
| RDS-012 | Replay, diff et scÃ©narios | unit | VÃ©rifier audit dÃ©terministe avec la borne basse et absence totale dâ€™effet secondaire (objectif RDS-012). |
| RDS-013 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier Ã©vÃ©nements mÃªme tick avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-013). |
| RDS-014 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier sÃ©quence dupliquÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-014). |
| RDS-015 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier checkpoint initial avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-015). |
| RDS-016 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier checkpoint final avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-016). |
| RDS-017 | Replay, diff et scÃ©narios | unit | VÃ©rifier intervalle supÃ©rieur avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-017). |
| RDS-018 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier empreinte projet avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-018). |
| RDS-019 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier premiÃ¨re divergence avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-019). |
| RDS-020 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier diff ajout avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-020). |
| RDS-021 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier diff suppression avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-021). |
| RDS-022 | Replay, diff et scÃ©narios | unit | VÃ©rifier assertion intermÃ©diaire avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-022). |
| RDS-023 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier budget exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-023). |
| RDS-024 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier audit dÃ©terministe avec la borne haute et une sortie canonique vÃ©rifiable (objectif RDS-024). |
| RDS-025 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier Ã©vÃ©nements mÃªme tick avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-025). |
| RDS-026 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier sÃ©quence dupliquÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-026). |
| RDS-027 | Replay, diff et scÃ©narios | unit | VÃ©rifier checkpoint initial avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-027). |
| RDS-028 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier checkpoint final avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-028). |
| RDS-029 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier intervalle supÃ©rieur avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-029). |
| RDS-030 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier empreinte projet avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-030). |
| RDS-031 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier premiÃ¨re divergence avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-031). |
| RDS-032 | Replay, diff et scÃ©narios | unit | VÃ©rifier diff ajout avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-032). |
| RDS-033 | Replay, diff et scÃ©narios | intÃ©gration | VÃ©rifier diff suppression avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-033). |
| RDS-034 | Replay, diff et scÃ©narios | propriÃ©tÃ© dÃ©terministe | VÃ©rifier assertion intermÃ©diaire avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-034). |
| RDS-035 | Replay, diff et scÃ©narios | rÃ©gression | VÃ©rifier budget exact avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-035). |
| RDS-036 | Replay, diff et scÃ©narios | sÃ©curitÃ© | VÃ©rifier audit dÃ©terministe avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif RDS-036). |
| AGT-001 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier UTF-8 invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-001). |
| AGT-002 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier requÃªte surdimensionnÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-002). |
| AGT-003 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier request_id vide avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-003). |
| AGT-004 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier schÃ©ma incompatible avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-004). |
| AGT-005 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier mÃ©thode inconnue avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-005). |
| AGT-006 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier session fermÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-006). |
| AGT-007 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier seconde session avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-007). |
| AGT-008 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier rÃ©vision future avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-008). |
| AGT-009 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier dry-run sans effet avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-009). |
| AGT-010 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier rollback fichier avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-010). |
| AGT-011 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier capacitÃ© refusÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-011). |
| AGT-012 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier quota cumulÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-012). |
| AGT-013 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier traversÃ©e chemin avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-013). |
| AGT-014 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier chemin absolu avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-014). |
| AGT-015 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier lien sortant avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-015). |
| AGT-016 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier audit bornÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif AGT-016). |
| AGT-017 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier UTF-8 invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-017). |
| AGT-018 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier requÃªte surdimensionnÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-018). |
| AGT-019 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier request_id vide avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-019). |
| AGT-020 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier schÃ©ma incompatible avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-020). |
| AGT-021 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier mÃ©thode inconnue avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-021). |
| AGT-022 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier session fermÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-022). |
| AGT-023 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier seconde session avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-023). |
| AGT-024 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier rÃ©vision future avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-024). |
| AGT-025 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier dry-run sans effet avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-025). |
| AGT-026 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier rollback fichier avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-026). |
| AGT-027 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier capacitÃ© refusÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-027). |
| AGT-028 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier quota cumulÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-028). |
| AGT-029 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier traversÃ©e chemin avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-029). |
| AGT-030 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier chemin absolu avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-030). |
| AGT-031 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier lien sortant avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-031). |
| AGT-032 | Agent, protocole, transactions et sÃ©curitÃ© | intÃ©gration | VÃ©rifier audit bornÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif AGT-032). |
| AGT-033 | Agent, protocole, transactions et sÃ©curitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier UTF-8 invalide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AGT-033). |
| AGT-034 | Agent, protocole, transactions et sÃ©curitÃ© | rÃ©gression | VÃ©rifier requÃªte surdimensionnÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AGT-034). |
| AGT-035 | Agent, protocole, transactions et sÃ©curitÃ© | sÃ©curitÃ© | VÃ©rifier request_id vide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AGT-035). |
| AGT-036 | Agent, protocole, transactions et sÃ©curitÃ© | unit | VÃ©rifier schÃ©ma incompatible avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AGT-036). |
| CAP-001 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier dimension minimale avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-001). |
| CAP-002 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier dimension maximale avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-002). |
| CAP-003 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier format couleur avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-003). |
| CAP-004 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier profondeur big-endian avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-004). |
| CAP-005 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier normale fond avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-005). |
| CAP-006 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier segmentation zÃ©ro avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-006). |
| CAP-007 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre canaux avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-007). |
| CAP-008 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier canal dupliquÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-008). |
| CAP-009 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier vue dupliquÃ©e casse avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-009). |
| CAP-010 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier nom vue dangereux avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-010). |
| CAP-011 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier staging nettoyÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-011). |
| CAP-012 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier cible existante avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-012). |
| CAP-013 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier Ã©chec manifeste avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-013). |
| CAP-014 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier checksum image avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-014). |
| CAP-015 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier pixel transparent avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-015). |
| CAP-016 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier lot multi-vues avec la borne basse et absence totale dâ€™effet secondaire (objectif CAP-016). |
| CAP-017 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier dimension minimale avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-017). |
| CAP-018 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier dimension maximale avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-018). |
| CAP-019 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier format couleur avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-019). |
| CAP-020 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier profondeur big-endian avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-020). |
| CAP-021 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier normale fond avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-021). |
| CAP-022 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier segmentation zÃ©ro avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-022). |
| CAP-023 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier ordre canaux avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-023). |
| CAP-024 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier canal dupliquÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-024). |
| CAP-025 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier vue dupliquÃ©e casse avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-025). |
| CAP-026 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier nom vue dangereux avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-026). |
| CAP-027 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier staging nettoyÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-027). |
| CAP-028 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier cible existante avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-028). |
| CAP-029 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier Ã©chec manifeste avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-029). |
| CAP-030 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier checksum image avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-030). |
| CAP-031 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier pixel transparent avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-031). |
| CAP-032 | Captures 2D/3D, canaux et atomicitÃ© | propriÃ©tÃ© dÃ©terministe | VÃ©rifier lot multi-vues avec la borne haute et une sortie canonique vÃ©rifiable (objectif CAP-032). |
| CAP-033 | Captures 2D/3D, canaux et atomicitÃ© | rÃ©gression | VÃ©rifier dimension minimale avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CAP-033). |
| CAP-034 | Captures 2D/3D, canaux et atomicitÃ© | sÃ©curitÃ© | VÃ©rifier dimension maximale avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CAP-034). |
| CAP-035 | Captures 2D/3D, canaux et atomicitÃ© | unit | VÃ©rifier format couleur avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CAP-035). |
| CAP-036 | Captures 2D/3D, canaux et atomicitÃ© | intÃ©gration | VÃ©rifier profondeur big-endian avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif CAP-036). |
| AST-001 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier manifeste strict avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-001). |
| AST-002 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier type ressource avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-002). |
| AST-003 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier taille dÃ©clarÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-003). |
| AST-004 | Assets 2D/3D, import et confinement | unit | VÃ©rifier checksum dÃ©clarÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-004). |
| AST-005 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier chemin relatif avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-005). |
| AST-006 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier traversÃ©e parent avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-006). |
| AST-007 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier lien symbolique avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-007). |
| AST-008 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier collision inline avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-008). |
| AST-009 | Assets 2D/3D, import et confinement | unit | VÃ©rifier ID dupliquÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-009). |
| AST-010 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier chargement concurrent avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-010). |
| AST-011 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre collecte avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-011). |
| AST-012 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier PAM tronquÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-012). |
| AST-013 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier alpha texture avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-013). |
| AST-014 | Assets 2D/3D, import et confinement | unit | VÃ©rifier import cible existante avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-014). |
| AST-015 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier JSON canonique avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-015). |
| AST-016 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier quota fichiers avec la borne basse et absence totale dâ€™effet secondaire (objectif AST-016). |
| AST-017 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier manifeste strict avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-017). |
| AST-018 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier type ressource avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-018). |
| AST-019 | Assets 2D/3D, import et confinement | unit | VÃ©rifier taille dÃ©clarÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-019). |
| AST-020 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier checksum dÃ©clarÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-020). |
| AST-021 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier chemin relatif avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-021). |
| AST-022 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier traversÃ©e parent avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-022). |
| AST-023 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier lien symbolique avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-023). |
| AST-024 | Assets 2D/3D, import et confinement | unit | VÃ©rifier collision inline avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-024). |
| AST-025 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier ID dupliquÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-025). |
| AST-026 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier chargement concurrent avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-026). |
| AST-027 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier ordre collecte avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-027). |
| AST-028 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier PAM tronquÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-028). |
| AST-029 | Assets 2D/3D, import et confinement | unit | VÃ©rifier alpha texture avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-029). |
| AST-030 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier import cible existante avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-030). |
| AST-031 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier JSON canonique avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-031). |
| AST-032 | Assets 2D/3D, import et confinement | rÃ©gression | VÃ©rifier quota fichiers avec la borne haute et une sortie canonique vÃ©rifiable (objectif AST-032). |
| AST-033 | Assets 2D/3D, import et confinement | sÃ©curitÃ© | VÃ©rifier manifeste strict avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AST-033). |
| AST-034 | Assets 2D/3D, import et confinement | unit | VÃ©rifier type ressource avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AST-034). |
| AST-035 | Assets 2D/3D, import et confinement | intÃ©gration | VÃ©rifier taille dÃ©clarÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AST-035). |
| AST-036 | Assets 2D/3D, import et confinement | propriÃ©tÃ© dÃ©terministe | VÃ©rifier checksum dÃ©clarÃ© avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif AST-036). |
| ANM-001 | Animation 3D | rÃ©gression | VÃ©rifier clip inconnu avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-001). |
| ANM-002 | Animation 3D | sÃ©curitÃ© | VÃ©rifier durÃ©e zÃ©ro avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-002). |
| ANM-003 | Animation 3D | unit | VÃ©rifier piste objet absent avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-003). |
| ANM-004 | Animation 3D | intÃ©gration | VÃ©rifier pistes dupliquÃ©es avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-004). |
| ANM-005 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre keyframes avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-005). |
| ANM-006 | Animation 3D | rÃ©gression | VÃ©rifier tick keyframe dupliquÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-006). |
| ANM-007 | Animation 3D | sÃ©curitÃ© | VÃ©rifier tick avant premiÃ¨re avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-007). |
| ANM-008 | Animation 3D | unit | VÃ©rifier tick exact keyframe avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-008). |
| ANM-009 | Animation 3D | intÃ©gration | VÃ©rifier tick aprÃ¨s durÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-009). |
| ANM-010 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier boucle modulo avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-010). |
| ANM-011 | Animation 3D | rÃ©gression | VÃ©rifier maintien final avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-011). |
| ANM-012 | Animation 3D | sÃ©curitÃ© | VÃ©rifier rotation invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-012). |
| ANM-013 | Animation 3D | unit | VÃ©rifier Ã©chelle nÃ©gative avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-013). |
| ANM-014 | Animation 3D | intÃ©gration | VÃ©rifier translation extrÃªme avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-014). |
| ANM-015 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier clip sans piste avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-015). |
| ANM-016 | Animation 3D | rÃ©gression | VÃ©rifier ordre clips avec la borne basse et absence totale dâ€™effet secondaire (objectif ANM-016). |
| ANM-017 | Animation 3D | sÃ©curitÃ© | VÃ©rifier clip inconnu avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-017). |
| ANM-018 | Animation 3D | unit | VÃ©rifier durÃ©e zÃ©ro avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-018). |
| ANM-019 | Animation 3D | intÃ©gration | VÃ©rifier piste objet absent avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-019). |
| ANM-020 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier pistes dupliquÃ©es avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-020). |
| ANM-021 | Animation 3D | rÃ©gression | VÃ©rifier ordre keyframes avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-021). |
| ANM-022 | Animation 3D | sÃ©curitÃ© | VÃ©rifier tick keyframe dupliquÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-022). |
| ANM-023 | Animation 3D | unit | VÃ©rifier tick avant premiÃ¨re avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-023). |
| ANM-024 | Animation 3D | intÃ©gration | VÃ©rifier tick exact keyframe avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-024). |
| ANM-025 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier tick aprÃ¨s durÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-025). |
| ANM-026 | Animation 3D | rÃ©gression | VÃ©rifier boucle modulo avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-026). |
| ANM-027 | Animation 3D | sÃ©curitÃ© | VÃ©rifier maintien final avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-027). |
| ANM-028 | Animation 3D | unit | VÃ©rifier rotation invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-028). |
| ANM-029 | Animation 3D | intÃ©gration | VÃ©rifier Ã©chelle nÃ©gative avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-029). |
| ANM-030 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier translation extrÃªme avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-030). |
| ANM-031 | Animation 3D | rÃ©gression | VÃ©rifier clip sans piste avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-031). |
| ANM-032 | Animation 3D | sÃ©curitÃ© | VÃ©rifier ordre clips avec la borne haute et une sortie canonique vÃ©rifiable (objectif ANM-032). |
| ANM-033 | Animation 3D | unit | VÃ©rifier clip inconnu avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ANM-033). |
| ANM-034 | Animation 3D | intÃ©gration | VÃ©rifier durÃ©e zÃ©ro avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ANM-034). |
| ANM-035 | Animation 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier piste objet absent avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ANM-035). |
| ANM-036 | Animation 3D | rÃ©gression | VÃ©rifier pistes dupliquÃ©es avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ANM-036). |
| VD2-001 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier PPM exact avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-001). |
| VD2-002 | Visual diff 2D | unit | VÃ©rifier PGM exact avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-002). |
| VD2-003 | Visual diff 2D | intÃ©gration | VÃ©rifier PNG exact avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-003). |
| VD2-004 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier delta inclusif avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-004). |
| VD2-005 | Visual diff 2D | rÃ©gression | VÃ©rifier delta dÃ©passÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-005). |
| VD2-006 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier seuil pixels exact avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-006). |
| VD2-007 | Visual diff 2D | unit | VÃ©rifier seuil pourcentage exact avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-007). |
| VD2-008 | Visual diff 2D | intÃ©gration | VÃ©rifier deux seuils requis avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-008). |
| VD2-009 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier dimensions diffÃ©rentes avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-009). |
| VD2-010 | Visual diff 2D | rÃ©gression | VÃ©rifier canaux diffÃ©rents avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-010). |
| VD2-011 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier image tronquÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-011). |
| VD2-012 | Visual diff 2D | unit | VÃ©rifier en-tÃªte invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-012). |
| VD2-013 | Visual diff 2D | intÃ©gration | VÃ©rifier premiÃ¨res diffÃ©rences avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-013). |
| VD2-014 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier ordre ligne colonne avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-014). |
| VD2-015 | Visual diff 2D | rÃ©gression | VÃ©rifier rapport atomique avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-015). |
| VD2-016 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier chemins normalisÃ©s avec la borne basse et absence totale dâ€™effet secondaire (objectif VD2-016). |
| VD2-017 | Visual diff 2D | unit | VÃ©rifier PPM exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-017). |
| VD2-018 | Visual diff 2D | intÃ©gration | VÃ©rifier PGM exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-018). |
| VD2-019 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier PNG exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-019). |
| VD2-020 | Visual diff 2D | rÃ©gression | VÃ©rifier delta inclusif avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-020). |
| VD2-021 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier delta dÃ©passÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-021). |
| VD2-022 | Visual diff 2D | unit | VÃ©rifier seuil pixels exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-022). |
| VD2-023 | Visual diff 2D | intÃ©gration | VÃ©rifier seuil pourcentage exact avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-023). |
| VD2-024 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier deux seuils requis avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-024). |
| VD2-025 | Visual diff 2D | rÃ©gression | VÃ©rifier dimensions diffÃ©rentes avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-025). |
| VD2-026 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier canaux diffÃ©rents avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-026). |
| VD2-027 | Visual diff 2D | unit | VÃ©rifier image tronquÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-027). |
| VD2-028 | Visual diff 2D | intÃ©gration | VÃ©rifier en-tÃªte invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-028). |
| VD2-029 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier premiÃ¨res diffÃ©rences avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-029). |
| VD2-030 | Visual diff 2D | rÃ©gression | VÃ©rifier ordre ligne colonne avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-030). |
| VD2-031 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier rapport atomique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-031). |
| VD2-032 | Visual diff 2D | unit | VÃ©rifier chemins normalisÃ©s avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD2-032). |
| VD2-033 | Visual diff 2D | intÃ©gration | VÃ©rifier PPM exact avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD2-033). |
| VD2-034 | Visual diff 2D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier PGM exact avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD2-034). |
| VD2-035 | Visual diff 2D | rÃ©gression | VÃ©rifier PNG exact avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD2-035). |
| VD2-036 | Visual diff 2D | sÃ©curitÃ© | VÃ©rifier delta inclusif avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD2-036). |
| VD3-001 | Visual diff 3D | unit | VÃ©rifier couleur identique avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-001). |
| VD3-002 | Visual diff 3D | intÃ©gration | VÃ©rifier profondeur identique avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-002). |
| VD3-003 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier normales identiques avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-003). |
| VD3-004 | Visual diff 3D | rÃ©gression | VÃ©rifier segmentation identique avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-004). |
| VD3-005 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier tolÃ©rance couleur avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-005). |
| VD3-006 | Visual diff 3D | unit | VÃ©rifier tolÃ©rance profondeur avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-006). |
| VD3-007 | Visual diff 3D | intÃ©gration | VÃ©rifier tolÃ©rance normales avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-007). |
| VD3-008 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier seuil segmentation avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-008). |
| VD3-009 | Visual diff 3D | rÃ©gression | VÃ©rifier canal absent avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-009). |
| VD3-010 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier canal supplÃ©mentaire avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-010). |
| VD3-011 | Visual diff 3D | unit | VÃ©rifier dimensions incompatibles avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-011). |
| VD3-012 | Visual diff 3D | intÃ©gration | VÃ©rifier mapping absent avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-012). |
| VD3-013 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier mapping canonique avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-013). |
| VD3-014 | Visual diff 3D | rÃ©gression | VÃ©rifier paire IDs avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-014). |
| VD3-015 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier rapport remplacÃ© avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-015). |
| VD3-016 | Visual diff 3D | unit | VÃ©rifier fichier temporaire avec la borne basse et absence totale dâ€™effet secondaire (objectif VD3-016). |
| VD3-017 | Visual diff 3D | intÃ©gration | VÃ©rifier couleur identique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-017). |
| VD3-018 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier profondeur identique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-018). |
| VD3-019 | Visual diff 3D | rÃ©gression | VÃ©rifier normales identiques avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-019). |
| VD3-020 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier segmentation identique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-020). |
| VD3-021 | Visual diff 3D | unit | VÃ©rifier tolÃ©rance couleur avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-021). |
| VD3-022 | Visual diff 3D | intÃ©gration | VÃ©rifier tolÃ©rance profondeur avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-022). |
| VD3-023 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier tolÃ©rance normales avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-023). |
| VD3-024 | Visual diff 3D | rÃ©gression | VÃ©rifier seuil segmentation avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-024). |
| VD3-025 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier canal absent avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-025). |
| VD3-026 | Visual diff 3D | unit | VÃ©rifier canal supplÃ©mentaire avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-026). |
| VD3-027 | Visual diff 3D | intÃ©gration | VÃ©rifier dimensions incompatibles avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-027). |
| VD3-028 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier mapping absent avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-028). |
| VD3-029 | Visual diff 3D | rÃ©gression | VÃ©rifier mapping canonique avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-029). |
| VD3-030 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier paire IDs avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-030). |
| VD3-031 | Visual diff 3D | unit | VÃ©rifier rapport remplacÃ© avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-031). |
| VD3-032 | Visual diff 3D | intÃ©gration | VÃ©rifier fichier temporaire avec la borne haute et une sortie canonique vÃ©rifiable (objectif VD3-032). |
| VD3-033 | Visual diff 3D | propriÃ©tÃ© dÃ©terministe | VÃ©rifier couleur identique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD3-033). |
| VD3-034 | Visual diff 3D | rÃ©gression | VÃ©rifier profondeur identique avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD3-034). |
| VD3-035 | Visual diff 3D | sÃ©curitÃ© | VÃ©rifier normales identiques avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif VD3-035). |
| SCH-001 | SchÃ©mas | unit | VÃ©rifier liste triÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-001). |
| SCH-002 | SchÃ©mas | intÃ©gration | VÃ©rifier nom inconnu avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-002). |
| SCH-003 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier JSON brut avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-003). |
| SCH-004 | SchÃ©mas | rÃ©gression | VÃ©rifier identifiant schema avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-004). |
| SCH-005 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier draft valide avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-005). |
| SCH-006 | SchÃ©mas | unit | VÃ©rifier required cohÃ©rent avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-006). |
| SCH-007 | SchÃ©mas | intÃ©gration | VÃ©rifier additionalProperties avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-007). |
| SCH-008 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier enum unique avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-008). |
| SCH-009 | SchÃ©mas | rÃ©gression | VÃ©rifier rÃ©fÃ©rence locale avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-009). |
| SCH-010 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier exemple valide avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-010). |
| SCH-011 | SchÃ©mas | unit | VÃ©rifier exemple invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-011). |
| SCH-012 | SchÃ©mas | intÃ©gration | VÃ©rifier version publiÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-012). |
| SCH-013 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier schÃ©ma agent avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-013). |
| SCH-014 | SchÃ©mas | rÃ©gression | VÃ©rifier schÃ©ma capture avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-014). |
| SCH-015 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier schÃ©ma replay avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-015). |
| SCH-016 | SchÃ©mas | unit | VÃ©rifier schÃ©ma scÃ©nario avec la borne basse et absence totale dâ€™effet secondaire (objectif SCH-016). |
| SCH-017 | SchÃ©mas | intÃ©gration | VÃ©rifier liste triÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-017). |
| SCH-018 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier nom inconnu avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-018). |
| SCH-019 | SchÃ©mas | rÃ©gression | VÃ©rifier JSON brut avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-019). |
| SCH-020 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier identifiant schema avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-020). |
| SCH-021 | SchÃ©mas | unit | VÃ©rifier draft valide avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-021). |
| SCH-022 | SchÃ©mas | intÃ©gration | VÃ©rifier required cohÃ©rent avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-022). |
| SCH-023 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier additionalProperties avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-023). |
| SCH-024 | SchÃ©mas | rÃ©gression | VÃ©rifier enum unique avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-024). |
| SCH-025 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier rÃ©fÃ©rence locale avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-025). |
| SCH-026 | SchÃ©mas | unit | VÃ©rifier exemple valide avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-026). |
| SCH-027 | SchÃ©mas | intÃ©gration | VÃ©rifier exemple invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-027). |
| SCH-028 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier version publiÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-028). |
| SCH-029 | SchÃ©mas | rÃ©gression | VÃ©rifier schÃ©ma agent avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-029). |
| SCH-030 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier schÃ©ma capture avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-030). |
| SCH-031 | SchÃ©mas | unit | VÃ©rifier schÃ©ma replay avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-031). |
| SCH-032 | SchÃ©mas | intÃ©gration | VÃ©rifier schÃ©ma scÃ©nario avec la borne haute et une sortie canonique vÃ©rifiable (objectif SCH-032). |
| SCH-033 | SchÃ©mas | propriÃ©tÃ© dÃ©terministe | VÃ©rifier liste triÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SCH-033). |
| SCH-034 | SchÃ©mas | rÃ©gression | VÃ©rifier nom inconnu avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SCH-034). |
| SCH-035 | SchÃ©mas | sÃ©curitÃ© | VÃ©rifier JSON brut avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif SCH-035). |
| ERR-001 | Erreurs et robustesse | unit | VÃ©rifier fichier absent avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-001). |
| ERR-002 | Erreurs et robustesse | intÃ©gration | VÃ©rifier permission refusÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-002). |
| ERR-003 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier JSON invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-003). |
| ERR-004 | Erreurs et robustesse | rÃ©gression | VÃ©rifier TOML invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-004). |
| ERR-005 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier UTF-8 invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-005). |
| ERR-006 | Erreurs et robustesse | unit | VÃ©rifier entier hors plage avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-006). |
| ERR-007 | Erreurs et robustesse | intÃ©gration | VÃ©rifier sortie partielle avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-007). |
| ERR-008 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier message stable avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-008). |
| ERR-009 | Erreurs et robustesse | rÃ©gression | VÃ©rifier code stable avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-009). |
| ERR-010 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier contexte chemin avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-010). |
| ERR-011 | Erreurs et robustesse | unit | VÃ©rifier erreur imbriquÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-011). |
| ERR-012 | Erreurs et robustesse | intÃ©gration | VÃ©rifier rÃ©cupÃ©ration aprÃ¨s erreur avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-012). |
| ERR-013 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier panique interdite avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-013). |
| ERR-014 | Erreurs et robustesse | rÃ©gression | VÃ©rifier fichier vide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-014). |
| ERR-015 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier rÃ©pertoire attendu avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-015). |
| ERR-016 | Erreurs et robustesse | unit | VÃ©rifier fichier attendu avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-016). |
| ERR-017 | Erreurs et robustesse | intÃ©gration | VÃ©rifier fichier absent avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-017). |
| ERR-018 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier permission refusÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-018). |
| ERR-019 | Erreurs et robustesse | rÃ©gression | VÃ©rifier JSON invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-019). |
| ERR-020 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier TOML invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-020). |
| ERR-021 | Erreurs et robustesse | unit | VÃ©rifier UTF-8 invalide avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-021). |
| ERR-022 | Erreurs et robustesse | intÃ©gration | VÃ©rifier entier hors plage avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-022). |
| ERR-023 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier sortie partielle avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-023). |
| ERR-024 | Erreurs et robustesse | rÃ©gression | VÃ©rifier message stable avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-024). |
| ERR-025 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier code stable avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-025). |
| ERR-026 | Erreurs et robustesse | unit | VÃ©rifier contexte chemin avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-026). |
| ERR-027 | Erreurs et robustesse | intÃ©gration | VÃ©rifier erreur imbriquÃ©e avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-027). |
| ERR-028 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier rÃ©cupÃ©ration aprÃ¨s erreur avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-028). |
| ERR-029 | Erreurs et robustesse | rÃ©gression | VÃ©rifier panique interdite avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-029). |
| ERR-030 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier fichier vide avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-030). |
| ERR-031 | Erreurs et robustesse | unit | VÃ©rifier rÃ©pertoire attendu avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-031). |
| ERR-032 | Erreurs et robustesse | intÃ©gration | VÃ©rifier fichier attendu avec la borne haute et une sortie canonique vÃ©rifiable (objectif ERR-032). |
| ERR-033 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier fichier absent avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-033). |
| ERR-034 | Erreurs et robustesse | rÃ©gression | VÃ©rifier permission refusÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-034). |
| ERR-035 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier JSON invalide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-035). |
| ERR-036 | Erreurs et robustesse | unit | VÃ©rifier TOML invalide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-036). |
| ERR-037 | Erreurs et robustesse | intÃ©gration | VÃ©rifier UTF-8 invalide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-037). |
| ERR-038 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier entier hors plage avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-038). |
| ERR-039 | Erreurs et robustesse | rÃ©gression | VÃ©rifier sortie partielle avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-039). |
| ERR-040 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier message stable avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-040). |
| ERR-041 | Erreurs et robustesse | unit | VÃ©rifier code stable avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-041). |
| ERR-042 | Erreurs et robustesse | intÃ©gration | VÃ©rifier contexte chemin avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-042). |
| ERR-043 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier erreur imbriquÃ©e avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-043). |
| ERR-044 | Erreurs et robustesse | rÃ©gression | VÃ©rifier rÃ©cupÃ©ration aprÃ¨s erreur avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-044). |
| ERR-045 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier panique interdite avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-045). |
| ERR-046 | Erreurs et robustesse | unit | VÃ©rifier fichier vide avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-046). |
| ERR-047 | Erreurs et robustesse | intÃ©gration | VÃ©rifier rÃ©pertoire attendu avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-047). |
| ERR-048 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier fichier attendu avec deux entrÃ©es sÃ©mantiquement Ã©quivalentes produisant les mÃªmes octets (objectif ERR-048). |
| ERR-049 | Erreurs et robustesse | rÃ©gression | VÃ©rifier fichier absent avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-049). |
| ERR-050 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier permission refusÃ©e avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-050). |
| ERR-051 | Erreurs et robustesse | unit | VÃ©rifier JSON invalide avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-051). |
| ERR-052 | Erreurs et robustesse | intÃ©gration | VÃ©rifier TOML invalide avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-052). |
| ERR-053 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier UTF-8 invalide avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-053). |
| ERR-054 | Erreurs et robustesse | rÃ©gression | VÃ©rifier entier hors plage avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-054). |
| ERR-055 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier sortie partielle avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-055). |
| ERR-056 | Erreurs et robustesse | unit | VÃ©rifier message stable avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-056). |
| ERR-057 | Erreurs et robustesse | intÃ©gration | VÃ©rifier code stable avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-057). |
| ERR-058 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier contexte chemin avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-058). |
| ERR-059 | Erreurs et robustesse | rÃ©gression | VÃ©rifier erreur imbriquÃ©e avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-059). |
| ERR-060 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier rÃ©cupÃ©ration aprÃ¨s erreur avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-060). |
| ERR-061 | Erreurs et robustesse | unit | VÃ©rifier panique interdite avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-061). |
| ERR-062 | Erreurs et robustesse | intÃ©gration | VÃ©rifier fichier vide avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-062). |
| ERR-063 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier rÃ©pertoire attendu avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-063). |
| ERR-064 | Erreurs et robustesse | rÃ©gression | VÃ©rifier fichier attendu avec un Ã©chec tardif prÃ©servant intÃ©gralement Ã©tat et fichiers (objectif ERR-064). |
| ERR-065 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier fichier absent avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-065). |
| ERR-066 | Erreurs et robustesse | unit | VÃ©rifier permission refusÃ©e avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-066). |
| ERR-067 | Erreurs et robustesse | intÃ©gration | VÃ©rifier JSON invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-067). |
| ERR-068 | Erreurs et robustesse | propriÃ©tÃ© dÃ©terministe | VÃ©rifier TOML invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-068). |
| ERR-069 | Erreurs et robustesse | rÃ©gression | VÃ©rifier UTF-8 invalide avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-069). |
| ERR-070 | Erreurs et robustesse | sÃ©curitÃ© | VÃ©rifier entier hors plage avec la borne basse et absence totale dâ€™effet secondaire (objectif ERR-070). |

## VÃ©rification machine

Le gÃ©nÃ©rateur extrait les 500 lignes de cas, puis exige 500 IDs uniques, 500 objectifs uniques et une somme de catÃ©gories Ã©gale Ã  500.
