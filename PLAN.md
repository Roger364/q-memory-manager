# Plan d'action RamScope

Objectif : rendre l'inventaire mémoire et les conseils de démarrage plus exacts, compréhensibles et prudents, sans transformer RamScope en nettoyeur automatique. Toute modification doit rester réversible, explicite, et ne doit jamais se baser sur un seul nom de processus ou un seuil de RAM.

Ce document est la version 2 du plan. Il remplace la version précédente, dont l'étape 0 annonçait comme livrés un README de présentation et un `SAFETY.md` qui n'existent pas dans le dépôt. Un plan qui déclare terminé ce qui n'existe pas ne peut pas servir de référence : la règle de véracité ci-dessous existe pour empêcher que cela se reproduise.

## 0. Comment lire et tenir ce plan

Chaque étape suit le même gabarit, et aucun de ces cinq points ne peut être omis :

1. **Objectif** — le comportement observable à changer, en une phrase.
2. **Périmètre** — les fichiers et fonctions réellement touchés.
3. **Changements** — ce qui est modifié, dans l'ordre où on le fait.
4. **Critère de sortie** — une condition qu'on peut vérifier sans interprétation : commande, code retour, sortie HTTP, test nommé, `grep`.
5. **Test de réversion** — le test qui échoue si le changement est annulé. Sans ce test, le critère de sortie n'est pas considéré comme couvert.

Règles de tenue :

- Une étape = un commit. Le message cite les identifiants de défauts traités (`D1`, `D4`…).
- Un élément n'est marqué « terminé » que si le journal (section 7) contient le chemin de l'artefact et le code retour de la commande. « À consigner après exécution » n'est plus acceptable : soit la ligne existe avec son code retour, soit l'étape reste « à faire ».
- Une étape qui échoue reste ouverte. On ne passe pas à la suivante en laissant un contrôle rouge derrière soi.
- Les chiffres affichés restent annotés comme mesurés, estimés ou indisponibles. Une estimation n'est jamais présentée comme une mesure.
- Si l'identité d'un élément est incertaine, la recommandation est « rien à faire » ou « vérifier manuellement », jamais une action automatique.
- Ne pas prétendre avoir testé ce qui n'a pas été exécuté. La compilation Windows est vérifiable sur cette machine ; l'exécution réelle des actions exige une VM ou un instantané.

## 1. État vérifié du dépôt (référence de départ)

Constat établi le 2026-11-13, sur la machine de développement Windows, sans modification du code.

| Élément | Constat |
|---|---|
| Volume | 5 655 lignes : `rules.rs` 1 017, `win.rs` 1 457, `app.js` 749, `server.rs` 366, `demo.rs` 506, `ffi.rs` 462 |
| Base de règles | 262 services, 93 entrées de démarrage, 50 tâches |
| `cargo test` | **code retour 0** — 5 tests, 5 réussis, 0 échec |
| `cargo fmt --check` | **code retour 1** — 66 hunks dans 7 fichiers (`demo.rs`, `ffi.rs`, `main.rs`, `rules.rs`, `server.rs`, `util.rs`, `win.rs`) |
| `cargo clippy` | **non exécuté** à ce stade |
| Build release | **non exécuté** à ce stade |
| Rapport présent | `ramscope-data/rapports/ramscope-2026-11-13-19-14-44.json` porte `"demo": true` : ce sont des données simulées, pas une mesure de la machine |
| `README.md` | 2 lignes, titre dupliqué : ne documente ni usage, ni interprétation des mesures |
| `SAFETY.md` | **absent** |
| Compilation Windows | `cargo test` compile `win.rs` : la cible Windows est vérifiable ici. Seule l'application réelle des actions demande une VM ou un instantané |

Conséquence de méthode : la vérification n'est plus regroupée en fin de parcours. Chaque étape se termine par son propre contrôle, et `cargo fmt --check` doit repasser à 0 dès l'étape 0 — sinon l'étape 6 échouerait après six étapes de travail non validées.

## 2. Invariants de conception

Ces règles priment sur toute considération d'ergonomie ou de gain affiché.

