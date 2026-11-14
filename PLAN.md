# Plan d'action RamScope

Objectif : rendre l'inventaire mémoire et les conseils de démarrage plus exacts, compréhensibles et prudents, sans transformer RamScope en nettoyeur automatique. Toute modification doit rester réversible, explicite, et ne doit jamais se baser sur un seul nom de processus ou un seuil de RAM.

Ce document est la version 3.2 du plan. La version 2 corrigeait la version 1, dont l'étape 0 annonçait comme livrés un README de présentation et un `SAFETY.md` qui n'existaient pas dans le dépôt. Un plan qui déclare terminé ce qui n'existe pas ne peut pas servir de référence : la règle de véracité ci-dessous existe pour empêcher que cela se reproduise.

La version 3 (2026-11-13, audit du code contre le registre) apporte cinq amendements : (1) le versionnage : à l'audit, seul `README.md` était suivi par git, le projet et ce plan étaient hors de tout versionnage — un commit de baseline (`e5198a9`) existe désormais et sert de référence à la section 1 ; (2) l'agrégation des gains par hôte est précisée partout avec la même sémantique — somme des parts plafonnée à la RAM de l'hôte — car le critère de sortie de l'étape 3 se contredisait sur ce point ; (3) deux défauts du rapport exporté sont enregistrés (`D15`, `D16`) ; (4) la section 1 est révisée, datée et épinglée au commit de baseline ; (5) le décompte de règles inclut la table `PROCESSES` (602 règles, pas 405 ; 601 après fusion du doublon `eventsystem`/`eventsysevent` à l'étape 4, voir 7.6).

La version 3.1 (2026-11-14, relecture de la v3 contre le code) corrige quatre points : (1) le plafond `min(somme des parts, RAM de l'hôte)` ne peut jamais mordre sur les parts réelles — toutes égales à `host_ram / shared` — et laisse intact le vrai vice : un hôte dont un service reste nécessaire ne libère rien ; la règle devient « tous les services de l'hôte dans le lot, sinon zéro » ; (2) la section 1 est remise à l'état d'après l'étape 1, sans mélange d'époques ; (3) `D16` est rattaché à l'étape 3, qui le corrige, l'étape 5 n'en assurant que la garde ; (4) `demo.rs` entre aux périmètres des étapes 2 et 3. Elle consigne aussi la vérification navigateur de l'étape 1 (7.2).

La version 3.2 (2026-11-14, revue post-livraison) ne rouvre aucune étape fonctionnelle. Elle corrige un décompte de lignes périmé, consigne le point de contrôle `c4fc786` qui regroupe les étapes 2 à 5, enregistre trois dettes de durcissement (`D21`–`D23`) et ajoute un lot de clôture priorisé (section 8). Aucun code n'a été modifié par cette revue.

## 0. Comment lire et tenir ce plan

Chaque étape suit le même gabarit, et aucun de ces cinq points ne peut être omis :

1. **Objectif** — le comportement observable à changer, en une phrase.
2. **Périmètre** — les fichiers et fonctions réellement touchés.
3. **Changements** — ce qui est modifié, dans l'ordre où on le fait.
4. **Critère de sortie** — une condition qu'on peut vérifier sans interprétation : commande, code retour, sortie HTTP, test nommé, `grep`.
5. **Test de réversion** — le test qui échoue si le changement est annulé. Sans ce test, le critère de sortie n'est pas considéré comme couvert.

Règles de tenue :

- Une étape = un commit lorsque le découpage est possible. Le message cite les identifiants de défauts traités (`D1`, `D4`…).
- Exception historique : les étapes 0 et 1 ont été livrées avant la mise sous versionnage et partagent le commit `e5198a9`, postérieur à leur exécution. Les étapes 2 à 5, développées avant leur premier commit, partagent le point de contrôle `c4fc786` ; ce regroupement est consigné ici et ne change pas la règle pour les étapes suivantes. Ces étapes n'ont pas de test de réversion rejouable séparément.
- Un élément n'est marqué « terminé » que si le journal (section 7) contient le chemin de l'artefact et le code retour de la commande. « À consigner après exécution » n'est plus acceptable : soit la ligne existe avec son code retour, soit l'étape reste « à faire ».
- Une étape qui échoue reste ouverte. On ne passe pas à la suivante en laissant un contrôle rouge derrière soi.
- Les chiffres affichés restent annotés comme mesurés, estimés ou indisponibles. Une estimation n'est jamais présentée comme une mesure.
- Si l'identité d'un élément est incertaine, la recommandation est « rien à faire » ou « vérifier manuellement », jamais une action automatique.
- Ne pas prétendre avoir testé ce qui n'a pas été exécuté. La compilation Windows est vérifiable sur cette machine ; l'exécution réelle des actions exige une VM ou un instantané.
- Avant chaque étape, confronter la section 1 au journal et au code : en cas d'écart, le journal prime et la section 1 est révisée avec une nouvelle date. Une affirmation de la documentation (README, `SAFETY.md`) qui n'est pas encore vraie dans le code est un défaut de la documentation, pas du code attendu.

## 1. État vérifié du dépôt (point de référence)

Constat initial établi le 2026-11-13 en cours d'après-midi, sans modification du code. **Révisé le 2026-11-14 après le lot 8.1/8.2** ; le point de référence reste le commit `e5198a9`, postérieur aux étapes 0 et 1 (section 7.0). Les lignes ci-dessous décrivent l'arbre à cette révision.

| Élément | Constat révisé (2026-11-14) |
|---|---|
| Volume | 7 875 lignes de Rust et de JS (hors `index.html`, 62 lignes, et `style.css`, 203) : `win.rs` 1 830, `rules.rs` 1 365, `demo.rs` 1 191, `app.js` 884, `server.rs` 640, `ffi.rs` 480, `model.rs` 459, `plan.rs` 451, `util.rs` 180, `report.rs` 202, `main.rs` 102, `platform.rs` 91 |
| Base de règles | 601 règles : 261 services, 197 processus, 93 entrées de démarrage, 50 tâches. Le plan annonçait 602 ; l'audit de l'étape 4 a révélé le doublon `eventsystem` / `eventsysevent`, fusionné (7.6) |
| Tests | 43 fonctions de test : 13 dans `plan.rs`, 12 dans `rules.rs`, 10 dans `server.rs`, 2 dans `demo.rs`, 1 dans `report.rs`, 5 dans `win.rs` ; `cargo test` code retour 0 sur les quarante-trois, consigné au journal (7.12) |
| `cargo fmt --check` | **code retour 0** : D13 corrigé et consigné au journal (7.1) |
| `cargo clippy` | **code retour 0** : `cargo clippy --all-targets -- -D warnings`, consigné au journal (7.6) |
| Build release | **code retour 0** : binaire produit, consigné au journal (7.6) |
| Rapport présent | `ramscope/ramscope-data/rapports/ramscope-2026-11-13-19-14-44.json` porte `"demo": true` : ce sont des données simulées, pas une mesure de la machine |
| `README.md` | réécrit à l'étape 0 ; `Host`, origine et jeton sont couverts par des tests nommés dans `server.rs`. La validation serveur et les garde-fous moteur sont décrits (deux verrous, étape 2) ; les gains typés (étape 3), le cadenas distinct du niveau et la provenance de famille (étape 4), puis l'interface de décision (étape 5) et la recette réelle en lecture seule (étape 6) sont documentés |
| `SAFETY.md` | créé à l'étape 0 ; complété par les limites des gains typés (étape 3), la séparation verrou/gravité et les limites de provenance des règles (étape 4), puis les garde-fous de sélection (étape 5) et la lecture des tâches sans administrateur (étape 6) |
| Versionnage | avant le commit de baseline : un seul fichier suivi (`README.md`) ; remote réel (`Roger364/q-memory-manager`) ; l'ancienne mention de dépôt était erronée et a été corrigée le 2026-11-14 |
| Compilation Windows | `cargo test` compile `win.rs` : la cible Windows est vérifiable ici. Seule l'application réelle des actions demande une VM ou un instantané |

Conséquence de méthode : la vérification n'est plus regroupée en fin de parcours. Chaque étape se termine par son propre contrôle, et `cargo fmt --check` doit repasser à 0 dès l'étape 0 — sinon l'étape 6 échouerait après six étapes de travail non validées.

Écart relevé à la révision précédente, clos par correction de la documentation : `README.md` affirmait le refus « par le moteur et par le serveur », alors que `server.rs` ne validait pas lui-même. La vérification montre que `win.rs` — le moteur — appelle `plan::validate` (ligne 1462) avant d'agir : le refus est réel quel que soit le point d'entrée, mais il n'existe pas de seconde validation dans `server.rs`. Le README disait alors « par le moteur, quel que soit le point d'entrée » ; l'étape 2 a ajouté la validation serveur avant `backend.apply` (7.4).

## 2. Invariants de conception

Ces règles priment sur toute considération d'ergonomie ou de gain affiché.

1. **La RAM est une observation, pas une preuve de danger.** Un processus gourmand n'est pas un processus inutile.
2. **Verrou et gravité sont deux notions distinctes.** `locked` signifie « RamScope refuse d'agir » ; `Level` signifie « à quel point c'est important pour Windows ». Un service verrouillé n'est pas forcément vital et ne doit pas être affiché comme tel.
3. **Les gains ne s'additionnent pas naïvement entre services d'un même hôte.** La RAM d'un `svchost.exe` est partagée : le gain différé d'un hôte n'est compté que si **tous** ses services figurent dans le lot — il vaut alors la RAM de l'hôte, en estimation — et zéro sinon. Le total du lot est la somme des hôtes ainsi complets. Tant que l'hôte reste en vie, aucun service de cet hôte n'apporte de gain immédiat ; un service retiré d'un hôte qui redémarre pour les autres ne libère rien. Le plafond `min(somme des parts, RAM de l'hôte)` retenu par la v3 ne suffisait pas : les parts réelles sont égales (`host_ram / shared`) et le plafond ne peut jamais mordre.
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
| D1 | P0 | `server.rs` — `handle()`, injection du jeton dans `index.html` | Le jeton de session est servi dans la page, l'en-tête `Host` n'est jamais validé, et le contrôle d'origine fait `origin.ends_with(host)`. Une page malveillante résolue vers `127.0.0.1` (DNS rebinding) lit `/`, récupère le jeton, puis poste sur `/api/apply` : son origine se termine par le même hôte, le contrôle passe. Corrigé à l'étape 1 (7.2) |
| D2 | P0 | `win.rs` — `kill_process`, `model.rs` — `Action::Kill` | L'action ne transporte que `pid` et `name`. Le contrôle porte sur le nom protégé, `pid > 8` et l'auto-terminaison, pas sur l'identité du processus. Un PID recyclé entre la collecte et le clic fait terminer le mauvais programme. Corrigé à l'étape 2 (7.4) : `image_path` et `created_at` sont revérifiés sur le PID rouvert avant `TerminateProcess` |
| D3 | P1 | `web/app.js` — `selTotals()`, `confirmApply()` | Le gain différé d'un service est ajouté au gain « tout de suite » quand l'option « Libérer la RAM maintenant » est cochée, puis le total différé est mis à zéro. Du différé est présenté comme de l'immédiat, et le gain des entrées de démarrage sélectionnées disparaît. Corrigé à l'étape 3 (7.5) : conversion supprimée, deux lignes distinctes, delta mesuré après l'arrêt |
| D4 | P1 | `rules.rs` — `process_advice` | Toute application tierce sans fenêtre au-dessus de 150 Mo reçoit une action `Kill`. C'est une action destructive fondée sur un seuil de RAM et un état de fenêtre, ce que le plan interdit par ailleurs. Corrigé à l'étape 3 (7.5) : seuil et branches heuristiques supprimés, processus non répertorié sans fenêtre → `Unknown`, aucune action |
| D5 | P1 | `rules.rs` — `startup_advice` | Un gain de 60 Mo (ou 40 Mo selon la branche) est fabriqué quand aucune mesure n'existe. Une valeur inventée présentée comme un gain estimé. Corrigé à l'étape 3 (7.5) : constantes supprimées, gain `unavailable` → « non chiffrable » |
| D6 | P1 | `win.rs` — collecte des services, `plan_gain_next_boot` | La part d'un service vaut `host_ram / shared`, puis ces parts sont additionnées. Trois services d'un même hôte produisent un total supérieur à la RAM libérable. Le plafond `min(somme, RAM)` proposé par la v3 est inopérant sur des parts égales ; la v3.1 prescrit « tous les services de l'hôte ou zéro ». Corrigé à l'étape 3 (7.5) : `plan::total_gains` et l'interface appliquent la règle, hôte complet seulement |
| D7 | P1 | `rules.rs` — `is_protected_service` / `service_advice` | Les services protégés sortent en `Level::Critical`, donc étiquetés « Vital », alors que la table les classe `Keep` : `wuauserv`, `themes`, `msiserver`, `defragsvc`, `w32time`. Protection et gravité sont confondues. Corrigé à l'étape 4 (7.6) : `Advice.locked` porte le refus, `Level` reste celui de la table ; `wuauserv` reste « À garder » + cadenas, `Critical` réservé aux composants vitaux |
| D8 | P1 | `main.rs` — aide, `web/app.js` — en-tête de confirmation | L'aide affirme « Toutes les modifications sont réversibles » et la modale commence par « Rien n'est supprimé », alors que `Action::Kill` n'a pas d'annulation. La modale se contredit deux lignes plus bas. Corrigé à l'étape 2 (7.4) : aide et modale distinguent les changements de configuration de la terminaison. Volet interface fermé à l'étape 5 (7.7) : sélections de masse renommées, `Kill` hors des lots et confirmation comptant réversible/irréversible |
| D9 | P2 | `rules.rs` — `startup_advice`, `service_advice` | Correspondance par sous-chaîne sur le nom et sur la ligne de commande entière, avec des clés courtes (`hp`, `wps`, `wsl`, `zoom`, `plex`). Une règle peut s'appliquer à un programme qui n'a rien à voir. Corrigé à l'étape 4 (7.6) : modes explicites et comparaisons exactes ; test `php` ≠ `hp` |
| D10 | P2 | `rules.rs` — table `SERVICES` | La clé `ngcctorSvc` est la seule en majuscules alors que le nom du service est mis en minuscules : la règle ne peut jamais correspondre. Corrigé à l'étape 4 (7.6) : `ngcctorsvc` ; l'audit de casse couvre désormais les quatre tables |
| D11 | P2 | `rules.rs` — `process_advice` | La branche spécifique `svchost.exe` est inatteignable, `svchost.exe` figurant dans `PROTECTED_PROCESSES` et traité en amont. Corrigé à l'étape 4 (7.6) : branche supprimée, liste des services hébergés reprise dans le conseil, test dédié |
| D17 | P2 | `rules.rs` — table `SERVICES` | La table comptait `eventsystem` et `eventsysevent`, deux clés pour le même service COM+ Event System ; la seconde ne correspond à aucun service Windows. L'audit d'unicité de l'étape 4 l'a révélée. Corrigé à l'étape 4 (7.6) : doublon fusionné, total réel 601 règles et non 602 |
| D12 | P2 | `main.rs`, `server.rs` — option `--host` | `--host 0.0.0.0` est accepté alors que l'aide annonce une écoute en boucle locale uniquement. Cela exposerait jeton et actions au réseau local. Corrigé à l'étape 1 (7.2) : seules les adresses de boucle locale sont acceptées |
| D13 | P2 | Dépôt entier | `cargo fmt --check` échouait : 66 hunks dans 7 fichiers. Corrigé dans l'arbre de travail (code retour 0 à 23 h 47) ; fermeture au journal à l'étape 0 |
| D14 | P2 | `README.md`, `SAFETY.md` | Le plan v1 les déclarait livrés ; le README était un stub de deux lignes et `SAFETY.md` était absent. Les deux existent depuis l'étape 0, avec la réserve de vérité notée en section 1 |
| D15 | P1 | `report.rs` — `build()`, colonnes « Gain » | Le rapport exporté additionne gain immédiat et gain différé dans une même cellule (`gain + gain_next_boot`), pour les services comme pour le démarrage. Même vice que D3, dans l'export ; l'étape 3 cite le rapport dans son critère de sortie sans l'avoir à son périmètre. Le tri et le total de section de `web/app.js` (lignes 289 et 394-395) mélangent les mêmes deux gains. Corrigé à l'étape 3 (7.5) : colonnes séparées dans le rapport et l'interface, tri sur le gain immédiat |
| D16 | P2 | `report.rs` — ligne « rien à optimiser » | « Rien à optimiser sans risque » emploie une formulation que l'étape 5 bannit, hors du périmètre de son contrôle (limité à `src/web/`). À corriger à l'étape 3 avec le reste du rapport ; l'étape 5 n'en assurera que la garde de régression. La justification `rules.rs` « Désactivable sans risque » est conservée : elle qualifie un élément précis, pas une sélection de masse. Corrigé à l'étape 3 (7.5) : « Rien à optimiser : configuration déjà saine » |
| D18 | P2 | `web/app.js` — fermeture du fond de modale | Un clic sur un bouton qui ouvre une modale après un appel réseau (export) peut être reçu par le masque apparu entre l'appui et le relâchement : la modale s'ouvre puis se referme aussitôt. Observé pendant la revue de l'étape 5. Corrigé à l'étape 5 (7.7) : fermeture seulement si l'appui a commencé sur le fond ; test `le_fond_de_modale_ne_se_ferme_pas_sur_un_clic_herite` |
| D19 | P2 | `web/style.css` — media query petit écran | À 390 px de large, `.hero` conservait une première colonne de 320 px et des cartes de 150 px : la grille dépassait la fenêtre et la deuxième carte était coupée. Observé pendant la revue de l'étape 5. Corrigé à l'étape 5 (7.7) : `.hero` sur une colonne, machine pleine largeur et chemins d'export coupés ; capture `docs/evidence/ramscope-step5-mobile-fixed.png` |
| D20 | P2 | `win.rs` — `collect_tasks` | En lecture seule, `C:\\Windows\\System32\\Tasks` est refusé par les ACL : l'interface affichait 0 tâche alors que Windows en expose 201. Observé à la recette de l'étape 6. Corrigé à l'étape 6 (7.8) : repli sur `schtasks /query /xml`, même analyse XML, test `decode_une_tache_xml` ; 201 tâches et 167 activées retrouvées, identiques à `Get-ScheduledTask` |
| D21 | P2 | `win.rs` — `tasks_from_schtasks_xml`, `run_schtasks` | L'outil est résolu par son nom (`Command::new("schtasks.exe")`). Sous Windows, un `schtasks.exe` placé dans le répertoire courant ou le `PATH` peut être exécuté à la place de l'outil système, y compris pendant une collecte en lecture seule. Découvert par la revue post-livraison. **Corrigé le 2026-11-14** (8.1) : chemin absolu via `system32_path`, test `chemins_des_outils_systeme_sont_absolus`. |
| D22 | P2 | `win.rs` — `collect_tasks` | Le repli `schtasks` n'est déclenché que si `out.is_empty()`. Si un seul fichier XML reste lisible et que les autres sont refusés, la liste est silencieusement partielle. Découvert par la revue post-livraison. **Corrigé le 2026-11-14** (8.2) : repli dès qu'une lecture est refusée, fusion par chemin, déduplication ; tests `le_repli_couvre_une_acl_partielle` et `fusionne_les_taches_par_chemin_sans_doublon`. |
| D23 | P2 | `web/app.js`, `web/index.html`, `server.rs` — CSP | La CSP conservait `'unsafe-inline'` pour les styles et un gestionnaire inline (`elevate()`). L'étape 1 annonçait un resserrage à l'étape 5, qui n'a pas été fait. Découvert par la revue post-livraison. **Corrigé le 2026-11-14** (8.4) : script inline supprimé, jeton dans une meta, écouteur `btnElevate` dans `app.js`, `script-src 'self'` ; `style-src 'unsafe-inline'` reste justifié par les styles calculés. |

D13 et D14 sont corrigés par l'étape 0 (contrôles en 7.1) : le formatage est conforme et la documentation existe. D1 et D12 sont corrigés par l'étape 1 (contrôles en 7.2). D2 et D8 sont corrigés par l'étape 2 (7.4), le volet interface de D8 à l'étape 5 (7.7). D3, D4, D5, D6, D15 et D16 sont corrigés par l'étape 3 (7.5). D7, D9, D10, D11 et D17 sont corrigés par l'étape 4 (7.6). D18 et D19 sont corrigés par l'étape 5 (7.7). D20 est corrigé par l'étape 6 (7.8). D21 et D22 sont corrigés par le lot 8 (7.10). D23 est corrigé par le lot 8.4 (7.12).

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

**État.** Terminée le 2026-11-14 (contrôles HTTP du 13, contrôle navigateur du 14) — codes retour en 7.2.

**Objectif.** Rendre l'API locale inatteignable depuis une page web tierce, y compris par rebinding DNS.

**Périmètre.** `server.rs` — `handle()`, `serve()`, `send()` ; `main.rs` — options ; `web/index.html` si le jeton doit sortir du HTML.

**Changements.**

- Valider `Host` avant tout traitement : liste exacte `127.0.0.1:<port>` et `localhost:<port>`. Toute autre valeur → 403, y compris sur les pages statiques (D1). La variante `[::1]` n'a pas été retenue : le format d'URL utilisé ne la produit pas.
- Remplacer `origin.ends_with(host)` par une comparaison stricte après analyse : schéma, hôte, port. Pas de sous-chaîne, pas de suffixe.
- Exiger le jeton sur `GET /api/snapshot` comme sur les autres routes `/api/*` ; la page le transmet en en-tête. L'injection `__TOKEN__` reste : la validation de `Host` empêche qu'elle soit servie à un hôte étranger.
- Comparer le jeton sans sortie anticipée dépendante de la position du premier octet différent.
- En-têtes de durcissement : `Content-Security-Policy: default-src 'self'`, `X-Frame-Options: DENY`, `Cross-Origin-Resource-Policy: same-origin`, en plus de `nosniff`, `no-referrer` et `no-store` déjà présents.
- Fermer `--host` (D12) : seules les adresses de boucle locale (`127.0.0.1`, `localhost`) sont acceptées, toute autre valeur refuse le démarrage, et l'aide décrit exactement ce comportement. La variante « retirer l'option » n'a pas été retenue : le nom est conservé, les valeurs sont fermées.

**Livré le 2026-11-13.** `--host` n'accepte plus que `127.0.0.1` et `localhost` ; toute autre valeur refuse le démarrage (test unitaire). `Host` est validé exactement pour toutes les routes, pages comprises. L'origine est comparée en schéma, hôte et port, sans suffixe. Le jeton est exigé sur tout `/api/` et comparé sans sortie anticipée. CSP, `X-Frame-Options` et `Cross-Origin-Resource-Policy` sont ajoutés. Le jeton reste dans la page, ce que couvre la validation de `Host` ; la CSP conserve `'unsafe-inline'` pour les styles et le gestionnaire inline restant ; ce resserrage n'a pas été fait à l'étape 5 et reste ouvert.

**Critère de sortie.**

- `curl.exe -s -o NUL -w "%{http_code}" -H "Host: attaquant.tld" http://127.0.0.1:<port>/api/snapshot` → `403`.
- Même requête sur `/` → `403`, et la réponse ne contient ni jeton ni inventaire.
- `POST /api/apply` avec `Origin: http://attaquant.tld:<port>` et `Host: attaquant.tld:<port>` → `403`.
- `POST /api/apply` sans jeton, depuis l'origine réelle → `403`.
- L'interface normale, ouverte via `127.0.0.1` dans Chromium, charge sans erreur de console et applique une action en mode démo (consigné en 7.2).

**Test de réversion.** Un test d'intégration qui envoie un `Host` inconnu et attend 403 ; il échoue si le contrôle est retiré. Un test unitaire sur la fonction de comparaison d'origine couvre le cas du suffixe trompeur.

**Effort indicatif.** 1 jour.

---

### Étape 2 — Identité des actions et irréversibilité (P0)

**Objectif.** Garantir qu'une action s'applique à la cible qui a été montrée et confirmée, et que l'utilisateur sait ce qui n'est pas annulable.

**Périmètre.** `model.rs` — `Action`, `RollbackEntry` ; `win.rs` — `kill_process`, `apply`, `apply_service`, `apply_startup` ; `rules.rs` — construction des actions ; `server.rs` — validation avant `backend.apply` ; `web/app.js` — confirmation ; `demo.rs` — identité simulée et cas divergent.

**Changements.**

- `Action::Kill` transporte `pid`, `name`, `image_path` et `created_at` (heure de création du processus). Avant `TerminateProcess`, rouvrir le PID en `PROCESS_QUERY_LIMITED_INFORMATION`, comparer chemin et heure de création ; en cas d'écart, refuser avec un message explicite (« l'identité du processus a changé depuis l'analyse ») (D2).
- Pour `Service`, `Startup` et `Task`, relire l'état courant juste avant d'écrire et refuser si le type de démarrage, l'activation ou l'existence ne correspond plus à ce qui a été confirmé.
- Étendre `plan::validate` (créé à l'étape 0) au snapshot et à l'identité, et l'appeler côté serveur avant `backend.apply` : verrou des services protégés, verrou des processus protégés, `pid > 8`, auto-terminaison, identité, doublons dans le lot. `win.rs` conserve ses garde-fous : deux verrous, pas un.
- `demo.rs` : doter le processus simulé d'une identité (`image_path`, `created_at`) et rendre reproductible un cas divergent, sans quoi le test d'intégration ci-dessous ne peut pas s'exécuter.
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

**État.** Terminée le 2026-11-13 — contrôles et codes retour en 7.5.

**Objectif.** Qu'aucun gain affiché ne soit faux : ni inventé, ni additionné à tort, ni présenté comme immédiat quand il est différé.

**Périmètre.** `model.rs` — `Advice`, `MemoryInfo` ; `rules.rs` — `service_advice`, `process_advice`, `startup_advice` ; `win.rs` — `memory_info`, collecte des services, agrégation du plan ; `web/app.js` — totaux et rendu ; `report.rs` — colonnes « Gain » du rapport exporté (D15, D16) ; `demo.rs` — agrégation simulée.

**Changements.**

- Remplacer les entiers nus de gain par une mesure typée : valeur, nature (`mesuré`, `estimé`, `indisponible`), portée (`privé`, `working set`, `part d'hôte`), additivité (oui/non). Le champ reste sérialisable pour ne pas casser le rapport exporté plus que nécessaire.
- Calculer le total d'un lot par hôte : un hôte ne compte que si **tous** ses services figurent dans le lot ; son gain vaut alors la RAM de l'hôte (estimation), et zéro sinon. Jamais la somme des parts `host_ram / shared`, même plafonnée : les parts réelles sont égales et un hôte maintenu en vie ne libère rien (D6). Le gain immédiat d'un service d'un hôte partagé reste nul tant que l'hôte reste en vie.
- Supprimer les constantes de repli 60 Mo et 40 Mo (D5). Sans mesure : `indisponible`, affiché « non chiffrable ».
- Supprimer la règle des 150 Mo (D4). Un processus tiers non répertorié sans fenêtre devient `Unknown` avec « vérifier manuellement », pas une action.
- Supprimer la conversion différé → immédiat de `stopNow`. Remplacer par une mesure : mémoire disponible avant, application, attente bornée, mémoire disponible après ; afficher le delta comme **mesuré**, et le reste comme **estimé** (D3). Un service arrêté ne libère pas sa part d'un hôte partagé : ne pas l'annoncer.
- `report.rs` : séparer gain immédiat et gain différé dans le rapport exporté, jamais une somme des deux dans une même cellule ; reformuler « Rien à optimiser sans risque » en « Rien à optimiser : configuration déjà saine » (D15, D16).
- Étiqueter `compressed` pour ce qu'il est — le working set du processus « Memory Compression », une approximation — ou utiliser le compteur système correspondant si disponible.
- Vérifier que `used = total - available` est bien la définition annoncée dans l'interface, et que `commit`, `kernel` et `cache` portent leur nom Windows exact dans l'infobulle.

**Critère de sortie.**

- Test unitaire, sémantique « tous les services ou zéro » : hôte de 400 Mo avec quatre services de 100 Mo — un seul sélectionné → 0, trois → 0, les quatre → 400 Mo ; deux hôtes entièrement sélectionnés de 100 et 200 Mo → 300 Mo ; un hôte dont un service non sélectionné reste nécessaire n'ajoute rien, ni immédiat ni différé.
- `grep -n "150 \* 1024 \* 1024\|60 \* 1024 \* 1024\|40 \* 1024 \* 1024" src/` → aucun résultat.
- En mode démo, l'écran de confirmation affiche deux lignes distinctes, « tout de suite : X » et « au prochain démarrage : Y », et X n'inclut jamais un gain de type démarrage.
- Le rapport exporté contient la nature de chaque gain.
- En mode démo, le total du plan utilise la même agrégation que le backend réel : aucun `gain_next_boot` de service n'est additionné nu.

**Test de réversion.** Le test d'agrégation par hôte échoue si un hôte partiellement sélectionné compte encore (retour à une somme de parts) ; le test de `stopNow` échoue si un gain différé alimente le total immédiat.

**Effort indicatif.** 1,5 jour.

---

### Étape 4 — Règles : schéma et provenance

**État.** Terminée le 2026-11-13 avec un écart de comptage documenté — contrôles et codes retour en 7.6. Le total réel est 601 règles : l'audit a révélé un doublon `eventsystem` / `eventsysevent` dans les 602 annoncées (D17).

**Objectif.** Savoir pourquoi une règle existe, sur quoi elle s'applique exactement, et depuis quand elle n'a pas été revue.

**Périmètre.** `rules.rs` — tables `SERVICES`, `STARTUP`, `TASKS`, `PROCESSES`, `PROTECTED_*` ; `model.rs` — `Advice.locked`, distinct de `Level`.

**Changements.**

- Passer les tables à une structure nommée : clé, niveau, action conseillée, mode de correspondance, source, date de revue, justification. Le mode de correspondance est explicite : nom exact de service, nom d'exécutable, nom de tâche, préfixe de chemin. Plus aucune correspondance par sous-chaîne sur une ligne de commande entière (D9).
- Corriger la clé `ngcctorSvc` (D10) et supprimer la branche `svchost.exe` inatteignable, en conservant l'information utile (liste des services hébergés) dans le conseil de processus via le champ `services` (D11).
- Séparer `locked` de `Level` : `locked` porte le refus, `Level` porte la gravité. L'interface cesse d'afficher « Vital » pour un service seulement verrouillé (D7). `Critical` reste réservé aux composants dont l'arrêt casse Windows.
- Documenter, pour chaque règle qui propose une action, la source de la justification. Une règle sans source reste `Unknown` et ne propose rien.
- Tests d'invariant : clés uniques, clés en minuscules, longueur minimale pour les modes non exacts, toute action adossée à une justification non vide, tout `Critical` présent dans les listes de verrouillage, tout service verrouillé présent dans la table.

**Critère de sortie.**

- Audit automatique des 602 règles (262 services + 197 processus + 93 démarrage + 50 tâches) : invariants vérifiés, zéro doublon, zéro clé en majuscules, zéro mode de correspondance ambigu.
- Test qui prouve qu'une entrée de démarrage dont la commande contient `php` ne correspond plus à la règle `hp`.
- `grep` : plus aucun `contains` sur `command` dans la fonction de conseil de démarrage.
- Le nombre de règles sans source est affiché dans le journal de l'étape.

**Test de réversion.** Le test « `php` ≠ `hp` » échoue si la correspondance par sous-chaîne revient ; le test d'unicité échoue sur un doublon réintroduit.

**Effort indicatif.** 2 jours.

---

### Étape 5 — Interface de décision

**État.** Terminée le 2026-11-14 — contrôles et codes retour en 7.7. La revue visuelle a révélé et fait corriger deux défauts d'interface (D18 : fermeture prématurée de la modale d'export ; D19 : débordement horizontal à 390 px).

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

- `grep -rn "sans risque\|aucun risque réel\|gain net" src/web/ src/report.rs` → aucun résultat. La justification `rules.rs` « Désactivable sans risque » reste hors bannissement (D16) : elle qualifie un élément précis, pas une sélection de masse. `report.rs` n'est pas modifié ici : sa formulation est corrigée à l'étape 3, ce grep est la garde de régression.
- En mode démo, la revue visuelle consignée montre : une sélection vide par défaut, un total séparé immédiat/différé, un `Kill` signalé comme irréversible.
- Les parcours charger / filtrer / sélectionner / appliquer / restaurer / exporter sont exercés en démo et consignés.

**Test de réversion.** Un test de chaînes interdit les formulations bannies ; il échoue si l'une revient.

**Effort indicatif.** 1,5 jour.

---

### Étape 6 — Recette et documentation

**État.** Terminée le 2026-11-14 — contrôles et codes retour en 7.8. La recette réelle a révélé D20 (tâches illisibles sans administrateur), corrigé dans la même étape. La revue post-livraison a ouvert D21–D23 ; D21 et D22 sont corrigés par le lot 8.1/8.2 (7.10), D23 par le lot 8.4 (7.12).

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

**Fusions de services.** `svchost.exe` hébergeant plusieurs services, répartition approximative, hôte entièrement sélectionné (gain = RAM de l'hôte) ou partiellement sélectionné (gain = 0), arrêt d'un service d'un hôte partagé.

**Fermeture normale et terminaison.** La même application ne doit pas être présentée comme sûre à terminer sans avertissement ; texte de confirmation distinct selon le type d'action.

**Rollback.** Point de restauration correct, restauration partielle, restauration totale, échec d'application, action irréversible absente du point.

**Rapport exporté.** Gains immédiat et différé séparés, jamais additionnés dans une même cellule (D15) ; formulations bannies absentes (D16).

**Surface HTTP.** `Host` inconnu, origine trompeuse par suffixe, absence de jeton, jeton invalide, corps trop volumineux, plus de 400 actions, méthode non autorisée.

**Tâches et outils externes.** ACL complète, ACL partielle, `schtasks` absent ou en échec, doublons de chemins, résolution absolue de `schtasks.exe` et `explorer.exe`. La fixture doit couvrir le wrapper, pas seulement le parseur XML.

**Règles.** Unicité, casse, mode de correspondance, absence de source, faux positif `php` déclenchant `hp`.

## 6. Risques et parades

| Risque | Parade |
|---|---|
| Casser un service utile en le passant en manuel | Verrou côté serveur, justification par règle, restauration testée, mode démo pour répéter |
| Faux positif d'une règle de correspondance | Correspondance exacte ou par nom d'exécutable, test dédié par famille de faux positifs connus |
| Exposition locale résiduelle malgré le durcissement | Validation `Host` + jeton sur toutes les routes API + CSP ; test d'intégration rejoué à chaque étape |
| Exécution d'un outil système détourné (`schtasks.exe`, `explorer.exe`) | Résolution par chemin absolu `%SystemRoot%\System32\...`, jamais par nom nu ; D21, lot 8.1 |
| Liste de tâches incomplète sur ACL partielles | Repli dès qu'une lecture est refusée, fusion par chemin et déduplication ; D22, lot 8.2 |
| Contournement de la CSP par un gestionnaire inline | Suppression de `onclick="elevate()"` et passage de `script-src` à `'self'` ; D23, lot 8.4 |
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

### 7.0 Point de référence (2026-11-14, 00 h 01)

Commit `e5198a9` — « Baseline : rien du projet n'était versionné » : `ramscope/`, ce plan, `README.md` et `SAFETY.md` versionnés (20 fichiers, 7 556 lignes), avec `.gitignore` excluant les artefacts de build, les données d'exécution machine et les espaces de travail des agents. Avant ce commit, git ne suivait qu'un `README.md` de deux lignes : aucun point de retour n'existait pour les tests de réversion, et l'état d'avant l'étape 0 n'est pas reconstructible. Le commit est postérieur aux étapes 0 et 1 : c'est un point de contrôle, pas une baseline d'avant l'étape 0, et il ne permet pas de rejouer leur test de réversion. Remote réel (`Roger364/q-memory-manager`) ; l'affirmation « push non effectué » était fausse : un push initial existait déjà sur `origin/main`. Les commits locaux ont ensuite dépassé le distant ; correction consignée en section 11.

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

### 7.2 Étape 1 — contrôles exécutés (2026-11-13, complétés le 2026-11-14)

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
| Interface dans Chromium | — | page chargée, console vide (aucune violation CSP), `/api/snapshot` et `/api/rollbacks` → 200 |
| Application depuis l'interface (démo) | 200 | `POST /api/apply` → 200, « 1/1 réussi », point de restauration `c628866dd3b054e5` créé, aucun message console |

Le contrôle navigateur a utilisé des clics DOM (`element.click()`) : les clics pointeur du pilote n'ont pas changé la vue, sans erreur console. Les gestionnaires et le `fetch` de l'application ont bien été exécutés ; l'interaction souris réelle reste non vérifiée.

Non exécuté à cette étape : lancement avec `--host 0.0.0.0` (le refus est couvert par un test unitaire ; un lancement réel afficherait une boîte de dialogue bloquante sur la machine).

### 7.3 Revue de la v3 et corrections du plan (2026-11-14)

Relecture du plan v3 contre le code, sans modification du code applicatif. Confirmés : décompte 602 (262 + 197 + 93 + 50), D15 (`report.rs:92,104`), D16 (`report.rs:121`), D6 (`win.rs:1085-1096`). Corrigés dans le plan : règle d'agrégation « tous les services de l'hôte ou zéro », section 1 remise à l'état d'après l'étape 1, rattachement de D16 à l'étape 3, périmètres 2 et 3 incluant `demo.rs`, lignes D1/D12 marquées corrigées, `app.js:289,394-395` enregistré dans D15. `README.md` et `SAFETY.md` corrigés. Aucune ligne de code Rust ou JS modifiée.

### 7.4 Étape 2 — contrôles exécutés (2026-11-13)

| Commande | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 22 tests, 22 réussis, dont 7 nouveaux : 5 dans `plan.rs`, 2 dans `demo.rs` |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9 s |
| `POST /api/apply` — `Kill` sans `image_path` | 403 | « Identité du processus incomplète : chemin manquant. » |
| `POST /api/apply` — lot de 401 actions | 400 | « trop d'actions d'un coup (max 400) » (plafond du serveur, avant `plan::validate`) |
| `POST /api/apply` — doublon de service | 403 | « Action présente plusieurs fois dans le lot. » |
| `POST /api/apply` — service avec `from` périmé | 200 | `ok: false`, « l'état du service a changé depuis l'analyse : action refusée » |
| `POST /api/apply` — service déjà dans l'état demandé | 403 | « Le service est déjà dans l'état demandé. » |
| `POST /api/apply` — `Kill` d'identité correcte (démo) | 200 | `ok: true`, `rollback_id: null` : aucune action réversible, aucun point vide |
| `POST /api/apply` — `Kill` avec `created_at` divergent (démo) | 200 | `ok: false`, « l'identité du processus a changé depuis l'analyse : action refusée », `rollback_id: null` |
| `grep -n "réversible" src/main.rs src/web/app.js` | 0 | chaque occurrence est qualifiée ; plus aucune promesse générale |
| Interface — bouton « Sélectionner les conseillés » de l'onglet Processus | — | message explicite, `0 sélectionné`, aucune case cochée |
| Interface — modale d'un `Kill` (Widgets.exe) | — | « La terminaison d'un processus est définitive » ; plus de « Rien n'est supprimé » ; avertissement sur le travail non enregistré |

Le cas « PID recyclé » n'est pas reproductible sur une machine réelle dans ce contrôle : il est couvert par le test démo et par la comparaison `created_at` + chemin dans `win.rs`, non exécutée sur un processus réel. Aucune action réelle n'a été appliquée.

### 7.5 Étape 3 — contrôles exécutés (2026-11-13)

| Commande | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 29 tests, 29 réussis, dont 7 nouveaux : 3 dans `plan.rs`, 2 dans `rules.rs`, 1 dans `server.rs`, 1 dans `report.rs` |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9 s |
| `node --check src/web/app.js` | 0 | syntaxe JavaScript valide |
| `grep -rn "150 \* 1024 \* 1024\|60 \* 1024 \* 1024\|40 \* 1024 \* 1024" src/` | 1 | aucun résultat |
| Snapshot démo — agrégation | — | `plan_gain_next_boot` = 1 148 190 720 octets ; même valeur recalculée à partir des entrées ; somme brute des parts = 212 860 926 octets ; 1 hôte complet, 10 incomplets |
| Interface — 2 des 3 services de l'hôte 4810 sélectionnés | — | « Gain : non chiffrable » |
| Interface — 3 des 3 services du même hôte sélectionnés | — | « 18 Mo (estimé) au prochain démarrage », une seule fois |
| Interface — modale avec arrêt immédiat (Bonjour Service) | — | « Tout de suite : non chiffrable » / « Au prochain démarrage : 14 Mo (estimé) » : le différé n'est plus déplacé |
| Interface — résultat d'un arrêt immédiat | — | « Mémoire disponible mesurée : 7,46 Go → 7,46 Go (+0 Mo, mesuré) » |
| Rapport exporté | — | colonnes « Tout de suite » et « Au prochain démarrage », nature `(estimé)`/`non chiffrable`, « Rien à optimiser sans risque » absent |
| `used = total - available` sur le snapshot démo | — | vrai |

La seule occurrence restante de « sans risque » dans le rapport est la justification « Désactivable sans risque » d'une entrée de démarrage, explicitement autorisée par le plan. Aucune action réelle n'a été appliquée ; les contrôles d'arrêt et de mesure ont été exécutés en mode démonstration, où la mémoire simulée ne varie pas (delta mesuré nul).

### 7.6 Étape 4 — contrôles exécutés (2026-11-13)

| Commande | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 34 tests, 34 réussis, dont 5 nouveaux dans `rules.rs` : audit des tables, verrou non vital, liste `svchost`, `php` ≠ `hp`, absence d'action des heuristiques sans source |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit |
| `node --check src/web/app.js` | 0 | syntaxe JavaScript valide |
| Audit `audit_des_regles` | 0 | 601 règles : 261 services, 197 processus, 93 démarrages, 50 tâches ; zéro doublon par table, zéro clé en majuscules, modes fixes par table, toute action avec justification, tout `Critical` dans une liste de verrouillage, tout service verrouillé dans la table |
| `grep -nE "command.*contains"` sur `startup_advice` | 1 | aucun résultat : plus de `contains` sur la ligne de commande ; les heuristiques de repli ont été supprimées, hors table aucune action |
| `grep -rn "key_matches" src/` | 1 | aucun résultat : l'ancien matching par sous-chaîne a été supprimé |
| `grep -rn "ngcctorSvc\|eventsysevent" src/` | 1 | aucun résultat : clés corrigées ou fusionnées |
| Règles sans source | — | 0 / 601 : chaque entrée porte une provenance de famille non vide |
| Règles avec date de revue individuelle | — | 0 / 601 : la valeur est `non datée`. La migration de schéma n'est pas une revue de fond ; aucune date de revue ne lui est attribuée |

Le plan annonçait 602 règles. L'audit a montré que `eventsystem` et `eventsysevent` désignaient le même service COM+ Event System ; la seconde clé ne correspond à aucun service Windows (vérifié sur la documentation de service Windows 11). Le doublon a été fusionné : le total réel est 601. Conserver une entrée fictive pour atteindre 602 aurait été un chiffre faux.

La provenance est une **source de famille**, pas une URL vérifiée règle par règle : « table RamScope héritée » plus une référence de famille (Microsoft Learn, documentation des éditeurs, Gestionnaire des tâches, Planificateur de tâches). Les justifications n'ont pas été revérifiées individuellement ; `reviewed = "non datée"` rend ce doute visible au lieu de le masquer. Toutes les actions restantes viennent d'une règle de table à source non vide ; les heuristiques de repli qui proposaient une action sans règle ni source ont été supprimées. Une future entrée sans source devra rester `Unknown`.

Le mode des tâches est un suffixe exact à frontière de composant (ou un nom de composant exact), plus strict que le « préfixe de chemin » envisagé dans le plan : il évite qu'une clé courte ne s'applique à un chemin plus long. Les listes `PROTECTED_*` restent des listes de noms, sans niveau ni action ; l'audit vérifie leur cohérence avec les tables. Le cadenas est affiché par `web/app.js` (`Advice.locked`) ; l'interface n'a pas été ouverte dans un navigateur pour cette étape, seul `node --check` a validé la syntaxe.

### 7.7 Étape 5 — contrôles exécutés (2026-11-14)

Commit : `c4fc786` (point de contrôle des étapes 2 à 5).

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence après reformatage du test ajouté |
| `cargo test` | 0 | 37 tests, 37 réussis, dont 3 ajoutés dans `server.rs` : formulations bannies, refus de `Kill` dans un lot, fermeture du fond de modale sur un appui commencé sur le fond |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9,48 s après arrêt du serveur qui verrouillait l'exécutable ; premier essai à 101 (`Accès refusé`) tant que l'ancien binaire tournait |
| `node --check src/web/app.js` | 0 | syntaxe JavaScript valide |
| `grep -rn "sans risque\|aucun risque réel\|gain net" src/web/ src/report.rs` | 1 | aucun résultat |
| Démo — `GET /` | 200 | page servie sur `http://127.0.0.1:8876` en mode démonstration |
| Démo — chargement et vue d'ensemble | — | 5 cartes mémoire ; « Récupérable au prochain démarrage » 1,07 Go, « Récupérable tout de suite » 4,34 Go ; aucune ligne sélectionnée au départ |
| Démo — filtre `spotify` | — | une seule ligne, le démarrage Spotify ; les autres sections disparaissent |
| Démo — sélection de masse | — | 35 éléments sélectionnés ; barre : « 35 sélectionnés », « Gain : 1,07 Go estimé », « Au prochain démarrage » ; aucun processus dans la sélection |
| Démo — confirmation du lot | — | « Confirmer 35 changements », « 35 actions réversibles · 0 irréversible », « Aucun processus n'est terminé », point de restauration annoncé |
| Démo — application d'un lot | — | un service sélectionné (`dmwappushservice`) : « 1 action réversible · 0 irréversible », puis « Application : 1/1 réussi » et point de restauration `c65a25ad12f78b45` créé ; message de résultat corrigé en « Un service reconfiguré (manuel ou désactivé)… » |
| Démo — panneau de détails | — | `dmwappushservice` : source détectée, état « Arrêté · Manuel », effet tout de suite et au prochain démarrage, réversibilité, justification, nom court, compte, PID hôte |
| Démo — `Kill` seul | — | « Confirmer 1 changement », « 0 action réversible · 1 irréversible », processus nommé `chrome.exe (PID 5520)`, avertissement de perte de travail, « Tout de suite : 402 Mo estimé », « Au prochain démarrage : non chiffrable » |
| Démo — restauration | — | « Restauration : 1/1 réussi », point consommé, `dmwappushservice` revenu à `Manuel`, compteur `Restauration` passé de 7 à 6 ; un second essai sur un point antérieur avait déjà donné 1/1 |
| Démo — export | — | modale « Rapport exporté » visible après correction de D18 ; chemins HTML et JSON renvoyés sous `%LOCALAPPDATA%\RamScope\rapports\` ; fichiers non rouverts dans ce contrôle |
| Démo — petit écran 390 × 844 | — | après correction de D19 : une seule colonne, en-tête replié, tableaux défilant horizontalement, modale d'export sans débordement |
| Démo — clavier | — | en-têtes de tri `role="button"`, `tabindex="0"` ; un événement `Enter` sur `PID` change le premier PID de 5520 à 5920 ; `Échap` ferme la modale et rend le focus au bouton d'ouverture. Événements clavier synthétiques, pas un parcours de frappes réelles |
| Contrastes | — | calcul des couleurs : `#e8edf5` sur `#111827` 15,09:1 ; `#8b98ad` sur `#111827` 6,08:1 ; `#7b879c` sur `#111827` 4,89:1 ; badges 5,65 à 11,23:1 ; `#fcd34d` sur `#1c1706` 12,40:1 |

La revue a exercé les parcours charger, filtrer, sélectionner, confirmer, appliquer et restaurer en démonstration, puis exporter ; aucune action réelle n'a été demandée ni exécutée hors démonstration. Le message de résultat d'un service désactivé, qui parlait encore de « service passé en manuel », a été corrigé pendant la revue. Deux défauts non prévus ont été trouvés et corrigés dans la même étape. **D18** : la modale d'export s'ouvrait après un appel réseau et le clic en cours pouvait être reçu par le masque, qui la refermait aussitôt ; la fermeture exige désormais un appui commencé sur le fond (`pointerdown`), et un test statique interdit le retour à l'ancien gestionnaire. **D19** : à 390 px, la grille `.hero` conservait des largeurs minimales qui la faisaient déborder ; elle passe sur une colonne et les chemins d'export se coupent.

Limites : la restauration et l'export ont été exercés en mode démonstration ; le contenu des deux fichiers exportés n'a pas été rouvert. La navigation clavier n'a pas été parcourue avec de vraies frappes dans ce navigateur : les gestionnaires ont été sollicités par événements synthétiques. Le rendu a été regardé à 1600 × 900 et 390 × 844, pas sur d'autres tailles ni avec un lecteur d'écran. Aucun contraste n'a été mesuré sur une capture, seulement calculé à partir des couleurs du thème. La CSP n'a pas été resserrée : `'unsafe-inline'` reste nécessaire pour les styles et un gestionnaire inline (`elevate()`), ce point reste ouvert.

Captures versionnées dans `docs/evidence/` : `ramscope-step5-final-2026-11-14.png` (bureau), `ramscope-step5-mobile-fixed.png` (390 px), `ramscope-step5-export-mobile.png` (export), `ramscope-step5-confirm.png` (lot réversible), `ramscope-step5-applied.png` (application 1/1), `ramscope-step5-kill.png` (`Kill` seul) et `ramscope-step5-restored.png` (restauration). L'index et les empreintes sont dans `docs/evidence/README.md` et `docs/evidence/SHA256SUMS`.

### 7.8 Étape 6 — recette finale (2026-11-14)

Commit : `6dfbaa2`.

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 38 tests, 38 réussis, dont `win::tests::decode_une_tache_xml` ajouté pour D20 |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9,55 s après arrêt des deux serveurs qui verrouillaient l'exécutable |
| `node --check src/web/app.js` | 0 | syntaxe JavaScript valide |
| `ramscope.exe --help` | 0 | options et limites d'élévation affichées |
| Démo — `GET /` | 200 | binaire final rechargé sur `http://127.0.0.1:8876` ; plan de 35 éléments, sélection vide ; parcours détaillé en 7.7, interface inchangée depuis |
| Réel — `--no-admin` sur `http://127.0.0.1:8877` | 200 | lecture seule, aucun UAC, aucune action appliquée |
| Réel — mémoire totale | — | RamScope `34 281 054 208` octets = `TotalVisibleMemorySize` `33 477 592` Ko ; disponible `20 923 269 120` octets, même ordre que les `20 413 864` Ko de PowerShell pris à un autre instant (écart < 0,1 %) |
| Réel — services | — | 289 dans RamScope, 289 via `Get-Service` |
| Réel — tâches | — | après D20 : 201 dans RamScope, 201 via `Get-ScheduledTask` ; 167 activées des deux côtés ; zéro doublon de chemin |
| Réel — démarrage | — | 7 entrées RamScope, 6 via `Win32_StartupCommand` : `IJNetworkScannerSelectorEX2`, absent de l'API, est lu dans le registre 32 bits ; RamScope est plus complet sur ce point |
| Réel — processus | — | 238–239 selon l'instant, contre 237–238 via `Get-Process` ; écart dû au décalage des collectes |
| Réel — durée de collecte | — | 844 ms avec le repli `schtasks` au dernier passage, 1 004 ms au premier, contre 237 ms avant ; le repli ajoute environ 0,6–0,8 s, consigné comme régression de performance acceptée pour la lecture seule |
| Surface HTTP | 403 | `Host: attaquant.tld` sur `/` et `/api/snapshot` ; absence de jeton ; origine étrangère ; `POST /api/apply` en lecture seule |

D20 a été trouvé par cette recette : sans administrateur, `C:\\Windows\\System32\\Tasks` est refusé par les ACL et RamScope affichait 0 tâche alors que Windows en expose 201. Le repli `schtasks /query /xml` analyse le même XML, conserve les chemins et les états, et retrouve exactement 201 tâches et 167 activées. Le test `decode_une_tache_xml` couvre le parseur ; le chemin administrateur (lecture directe des fichiers XML) n'a pas été comparé à `schtasks`. La note « tâche non lisible » n'est plus affichée quand le repli retrouve la liste : elle ne subsiste que si le repli échoue, pour ne pas alarmer sans raison.

La collecte réelle n'a subi aucune action : seuls des `GET` et des `POST` refusés en lecture seule ont été exécutés. Le parcours de démonstration complet (charger, filtrer, sélectionner, confirmer, appliquer, restaurer, exporter) est celui de 7.7 ; l'interface n'a pas changé depuis, seul `win.rs` a été modifié. Les limites restantes sont consignées dans le README : pas d'application réelle, pas de comparaison du chemin administrateur des tâches, CSP non resserrée, pas de lecteur d'écran.

### 7.9 Revue post-livraison (2026-11-14)


| Constat | Preuve | Décision |
|---|---|---|
| Décompte de la section 1 périmé | le plan annonçait 7 717 lignes et `win.rs` 1 734 ; `wc -l` donne 7 756 et 1 736 | corrigé dans la section 1 ; les chiffres restent datés |
| Outils externes résolus par leur nom | `Command::new("schtasks.exe")` dans deux fonctions, `explorer.exe` dans `/api/open-dir` | D21, lot 8.1 |
| Repli déclenché seulement sur liste vide | `if out.is_empty()` dans `collect_tasks` | D22, lot 8.2 |
| Test D20 limité au parseur | `decode_une_tache_xml` ne teste ni le wrapper ni le déclenchement | lot 8.3 |
| Budget de collecte non défini | 844–1 004 ms en lecture seule contre 237 ms avant | budget ≤ 1,5 s, lot 8.3 |
| CSP encore ouverte | `'unsafe-inline'` et `onclick="elevate()"` | D23, lot 8.4 |
| Preuves non versionnées | 7 captures, 1,3 Mo, non suivies | lot 8.5 |
| Règle « une étape = un commit » contredite par le point de contrôle | `c4fc786` regroupe les étapes 2 à 5 | exception consignée dans la section 0 |

Verdict : le socle fonctionnel reste livré, mais aucune nouvelle étape fonctionnelle n'est justifiée. Le lot 8 est un lot de clôture de 1,5 à 2 jours ; 8.1 et 8.2 passent avant toute distribution. Aucun code n'a été modifié pendant cette revue ; D21–D23 restent ouverts.

### 7.10 Lot 8.1 et 8.2 — contrôles exécutés (2026-11-14)

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence après formatage des fichiers touchés |
| `cargo test` | 0 | 41 tests, 41 réussis ; trois tests ajoutés dans `win.rs` |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9,67 s après arrêt de la démonstration qui verrouillait `target/release/ramscope.exe` |
| `grep -rn 'Command::new("schtasks.exe")\|Command::new("explorer.exe")' src/` | 1 | aucun résultat : les outils passent par `system32_path` |
| Réel — `--no-admin` après correction | 200 | `demo=false`, `admin=false`, 201 tâches, 167 activées, zéro doublon de chemin ; collecte 806 ms ; seule note : mode lecture seule |
| Réel — `GET /` | 200 | page servie avec jeton de session ; `GET /api/health` confirme `demo=false`, `admin=false` |

Les tests ajoutés sont `chemins_des_outils_systeme_sont_absolus`, `le_repli_couvre_une_acl_partielle` et `fusionne_les_taches_par_chemin_sans_doublon`. Le test de fusion utilise deux tâches directes et deux tâches de repli, dont un doublon de casse : la tâche directe est conservée et le chemin n'apparaît qu'une fois. Le cas ACL partielle est testé au niveau de la décision de repli et de la fusion, pas en injectant une ACL Windows réelle. Aucune action réelle n'a été appliquée ; le serveur réel a été arrêté après la mesure, puis la démonstration a été relancée sur `http://127.0.0.1:8876`.

### 7.11 Lot 8.3 — contrôles exécutés (2026-11-14)

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 42 tests, 42 réussis ; `decode_la_sortie_de_schtasks` ajouté |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9,57 s |
| Réel — trois collectes `--no-admin` | 200 | 833, 769 et 810 ms ; 201 tâches, 167 activées, zéro doublon à chaque fois |
| Démo après mesure | 200 | serveur relancé sur `http://127.0.0.1:8876` |

`tasks_from_schtasks_output` transforme désormais une sortie `schtasks` en tâches sans lancer de processus ; `tasks_from_schtasks_xml` ne fait plus que l'exécution et l'appel. La fixture contient deux blocs `<!-- \Chemin -->`, dont un avec `<Enabled>false</Enabled>` : les deux chemins, l'état activé/désactivé et la commande sont vérifiés. Le wrapper est testé sur une fixture, pas sur une sortie réelle de `schtasks` en échec. Aucune action réelle n'a été appliquée. La collecte reste sous le budget de 1,5 s : aucun cache n'a été ajouté.

### 7.12 Lot 8.4 — contrôles exécutés (2026-11-14)

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `cargo fmt --check` | 0 | aucune différence |
| `cargo test` | 0 | 43 tests, 43 réussis ; test CSP ajouté |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucune erreur |
| `cargo build --release` | 0 | binaire produit en 9,63 s |
| `GET /` | 200 | en-tête `script-src 'self'`, zéro script inline, une meta de jeton |
| Démo Chromium | 200 | `#badges` affiche « DÉMO Administrateur », chargement terminé, 155 nœuds, zéro gestionnaire inline, console vide |
| `GET /api/health` | 200 | `demo=true` |

Le script inline qui posait `window.RAMSCOPE_TOKEN` a été supprimé : le jeton est maintenant lu dans `meta[name="ramscope-token"]`, et le bouton « Passer en administrateur » est lié par `#btnElevate` dans `app.js`. Les styles inline restent nombreux (largeurs de jauges, couleurs de points) et justifient `style-src 'unsafe-inline'` ; `script-src` n'a plus besoin de cette exception. Le bouton d'élévation n'a pas été cliqué : en démonstration il n'est pas affiché, et un clic réel déclencherait UAC. Aucune action réelle n'a été appliquée.

### 7.13 Lot 8.5 — preuves versionnées (2026-11-14)

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `file docs/evidence/*.png` | 0 | sept fichiers reconnus comme PNG 1600 × 900, 8 bits RGB |
| `sha256sum -c docs/evidence/SHA256SUMS` | 0 | sept fichiers OK |
| `du -sh docs/evidence` | 0 | 1,3 Mo au total |
| `git diff --check` | 0 | aucune erreur d'espacement |

Les captures ont été déplacées du répertoire racine vers `docs/evidence/` sans modification de contenu ; un index `README.md` précise qu'elles viennent de la démonstration et un manifeste `SHA256SUMS` fige leurs empreintes. Les images n'ont pas été rouvertes visuellement pendant ce lot. Aucun test Rust n'a été relancé : aucun fichier de code n'a changé. Après commit, `git status --short` ne doit plus montrer que les artefacts explicitement ignorés.

### Avancement

| Étape | État | Défauts traités | Vérification consignée |
|---|---|---|---|
| 0 — Socle vérifiable | terminée (2026-11-13) | D13, D14 | voir 7.1 |
| 1 — Surface locale | terminée (2026-11-13) | D1, D12 | voir 7.2 |
| 2 — Identité et irréversibilité | terminée (2026-11-13) | D2, D8 | voir 7.4 |
| 3 — Vérité des chiffres | terminée (2026-11-13) | D3, D4, D5, D6, D15, D16 | voir 7.5 |
| 4 — Règles et provenance | terminée avec écart 602→601 documenté (2026-11-13) | D7, D9, D10, D11, D17 | voir 7.6 |
| 5 — Interface de décision | terminée (2026-11-14) | D8 (volet interface), D18, D19 | voir 7.7 |
| 6 — Recette et documentation | terminée (2026-11-14) | D20 | voir 7.8 |
| Lot de clôture — durcissement | terminé : 8.1 à 8.5 (2026-11-14) | D21, D22, D23 | voir 7.10 à 7.13 et 8 |

Une ligne « à faire » ne devient « terminée » qu'avec la commande, son code retour et le chemin de l'artefact concerné.

## 8. Après livraison — lot de clôture

Le socle fonctionnel des étapes 0 à 6 est livré. Ce lot ne rouvre aucune étape : il traite les dettes de durcissement et de preuve. **8.1 à 8.5 sont terminés le 2026-11-14** (contrôles en 7.10 à 7.13) ; le lot de clôture est complet.

### 8.1 Chemins absolus des outils externes (D21) — P2, 0,25 jour — terminé

**Changement.** Résoudre `schtasks.exe` via `%SystemRoot%\System32\schtasks.exe` dans `tasks_from_schtasks_xml` et `run_schtasks`; appliquer le même principe à `explorer.exe` dans `/api/open-dir`. Un helper unique rend le chemin testable.

**Critère de sortie.** `grep -rn 'Command::new("schtasks.exe")\|Command::new("explorer.exe")' ramscope/src/` → aucun résultat ; test nommé du helper ; snapshot réel toujours à 201 tâches et 167 activées.

**Test de réversion.** Le test du helper échoue si le nom nu revient ; le snapshot échoue si le chemin absolu ne trouve plus `schtasks`.

**État (2026-11-14).** Terminé : `system32_path` est utilisé pour `Tasks`, `schtasks.exe` et `explorer.exe` ; test `chemins_des_outils_systeme_sont_absolus` ; `grep` sans résultat ; snapshot réel à 201 tâches et 167 activées. Voir 7.10.

### 8.2 Repli sur ACL partielles et déduplication (D22) — P2, 0,5 jour — terminé

**Changement.** Déclencher le repli dès que `denied > 0` ou `unreadable > 0`, pas seulement quand `out.is_empty()`. Fusionner les résultats directs et `schtasks` par chemin ; un chemin ne doit apparaître qu'une fois. En cas d'échec du repli, conserver une note explicite au lieu d'une liste vide silencieuse.

**Critère de sortie.** Test nommé avec une fixture représentant un fichier lisible et des fichiers refusés : la liste fusionnée est complète, sans doublon. Test d'intégration réel : 201 tâches, 167 activées, zéro doublon.

**Test de réversion.** La fixture échoue si le repli ne se déclenche que sur une liste vide, ou si la fusion réintroduit un doublon.

**État (2026-11-14).** Terminé : `needs_task_fallback` couvre la liste vide et les lectures refusées ; `merge_tasks` fusionne par chemin insensible à la casse et déduplique ; tests `le_repli_couvre_une_acl_partielle` et `fusionne_les_taches_par_chemin_sans_doublon` ; snapshot réel sans doublon. Voir 7.10.

### 8.3 Test du wrapper et budget de collecte (D22) — P2, 0,25 jour — terminé

**Changement.** Extraire la transformation « sortie `schtasks` → tâches » dans une fonction testable, puis la couvrir avec une fixture de deux tâches, dont une désactivée. Mesurer la collecte réelle : budget accepté **≤ 1,5 s** en lecture seule ; au-delà, mettre en cache la liste des tâches pour 60 s ou documenter le dépassement.

**Critère de sortie.** Test nommé du wrapper ; `meta.duration_ms` consigné sur trois collectes réelles ; aucun dépassement silencieux du budget.

**État (2026-11-14).** Terminé : `tasks_from_schtasks_output` est séparé de l'exécution ; test `decode_la_sortie_de_schtasks` avec deux tâches, dont une désactivée ; trois collectes réelles à 833, 769 et 810 ms, sous le budget ; aucun cache ajouté. Voir 7.11.

### 8.4 Resserrer la CSP (D23) — P2, 0,5 jour — terminé

**Changement.** Supprimer le script inline qui portait le jeton, déplacer le jeton dans une meta et le gestionnaire `onclick="elevate()"` vers un écouteur de `app.js`, puis passer `script-src` à `'self'` sans `'unsafe-inline'`. Les styles inline restent nécessaires aux largeurs et couleurs calculées ; `style-src 'unsafe-inline'` est conservé et justifié.

**Critère de sortie.** `grep -n 'onclick=' src/web/index.html src/web/app.js` → aucun résultat ; en-tête CSP servi sans `'unsafe-inline'` pour les scripts ; chargement Chromium sans erreur console ; parcours démo charger → sélectionner → confirmer → annuler encore fonctionnel.

**État (2026-11-14).** Terminé : `script-src 'self'`, zéro script inline, zéro gestionnaire HTML inline, jeton lu depuis la meta, bouton d'élévation lié dans `app.js` ; test `la_csp_n_autorise_pas_le_script_inline` ; démonstration chargée avec 155 nœuds et aucune erreur console. Voir 7.12.

### 8.5 Preuves et versionnage — P3, 0,1 jour — terminé

**Changement.** Versionner les sept captures de l'étape 5 sous `docs/evidence/`, avec un index et un manifeste `SHA256SUMS`. Vérifier que 7.7 et 7.8 citent bien `c4fc786` et `6dfbaa2`, puis que le prochain lot respecte « un commit par étape ».

**Critère de sortie.** `git status --short` ne montre que des fichiers explicitement ignorés ou versionnés ; le journal cite le hash du commit de chaque étape.

**État (2026-11-14).** Terminé : sept PNG valides (1600 × 900) déplacés dans `docs/evidence/`, index `README.md`, manifeste `SHA256SUMS` vérifié par `sha256sum -c` (code retour 0). Les captures n'ont pas été rouvertes visuellement pendant ce lot. Voir 7.13.

## 9. Hors périmètre

- Nettoyeur automatique, planification, exécution sans confirmation.
- Base de règles communautaire ou téléchargée.
- Refonte graphique complète, changement de framework front.
- Télémétrie, envoi de rapport à un serveur distant.
- Signature de code et distribution (sujet séparé, après stabilisation).

## 10. Annexe — commandes de vérification

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
grep -rn "sans risque\|aucun risque réel\|gain net" src/web/ src/report.rs
grep -rn "150 \* 1024 \* 1024\|60 \* 1024 \* 1024\|40 \* 1024 \* 1024" src/
```

Rappel : aucune de ces commandes ne doit être présentée comme exécutée avant de l'être. Le journal est la seule source de vérité sur ce point.

## 11. Correction documentaire et synchronisation (2026-11-14)

La revue WorkBuddy a été rédigée sur `d08e412`, puis commitée sous `ecccadc`. Elle signalait à juste titre que `PLAN.md` citait un ancien remote erroné, un décompte de tests inversé et un push non effectué. Les vérifications suivantes ont été exécutées :

| Commande / contrôle | Code retour | Observation |
|---|---|---|
| `git remote -v` | 0 | `origin` = `https://github.com/Roger364/q-memory-manager.git` |
| `git ls-remote origin HEAD` | 0 | `03bed3e` avant le push du lot 9 |
| `gh repo view <ancien-dépôt>` | 1 | `404 Not Found` : l'ancien dépôt n'existe plus |
| Recherche de l'ancienne URL dans `.git` | 1 | aucune trace dans la configuration ou les références Git |
| `cargo test -- --list` (dans `ramscope/`) | 0 | 43 tests : 13 dans `plan.rs`, 12 dans `rules.rs` |

La version du plan est corrigée en **3.2**, le décompte des tests est remis dans le bon ordre, et les mentions de l'ancien dépôt ont été retirées de la documentation. Le plan d'améliorations proposé est séparé dans `PLAN-LOT9.md` pour ne pas mélanger le journal historique et le travail à venir. Le push du lot 9 clôt cette correction en fast-forward.
