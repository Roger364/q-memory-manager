# Plan d'améliorations — RamScope, lot 9

**Statut :** plan révisé le 2026-11-14 après évaluation de la revue Hermes-Beam. C7 et C6 sont implémentés localement avec leurs tests ; C6 est validé dans Windows Sandbox pour la propagation des options. C2 est implémenté, poussé et validé le 2026-11-15 par des actions réelles de service, de tâche planifiée et d'entrée de démarrage, chacune suivie d'une restauration dans Windows Sandbox ; le correctif de repli des tâches issu de ces tests est inclus. D9-2a est implémenté dans 9.11 et couvert par 67 tests unitaires ; le test UAC sur un compte standard a été exécuté le 2026-11-15 dans un Windows Sandbox de test (acceptation et annulation réelles). C1–C5 et C8–C15 restent à traiter. L'industrialisation 9.10 est partiellement livrée.
**Base :** revue WorkBuddy (audit `d08e412`) et revue Hermes-Beam (audit `3673120`) ; plan révisé sur `main` au commit `761343e`, après la fusion de la mise à jour Dependabot.
**Objectif :** rendre RamScope distribuable sans refonte, en corrigeant d'abord la frontière de confiance, puis les écarts entre le code et la documentation, puis la qualité des tests.

> Les constats viennent d'une lecture de code et de contrôles rejoués, pas d'exploits exécutés. Le plan distingue donc ce qui doit être corrigé de ce qui doit d'abord être décidé.

## 1. Évaluation de la revue Hermes-Beam

### 1.1 Ce que la revue apporte

La revue est crédible et utile. Elle audite `3673120`, cite des lignes précises, rejoue les contrôles et confirme les constats de la revue WorkBuddy sans reprendre son ancien diagnostic de remote. Elle apporte trois éléments qui manquaient ou qui étaient trop implicites dans le premier plan :

- **C1** : `Rule.source` et `Rule.reviewed` sont effectivement du code mort, et l'audit de `rules.rs` impose `reviewed == "non datée"`. Une vraie date de revue ferait donc échouer un test censé garantir la qualité des règles.
- **C8** : plusieurs tests de `server.rs` vérifient la présence de chaînes dans les sources (`include_str!`) plutôt que le comportement. Ils peuvent casser sur un refactor inoffensif et passer sur une régression qui conserve la chaîne.
- **C15** : aucun fichier de licence n'est présent. C'est un choix juridique à documenter avant toute distribution publique.

Elle précise aussi deux points déjà présents dans le plan : le jeton de session n'authentifie pas un processus local, et le champ `from` fourni par le client peut être absent, ce qui saute la revérification d'état.

### 1.2 Ce que je corrige dans son classement

Je ne reprends pas son ordre tel quel :

- **C1 est une dette de vérité des données, pas une élévation de privilèges.** Il reste un critère de sortie avant distribution, mais il ne doit pas retarder C7 et C2, qui sont des risques d'exécution élevée et de frontière de confiance.
- **C2 est un risque local, pas une faille distante.** Le plan doit rendre explicite le modèle de menace et éviter de promettre une authentification que le jeton actuel ne fournit pas.
- **C15 est une décision, pas un correctif de code.** Le plan ne choisit pas une licence à la place de Ro ; il impose une décision avant distribution.
- **C8 est un chantier de qualité de tests**, séparé des correctifs de sécurité. Il ne doit pas être noyé dans la CI déjà livrée.

### 1.3 Verdict par constat

| Constat | Verdict | Destination | Effet sur le plan |
|---|---|---|---|
| C1 — provenance morte et audit figé | Confirmé, nouveau | 9.6 | Ajout d'un chantier distinct, critère de sortie avant distribution |
| C2 — API locale élevée et état fourni par le client | Confirmé, déjà connu | 9.3 | Précision sur `from`, `was_enabled`, le jeton et le canal local |
| C3 — origine optionnelle | Confirmé, déjà connu | 9.4 | Inchangé, test explicite |
| C4 — corps non borné | Confirmé, déjà connu | 9.4 | Inchangé, limite avant lecture |
| C5 — verrous non indépendants | Confirmé, déjà connu | 9.5 | Inchangé |
| C6 — options CLI perdues à l'élévation | Confirmé, déjà connu | 9.2 | Inchangé, test d'arguments |
| C7 — chemins système via l'environnement | Confirmé, déjà connu | 9.1 | Inchangé, priorité sécurité |
| C8 — tests sur le texte source | Confirmé, nouveau | 9.9 | Chantier dédié, distinct de la CI |
| C9 — `extract_exe` et Unicode | Confirmé, déjà connu | 9.8 | Inchangé |
| C10 — arrêt brutal avec threads actifs | Confirmé, déjà connu | 9.8 | Inchangé |
| C11 — plage de ports vide | Confirmé, déjà connu | 9.8 | Inchangé |
| C12 — faute « Irrréversible » | Confirmé, déjà connu | 9.8 | Inchangé |
| C13 — apostrophe non échappée | Confirmé, déjà connu | 9.8 | Inchangé |
| C14 — zéro mesuré confondu avec absence | Confirmé, déjà connu | 9.7 | Séparé de la provenance |
| C15 — licence absente | Confirmé, nouveau | 9.10 | Décision explicite avant distribution |

Aucun constat n'est rejeté. La revue ne remplace toutefois pas un test d'action réelle : ses constats de sécurité restent des lectures de code avec préconditions explicitées.

## 2. Principes

- Ne jamais appliquer d'action réelle à la machine de tous les jours ; les tests d'écriture se font dans une VM ou un Windows Sandbox jetable.
- Un correctif sans test de réversion nommé n'est pas terminé.
- Ne pas annoncer une garantie de sécurité qui n'a pas été testée.
- Conserver les garde-fous existants : `Host` exact, jeton, origine stricte, validation serveur et verrous moteur.
- Un commit par sujet, avec la commande et le code retour consignés dans `PLAN.md`.
- Corriger d'abord les risques concrets et peu risqués ; les décisions d'architecture viennent après.
- Ne pas marquer un constat « corrigé » sans test qui échoue si la correction est annulée.