1. **La RAM est une observation, pas une preuve de danger.** Un processus gourmand n'est pas un processus inutile.
2. **Verrou et gravité sont deux notions distinctes.** `locked` signifie « RamScope refuse d'agir » ; `Level` signifie « à quel point c'est important pour Windows ». Un service verrouillé n'est pas forcément vital et ne doit pas être affiché comme tel.
3. **Les gains ne s'additionnent pas entre services d'un même hôte.** La RAM d'un `svchost.exe` est partagée : le total d'un lot se calcule par hôte avec un maximum, jamais par somme de parts.
4. **Aucune action destructive ne repose sur un seuil de RAM ou un nom seul.** Il faut une identité vérifiée et une justification traçable.
5. **L'identité d'une cible est revérifiée au moment d'agir**, pas seulement au moment de collecter : PID recyclé, service reconfiguré entre-temps, tâche supprimée.
6. **Aucune promesse de réversibilité pour une terminaison de processus.** Le texte d'aide et les confirmations le disent explicitement.
7. **Mesuré, estimé et indisponible sont distincts partout** — modèle de données, API, interface, rapport exporté.
8. **Un élément non identifié ne reçoit aucune action.** L'interface peut proposer « vérifier manuellement », pas « terminer ».
9. **Toute affirmation « terminé » est adossée à un artefact et à un code retour.**
10. **L'application ne modifie jamais un réglage système sans confirmation explicite**, et jamais dans un lot qui mélange réversible et irréversible sans le dire.

## 3. Registre des défauts

Chaque défaut est rattaché à une étape. Un défaut n'est fermé que lorsque son test de réversion existe et passe.

| ID | Sévérité | Localisation | Constat |
|---|---|---|---|
| D1 | P0 | `server.rs` — `handle()`, injection du jeton dans `index.html` | Le jeton de session est servi dans la page, l'en-tête `Host` n'est jamais validé, et le contrôle d'origine fait `origin.ends_with(host)`. Une page malveillante résolue vers `127.0.0.1` (DNS rebinding) lit `/`, récupère le jeton, puis poste sur `/api/apply` : son origine se termine par le même hôte, le contrôle passe |
| D2 | P0 | `win.rs` — `kill_process`, `model.rs` — `Action::Kill` | L'action ne transporte que `pid` et `name`. Le contrôle porte sur le nom protégé, `pid > 8` et l'auto-terminaison, pas sur l'identité du processus. Un PID recyclé entre la collecte et le clic fait terminer le mauvais programme |
| D3 | P1 | `web/app.js` — `selTotals()`, `confirmApply()` | Le gain différé d'un service est ajouté au gain « tout de suite » quand l'option « Libérer la RAM maintenant » est cochée, puis le total différé est mis à zéro. Du différé est présenté comme de l'immédiat, et le gain des entrées de démarrage sélectionnées disparaît |
| D4 | P1 | `rules.rs` — `process_advice` | Toute application tierce sans fenêtre au-dessus de 150 Mo reçoit une action `Kill`. C'est une action destructive fondée sur un seuil de RAM et un état de fenêtre, ce que le plan interdit par ailleurs |
| D5 | P1 | `rules.rs` — `startup_advice` | Un gain de 60 Mo (ou 40 Mo selon la branche) est fabriqué quand aucune mesure n'existe. Une valeur inventée présentée comme un gain estimé |
| D6 | P1 | `win.rs` — collecte des services, `plan_gain_next_boot` | La part d'un service vaut `host_ram / shared`, puis ces parts sont additionnées. Trois services d'un même hôte produisent un total supérieur à la RAM libérable |
| D7 | P1 | `rules.rs` — `is_protected_service` / `service_advice` | Les services protégés sortent en `Level::Critical`, donc étiquetés « Vital », alors que la table les classe `Keep` : `wuauserv`, `themes`, `msiserver`, `defragsvc`, `w32time`. Protection et gravité sont confondues |
| D8 | P1 | `main.rs` — aide, `web/app.js` — en-tête de confirmation | L'aide affirme « Toutes les modifications sont réversibles » et la modale commence par « Rien n'est supprimé », alors que `Action::Kill` n'a pas d'annulation. La modale se contredit deux lignes plus bas |
| D9 | P2 | `rules.rs` — `startup_advice`, `service_advice` | Correspondance par sous-chaîne sur le nom et sur la ligne de commande entière, avec des clés courtes (`hp`, `wps`, `wsl`, `zoom`, `plex`). Une règle peut s'appliquer à un programme qui n'a rien à voir |
| D10 | P2 | `rules.rs` — table `SERVICES` | La clé `ngcctorSvc` est la seule en majuscules alors que le nom du service est mis en minuscules : la règle ne peut jamais correspondre |
| D11 | P2 | `rules.rs` — `process_advice` | La branche spécifique `svchost.exe` est inatteignable, `svchost.exe` figurant dans `PROTECTED_PROCESSES` et traité en amont |
| D12 | P2 | `main.rs`, `server.rs` — option `--host` | `--host 0.0.0.0` est accepté alors que l'aide annonce une écoute en boucle locale uniquement. Cela exposerait jeton et actions au réseau local |
| D13 | P2 | Dépôt entier | `cargo fmt --check` échoue : 66 hunks dans 7 fichiers |
| D14 | P2 | `README.md`, `SAFETY.md` | Le plan v1 les déclarait livrés ; le README est un stub de deux lignes et `SAFETY.md` est absent |

