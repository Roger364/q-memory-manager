# Plan d'améliorations — RamScope, lot 9

**Statut :** proposition, à valider avant implémentation. Aucun code n'a été modifié.
**Base :** revue technique WorkBuddy du 2026-11-14, audit du code au commit `d08e412` ; plan rédigé alors que le HEAD local était `ecccadc`.
**Objectif :** rendre RamScope distribuable sans refonte, en corrigeant d'abord la frontière de confiance, puis les écarts entre le code et la documentation, puis l'industrialisation des tests.

> Les constats de la revue sont issus d'une lecture de code, pas d'exploits exécutés. Le plan distingue donc ce qui doit être corrigé de ce qui doit d'abord être décidé.

## 1. Principes

- Ne jamais appliquer d'action réelle à la machine de tous les jours ; les tests d'écriture se font dans une VM ou un Windows Sandbox jetable.
- Un correctif sans test de réversion nommé n'est pas terminé.
- Ne pas annoncer une garantie de sécurité qui n'a pas été testée.
- Conserver les garde-fous existants : `Host` exact, jeton, origine stricte, validation serveur et verrous moteur.
- Un commit par sujet, avec la commande et le code retour consignés dans `PLAN.md`.
- Corriger d'abord ce qui est concret et peu risqué ; les décisions d'architecture viennent après.

## 2. Décisions à prendre avant de coder

### D9-1 — Les processus locaux sont-ils dans le modèle de menace ?

- **Si oui** : P1-2 est une vraie élévation locale de privilèges et doit être traitée structurellement (canal local authentifié, pas seulement un jeton lisible dans une page).
- **Si non** : documenter explicitement que RamScope fait confiance aux processus de l'utilisateur, et appliquer seulement les mesures de réduction de surface. Cette position doit être écrite dans `SAFETY.md` : elle ne doit pas être implicite.

Recommandation : considérer les processus locaux comme non fiables tant que RamScope s'élève par défaut. C'est le cas le plus prudent et le plus cohérent avec une API qui peut arrêter des services.

### D9-2 — Conserver l'élévation UAC par défaut ?

Si oui, P1-2 et P2-4 sont prioritaires. Si l'usage recommandé devient `--no-admin`, il faut le dire clairement dans le README et ne plus présenter l'élévation comme le chemin normal.

### D9-3 — API HTTP ou canal local dédié ?

Le remplacement complet de HTTP par un named pipe avec DACL est structurel et coûteux. Le plan propose une étape minimale d'abord, puis une décision séparée sur le canal. Ne pas lancer cette refonte sans avoir mesuré le besoin réel.

### D9-4 — Portée du lot

Le lot 9 ne doit pas contenir de refonte graphique, de nouvelle base de règles, de signature de code ni de télémétrie.

## 3. Vue d'ensemble

| Priorité | Chantier | Constat | Effort indicatif |
|---|---|---|---:|
| 1 | Chemins système et dossier de données non influençables | P1-1 | 0,5 j |
| 2 | Options CLI conservées à l'élévation | P2-4 | 0,25 j |
| 3 | Frontière de confiance de l'API élevée | P1-2 | 1,5–2 j |
| 4 | Origine obligatoire et corps borné | P2-1, P2-2 | 0,5 j |
| 5 | Verrous réellement indépendants | P2-3 | 0,5 j |
| 6 | Provenance des règles et zéro mesuré | P2-5, P2-6 | 1 j |
| 7 | Robustesse et finitions | P3-3 | 0,5 j |
| 8 | CI, documentation, sauvegarde distante | P3-1, P3-2 | 1,25 j |

Total indicatif : **6 à 7 jours**, hors tests réels en VM et hors décision sur le canal local.

## 4. Chantiers détaillés

### 9.1 — Chemins système et données : P1-1

**Objectif.** Empêcher un environnement forgé de faire exécuter un binaire élevé ou de rediriger les points de restauration.

**Périmètre.** `win.rs` : `system32_path`, `data_dir`, éventuellement `elevate` ; `server.rs` pour `explorer.exe` ; tests de `win.rs`.