## 3. Décisions à prendre avant de coder

### D9-1 — Les processus locaux sont-ils dans le modèle de menace ?

**Décision (2026-11-15) : non fiables.** Un processus local du même compte peut lire la page, récupérer le jeton et forger un `Origin` : le jeton HTTP ne l'authentifie pas. RamScope ne doit donc pas prétendre protéger une session élevée contre un programme malveillant déjà exécuté sous ce compte.

Conséquences retenues pour la v1 :

- C2 est appliqué au niveau HTTP : jeton CSPRNG, état observé obligatoire, action limitée à celles proposées par le snapshot courant, restauration interne séparée.
- Le risque résiduel « processus local hostile » est **documenté et accepté pour la v1**, pas nié. Il ne doit pas être présenté comme un point corrigé.
- Le named pipe/DACL n'est **pas** un prérequis de la v1 : une DACL ouverte au navigateur du même utilisateur ne bloquerait pas forcément un processus du même compte. Cette refonte, ou un helper élevé avec consentement par lot, fera l'objet d'un lot séparé si le besoin est confirmé.
- La réduction de la durée de la session administrateur et l'élévation explicite restent recommandées avant une distribution publique (voir D9-2), mais ne sont pas mélangées à C2.

### D9-2 — Conserver l'élévation UAC par défaut ?

**Décision validée le 2026-11-15 (option A) et implémentée :** ne plus élever au lancement par défaut. Le lancement normal devient une session de lecture seule ; l'utilisateur demande explicitement l'élévation depuis l'interface après avoir consulté l'inventaire. Un nouveau `--admin` permet l'élévation au lancement pour ceux qui la veulent dès le départ, et `--no-admin` devient un mode strict qui interdit toute élévation ultérieure.

Cette étape réduit la durée d'exposition avant l'UAC, pas encore la durée de la session élevée. Un helper d'application one-shot, qui ne garderait les droits administrateur que quelques secondes, reste un lot séparé (D9-2b) : il touche au canal local et à l'authentification, et ne doit pas être mélangé à C2/D9-1. Le détail du lot est en **9.11** ; le test UAC sur compte standard a été exécuté le 2026-11-15.

### D9-3 — API HTTP ou canal local dédié ?

Le remplacement complet de HTTP par un named pipe avec DACL est structurel et coûteux. Le plan propose une étape minimale d'abord, puis une décision séparée sur le canal. Ne pas lancer cette refonte sans avoir mesuré le besoin réel.

### D9-4 — Portée du lot

Le lot 9 ne doit pas contenir de refonte graphique, de nouvelle base de règles, de signature de code ni de télémétrie.

### D9-5 — Licence et distribution

Décider si le dépôt reste privé, devient public, ou est distribué sous forme de binaire. En l'absence de décision, aucune licence permissive ne doit être ajoutée par défaut. C15 reste ouvert jusqu'à cette décision.

### D9-6 — Portée des tests comportementaux

C8 ne signifie pas automatiser tout le navigateur. Le plan privilégie des fonctions pures testables (sélection, formulation, calcul de gains) et ne réserve les tests DOM ou navigateur qu'aux parcours qui ne peuvent pas être testés autrement.

## 4. Vue d'ensemble

| Ordre | Chantier | Constats | Effort indicatif |
|---:|---|---|---:|
| 1 | Chemins système et données non influençables | C7 | 0,5 j |
| 2 | Options CLI conservées à l'élévation | C6 | 0,25 j |
| 3 | Frontière de confiance de l'API élevée | C2 | 1,5–2 j |
| 4 | Origine obligatoire et corps borné | C3, C4 | 0,5 j |
| 5 | Verrous réellement indépendants | C5 | 0,5 j |
| 6 | Provenance réelle des règles | C1 | 0,5 j |
| 7 | Zéro mesuré respecté | C14 | 0,25 j |
| 8 | Robustesse et finitions | C9–C13 | 0,5 j |
| 9 | Tests comportementaux | C8 | 1 j |
| 10 | Industrialisation et licence | C15 + état CI | 0,1 j |
| 11 | Réduction de l'élévation UAC | D9-2a | 0,5–0,75 j |

Total indicatif : **7 à 8 jours**, hors tests réels en VM et hors décision sur le canal local et le helper one-shot.

## 5. Chantiers détaillés

### 9.1 — Chemins système et données : C7

**État :** implémenté le 2026-11-14, tests ajoutés.

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
- Vérification FFI ou manuelle : le chemin retourné est identique avec un `SystemRoot` normal et forgé.

**Critère de sortie.**
- `grep` : plus de `std::env::var("SystemRoot")` ni de `std::env::var("LOCALAPPDATA")` dans les chemins d'autorité.
- Les nouveaux tests passent avec un environnement forgé.
- Aucun changement de comportement sur une collecte réelle en lecture seule.

**Réversion.** Le test avec `SystemRoot` forgé échoue si le code revient à la variable d'environnement.

### 9.2 — Élévation : C6

**État :** implémenté le 2026-11-14, tests ajoutés. Validation Sandbox effectuée le 2026-11-15 ; binaire release autonome vérifié.

**Objectif.** Les options choisies ne doivent pas disparaître au moment où RamScope demande l'élévation.

**Périmètre.** `main.rs` : construction des arguments ; `win.rs` : `elevate` ; tests d'arguments.

**Changements.**
- Transmettre à l'instance élevée la ligne de commande complète, pas seulement `--elevated`.
- Éviter la duplication de `--elevated` et toute boucle d'élévation.
- Échapper correctement les arguments contenant des espaces.
- Tester `--port`, `--host localhost`, `--no-browser` et la combinaison avec `--elevated`.
- Consigner le port et l'hôte effectifs dans un log de démarrage ou les exposer dans `/api/health` pour rendre le test vérifiable.

