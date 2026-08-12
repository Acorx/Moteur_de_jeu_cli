# Scripts déterministes

`script-run --script FILE [--dry-run] [--report FILE]` exécute le format JSON `aetherion.script/v1` sans shell ni processus externe. Les commandes admises sont `true`, `false`, `echo` et `noop`; elles peuvent être des chaînes séparées par espaces ou des listes d'arguments. Les variables `{{nom}}` sont substituées de manière déterministe.

Les budgets `max_commands` et `max_ticks_total` sont appliqués; une divergence de budget retourne le code 3. Un échec de commande retourne 1 et produit, si demandé, un rapport atomique `aetherion.script-report/v1`. `--dry-run` valide et rapporte les commandes sans les exécuter.