**Changements.**
- Résoudre `System32` avec `GetSystemDirectoryW`, jamais avec `SystemRoot` ou `windir`.
- Résoudre `%LOCALAPPDATA%` avec `SHGetKnownFolderPath(FOLDERID_LocalAppData)`.
- Conserver un helper unique et testable pour les chemins d'outils (`schtasks.exe`, `explorer.exe`, `Tasks`).
- Supprimer le repli silencieux vers `C:\Windows` et `"."` sur Windows : un échec de résolution doit produire une erreur explicite ou une note de collecte, pas un chemin choisi par l'environnement.
- Vérifier que le processus élevé ne dépend d'aucune variable d'environnement héritée pour localiser un outil système.

**Tests.**
- Test nommé : `systemroot_forge_ne_change_pas_le_chemin_systeme`.
- Test nommé : `localappdata_forge_ne_change_pas_le_dossier_donnees`.
- Test du helper pour `schtasks.exe` et `explorer.exe`.
- Vérification manuelle ou test FFI : le chemin retourné est identique avec un `SystemRoot` normal et forgé.

**Critère de sortie.**
- `grep` : plus de `std::env::var("SystemRoot")` ni de `std::env::var("LOCALAPPDATA")` dans les chemins d'autorité.
- Les nouveaux tests passent avec un environnement forgé.
- Aucun changement de comportement sur une collecte réelle en lecture seule.

**Réversion.** Le test avec `SystemRoot` forgé échoue si le code revient à la variable d'environnement.

### 9.2 — Élévation : P2-4

**Objectif.** Les options choisies ne doivent pas disparaître au moment où RamScope demande l'élévation.

**Périmètre.** `main.rs` : construction des arguments ; `win.rs` : `elevate` ; tests d'arguments.

**Changements.**
- Transmettre à l'instance élevée la ligne de commande complète, pas seulement `--elevated`.
- Éviter la duplication de `--elevated` et toute boucle d'élévation.
- Échapper correctement les arguments contenant des espaces.
- Tester `--port`, `--host localhost`, `--no-browser` et la combinaison avec `--elevated`.
- Consigner le port et l'hôte effectifs dans un log de démarrage ou les exposer dans `/api/health` pour rendre le test vérifiable.

**Tests.**
- Test unitaire de la fonction de reconstruction des arguments : la liste d'arguments est préservée, `--elevated` n'apparaît qu'une fois.
- Test manuel ou automatisé avec un faux `ShellExecuteW` : les paramètres reçus contiennent le port et l'hôte.
- Test réel en VM : `--port 8917 --host localhost --no-browser` avec élévation, vérifier l'écoute sur `localhost:8917` et l'absence d'ouverture de fenêtre.

**Critère de sortie.** Le chemin par défaut n'ignore plus aucune option documentée.

**Réversion.** Le test d'arguments échoue si la reconstruction ne conserve pas le port.

### 9.3 — Frontière de confiance de l'API élevée : P1-2

**Objectif.** Un processus local non élevé ne doit pas pouvoir faire exécuter des actions au serveur élevé sans preuve d'autorisation, et le serveur ne doit pas croire l'état fourni par le client.

**Périmètre.** `server.rs`, `plan.rs`, `win.rs`, `util.rs`, `model.rs`, `SAFETY.md`, `README.md`, tests HTTP et tests unitaires.

**Changements minimaux, dans cet ordre.**
1. **Jeton** : générer le jeton avec `BCryptGenRandom` (Windows) plutôt qu'avec `new_id()` horloge/PID/xorshift.
2. **Validation d'état côté serveur** : pour `Service`, `Startup` et `Task`, exiger `from`/`was_enabled` sur toute action ordinaire, puis les comparer au snapshot serveur. Une action sans état observé est refusée, sauf les actions internes de restauration qui ne passent pas par cette route.
3. **Documentation du modèle** : indiquer noir sur blanc dans `SAFETY.md` si les processus locaux sont supposés fiables. Si le choix est « non fiables », ajouter :
   - un contrôle d'intégrité du client ou un canal local authentifié (named pipe avec DACL) ;
   - une limitation claire de l'API HTTP à la session navigateur ;
   - un refus des mutations sans `Origin` (voir 9.4).