**Tests.**
- Test unitaire de la fonction de reconstruction des arguments : la liste est préservée et `--elevated` n'apparaît qu'une fois.
- Test avec un faux `ShellExecuteW` : les paramètres reçus contiennent le port et l'hôte.
- Test réel en VM : `--port 8917 --host localhost --no-browser` avec élévation, vérifier l'écoute sur `localhost:8917` et l'absence d'ouverture de fenêtre.

**Validation Windows Sandbox (2026-11-15, binaire CRT statique).**
- Commande initiale : `--port 8917 --host 127.0.0.1 --no-browser --no-admin --demo`.
- Processus élevé observé, PID `9060` :
  `"C:\Users\WDAGUtilityAccount\AppData\Local\Temp\ramscope-c6.exe" --port 8917 --host 127.0.0.1 --no-browser --demo --elevated`.
- `POST /api/elevate` : HTTP `200` ; ancien PID `4792` ; processus élevé unique.
- Contrôles : port conservé, hôte `127.0.0.1` conservé, `--no-browser` conservé, `--no-admin` retiré, `--elevated` unique : **tous vrais**.
- Le serveur a répondu sur `127.0.0.1:8917` ; le mode `--demo` évite toute action sur le système invité.
- Le test a été exécuté avec le binaire release de `920 576` octets et **sans `vcruntime140.dll`** copiée dans l'invité ; le script échoue désormais si cette DLL est présente.

**Limites du test.** Le compte du Sandbox est administrateur : ce test valide la reconstruction et la transmission des arguments par `ShellExecuteW`, pas le consentement UAC d'un compte standard. Le test n'a pas été rejoué avec `--host localhost` ; l'hôte effectif consigné est `127.0.0.1`. Le binaire release importait auparavant `VCRUNTIME140.dll` ; `ramscope/.cargo/config.toml` force désormais la CRT statique (`-C target-feature=+crt-static`), et les imports `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll`, `MSVCP140.dll` et les DLL UCRT dynamiques vérifiées sont absents du binaire reconstruit.

**Critère de sortie.** Le chemin par défaut n'ignore plus aucune option documentée ; le binaire release ne dépend plus de la CRT VC++ dynamique.

**Réversion.** Le test d'arguments échoue si la reconstruction ne conserve pas le port ; retirer `ramscope/.cargo/config.toml` réintroduit l'import `VCRUNTIME140.dll`.

### 9.3 — Frontière de confiance de l'API élevée : C2

**Objectif.** Le serveur ne doit pas croire l'état fourni par le client et ne doit accepter que les actions réellement proposées par l'analyse courante. Un processus local hostile reste hors du modèle de menace de la v1 : le jeton protège la session navigateur, pas l'identité d'un programme local (D9-1).

**Périmètre.** `server.rs`, `plan.rs`, `win.rs`, `util.rs`, `model.rs`, `SAFETY.md`, `README.md`, tests HTTP et tests unitaires.

**Changements minimaux, dans cet ordre.**
1. **Jeton** : générer le jeton avec `BCryptGenRandom` (Windows) plutôt qu'avec `new_id()` horloge/PID/xorshift.
2. **Validation d'état côté serveur** : pour `Service`, `Startup` et `Task`, exiger `from`/`was_enabled` sur toute action ordinaire, puis les comparer au snapshot serveur fraîchement collecté. Une action sans état observé est refusée, sauf les actions internes de restauration qui ne passent pas par cette route. Une action doit aussi être exactement celle proposée par le snapshot (`advice.action`), et non une cible arbitraire du même type.
3. **Restauration** : distinguer explicitement le chemin interne de restauration du chemin public `/api/apply` ; ne pas accepter `from: None` comme contournement sur une action utilisateur. Le démarrage d'un service (`StartService`) est réservé à ce chemin interne.
4. **Documentation du modèle** : acter D9-1 dans `SAFETY.md` et le README : processus locaux non fiables, jeton limité à la session navigateur, risque résiduel accepté pour la v1. Ne pas annoncer un contrôle d'intégrité du client ni un canal authentifié qui n'existe pas encore.
5. **Décision structurelle séparée** : le named pipe/DACL ou un helper élevé avec consentement par lot reste un lot dédié. Ne pas mélanger cette refonte avec les correctifs ci-dessus.

**Tests.**
- `refuse_un_from_null_sur_une_action_utilisateur` : `from: null` → refus.
- `refuse_un_etat_divergent_du_snapshot` : état différent de l'état serveur → refus.
- `accepte_les_actions_et_l_etat_du_snapshot` : les trois types d'action proposés passent avec l'état courant.
- `refuse_une_action_non_proposee_meme_avec_le_bon_etat` : cible réelle mais action hors snapshot → refus.
- `refuse_le_demarrage_d_un_service_sur_la_route_utilisateur` : `StartService` réservé à la restauration.
- `accepte_un_arret_immediat_propose_et_refuse_un_arret_arbitraire` : l'arrêt immédiat suit la même règle de proposition.
- `accepte_une_terminaison_proposee_par_le_snapshot` : la terminaison doit correspondre à l'identité et à l'action proposées.
- `une_restauration_interne_rejoue_un_undo_sans_etat_observe` : les inverses `from: None` restent applicables par le moteur.
- `lit_l_etat_natif_d_une_tache_pour_le_rollback` : le parseur d'état accepte `Enabled=false`, `true` et l'absence de balise (activée par défaut).
- `le_jeton_de_session_est_long_hexadecimal_et_imprevisible` : 32 octets CSPRNG, 64 caractères, deux jetons différents.
- Les tests existants de `Host`, d'origine et de jeton restent verts ; un jeton absent ou invalide est toujours refusé.