D13 et D14 sont corrigés par l'étape 0 (contrôles en 7.1) : le formatage est conforme et la documentation existe. D1 et D12 sont corrigés par l'étape 1 (contrôles en 7.2).

## 4. Étapes

### Étape 0 — Socle vérifiable

**État.** Terminée le 2026-11-13 — contrôles et codes retour en 7.1.

**Objectif.** Partir d'une base propre et testable hors Windows, pour que les étapes suivantes aient des contrôles réels au lieu d'intentions.

**Périmètre.** `Cargo.toml`, nouveau module `src/plan.rs`, tous les fichiers `.rs` (formatage), `README.md`, `SAFETY.md`.

**Changements.**

- `cargo fmt` sur le dépôt (D13) et vérification que le formatage n'a rien changé au comportement (`cargo test` avant/après).
- Extraire dans `src/plan.rs`, compilé sur toutes les plateformes, la logique pure : validation d'une liste d'actions, totalisation des gains d'un lot (dédoublonnage ; la non-additivité par hôte reste l'étape 3), normalisation et correspondance des règles. Le code Win32 reste dans `win.rs` ; `plan.rs` ne doit contenir aucun appel FFI.
- Créer un `SAFETY.md` court et vrai : ce qui est mesuré, ce qui est estimé, ce qui n'est jamais automatique, ce que la terminaison d'un processus fait perdre.
- Réécrire `README.md` : lancement, options, ce que veut dire chaque colonne, limites connues, différence entre mode démo et réel.
- Ajouter un test d'invariant minimal sur les tables (clés uniques) pour que l'étape 4 puisse l'étendre.

**Livré le 2026-11-13.** `src/plan.rs` créé (validation, totalisation, normalisation et correspondance) avec 5 tests ; `win.rs` appelle `plan::validate` et `plan::total_gains` en conservant les garde-fous existants ; `rules.rs` délègue `lc` et les trois correspondances au module ; `cargo fmt` appliqué aux 7 fichiers concernés ; `README.md` réécrit et `SAFETY.md` créé.

**Critère de sortie.**

- `cargo fmt --check` → code retour 0.
- `cargo test` → code retour 0, 5 tests d'origine toujours présents et nommés dans le journal.
- `src/plan.rs` ne contient aucune occurrence de `unsafe`, `#[cfg(windows)]` ni `crate::ffi`.
- `SAFETY.md` et `README.md` existent et décrivent le comportement réellement livré, sans annoncer de test non exécuté.

**Test de réversion.** Le test d'unicité des clés échoue si l'on réintroduit un doublon ; `cargo fmt --check` échoue si l'on annule le formatage.

**Effort indicatif.** 0,5 jour.

---

### Étape 1 — Verrouiller la surface locale (P0)

**État.** Terminée le 2026-11-13 — contrôles et codes retour en 7.2.

**Objectif.** Rendre l'API locale inatteignable depuis une page web tierce, y compris par rebinding DNS.

**Périmètre.** `server.rs` — `handle()`, `serve()`, `send()` ; `main.rs` — options ; `web/index.html` si le jeton doit sortir du HTML.

**Changements.**