4. **Décision structurelle séparée** : si le canal nommé est retenu, créer un lot dédié. Ne pas mélanger cette refonte avec les correctifs ci-dessus.

**Tests.**
- `apply` avec `from: null` → refus.
- `apply` avec `from` différent de l'état serveur → refus.
- `restore` continue de fonctionner sans `from`.
- Jeton absent, invalide, ou provenant d'un autre port → refus.
- Test de non-régression : un `Host` autorisé et un jeton valide restent nécessaires.
- Test de génération : deux jetons successifs ne sont pas prévisibles par l'horloge/PID ; au minimum, vérifier la source CSPRNG et sa longueur.

**Critère de sortie.**
- Le scénario « processus local non élevé lit le jeton puis applique une action » est soit bloqué, soit explicitement documenté comme hors modèle de menace.
- Aucune action de service/startup/tâche ne passe sans état observé côté serveur.
- `SAFETY.md` ne prétend plus que le jeton authentifie l'appelant.

**Réversion.** Le test `from: null` échoue si la validation d'état est retirée.

### 9.4 — Origine et taille des requêtes : P2-1, P2-2

**Objectif.** Réduire la surface de mutation et empêcher un déni de service local par corps géant.

**Périmètre.** `server.rs`, `README.md`, `SAFETY.md`, tests HTTP.

**Changements.**
- Exiger `Origin` sur toute mutation (`POST`), avec la même comparaison exacte que pour l'API.
- Ajouter `Sec-Fetch-Site: same-origin` en défense en profondeur si le client navigateur le fournit ; ne pas en faire la seule authentification.
- Rejeter `Content-Length` supérieur à une limite documentée (par exemple 1 Mio) avant lecture.
- Borner aussi la lecture réelle (`take`) pour couvrir les corps sans `Content-Length` ou en chunked.
- Choisir et documenter le code retour : `400` ou `413`, mais rester cohérent et testé.
- Mettre à jour les exemples `curl` de `PLAN.md` et le README pour inclure `Origin`.

**Tests.**
- `POST /api/apply` sans `Origin` → `403`.
- `POST /api/apply` avec `Origin` inconnue → `403`.
- `POST` avec `Content-Length` > 1 Mio → `413` ou `400` sans lecture complète.
- Test de corps chunked surdimensionné.
- Les tests existants d'application et de restauration envoient l'origine correcte.

**Critère de sortie.** Aucun `POST` sans origine valide ne peut atteindre une mutation ; aucun corps arbitrairement grand n'est lu.

**Réversion.** Le test sans `Origin` échoue si le contrôle redevient optionnel.

### 9.5 — Verrous réellement indépendants : P2-3

**Objectif.** Soit les deux niveaux de refus sont réellement indépendants, soit la documentation cesse de le prétendre.

**Périmètre.** `win.rs`, `server.rs`, `SAFETY.md`, `PLAN.md`, tests.

**Changements.**
- Ajouter dans `apply_service`, `apply_startup`, `apply_task` une vérification directe des listes protégées, indépendante de `plan::validate`.
- Vérifier de nouveau l'état courant juste avant l'écriture, même si le serveur a déjà validé.
- Si cette duplication n'est pas souhaitée, corriger la formulation « deux verrous » en « validation aux deux points d'entrée », avec un renvoi à `plan::validate`.
- Documenter précisément ce que chaque couche garantit.

**Tests.**
- Test qui appelle directement `win::apply` avec un service protégé, sans passer par `server`, et attend un refus.
- Test qui appelle directement `win::apply` avec un état `from` devenu faux et attend un refus.
- Test de non-régression pour l'application normale d'un service non protégé.

**Critère de sortie.** Un défaut dans `plan::validate` ne suffit plus à contourner tous les refus sur les services, ou la documentation ne parle plus de verrous indépendants.