**Critère de sortie.**
- Le scénario « processus local non élevé lit le jeton puis applique une action » est explicitement documenté comme hors modèle de menace de la v1 (D9-1), avec le risque résiduel nommé.
- Aucune action de service/startup/tâche ne passe sans état observé, et aucune action hors snapshot n'est acceptée par `/api/apply`.
- `SAFETY.md` et le README ne prétendent plus que le jeton authentifie l'appelant.

**Réversion.** `refuse_un_from_null_sur_une_action_utilisateur` et `refuse_une_action_non_proposee_meme_avec_le_bon_etat` échouent si la validation d'état ou la limitation au snapshot est retirée.

**Vérification HTTP en démonstration (2026-11-15).** Le binaire release a été lancé avec `--demo --no-browser --port 0` (port effectif 63242), puis un client Python a lu le jeton dans la page et envoyé les requêtes avec `Host`, `Origin` et `X-RamScope-Token` corrects :

- `from: null` → `409`, « État constaté du service manquant » ;
- `from` divergent → `409`, « L'état du service a changé depuis l'analyse » ;
- état courant conservé mais autre `start` → `409`, « Action non proposée par l'analyse courante » ;
- action exacte du snapshot (service `dmwappushservice`, Manuel → Désactivé) → `200`, point de restauration créé ;
- restauration interne de ce point (`from: None`) → `200`, service revenu à Manuel.

Les tests unitaires couvrent la fonction pure et le moteur ; ce test HTTP couvre le branchement réel de la route. Depuis ce test, deux tests unitaires supplémentaires couvrent l'arrêt immédiat et la terminaison proposés ; le test de la tâche planifiée en Sandbox a ensuite motivé un test de lecture d'état natif (57 tests au total).

**Vérification interface (2026-11-15).** L'interface a été ouverte dans Chromium sur le binaire release en mode démonstration. Le service `dmwappushservice` (Manuel → Désactivé) a été sélectionné, confirmé et appliqué : « Application : 1/1 réussi », point de restauration `dfb8a171e56522c1` créé. Le même point a ensuite été restauré depuis l'onglet Restauration : « Restauration : 1/1 réussi », service revenu à Manuel. Le journal navigateur ne contient aucune erreur console et les appels `/api/apply` et `/api/restore` ont répondu `200`. Aucune action n'a été appliquée sur une machine réelle lors de ce test d'interface.

**Vérification réelle en Windows Sandbox (2026-11-15).** Le binaire release a été exécuté **sans `--demo`**, avec `--elevated`, dans le Sandbox C6 déjà ouvert (le second Sandbox a été refusé : une seule instance à la fois).

- **Service.** `dmwappushservice` a été choisi parmi les actions proposées : `manual` avant, `disabled` après `POST /api/apply` (`200`), confirmé par `Win32_Service.StartMode=Disabled`. Le point `526d88c293ac7808` a ensuite été restauré par `POST /api/restore` (`200`) : snapshot et `Win32_Service.StartMode` revenus à `manual`/`Manual`.
- **Tâche planifiée.** Le premier essai a révélé un défaut réel : la tâche `\Microsoft\Windows\Application Experience\StartupAppTask` était proposée depuis le repli `schtasks`, mais `current_task_enabled` ne lisait que `System32\Tasks`, donc l'application répondait « tâche introuvable ». La lecture d'état utilise désormais le même repli `schtasks /query /tn <chemin> /xml`. Après reconstruction, la tâche est passée de `Ready` à `Disabled` (`POST /api/apply`, `200`), puis de `Disabled` à `Ready` après restauration (`POST /api/restore`, `200`, point `145b6b0780efdd93`). L'état natif a été vérifié avec `Get-ScheduledTask` avant, après application et après restauration.
- **Entrée de démarrage.** Aucune entrée proposée n'était présente dans l'image du Sandbox ; le test a créé une entrée HKCU `RamScopeC2StartupTest` (commande `spotify.exe`) pour exercer le mécanisme réel `StartupApproved`. L'état natif avant était activé avec une valeur d'approbation absente ; après `POST /api/apply` (`200`), la valeur est passée à `3` (désactivée), puis à `2` (activée) après `POST /api/restore` (`200`, point `ae9d473adf0a7a32`). L'entrée de test et sa valeur d'approbation ont été supprimées après vérification ; le Run original n'a pas été modifié.

Le test service vérifie aussi l'absence de copie de `vcruntime140.dll`. Le Sandbox est resté ouvert. Limites : un seul service, une seule tâche et une entrée de démarrage synthétique, compte Sandbox administrateur, pas de test du consentement UAC standard ni d'action processus réelle.

### 9.4 — Origine et taille des requêtes : C3, C4

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

### 9.5 — Verrous réellement indépendants : C5

**Objectif.** Soit les deux niveaux de refus sont réellement indépendants, soit la documentation cesse de le prétendre.

**Périmètre.** `win.rs`, `server.rs`, `SAFETY.md`, `PLAN.md`, tests.

**Changements.**
- Ajouter dans `apply_service`, `apply_startup` et `apply_task` une vérification directe des listes protégées, indépendante de `plan::validate`.
- Vérifier de nouveau l'état courant juste avant l'écriture, même si le serveur a déjà validé.
- Si cette duplication n'est pas souhaitée, corriger la formulation « deux verrous » en « validation aux deux points d'entrée », avec un renvoi à `plan::validate`.
- Documenter précisément ce que chaque couche garantit.

**Tests.**
- Test qui appelle directement `win::apply` avec un service protégé, sans passer par `server`, et attend un refus.
- Test qui appelle directement `win::apply` avec un état `from` devenu faux et attend un refus.
- Test de non-régression pour l'application normale d'un service non protégé.

**Critère de sortie.** Un défaut dans `plan::validate` ne suffit plus à contourner tous les refus sur les services, ou la documentation ne parle plus de verrous indépendants.

**Réversion.** Le test d'appel direct à `win::apply` échoue si le contrôle moteur est retiré.