- Valider `Host` avant tout traitement : liste exacte `127.0.0.1:<port>`, `localhost:<port>`, `[::1]:<port>`. Toute autre valeur → 403, y compris sur les pages statiques (D1).
- Remplacer `origin.ends_with(host)` par une comparaison stricte après analyse : schéma, hôte, port. Pas de sous-chaîne, pas de suffixe.
- Exiger le jeton sur `GET /api/snapshot` comme sur les autres routes `/api/*` ; la page le transmet en en-tête. Supprimer l'injection `__TOKEN__` du HTML si elle n'est plus nécessaire.
- Comparer le jeton sans sortie anticipée dépendante de la position du premier octet différent.
- En-têtes de durcissement : `Content-Security-Policy: default-src 'self'`, `X-Frame-Options: DENY`, `Cross-Origin-Resource-Policy: same-origin`, en plus de `nosniff`, `no-referrer` et `no-store` déjà présents.
- Retirer `--host` ou le renommer `--unsafe-network`, avec un avertissement affiché dans l'interface et le jeton non servi dans ce mode (D12). L'aide doit décrire exactement ce que fait l'option.

**Livré le 2026-11-13.** `--host` n'accepte plus que `127.0.0.1` et `localhost` ; toute autre valeur refuse le démarrage (test unitaire). `Host` est validé exactement pour toutes les routes, pages comprises. L'origine est comparée en schéma, hôte et port, sans suffixe. Le jeton est exigé sur tout `/api/` et comparé sans sortie anticipée. CSP, `X-Frame-Options` et `Cross-Origin-Resource-Policy` sont ajoutés. Le jeton reste dans la page, ce que couvre la validation de `Host` ; la CSP conserve `'unsafe-inline'` pour les styles et gestionnaires inline actuels, à resserrer à l'étape 5.

**Critère de sortie.**

- `curl.exe -s -o NUL -w "%{http_code}" -H "Host: attaquant.tld" http://127.0.0.1:<port>/api/snapshot` → `403`.
- Même requête sur `/` → `403`, et la réponse ne contient ni jeton ni inventaire.
- `POST /api/apply` avec `Origin: http://attaquant.tld:<port>` et `Host: attaquant.tld:<port>` → `403`.
- `POST /api/apply` sans jeton, depuis l'origine réelle → `403`.
- L'interface normale, ouverte via `127.0.0.1`, charge et applique une action en mode démo.

**Test de réversion.** Un test d'intégration qui envoie un `Host` inconnu et attend 403 ; il échoue si le contrôle est retiré. Un test unitaire sur la fonction de comparaison d'origine couvre le cas du suffixe trompeur.

**Effort indicatif.** 1 jour.

---

### Étape 2 — Identité des actions et irréversibilité (P0)

**Objectif.** Garantir qu'une action s'applique à la cible qui a été montrée et confirmée, et que l'utilisateur sait ce qui n'est pas annulable.

**Périmètre.** `model.rs` — `Action`, `RollbackEntry` ; `win.rs` — `kill_process`, `apply`, `apply_service`, `apply_startup` ; `rules.rs` — construction des actions ; `server.rs` — validation avant `backend.apply` ; `web/app.js` — confirmation.

**Changements.**

- `Action::Kill` transporte `pid`, `name`, `image_path` et `created_at` (heure de création du processus). Avant `TerminateProcess`, rouvrir le PID en `PROCESS_QUERY_LIMITED_INFORMATION`, comparer chemin et heure de création ; en cas d'écart, refuser avec un message explicite (« l'identité du processus a changé depuis l'analyse ») (D2).
- Pour `Service`, `Startup` et `Task`, relire l'état courant juste avant d'écrire et refuser si le type de démarrage, l'activation ou l'existence ne correspond plus à ce qui a été confirmé.
- Introduire `plan::validate(snapshot, actions)`, appelée par le serveur avant `backend.apply` : verrou des services protégés, verrou des processus protégés, `pid > 8`, auto-terminaison, identité, doublons dans le lot. `win.rs` conserve ses garde-fous : deux verrous, pas un.
- Refuser un lot qui contient une action irréversible si l'interface n'a pas marqué ce caractère comme confirmé — le serveur ne peut pas vérifier une intention, mais il peut refuser un `Kill` non accompagné de son identité complète.
- Corriger l'aide et les textes : « réversible » ne s'applique qu'aux changements de configuration ; une terminaison de processus n'est jamais annulée par un point de restauration (D8).
- Interdire `Kill` dans toute sélection de masse.

