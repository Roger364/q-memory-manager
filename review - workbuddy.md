# Revue technique approfondie — RamScope

**Auteure** : Nadine (WorkBuddy) · **Date** : 2026-11-14 · **Révision auditée** : `d08e412` (`main`)
**Périmètre** : le projet `ramscope/` et sa documentation (`PLAN.md`, `README.md`, `SAFETY.md`, `docs/evidence/`), tels qu'ils sont *commités*. Cette revue porte sur le code livré, pas sur l'activité d'un agent ; elle ne revendique aucune partie de l'implémentation.
**Méthode** : lecture des 8 140 lignes de sources ciblées, confrontation systématique des affirmations du plan au code, rejeu des commandes de vérification, inspection de la dépendance HTTP (`tiny_http 0.12`). **Aucun fichier du projet n'a été modifié** ; aucune action réelle n'a été appliquée à la machine.

---

## 1. Verdict

**Le socle est solide et honnêtement documenté ; il n'est pas encore distribuable sans trois corrections de sécurité et une correction fonctionnelle.** Le travail de vérité accompli sur les chiffres, les verrous et la traçabilité est réel et vérifiable : les 43 tests passent, les invariants d'agrégation sont appliqués de façon cohérente côté Rust et côté JS, et le journal tient ses promesses de codes retour. La dette restante n'est pas dans les fonctionnalités livrées, mais dans **la frontière de confiance** : le durcissement D21 repose sur une variable d'environnement contrôlable, et l'API locale élevée n'authentifie pas son appelant. S'y ajoutent une perte d'options lors de l'élévation UAC, une provenance des règles qui n'est qu'un champ mort, et un écart documentaire sur l'état du dépôt distant.

Dix constats sont détaillés ci-dessous, dont **deux P1**, **cinq P2** et **trois P3**.

---

## 2. Mesures vérifiées (rejouées le 2026-11-14, 22 h 38–22 h 43)

| Contrôle | Code retour | Résultat |
|---|---|---|
| `cargo test` | 0 | **43 réussis, 0 échec**, 0 ignoré, 0,02 s |
| `cargo fmt --check` | 0 | aucune différence |
| `cargo clippy --all-targets -- -D warnings` | 0 | aucun avertissement |
| `cargo build --release` | 0 | binaire produit |
| `node --check src/web/app.js` | 0 | syntaxe valide |
| `sha256sum -c docs/evidence/SHA256SUMS` | 0 | **7/7 captures OK** |
| `git ls-remote origin HEAD` | 0 | `03bed3e` — le distant a **6 commits de retard** sur `d08e412` |

Volume mesuré : **7 875 lignes Rust + JS** (`win.rs` 1 830, `rules.rs` 1 365, `demo.rs` 1 191, `app.js` 884, `server.rs` 640, `ffi.rs` 480, `model.rs` 459, `plan.rs` 451, `report.rs` 202, `util.rs` 180, `main.rs` 102, `platform.rs` 91), plus 62 lignes HTML et 203 lignes CSS. Tests par fichier : `plan.rs` 13, `rules.rs` 12, `server.rs` 10, `win.rs` 5, `demo.rs` 2, `report.rs` 1. **601 règles** (261 services, 197 processus, 93 démarrages, 50 tâches). 29 fichiers suivis par git.

---

## 3. Ce qui tient réellement

1. **Surface web** : `Host` comparé à une liste exacte (`server.rs:92-94, 145-148`), jeton exigé sur tout `/api/` et comparé sans sortie anticipée (`server.rs:109-120, 151-156`), origine sans suffixe (`server.rs:98-107`), CSP `script-src 'self'` sans script inline, `--host` fermé aux adresses de boucle locale. Le scénario DNS-rebinding de D1 est traité.
2. **Identité d'un `Kill`** : `image_path` + `created_at` vérifiés sur le PID rouvert avant `TerminateProcess` (`win.rs:1581-1617`), avec des garde-fous indépendants côté moteur. C'est le seul verrou réellement redondant.
3. **Vérité des chiffres** : `Gain` typé (`model.rs:90-150`), suppression des constantes de repli 150/60/40 Mo, agrégation « tous les services de l'hôte ou zéro » implémentée à l'identique dans `plan.rs:151-192` et `app.js:123-144`, colonnes séparées dans l'export (`report.rs:91-131`). Le correctif v3.1 a bien tué la fausse sémantique du plafond.
4. **Provenance des actions** : 601 règles auditées (unicité, casse, mode, justification non vide, verrouillage des `Critical`), correspondances exactes, faux positif `php`/`hp` couvert.
5. **Discipline de preuve** : journal avec commandes et codes retour, 7 captures versionnées avec empreintes, section « ce qui n'a pas été testé » explicite dans le README. C'est rare et c'est le vrai atout du projet.