### 9.6 — Provenance réelle des règles : C1

**Objectif.** La provenance affichée doit être réelle, et l'audit ne doit plus figer la dette « non datée » en invariant impossible à faire évoluer.

**Périmètre.** `rules.rs`, `model.rs`, `report.rs`, `web/app.js`, tests.

**Changements.**
- Exposer `source` et `reviewed` dans `Advice` et dans le rapport exporté.
- Afficher ces champs dans le panneau de détails et dans l'export.
- Remplacer l'assertion `reviewed == REVIEWED` par : « `non datée` ou date ISO valide ».
- Ajouter un test avec une règle datée pour prouver que l'audit accepte une vraie revue.
- Ne pas ajouter une fausse date de revue aux 601 règles : la provenance de famille doit rester honnête.
- Supprimer `#[allow(dead_code)]` sur les champs lorsqu'ils sont effectivement consommés.

**Tests.**
- Une règle avec `reviewed = "non datée"` passe l'audit.
- Une règle avec une date ISO valide passe l'audit.
- Une date invalide échoue avec un message explicite.
- L'API et le rapport contiennent la source et la date de revue.

**Critère de sortie.** Le rapport et l'interface montrent la source et la date de revue ; l'audit accepte les deux états sans imposer la dette.

**Réversion.** Le test avec une règle datée échoue si l'assertion rigide est rétablie.

### 9.7 — Zéro mesuré respecté : C14

**Objectif.** Un zéro mesuré ne doit jamais être présenté comme une absence de mesure.

**Périmètre.** `model.rs`, `report.rs`, `web/app.js`, tests.

**Changements.**
- Faire dépendre l'affichage d'un gain de sa `nature` avant sa valeur.
- `Gain::measured(0)` doit rester `Measured` dans `merge`, le rapport et l'interface.
- Ne pas transformer `estimated(0)` en mesure : il reste indisponible, comme aujourd'hui.
- Tester `Gain::measured(0)` dans le rapport et le rendu JS.

**Critère de sortie.** Un gain mesuré nul affiche « 0 mesuré », jamais « non chiffrable ».

**Réversion.** Le test `measured(0)` échoue si le rendu redevient basé uniquement sur `bytes == 0`.

### 9.8 — Robustesse et finitions : C9–C13

**Objectif.** Supprimer des petits défauts qui peuvent produire un panic ou une expérience incohérente.

**Périmètre.** `util.rs`, `main.rs`, `server.rs`, `web/app.js`, `report.rs`.

**Changements.**
- C9 : `extract_exe` ne doit pas utiliser un index calculé sur `to_lowercase()` pour découper la chaîne originale ; utiliser `char_indices` ou une recherche ASCII insensible à la casse.
- C10 : remplacer `std::process::exit(0)` par un arrêt coopératif : signaler l'arrêt, attendre les threads, puis quitter.
- C11 : gérer proprement la plage de ports proche de 65 535, avec un message explicite si aucun port n'est disponible.
- C12 : corriger « Irrréversible » en « Irréversible ».
- C13 : harmoniser `esc()` et l'échappement JS pour l'apostrophe.
- Ajouter un test de ligne de commande Unicode dans `extract_exe` et un test de dernier port.

**Critère de sortie.**
- Aucun panic possible sur une ligne de commande Unicode dans `extract_exe`.
- Arrêt sans écriture de rollback tronquée.
- Textes utilisateur sans faute visible.

### 9.9 — Tests comportementaux : C8

**Objectif.** Réduire les tests qui vérifient des chaînes de source et augmenter la couverture des décisions réelles.

**Périmètre.** `server.rs`, `web/app.js`, éventuellement extraction de fonctions pures.

**Changements.**
- Recenser les tests `include_str!` de `server.rs` et conserver uniquement ceux qui protègent une contrainte de sécurité ou de formulation impossible à tester autrement.
- Extraire progressivement les décisions pures : `batchable`, calcul des gains, formulation utilisateur, fermeture de modale.
- Ajouter des tests unitaires sur ces fonctions pures.
- Pour les parcours DOM qui ne peuvent pas être testés sans navigateur, documenter la limite au lieu de prétendre les couvrir avec `contains`.
- Ne pas introduire une infra E2E lourde dans ce lot sans décision explicite.

**Critère de sortie.** Les tests de comportement nommés couvrent les décisions critiques ; les tests de chaînes restants sont justifiés et limités.

### 9.10 — Industrialisation et licence : C15

**Objectif.** Terminer ce qui est déjà livré et formaliser la décision de licence.

**Déjà livré.**
- CI Windows : formatage, 43 tests, clippy, build release, JS, sommes SHA-256, refus des fichiers non suivis.
- Dependabot pour Cargo et GitHub Actions.
- Remote `Roger364/q-memory-manager` synchronisé ; push fast-forward.
- Corrections documentaires de `PLAN.md` et du remote.

**Reste à décider.**
- C15 : choisir entre dépôt privé sans licence permissive, licence open source, ou distribution binaire avec conditions.
- Ne pas ajouter de `LICENSE` avant la décision de Ro.
- Une fois la décision prise, ajouter le fichier de licence et la mention correspondante dans le README.

**Critère de sortie.** La décision de licence est consignée ; la CI reste verte ; le dépôt distant est synchronisé.

### 9.11 — Réduction de l'élévation UAC : D9-2a

**État :** implémenté le 2026-11-15, tests unitaires au vert (67 tests). Le consentement UAC sur un compte standard a été testé le 2026-11-15 : acceptation et annulation réelles, avec `admin:true` après acceptation et `admin:false` / `can_elevate:true` après annulation. La route `POST /api/elevate` du bouton a aussi été testée ; elle a révélé puis fait corriger le conflit de port de la relance élevée. Le clic DOM et le rechargement de la page ont été validés avec Chrome for Testing portable : Edge n'a pas pu être installé dans le Sandbox (MSI 1603/1722).