**Critère de sortie.**

- Tests unitaires, exécutables sans Windows, sur `plan::validate` : service protégé, processus protégé, `pid <= 8`, PID propre, identité divergente, lot vide, lot de 401 actions, action déjà appliquée.
- Test d'intégration en mode démo : une action `Kill` dont l'identité ne correspond pas au processus simulé est refusée avec le message attendu.
- `grep -n "réversible" src/main.rs src/web/app.js` : chaque occurrence est soit exacte, soit qualifiée ; plus aucune promesse générale.
- L'interface n'offre aucun `Kill` présélectionné et n'en propose aucun dans un lot automatique.

**Test de réversion.** Le test d'identité divergente échoue si la comparaison chemin/date est retirée ; le test de sélection de masse échoue si `Kill` réapparaît dans `selectSafe` ou équivalent.

**Effort indicatif.** 1,5 jour.

---

### Étape 3 — Vérité des chiffres

**Objectif.** Qu'aucun gain affiché ne soit faux : ni inventé, ni additionné à tort, ni présenté comme immédiat quand il est différé.

**Périmètre.** `model.rs` — `Advice`, `MemoryInfo` ; `rules.rs` — `service_advice`, `process_advice`, `startup_advice` ; `win.rs` — `memory_info`, collecte des services, agrégation du plan ; `web/app.js` — totaux et rendu.

**Changements.**

- Remplacer les entiers nus de gain par une mesure typée : valeur, nature (`mesuré`, `estimé`, `indisponible`), portée (`privé`, `working set`, `part d'hôte`), additivité (oui/non). Le champ reste sérialisable pour ne pas casser le rapport exporté plus que nécessaire.
- Calculer le total d'un lot par hôte : un maximum par `pid` hôte, puis somme des hôtes distincts. Jamais la somme des parts `host_ram / shared` (D6).
- Supprimer les constantes de repli 60 Mo et 40 Mo (D5). Sans mesure : `indisponible`, affiché « non chiffrable ».
- Supprimer la règle des 150 Mo (D4). Un processus tiers non répertorié sans fenêtre devient `Unknown` avec « vérifier manuellement », pas une action.
- Supprimer la conversion différé → immédiat de `stopNow`. Remplacer par une mesure : mémoire disponible avant, application, attente bornée, mémoire disponible après ; afficher le delta comme **mesuré**, et le reste comme **estimé** (D3). Un service arrêté ne libère pas sa part d'un hôte partagé : ne pas l'annoncer.
- Étiqueter `compressed` pour ce qu'il est — le working set du processus « Memory Compression », une approximation — ou utiliser le compteur système correspondant si disponible.
- Vérifier que `used = total - available` est bien la définition annoncée dans l'interface, et que `commit`, `kernel` et `cache` portent leur nom Windows exact dans l'infobulle.

**Critère de sortie.**

- Test unitaire : deux services du même hôte avec des parts de 100 et 200 Mo donnent un total de 300 Mo, pas 300 Mo additionnés une deuxième fois par le plan ; trois services d'un hôte de 300 Mo donnent 300 Mo au maximum.
- `grep -n "150 \* 1024 \* 1024\|60 \* 1024 \* 1024\|40 \* 1024 \* 1024" src/` → aucun résultat.
- En mode démo, l'écran de confirmation affiche deux lignes distinctes, « tout de suite : X » et « au prochain démarrage : Y », et X n'inclut jamais un gain de type démarrage.
- Le rapport exporté contient la nature de chaque gain.

**Test de réversion.** Le test d'agrégation par hôte échoue si l'on revient à une somme de parts ; le test de `stopNow` échoue si un gain différé alimente le total immédiat.

**Effort indicatif.** 1,5 jour.

---

### Étape 4 — Règles : schéma et provenance

**Objectif.** Savoir pourquoi une règle existe, sur quoi elle s'applique exactement, et depuis quand elle n'a pas été revue.