**Réversion.** Le test d'appel direct à `win::apply` échoue si le contrôle moteur est retiré.

### 9.6 — Provenance et zéro mesuré : P2-5, P2-6

**Objectif.** La provenance affichée doit être réelle, et un zéro mesuré ne doit jamais être présenté comme une absence de mesure.

**Périmètre.** `rules.rs`, `model.rs`, `report.rs`, `web/app.js`, tests.

**Changements.**
- Exposer `source` et `reviewed` dans `Advice`.
- Afficher ces champs dans le panneau de détails et dans le rapport exporté.
- Remplacer l'assertion `reviewed == "non datée"` par : « non datée » ou date ISO valide.
- Ajouter un test avec une règle datée pour prouver que l'audit accepte une vraie revue.
- Faire dépendre l'affichage d'un gain de sa `nature` avant sa valeur : `measured(0)` affiche « 0 mesuré », pas « non chiffrable ».
- Tester `Gain::measured(0)` dans `merge`, `report.rs` et le rendu JS.
- Ne pas ajouter une fausse date de revue aux 601 règles : la provenance de famille doit rester honnête.

**Critère de sortie.**
- Le rapport et l'interface montrent la source et la date de revue.
- Un gain mesuré nul reste mesuré partout.
- Le test d'audit ne gèle plus la dette « non datée ».

**Réversion.** Le test `measured(0)` échoue si le rendu redevient basé uniquement sur `bytes == 0`.

### 9.7 — Robustesse et finitions : P3-3

**Objectif.** Supprimer des petits défauts qui peuvent produire un panic ou une expérience incohérente.

**Périmètre.** `util.rs`, `main.rs`, `server.rs`, `web/app.js`, `report.rs`.

**Changements.**
- `extract_exe` : ne pas utiliser un index calculé sur `to_lowercase()` pour découper la chaîne originale ; utiliser `char_indices` ou une recherche ASCII insensible à la casse.
- `main.rs` : gérer proprement la plage de ports proche de 65 535, avec un message explicite si aucun port n'est disponible.
- `server.rs` : remplacer `std::process::exit(0)` par un arrêt coopératif : signaler l'arrêt, attendre les threads, puis quitter.
- Corriger « Irrréversible » en « Irréversible ».
- Harmoniser `esc()` et l'échappement JS pour l'apostrophe.
- Ajouter un test de ligne de commande Unicode dans `extract_exe` et un test de dernier port.

**Critère de sortie.**
- Aucun panic possible sur une ligne de commande Unicode dans `extract_exe`.
- Arrêt sans écriture de rollback tronquée.
- Textes utilisateur sans faute visible.

### 9.8 — CI, documentation, sauvegarde distante : P3-1, P3-2

**Objectif.** Rendre la recette reproductible et remettre la documentation et le dépôt distant en accord avec la réalité.

**Périmètre.** `.github/workflows`, `PLAN.md`, `README.md`, `SAFETY.md`, remote Git.

**Changements.**
- Ajouter une CI minimale sur `windows-latest` : `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release`, `node --check src/web/app.js`, `sha256sum -c docs/evidence/SHA256SUMS` (ou équivalent PowerShell).
- Ajouter un job ou une étape qui refuse les fichiers de test non suivis.
- Corriger dans `PLAN.md` : nombre de tests `plan.rs`/`rules.rs`, URL du remote, état réel du push.
- Ne plus présenter les tests d'assertions sur le texte source comme des tests comportementaux ; extraire progressivement les décisions pures.
- Pousser les commits locaux vers `Roger364/q-memory-manager` **après accord explicite de Ro** ; vérifier ensuite `git ls-remote origin HEAD`.
- Ne pas ajouter de fichier de test temporaire à la racine ; le `.gitignore` doit couvrir les logs et dossiers de test éphémères.

**Critère de sortie.**
- CI verte sur le commit de clôture.
- `git ls-remote origin HEAD` et `git rev-parse HEAD` concordent après le push autorisé.
- `PLAN.md` ne contient plus les trois affirmations fausses relevées par la revue.