**Décision validée le 2026-11-15 (option A).** Le lancement sans option devient une session de **lecture seule** ; l'élévation n'est demandée qu'après une action explicite de l'utilisateur.

**Alternatives examinées.**
- **A. Lecture seule par défaut + `--admin` (recommandée).** Réduit l'exposition initiale, garde un parcours simple pour appliquer après revue, et laisse un mode strict pour l'audit.
- **B. Statu quo.** Élévation automatique au lancement, avec `--no-admin` strict. Changement minimal, mais ne réduit pas la durée d'exposition et ne répond pas à D9-2.
- **C. Lecture seule sans `--admin`.** Plus simple, mais casse les scripts et les utilisateurs qui veulent une session élevée dès le départ ; rejetée pour cette raison.

| Lancement | UAC au démarrage | Élévation depuis l'interface | Usage |
|---|---|---|---|
| `ramscope.exe` | non | oui | parcours normal : inventaire, puis « Passer en administrateur » |
| `ramscope.exe --admin` | oui, sauf si déjà élevé | déjà élevé | utilisateur qui veut appliquer tout de suite |
| `ramscope.exe --no-admin` | non | non | lecture stricte, audit, aucune relance élevée |
| `ramscope.exe --demo` | non | non | démonstration |
| `--elevated` | interne | déjà élevé | relance de l'instance élevée, ne jamais l'exposer comme option utilisateur |

**Constat avant D9-2a.**
- `main.rs` élevait automatiquement dès le lancement si `--demo`, `--no-admin` et `--elevated` étaient absents et que le processus n'était pas déjà administrateur.
- `--no-admin` empêchait seulement l'élévation initiale : l'interface affichait encore « Passer en administrateur » et `/api/elevate` pouvait relancer une instance élevée.
- Une fois élevée, l'instance reste administrateur jusqu'à sa fermeture ; C2 ne change pas cette durée.
- `--elevated` et `--no-admin` sont retirés des arguments de la relance, puis `--elevated` est ajouté.

**Changements implémentés.**
1. **Mode d'élévation explicite et testable.** Extraire le calcul dans une fonction pure (par exemple `elevation_mode(args, deja_admin, demo)`) qui distingue : `ReadOnly`, `AdminAtStart`, `StrictReadOnly`, `Demo`, `AlreadyElevated`. Refuser la combinaison `--admin --no-admin` avec un message clair ; `--demo` reste sans UAC.
2. **Défaut en lecture seule.** Ne plus appeler `win::elevate()` dans le chemin par défaut. Seul `--admin` demande l'UAC au lancement. Si l'UAC est refusé, continuer en lecture seule et laisser l'interface expliquer l'état ; ne pas boucler.
3. **Mode strict.** `--no-admin` interdit `/api/elevate` (réponse `403` explicite), masque le bouton d'élévation et reste en lecture seule même après une tentative d'appel direct. Le mode est une propriété du serveur, pas du client.
4. **Propagation au serveur et à l'interface.** Ajouter un booléen `can_elevate` à la configuration du serveur et à `meta` (ou à `/api/health`), puis l'utiliser dans `app.js` pour masquer le bouton et afficher un message adapté. `/api/apply` et `/api/restore` conservent leurs garde-fous actuels.
5. **Arguments de relance.** `elevated_args` retire `--admin` et `--no-admin`, conserve le port, l'hôte, `--no-browser`, `--demo`, et n'ajoute qu'un seul `--elevated`.
6. **Documentation.** Mettre à jour `--help`, `README.md` et `SAFETY.md` après implémentation : le lancement normal ne demande plus UAC, `--no-admin` est strict, et la session élevée reste longue jusqu'à D9-2b.

**Tests ajoutés.**
- `le_defaut_ne_demande_pas_uac` : lancement sans option → `ReadOnly`.
- `admin_demande_l_uac_une_seule_fois` : mode `AdminAtStart`, pas de boucle.
- `no_admin_est_strict` : mode `StrictReadOnly`, `can_elevate=false`, `/api/elevate` → `403`.
- `admin_et_no_admin_sont_rejetes` : erreur de démarrage explicite.
- `elevated_args_retire_admin_et_no_admin` : port/hôte/`--no-browser` conservés, `--elevated` unique.
- `le_serveur_strict_ne_propose_pas_elevate` : contrôle serveur/JS, pas seulement le texte du README.
- `le_snapshot_annonce_le_droit_d_elevation` : propagation de `can_elevate` au snapshot.
- `une_relance_elevee_reessaie_le_port_avant_de_changer` : le retry de port est limité à `--elevated`, et seulement tant que le quota de tentatives n'est pas épuisé.
- Test dans une VM avec un **compte standard** : défaut sans UAC ; clic « Passer en administrateur » → UAC ; annulation → l'ancien serveur lecture seule reste vivant ; acceptation → une seule instance élevée ; `--admin` → UAC au démarrage ; `--no-admin` → aucune proposition d'élévation. **Exécuté le 2026-11-15 dans un Windows Sandbox de test : les scénarios `--admin` d'acceptation et d'annulation, `POST /api/elevate` avec `admin:false` → UAC acceptée → `admin:true` sur le même port, le clic DOM et le rechargement avec Chrome for Testing portable sont validés ; `--no-admin` sur VM standard reste à rejouer.**

**Résultats du test standard (2026-11-15).** Le Sandbox est livré avec UAC désactivé ; `EnableLUA=1` a été appliqué puis le Sandbox a été redémarré. Un compte `RamScopeStd` (Utilisateurs) et un compte administrateur distinct ont été créés. Les deux scénarios ont produit une vraie fenêtre `consent.exe` (`Credential Dialog Xaml Host`) avec les boutons Oui/Non :