**Périmètre.** `rules.rs` — tables `SERVICES`, `STARTUP`, `TASKS`, `PROCESSES`, `PROTECTED_*` ; `model.rs` — `Level`, nouveau champ de verrou.

**Changements.**

- Passer les tables à une structure nommée : clé, niveau, action conseillée, mode de correspondance, source, date de revue, justification. Le mode de correspondance est explicite : nom exact de service, nom d'exécutable, nom de tâche, préfixe de chemin. Plus aucune correspondance par sous-chaîne sur une ligne de commande entière (D9).
- Corriger la clé `ngcctorSvc` (D10) et supprimer la branche `svchost.exe` inatteignable, en conservant l'information utile (liste des services hébergés) dans le conseil de processus via le champ `services` (D11).
- Séparer `locked` de `Level` : `locked` porte le refus, `Level` porte la gravité. L'interface cesse d'afficher « Vital » pour un service seulement verrouillé (D7). `Critical` reste réservé aux composants dont l'arrêt casse Windows.
- Documenter, pour chaque règle qui propose une action, la source de la justification. Une règle sans source reste `Unknown` et ne propose rien.
- Tests d'invariant : clés uniques, clés en minuscules, longueur minimale pour les modes non exacts, toute action adossée à une justification non vide, tout `Critical` présent dans les listes de verrouillage, tout service verrouillé présent dans la table.

**Critère de sortie.**

- Audit automatique des 405 règles (262 + 93 + 50) : invariants vérifiés, zéro doublon, zéro clé en majuscules, zéro mode de correspondance ambigu.
- Test qui prouve qu'une entrée de démarrage dont la commande contient `php` ne correspond plus à la règle `hp`.
- `grep` : plus aucun `contains` sur `command` dans la fonction de conseil de démarrage.
- Le nombre de règles sans source est affiché dans le journal de l'étape.

**Test de réversion.** Le test « `php` ≠ `hp` » échoue si la correspondance par sous-chaîne revient ; le test d'unicité échoue sur un doublon réintroduit.

**Effort indicatif.** 2 jours.

---

### Étape 5 — Interface de décision

**Objectif.** Que l'utilisateur comprenne pourquoi un élément est listé, ce que l'action change, ce qu'elle ne change pas, et ce qui est réversible — avant de confirmer.

**Périmètre.** `web/app.js`, `web/index.html`, `web/style.css`.

**Changements.**

- Renommer les sélections de masse : « Tout le sans risque » devient « Sélectionner les éléments réversibles à gain différé ». Bannir « sans risque », « aucun risque réel », « gain net » (D8, D4).
- Séparer partout « tout de suite » et « au prochain démarrage », avec badge mesuré / estimé / non chiffrable.
- Par élément : source détectée, état actuel, effet immédiat ou différé, réversibilité, justification. Regrouper ces informations dans un panneau dépliable plutôt que dans une colonne unique tronquée.
- Aucune action à risque présélectionnée ; `Kill` visuellement distinct et jamais dans un lot.
- Confirmation détaillée : nombre d'actions réversibles, nombre d'actions irréversibles, nom des processus concernés, rappel de la perte de travail non enregistré.
- Navigation clavier cohérente, lisibilité en petit écran, contrastes suffisants.

**Critère de sortie.**

- `grep -rn "sans risque\|aucun risque réel\|gain net" src/web/` → aucun résultat.
- En mode démo, la revue visuelle consignée montre : une sélection vide par défaut, un total séparé immédiat/différé, un `Kill` signalé comme irréversible.
- Les parcours charger / filtrer / sélectionner / appliquer / restaurer / exporter sont exercés en démo et consignés.

**Test de réversion.** Un test de chaînes interdit les formulations bannies ; il échoue si l'une revient.

**Effort indicatif.** 1,5 jour.

---

### Étape 6 — Recette et documentation

**Objectif.** Livrer avec des preuves, pas avec des promesses.

**Périmètre.** Dépôt entier, `README.md`, `SAFETY.md`, section 7 de ce plan.

**Changements.**