## 5. Ordre de livraison recommandé

1. **9.1** : chemins système et données (indépendant, testable rapidement).
2. **9.2** : propagation des options CLI (correctif fonctionnel concret).
3. **9.4** : origine obligatoire et corps borné (petit périmètre HTTP).
4. **9.3** : validation d'état, jeton et décision sur le modèle de confiance.
5. **9.5** : verrous indépendants ou correction documentaire.
6. **9.6** : provenance et zéro mesuré.
7. **9.7** : robustesse.
8. **9.8** : CI, documentation et push.

Chaque étape doit être un commit séparé, avec son test nommé, sa commande et son code retour. Ne pas regrouper 9.1 à 9.4 dans un même commit : une revue de sécurité doit pouvoir être relue isolément.

## 6. Matrice de tests à ajouter

| Domaine | Test attendu |
|---|---|
| Chemins | `SystemRoot` forgé, `LOCALAPPDATA` forgé, `schtasks`/`explorer` absolus |
| Élévation | arguments conservés, `--elevated` unique, port/hôte effectifs |
| API | `Host` invalide, jeton absent/invalide, origine absente/invalide |
| Corps | `Content-Length` énorme, corps chunked énorme |
| État | `from` nul, `from` divergent, restauration sans `from` |
| Verrous | appel direct à `win::apply` sur cible protégée |
| Gains | hôte complet/partiel, `measured(0)`, `estimated`, `unavailable` |
| Provenance | source exposée, date « non datée » et date ISO acceptées |
| Robustesse | `extract_exe` Unicode, port 65 535, arrêt propre |
| Preuves | `sha256sum`, CI, absence de fichiers de test non suivis |

Les tests d'écriture réelle restent limités à une VM ou un Sandbox jetable. La collecte réelle sur l'hôte reste en `--no-admin` et ne doit jamais appliquer d'action.

## 7. Critères de sortie du lot 9

- [ ] Les 43 tests existants passent toujours.
- [ ] Les nouveaux tests de réversion passent.
- [ ] `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release` : code retour 0.
- [ ] La CI est verte et reproductible.
- [ ] Aucun P1 ouvert ; les P2 restants sont soit corrigés, soit explicitement acceptés dans `SAFETY.md`.
- [ ] La documentation ne promet plus de verrou, d'authentification ou de garantie non testée.
- [ ] Une action réelle a été testée dans une VM/Sandbox jetable, avec restauration.
- [ ] Le dépôt distant est à jour après accord de Ro.
- [ ] `PLAN.md` contient les commandes, codes retour et artefacts du lot.

## 8. Hors périmètre

- Refonte de l'interface ou changement de framework front.
- Remplacement complet de HTTP par un canal local dédié, sauf décision explicite après 9.3.
- Nouvelles règles métier ou nouvelle base de règles.
- Signature de code, installeur, distribution publique.
- Télémétrie, collecte distante ou nettoyage automatique.

## 9. Risques et parades

| Risque | Parade |
|---|---|
| Casser le chemin par défaut en corrigeant l'élévation | test d'arguments + test réel en VM avec port/hôte explicites |
| Introduire une régression de sécurité en déplaçant la validation | tests `from` nul/divergent, appel direct au moteur, revue dédiée |
| Refonte du canal local trop large | décision séparée après mesure ; ne pas mélanger avec 9.1–9.4 |
| Tests de texte source qui remplacent de vrais tests | extraire les fonctions pures et limiter `include_str!` aux gardes de formulation |
| Fausse preuve de sécurité | ne jamais marquer un P1 « corrigé » sans test de réversion et, si possible, PoC local |
| Perte de sauvegarde distante | push après accord, vérifier `git ls-remote` |

## 10. Prochaine action

Commencer par **9.1 et 9.2**, qui sont bornés, testables sans machine jetable et corrigent deux risques concrets. Ne pas ouvrir 9.3 avant d'avoir tranché la question D9-1 : selon la réponse, le travail va d'un simple durcissement à une modification de la frontière de confiance.