| Scénario | Action UAC | Résultat RamScope |
|---|---|---|
| Acceptation, `--admin --no-browser --port 8899` | clic Oui | instance élevée : `admin:true`, `can_elevate:false`, `ok:true` ; processus standard sorti ; `/api/quit` 200 |
| Annulation, `--admin --no-browser --port 8898` | clic Non | instance standard toujours vivante : `admin:false`, `can_elevate:true`, `ok:true` ; `/api/quit` 200 |
| Route du bouton, `POST /api/elevate` sur défaut 8898 | clic Oui | avant `admin:false`, `can_elevate:true` ; après `admin:true`, `can_elevate:false` sur 8898 ; `original_exited:true` ; `/api/quit` 200 |
| Clic DOM, Chrome for Testing, défaut 8898 | clic Oui | `button_text=Passer en administrateur`, `click_method=cdp_dom` ; après `admin:true`, `can_elevate:false`, `original_exited:true` ; rechargement : badge `Administrateur`, bouton absent ; `/api/quit` 200 |

Le bureau sécurisé a été désactivé uniquement dans ce Sandbox pour permettre l'automatisation du clic ; le chemin applicatif reste `ShellExecuteW("runas")` puis la relance `--elevated`. Les deux premiers scénarios utilisent `--admin` ; le bouton « Passer en administrateur » a été cliqué dans Chrome for Testing portable, et la page rechargée affiche `Administrateur` sans bouton d'élévation. Edge n'a pas pu être installé dans le Sandbox (MSI 1603/1722) ; le test DOM utilise donc Chromium, pas Edge. Les preuves sont conservées dans `sandbox-test/d92/accept-evidence/`, `sandbox-test/d92/cancel-result.txt`, `sandbox-test/d92/endpoint-evidence/` et `sandbox-test/d92/button-evidence/`. Le Sandbox C6 est resté ouvert ; le Sandbox de test d92 a été fermé après relevé.

**Défaut de port corrigé.** Avant correction, `/api/elevate` répondait `ok:true` et lançait bien `--elevated`, mais l'instance élevée se rabattait sur 8899 parce que l'ancien serveur occupait encore 8898 ; le relevé après acceptation échouait. `main.rs` réessaie désormais le port demandé pendant 5 s pour `--elevated`. Après rebuild, le test d'endpoint a observé `admin:true`, `can_elevate:false`, `original_exited:true` et `/api/quit` 200 sur 8898.

**Réversion.** `le_defaut_ne_demande_pas_uac` échoue si le code réintroduit l'élévation automatique ; `no_admin_est_strict` échoue si `/api/elevate` reste accessible.

**Critère de sortie.**
- [x] Aucun UAC n'est demandé sans action explicite ou `--admin` (tests unitaires).
- [x] `--no-admin` est réellement strict et testé côté serveur (403 et `can_elevate=false`).
- [x] Les options, la relance et les parcours C2 restent inchangés.
- [x] Les scénarios `--admin` d'acceptation et d'annulation sur compte standard sont consignés ; C6, compte administrateur, ne peut pas valider le consentement UAC.
- [x] La route `POST /api/elevate` du bouton est testée sur compte standard, y compris le retour sur le même port après acceptation ; le défaut de port est corrigé.
- [x] Le clic DOM et le rechargement après acceptation sont validés avec Chrome for Testing portable ; Edge n'a pas pu être installé dans le Sandbox.
- [ ] `--no-admin` sur VM standard reste à rejouer.

**Effort indicatif :** 0,5 à 0,75 jour, plus le test VM. Aucun changement de dépendance.

**Risques et parades.**
- **Confusion utilisateur** : l'interface doit expliquer la lecture seule et proposer un bouton nommé, pas seulement refuser l'action.
- **UAC annulé** : conserver l'instance lecture seule ; ne pas laisser de processus élevé orphelin.
- **Fausse promesse de sécurité** : cette étape réduit la durée d'exposition avant l'UAC, pas la surface d'une instance déjà élevée. D9-2b (helper one-shot) reste séparé et ne doit pas être revendiqué avant tests.
- **D9-1 inchangé** : un processus local hostile du même compte reste hors modèle de menace.

**Hors périmètre de ce lot.** Le helper élevé par lot, le named pipe/DACL, la suppression de l'UAC et toute revendication de protection contre un malware local.

## 6. Ordre de livraison recommandé

1. **9.1** : chemins système et données (C7), indépendant et testable rapidement.
2. **9.2** : propagation des options CLI (C6), correctif fonctionnel concret.
3. **9.3** : frontière de confiance (C2), après décision D9-1.
4. **9.4** : origine obligatoire et corps borné (C3, C4).
5. **9.5** : verrous indépendants ou correction documentaire (C5).
6. **9.6** : provenance réelle des règles (C1), avant distribution.
7. **9.7** : zéro mesuré (C14).
8. **9.8** : robustesse et finitions (C9–C13).
9. **9.9** : tests comportementaux (C8).
10. **9.10** : décision de licence (C15).
11. **9.11** : réduction de l'élévation UAC (D9-2a), implémentée ; les parcours `--admin`, `POST /api/elevate`, le clic DOM et le rechargement sur compte standard sont testés.

Chaque étape doit être un commit séparé, avec son test nommé, sa commande et son code retour. Ne pas regrouper 9.1 à 9.4 dans un même commit : une revue de sécurité doit pouvoir être relue isolément.

C1 est un critère de sortie avant distribution, mais pas une raison de retarder C7/C2. La distribution publique reste bloquée tant que C1, C2 et C7 ne sont pas traités, et que C15 n'est pas décidé.

## 7. Matrice de tests à ajouter