- Séquence complète : `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release`. Consigner chaque code retour, y compris les avertissements restants.
- Parcours complet en `--demo` : chargement, filtres, sélection, confirmation, application, restauration, export. Consigner ce qui a été observé, pas ce qui était attendu.
- Sur Windows réel : exécution en lecture seule (`--no-admin`) et vérification de l'inventaire. Pour l'application réelle, **une VM ou un instantané uniquement**, jamais la machine de tous les jours.
- Mettre à jour README et SAFETY en fonction du comportement final. Ne rien y annoncer qui n'ait été exécuté.
- Remplir le journal (section 7) et fermer les défauts un par un, avec le test correspondant.

**Critère de sortie.**

- Les quatre commandes ci-dessus sont au vert, ou leurs échecs sont documentés avec la raison et l'étape concernée.
- Le journal contient une ligne par commande, avec code retour et observation.
- Aucun défaut P0 ou P1 du registre n'est encore ouvert sans justification écrite.
- La section « ce qui n'a pas été testé » existe dans le README.

**Test de réversion.** Sans objet : c'est l'étape de preuve. Son échec rouvre l'étape concernée.

**Effort indicatif.** 0,5 jour.

## 5. Matrice de tests de régression

À couvrir au fil des étapes, pas à la fin. Une ligne sans test nommé reste ouverte.