---

## 4. Constats

### P1-1 — D21 est contournable : le « chemin absolu » est enraciné dans une variable d'environnement

**Preuve.** `win.rs:900-905` construit le chemin avec `std::env::var("SystemRoot")` puis `windir`, repli `C:\Windows`. Ce chemin sert à `schtasks.exe`, à `Tasks` (`win.rs:907-909`) et à `explorer.exe` (`server.rs:374`). `win.rs:268-284` relance le processus en `runas` via `ShellExecuteW` avec `lpDirectory = null` : **l'environnement du parent non élevé est hérité par le processus élevé**. Le test `chemins_des_outils_systeme_sont_absolus` (`win.rs:1783-1790`) n'assertit que `is_absolute()` et le suffixe `system32\schtasks.exe` — il passe avec un `SystemRoot` forgé.

**Cause.** La propriété testée (« absolu ») n'est pas la propriété voulue (« non influençable par l'appelant »). Une variable d'environnement de processus n'est pas une source d'autorité système.

**Impact.** Un lanceur local (raccourci, script, logiciel déjà présent) qui positionne `SystemRoot=C:\Temp\X` fait exécuter `C:\Temp\X\System32\schtasks.exe` **avec les droits élevés** de RamScope. Précondition : pouvoir lancer RamScope avec un environnement modifié — pas d'exploitation à distance. `data_dir()` (`win.rs:1744-1750`) a le même défaut avec `LOCALAPPDATA`/`USERPROFILE`, ce qui permet de rediriger rapports et points de restauration.

**Recommandation.** Résoudre le répertoire système par `GetSystemDirectoryW` (ou `SHGetKnownFolderPath(FOLDERID_System)`), jamais par `%SystemRoot%`. Ajouter un test qui forge `SystemRoot` et vérifie que le chemin retourné ne change pas. Traitement identique pour le dossier de données via `KNOWNFOLDERID_LocalAppData`.

### P1-2 — L'API locale élevée n'authentifie pas son appelant, et l'état préalable est fourni par le client

**Preuve.** Le jeton est `new_id() + new_id()` (`server.rs:424`), où `new_id()` dérive de `SystemTime::now().as_nanos()`, du PID et d'un xorshift (`util.rs:67-78`) : aucune source cryptographique, aucune dépendance `getrandom`/`rand`. Surtout, il est **servi dans `GET /`** à tout client capable d'émettre un `Host` autorisé (`server.rs:161-163`) : n'importe quel processus local peut le lire, puis appeler `POST /api/apply`. `plan::validate` (`plan.rs:47-118`) vérifie les verrous et la forme des actions, mais **pas leur provenance** : un `Action::Startup`, `Action::Task` ou `Action::Service` visant une cible arbitraire est accepté. Enfin le contrôle de péremption repose sur le champ `from` / `was_enabled` **fourni par le client** : avec `from: None`, `win.rs:1672-1684` saute purement la revérification d'état ; les actions de restauration construisent d'ailleurs systématiquement `from: None` (`win.rs:1638-1652`).