| Domaine | Test attendu |
|---|---|
| Chemins | `SystemRoot` forgé, `LOCALAPPDATA` forgé, `schtasks`/`explorer` absolus |
| Élévation | arguments conservés, `--elevated` unique, port/hôte effectifs, défaut sans UAC, `--no-admin` strict |
| API | `Host` invalide, jeton absent/invalide, origine absente/invalide |
| État | `from` nul refusé, `from` divergent refusé, restauration interne autorisée |
| Corps | `Content-Length` énorme, corps chunked énorme |
| Verrous | appel direct à `win::apply` sur cible protégée |
| Provenance | source exposée, date « non datée » et date ISO acceptées |
| Gains | hôte complet/partiel, `measured(0)`, `estimated`, `unavailable` |
| Robustesse | `extract_exe` Unicode, port 65 535, arrêt propre |
| Tests | décisions pures extraites, tests de chaînes restants justifiés |
| Preuves | `sha256sum`, CI, absence de fichiers de test non suivis |

Les tests d'écriture réelle restent limités à une VM ou un Sandbox jetable. La collecte réelle sur l'hôte reste en `--no-admin` et ne doit jamais appliquer d'action.

## 8. Critères de sortie du lot 9

- [x] Les 43 tests historiques passent toujours ; les 67 tests actuels passent.
- [ ] Les nouveaux tests de réversion passent pour les autres chantiers.
- [x] `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release` : code retour 0.
- [x] C2 est implémenté et couvert par les tests de réversion nommés ; le risque local résiduel est documenté dans `SAFETY.md` (D9-1).
- [x] La CI est verte sur `c9771fc` : run Windows `43663058587`, toutes les étapes au vert.
- [ ] C1 et C7 sont corrigés ou explicitement documentés comme hors modèle de menace.
- [ ] C3–C6 et C14 sont corrigés ou explicitement acceptés dans `SAFETY.md`.
- [ ] C9–C13 sont corrigés.
- [ ] C8 est traité par des tests comportementaux ciblés, avec les limites documentées.
- [ ] C15 est décidé et consigné.
- [x] D9-2a est implémenté et couvert par les tests unitaires ; `--no-admin` est strict côté serveur, le défaut est en lecture seule et la relance ne boucle pas.
- [x] D9-2a est testé avec un compte standard (consentement, annulation, route `POST /api/elevate`, clic DOM et rechargement) ; Edge n'a pas pu être installé dans le Sandbox, le clic DOM a été validé sous Chrome for Testing. Le helper one-shot reste hors périmètre.
- [ ] La documentation ne promet plus de verrou, d'authentification ou de garantie non testée.
- [x] Des actions réelles ont été testées dans un Windows Sandbox jetable : `dmwappushservice` Manuel → Désactivé → Manuel, `StartupAppTask` Ready → Disabled → Ready, et une entrée de démarrage `StartupApproved` 3 → 2, avec les API natives `Win32_Service`, `Get-ScheduledTask` et le registre vérifiés avant, après application et après restauration.
- [x] Le dépôt distant est à jour après accord de Ro, y compris le compte rendu du test de démarrage.
- [ ] `PLAN.md` contient les commandes, codes retour et artefacts du lot.
- [x] C7 et C6 sont implémentés et couverts par des tests unitaires locaux ; C6 est validé dans Windows Sandbox pour la propagation des options (`--port`, `--host`, `--no-browser`, `--elevated` unique, `--no-admin` retiré). Le consentement UAC d'un compte standard et le test `localhost` restent à faire.

## 9. Hors périmètre

- Refonte de l'interface ou changement de framework front.
- Remplacement complet de HTTP par un canal local dédié, sauf décision explicite après 9.3.
- Nouvelles règles métier ou nouvelle base de règles.
- Signature de code, installeur, distribution publique tant que C1/C2/C7/C15 ne sont pas traités.
- Télémétrie, collecte distante ou nettoyage automatique.

## 10. Risques et parades

| Risque | Parade |
|---|---|
| Casser le chemin par défaut en corrigeant l'élévation | test d'arguments + test réel en VM avec port/hôte explicites |
| Introduire une régression de sécurité en déplaçant la validation | tests `from` nul/divergent, appel direct au moteur, revue dédiée |
| Refonte du canal local trop large | décision séparée après mesure ; ne pas mélanger avec 9.1–9.4 |
| Test d'audit qui verrouille la dette de provenance | accepter explicitement `non datée` ou une date ISO, avec un test daté |
| Tests de texte source qui remplacent de vrais tests | extraire les fonctions pures et limiter `include_str!` aux gardes justifiées |
| Fausse preuve de sécurité | ne jamais marquer un P1 « corrigé » sans test de réversion et, si possible, PoC local |
| Licence choisie par défaut sans décision | garder C15 comme décision explicite de Ro |
| Perte de sauvegarde distante | push après accord, vérifier `git ls-remote` |

## 11. Prochaine action

C7, C6, C2 et D9-2a sont traités. D9-2a est fermé avec les preuves UAC, route et DOM ; la seule réserve de test est Edge, qui n'est pas installable dans le Sandbox utilisé (le clic DOM a été validé sous Chrome for Testing, même moteur Chromium).

Ordre recommandé pour la suite :
1. **Clôturer les preuves D9-2a** : rejouer `--no-admin` sur une VM à compte standard, vérifier `--host localhost`, et faire la validation Edge spécifique sur une image où Edge est préinstallé. Ce sont des tests, pas des changements de produit ; ils ne bloquent pas le mode par défaut déjà validé sous Chromium.
2. **D9-2b — helper elevated one-shot**, mais uniquement après une note de conception dédiée : canal local, authentification, durée de vie et test de réversion. Ne pas le mélanger à D9-1 ni à C2.
3. **C3/C4/C5** : origine obligatoire, corps borné, verrous réellement indépendants. Faible risque, forte valeur de durcissement.
4. **C9–C13** : robustesse et finitions, avec tests ciblés.
5. **C1/C15** : provenance réelle des règles et décision de licence, avant toute distribution publique.

La revue Hermes-Beam ne change pas cette priorité : elle confirme C7 et C6, ajoute C1/C8/C15, et justifie de renforcer 9.3 sur l'état fourni par le client. Le plan est désormais aligné sur ces constats.