**Protections et verrous.** Services critiques Windows, processus système, composants de sécurité, noyau, shell, services verrouillés mais non vitaux (vérifier qu'ils ne sont plus étiquetés « Vital »).

**Éléments inconnus.** Nom non répertorié, chemin non répertorié, service tiers sans précision, processus sans fenêtre au-dessus de 150 Mo : aucune action proposée.

**Éléments déjà configurés.** Service déjà en manuel, démarrage déjà désactivé, tâche déjà inactive : conseil « déjà bien configuré », aucune action.

**Identité et péremption.** PID terminé entre collecte et action, PID recyclé avec un autre exécutable, chemin d'image modifié, heure de création différente, service reconfiguré entre-temps, tâche supprimée.

**Données RAM incomplètes.** Taille manquante, hôte sans service identifiable, valeurs nulles, cache non fourni : affichage « indisponible », jamais zéro.

**Fusions de services.** `svchost.exe` hébergeant plusieurs services, répartition approximative, total de lot non additif, arrêt d'un service d'un hôte partagé.

**Fermeture normale et terminaison.** La même application ne doit pas être présentée comme sûre à terminer sans avertissement ; texte de confirmation distinct selon le type d'action.

**Rollback.** Point de restauration correct, restauration partielle, restauration totale, échec d'application, action irréversible absente du point.

**Surface HTTP.** `Host` inconnu, origine trompeuse par suffixe, absence de jeton, jeton invalide, corps trop volumineux, plus de 400 actions, méthode non autorisée.

**Règles.** Unicité, casse, mode de correspondance, absence de source, faux positif `php` déclenchant `hp`.

## 6. Risques et parades

| Risque | Parade |
|---|---|
| Casser un service utile en le passant en manuel | Verrou côté serveur, justification par règle, restauration testée, mode démo pour répéter |
| Faux positif d'une règle de correspondance | Correspondance exacte ou par nom d'exécutable, test dédié par famille de faux positifs connus |
| Exposition locale résiduelle malgré le durcissement | Validation `Host` + jeton sur toutes les routes API + CSP ; test d'intégration rejoué à chaque étape |
| Rollback incomplet après application partielle | Le point ne contient que les actions réussies ; l'interface affiche les échecs et n'annonce pas une restauration totale |
| Régression de performance de la collecte | Mesure de `duration_ms` avant/après chaque étape touchant `win.rs`, consignée dans le journal |
| Plan qui redevient une liste d'intentions | Gabarit obligatoire, test de réversion par changement, journal avec codes retour réels |

## 7. Journal

### Vérifications de référence (2026-11-13)

| Commande | Code retour | Observation |
|---|---|---|
| `cargo test` (depuis `ramscope/`) | 0 | 5 tests, 5 réussis |
| `cargo fmt --check` | 1 | 66 hunks, 7 fichiers |
| `cargo clippy --all-targets -- -D warnings` | non exécuté | — |
| `cargo build --release` | non exécuté | — |
| Lancement de l'application | non exécuté | — |

### 7.1 Étape 0 — contrôles exécutés (2026-11-13)

| Commande | Code retour | Observation |
|---|---|---|
| `cargo fmt` | 0 | 7 fichiers reformatés |
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 10 tests, 10 réussis : 5 d'origine et 5 dans `plan.rs` |
| `cargo build --release` | 0 | binaire produit dans `target/release/ramscope.exe` |
| `grep -nE "unsafe\|cfg\(windows\)\|crate::ffi\|crate::win" src/plan.rs` | 1 | aucune occurrence |
| `ramscope.exe --demo --no-browser --port 8917` | arrêt propre | `/api/health` et `/api/snapshot` répondent ; arrêt par `/api/quit`, journal « RamScope arrêté » |

Non exécuté à cette étape : application réelle des actions sur la machine (voir étape 6), `cargo clippy`.

### 7.2 Étape 1 — contrôles exécutés (2026-11-13)

| Commande | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 15 tests, 15 réussis, dont 5 nouveaux dans `server.rs` |
| `cargo build --release` | 0 | binaire produit en 9 s |
| `GET /` avec `Host: attaquant.tld` | 403 | page refusée, aucun contenu servi |
| `GET /api/snapshot` avec `Host: attaquant.tld` | 403 | refus avant même le jeton |
| `GET /api/snapshot` sans jeton | 403 | jeton désormais exigé en lecture |
| `GET /api/snapshot` avec jeton | 200 | inventaire de démonstration |
| `POST /api/apply` sans jeton | 403 | origine valide, jeton absent |
| `POST /api/apply` avec jeton, `Host` et `Origin` attaquants | 403 | refus sur l'hôte |
| `POST /api/apply` avec jeton et origine valide | 200 | action appliquée en démonstration |
| En-têtes de `/` | — | CSP, `X-Frame-Options: DENY`, `Cross-Origin-Resource-Policy: same-origin` présents |

Non exécuté à cette étape : lancement avec `--host 0.0.0.0` (le refus est couvert par un test unitaire ; un lancement réel afficherait une boîte de dialogue bloquante sur la machine), vérification visuelle de l'interface dans un navigateur.

### Avancement

| Étape | État | Défauts traités | Vérification consignée |
|---|---|---|---|
| 0 — Socle vérifiable | terminée (2026-11-13) | D13, D14 | voir 7.1 |
| 1 — Surface locale | terminée (2026-11-13) | D1, D12 | voir 7.2 |
| 2 — Identité et irréversibilité | à faire | D2, D8 | — |
| 3 — Vérité des chiffres | à faire | D3, D4, D5, D6 | — |
| 4 — Règles et provenance | à faire | D7, D9, D10, D11 | — |
| 5 — Interface de décision | à faire | D8 (volet interface) | — |
| 6 — Recette et documentation | à faire | — | — |

Une ligne « à faire » ne devient « terminée » qu'avec la commande, son code retour et le chemin de l'artefact concerné.

## 8. Hors périmètre

- Nettoyeur automatique, planification, exécution sans confirmation.
- Base de règles communautaire ou téléchargée.
- Refonte graphique complète, changement de framework front.
- Télémétrie, envoi de rapport à un serveur distant.
- Signature de code et distribution (sujet séparé, après stabilisation).

## 9. Annexe — commandes de vérification

Depuis `ramscope/`, sous Windows :

```text
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Contrôles de la surface HTTP, avec `PORT` et `TOKEN` de la session de test :

```text
curl.exe -s -o NUL -w "%{http_code}" -H "Host: attaquant.tld" http://127.0.0.1:PORT/api/snapshot
curl.exe -s -o NUL -w "%{http_code}" -H "Host: attaquant.tld" http://127.0.0.1:PORT/
curl.exe -s -o NUL -w "%{http_code}" -X POST -H "Content-Type: application/json" -H "Origin: http://attaquant.tld:PORT" -H "Host: attaquant.tld:PORT" --data "{\"actions\":[]}" http://127.0.0.1:PORT/api/apply
```

Contrôles de formulation et de constantes :

```text
grep -rn "sans risque\|aucun risque réel\|gain net" src/web/
grep -rn "150 \* 1024 \* 1024\|60 \* 1024 \* 1024\|40 \* 1024 \* 1024" src/
```

Rappel : aucune de ces commandes ne doit être présentée comme exécutée avant de l'être. Le journal est la seule source de vérité sur ce point.