**Cause.** Le modèle de confiance protège contre une *page web tierce* (objectif de D1), mais traite le serveur comme une frontière de sécurité vis-à-vis des *processus locaux*. Or le serveur tourne élevé, et le client est seul juge de l'état observé. L'invariant 5 du plan (« identité revérifiée au moment d'agir ») n'est donc pas garanti côté serveur.

**Impact.** Élévation de privilèges locale : un processus non élevé peut faire désactiver des services, des tâches ou des entrées de démarrage, et arrêter des services, sans passer par l'interface. Ce n'est pas une faille distante ; c'est un modèle de confiance à expliciter.

**Recommandation.** *A minima* : documenter ce modèle dans `SAFETY.md` ; exiger `from`/`was_enabled` sur toute action hors restauration et les confronter au snapshot serveur (cache 2 s, `server.rs:34-45`) ; générer le jeton via `BCryptGenRandom` et ne l'exposer que dans la session navigateur. *Structurellement* : remplacer l'API TCP par un canal nommé avec DACL, ou vérifier le niveau d'intégrité / l'identité du pair. Conserver la validation `Host`/`Origin`, qui reste utile contre le navigateur.

### P2-1 — « L'origine est comparée exactement » : vrai seulement quand l'en-tête est présent

**Preuve.** `server.rs:196-206` : le contrôle est dans un `if let Some(o)`. Une mutation sans en-tête `Origin` est acceptée. `README.md:94` et `SAFETY.md:63` affirment la comparaison sans réserve. Aucun contrôle `Sec-Fetch-Site`.

**Impact.** Défense en profondeur absente ; le jeton et l'en-tête personnalisé empêchent aujourd'hui l'exploitation par formulaire cross-site, mais l'affirmation documentaire est fausse et un client non navigateur n'a aucune contrainte d'origine.

**Recommandation.** Exiger `Origin` sur toute mutation (403 si absent), ajouter `Sec-Fetch-Site: same-origin`, corriger les deux phrases de documentation, ajouter un test nommé.

### P2-2 — Corps de requête non borné

**Preuve.** `server.rs:208-211` : `read_to_string` sans limite. La source de `tiny_http 0.12` (`request.rs:191-215`) ne plafonne ni `Content-Length` ni le décodage chunked. La matrice de régression `PLAN.md:338` mentionne « corps trop volumineux » mais aucun code ni test ne le couvre — c'est une ligne de matrice non tenue.

**Impact.** Un client local peut provoquer l'épuisement mémoire du processus élevé (déni de service local).

**Recommandation.** Rejeter `Content-Length > 1 Mio` et borner la lecture (`take`), avec un test qui envoie un corps surdimensionné.

### P2-3 — Les « deux verrous » sont, pour les services, une seule fonction appelée deux fois

**Preuve.** `server.rs:233` et `win.rs:1622` appellent tous deux `plan::validate` avec le même PID. Pour `Service` / `Startup` / `Task`, le moteur n'a pas de contrôle de verrou indépendant : `win.rs:1670-1728` applique directement après validation. `SAFETY.md:22` et `PLAN.md:54` présentent cela comme deux niveaux de refus. Les vrais seconds verrous existent pour `Kill` (`win.rs:1582-1587`) et pour la péremption d'état (`from`), pas pour les listes protégées.

**Impact.** Un défaut dans `validate_one` contourne les deux « verrous » simultanément ; la redondance annoncée est illusoire sur la famille de actions la plus nombreuse.

**Recommandation.** Soit revérifier `is_protected_service` / `is_protected_process` dans `apply_service`, `run_schtasks`, `control_service` et `apply_startup`, soit corriger la formulation en « validation unique, appliquée aux deux points d'entrée ».

### P2-4 — Perte des options de ligne de commande lors de l'élévation UAC

**Preuve.** `main.rs:68` relance via `win::elevate()`, qui ne transmet que `--elevated` (`win.rs:268-284`). `/api/elevate` (`server.rs:333-343`) utilise la même fonction. Ni `--port`, ni `--host`, ni `--no-browser` ne sont forwardés.

**Impact.** Sur le chemin **par défaut** d'une machine réelle (sans `--no-admin`), les options documentées `README.md:26-33` sont silencieusement ignorées : l'instance élevée revient au port 8765, à `127.0.0.1` et ouvre une fenêtre. Le journal 7.8 a mesuré des serveurs sur 8876/8877 lancés en `--demo` / `--no-admin`, donc le cas n'a jamais été exercé.

**Recommandation.** Reconstruire la ligne de commande complète (`std::env::args().skip(1)` + `--elevated`) dans `elevate()` ; tester avec un lancement `--port` réel et consigner le port effectif.

### P2-5 — Provenance des règles : champs morts, dette gelée par un test

**Preuve.** `Rule.source` et `Rule.reviewed` portent `#[allow(dead_code)]` (`rules.rs:39-42`) ; aucune occurrence de `.source` / `.reviewed` hors `rules.rs`. Les macros (`rules.rs:59-113`) injectent une constante de famille et `REVIEWED = "non datée"` pour les 601 règles. L'audit (`rules.rs:1253-1254`) assertit `!source.is_empty()` — toujours vrai par construction — et `assert_eq!(r.reviewed, REVIEWED)` — **ce qui fait échouer le test si une règle reçoit une vraie date de revue**. `Advice` (`model.rs:152-167`) n'expose ni source ni date : l'API, l'interface et l'export ne les montrent jamais.

**Impact.** L'objectif de l'étape 4 (« savoir pourquoi une règle existe… et depuis quand elle n'a pas été revue ») est rempli pour la justification, pas pour la provenance. Deux assertions donnent une garantie illusoire, et la troisième transforme une dette assumée en invariant obligatoire.

**Recommandation.** Exposer `source` et `reviewed` dans `Advice` et dans le rapport ; remplacer l'assertion par « `non datée` ou date ISO valide » ; écrire un test avec une règle datée pour prouver que les deux états sont acceptés.

### P2-6 — Zéro mesuré confondu avec absence de mesure (latent)

**Preuve.** `model.rs:117-126` documente explicitement « un zéro mesuré reste une mesure ». Mais `Gain::merge` ignore tout gain dont `bytes == 0` (`model.rs:130`), `report.rs:13-17` affiche « non chiffrable » dès que `bytes == 0`, et `app.js:105-122` fait dépendre `gainText`/`gainLine`/`addGain` de la valeur plutôt que de la nature. L'unique `Gain::measured` est aujourd'hui dans `MemoryDelta` (`server.rs:255`), rendu séparément : le défaut est latent, mais il contredit l'invariant 7 et surgira au premier gain mesuré nul attaché à un `Advice`.

**Recommandation.** Tester `nature` avant `bytes` partout ; ajouter un test `Gain::measured(0)` traversant `merge`, `gain_text` et le rendu JS.

### P3-1 — Documentation d'état périmée et absence de sauvegarde distante

**Preuve.** `PLAN.md:49` et `PLAN.md:372` citaient un ancien remote erroné et « rien poussé ». Le remote réel est `https://github.com/Roger364/q-memory-manager.git`, et `git ls-remote origin HEAD` retourne `03bed3e` : **un push a eu lieu**, mais **6 commits** (`c4fc786` → `d08e412`, soit les étapes 2 à 6 et tout le lot 8) ne sont pas sur le distant. `PLAN.md:42` annonce « 12 dans `plan.rs`, 13 dans `rules.rs` » ; le comptage réel est l'inverse (13 / 12). Le nombre total de 43 et le volume de 7 875 lignes sont, eux, exacts.

**Impact.** La section censée être la référence vérifiée contient trois affirmations fausses ; surtout, **le travail livré n'a pas de sauvegarde hors machine**.

**Recommandation.** Corriger les trois lignes, puis pousser `d08e412` après accord de Ro. À noter : deux fichiers de test non suivis sont apparus à la racine pendant cette revue (`demo-test.log`, `ramscope-http-test.js`, puis `ramscope-readonly-test.js`, `readonly-test.log`, `sandbox-test/`, horodatés 22 h 41) — activité d'un autre agent, non touchée ici ; le `.gitignore` ne les couvre pas.

### P3-2 — Qualité de tests et absence d'automatisation

**Preuve.** 5 des 10 tests de `server.rs` sont des assertions sur le texte source (`include_str!(...).contains(...)`), par exemple `une_terminaison_ne_peut_pas_etre_selectionnee_dans_un_lot` (`server.rs:592-597`) qui vérifie la présence exacte de `item.advice.action.kind !== 'kill'`. Ces tests cassent sur un refactor inoffensif et passent sur une régression qui conserve la chaîne. Aucun fichier `.github/workflows`, `Makefile`, `rust-toolchain` ni licence n'est versionné : les quatre commandes de recette restent manuelles et leur résultat n'est garanti que par le journal rédigé à la main (rejoué ici, exact).

**Recommandation.** Extraire les décisions (`batchable`, formulation, fermeture de modale) dans des fonctions pures testables, et ajouter une CI minimale exécutant `fmt --check`, `test`, `clippy -D warnings`, `build --release`, `node --check` et `sha256sum -c`.

### P3-3 — Robustesse et finitions

- `util.rs:173-176` : l'indice de `.exe` est calculé sur `cmd.to_lowercase()` puis appliqué à `cmd` original. `to_lowercase` peut changer la longueur en octets de certains caractères Unicode (`İ`, `ẞ`) ; une ligne de commande non ASCII peut alors provoquer un slice hors frontière UTF-8, donc un panic dans la collecte. Recommandation : travailler sur la chaîne originale avec un prédicat insensible à la casse, ou utiliser `char_indices`.
- `server.rs:463` : `std::process::exit(0)` alors que 4 threads de service (`server.rs:441-452`) peuvent être en cours d'écriture d'un point de restauration. Risque de JSON tronqué. Recommandation : arrêt coopératif avec `join`.
- `main.rs:75` : `port..port.saturating_add(12)` produit une plage vide pour un port proche de 65 535 (échec sans tentative d'écoute).
- `app.js:605` et `app.js:704` : « Irrréversible » (trois `r`) dans deux textes présentés à l'utilisateur avant une action destructive.
- `report.rs:5-10` : `esc()` n'échappe pas `'`, sans conséquence aujourd'hui (aucune donnée injectée dans un attribut entre apostrophes), mais à harmoniser avec `app.js:36-38`.

---

## 5. Limites de cette revue

Les vérifications rejouées portent sur la compilation, les tests unitaires, le formatage, l'analyse statique, la syntaxe JS et l'intégrité des preuves. **Je n'ai pas** lancé l'application, ni exercé l'interface dans un navigateur, ni appliqué d'action réelle, ni testé les chemins FFI sur un processus vivant, ni injecté une ACL Windows ou un `SystemRoot` forgé, ni fait de fuzzing ou d'analyse de dépendances (`cargo audit`). Les constats P1 sont donc établis par lecture de code et de la dépendance HTTP, avec leurs préconditions explicitées — pas par exploitation. Le journal `PLAN.md:508-532` reste la source de vérité pour les mesures sur machine réelle en lecture seule (289 services, 201 tâches, 806–844 ms de collecte).

---

## 6. Priorisation proposée

| Ordre | Action | Effort | Pourquoi avant distribution |
|---|---|---|---|
| 1 | P1-1 : `GetSystemDirectoryW` + test d'environnement forgé | faible | exécution élevée d'un binaire contrôlé |
| 2 | P1-2 : documenter le modèle de confiance, exiger `from`, confronter au snapshot, jeton CSPRNG | moyen | frontière de sécurité réelle, pas théorique |
| 3 | P2-4 : propager les options CLI dans `elevate()` | faible | chemin par défaut cassé sur machine réelle |
| 4 | P2-1, P2-2, P2-3 : origine obligatoire, corps borné, verrous réellement indépendants | faible | trois écarts entre documentation et code |
| 5 | P2-5, P2-6 : provenance exposée, zéro mesuré respecté | moyen | dette de vérité des données |
| 6 | P3-1 : corriger `PLAN.md` §1/7.0 et **pousser les 6 commits** | immédiat | sauvegarde hors machine |
| 7 | P3-2, P3-3 : CI, tests comportementaux, finitions | moyen | rendre la recette reproductible |

**Conclusion.** Le projet mérite d'être conservé en l'état comme base de travail : je n'ai trouvé aucune régression fonctionnelle par rapport aux objectifs des étapes 0 à 6, et les affirmations du journal résistent à un rejeu indépendant. Les réserves portent sur des angles que le registre D1–D23 n'a pas examinés — l'autorité des chemins système, l'authentification du client local, la conservation des options d'exécution — et sur une dette de provenance transformée en invariant. Ce sont des corrections ciblées, pas une refonte ; le lot de clôture 8 peut rester fermé, un lot 9 de sécurité est en revanche justifié.

---

## 7. Post-scriptum de synchronisation (2026-11-14)

Cette revue a été rédigée sur `d08e412`, puis commitée sous `ecccadc`. Au moment de la rédaction, `origin/main` était à `03bed3e` et le décalage était de six commits ; il est de sept après le commit de revue. Le seul remote configuré est `Roger364/q-memory-manager`. L'ancien nom de dépôt cité dans `PLAN.md` était une erreur documentaire : le dépôt correspondant répond `404`, et aucune trace de cette URL n'existe dans `.git/config` ni dans les références Git. Les fichiers de test non suivis signalés dans P3-1 étaient des artefacts temporaires d'un autre agent ; ils ont été supprimés et ne font pas partie du dépôt.
