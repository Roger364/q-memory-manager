# RamScope

RamScope inventorie la mémoire, les services et le démarrage automatique d'un PC Windows 10/11, explique ce qui occupe la RAM, et propose des réglages réversibles. Il ne supprime aucun fichier et ne modifie aucun réglage sans confirmation explicite.

Un seul exécutable, aucune installation, aucune dépendance à l'exécution. L'interface est une page web servie sur la boucle locale et ouverte dans une fenêtre d'application.

## Compiler

Prérequis : Rust stable, cible Windows.

```text
cd ramscope
cargo build --release
```

Le binaire est produit dans `ramscope\target\release\ramscope.exe`.

## Lancer

```text
ramscope.exe                 # collecte réelle, élévation UAC demandée
ramscope.exe --demo          # données simulées, aucun accès à la machine
ramscope.exe --no-admin      # lecture seule, sans élévation
```

| Option | Effet |
|---|---|
| `--port N` | port d'écoute (8765 par défaut, essaie les 11 suivants s'il est occupé) |
| `--host ADRESSE` | adresse d'écoute : `127.0.0.1` (défaut) ou `localhost` ; toute autre valeur est refusée au démarrage |
| `--no-browser` | ne pas ouvrir la fenêtre automatiquement |
| `--no-admin` | rester en lecture seule, sans demander l'élévation |
| `--demo` | données de démonstration, aucun accès réel |
| `--help` | aide |

Sans `--demo` ni `--no-admin`, RamScope demande l'élévation (UAC) au lancement : modifier un service ou une tâche exige les droits administrateur. En lecture seule, l'inventaire reste consultable mais le serveur refuse toute application.

## Lire les chiffres

- **Mémoire physique** : total, disponible et charge, via `GlobalMemoryStatusEx`. « Utilisée » vaut total moins disponible.
- **Working set** : mémoire résidente d'un processus, la valeur que montre le Gestionnaire des tâches.
- **Mémoire privée** : mémoire non partagée, celle qu'un processus rend réellement en se fermant. C'est la base des gains « tout de suite ».
- **Commit, noyau, cache système** : compteurs système, via `GetPerformanceInfo`.
- **Mémoire compressée** : working set du processus « Memory Compression ». C'est une approximation du compteur système, pas le compteur lui-même.
- **RAM d'un service** : part du processus hôte (`RAM de l'hôte ÷ nombre de services hébergés`). Plusieurs services d'un même `svchost.exe` partagent la même mémoire : ces parts sont approximatives et ne s'additionnent pas.
- **Gains** : chaque gain porte sa nature — mesuré, estimé ou non chiffrable — et sa portée. Un gain « tout de suite » correspond à une fermeture ou à un arrêt ; un gain « au prochain démarrage » correspond à un service passé en manuel ou à un démarrage désactivé. Sans mesure, le gain est « non chiffrable », jamais inventé.
- **Hôtes partagés** : la part d'un service dans un `svchost.exe` n'est pas additive. Un hôte ne compte que si tous ses services figurent dans le lot ; son gain vaut alors la RAM de l'hôte, une seule fois. Sinon, aucun gain n'est annoncé pour cet hôte.
- **Arrêt immédiat** : cocher « Arrêter aussi les services maintenant » ne transforme pas un gain différé en gain immédiat. La mémoire disponible est mesurée avant et après l'application, avec une attente bornée ; le delta est affiché comme mesuré.

## Niveaux

| Niveau | Sens |
|---|---|
| Vital | Composant dont l'arrêt casse Windows ou la session. |
| Système | Composant Windows normal, à laisser tel quel. |
| À garder | Utile pour ton usage (pilote, sécurité, usage quotidien). |
| Selon usage | Le bon réglage dépend de ce que tu fais. |
| À alléger | Démarrage automatique ou agent d'arrière-plan superflu. |
| Superflu | Inutile dans la quasi-totalité des cas. |
| Non répertorié | RamScope ne sait pas : aucune action proposée. |

Un cadenas signale un élément **verrouillé** : le serveur puis le moteur refusent toute action, quel que soit le point d'entrée, et pas seulement dans l'interface. Le verrou est distinct du niveau : un service verrouillé mais non vital garde son niveau de la table (par exemple « À garder ») et n'est plus étiqueté « Vital ». Les règles portent une provenance de famille, pas une revue individuelle : voir `SAFETY.md`.

## Actions

- Type de démarrage d'un service : automatique, automatique différé, manuel, désactivé.
- Entrée de démarrage : activée ou désactivée via le mécanisme `StartupApproved`, le même que le Gestionnaire des tâches.
- Tâche planifiée : activée ou désactivée.
- Arrêt immédiat d'un service.
- Terminaison d'un processus : irréversible, sans restauration possible. Le PID est rouvert avant l'action ; si le chemin d'image ou l'heure de création diffèrent de l'analyse, la terminaison est refusée.

Chaque lot appliqué qui contient au moins une action réussie et réversible crée un point de restauration JSON ; un lot qui ne contient que des terminaisons n'en crée aucun. Le rapport exporté sépare le gain immédiat et le gain au prochain démarrage, avec la nature de chacun. La restauration rejoue les actions inverses ; elle ne recrée pas un processus terminé.

## Interface de décision

- La sélection de masse « Sélectionner les éléments réversibles à gain différé » ne coche que des services, démarrages et tâches classés « superflu » ou « à alléger » ; elle ne sélectionne jamais un processus.
- « Terminer » reste une action directe : bouton rouge, étiquette « irréversible », jamais inclus dans un lot. La confirmation nomme le processus et rappelle que le travail non enregistré sera perdu.
- Le bouton « Détails » ouvre, pour chaque ligne, la source détectée, l'état actuel, l'effet tout de suite, l'effet au prochain démarrage, la réversibilité et la justification.
- La barre de sélection sépare le gain tout de suite et le gain au prochain démarrage ; chaque valeur porte son badge mesuré, estimé ou non chiffrable.

## Fichiers écrits

Sous Windows, dans `%LOCALAPPDATA%\RamScope` :

- `rollbacks\<id>.json` — points de restauration ;
- `rapports\ramscope-<date>.json` et `.html` — exports (créés à la demande) ;
- `erreur.log` — écrit uniquement si l'interface n'a pas pu démarrer.

Hors Windows, `ramscope-data\` dans le répertoire courant, pour le développement uniquement.

## Sécurité

- Écoute uniquement sur la boucle locale (`127.0.0.1` ou `localhost`) ; toute autre adresse est refusée au démarrage.
- L'en-tête `Host` est validé contre les adresses de la session : une requête portant un autre hôte, y compris par rebinding DNS, reçoit `403`, pages comprises.
- Toute route `/api/` exige le jeton de session ; l'origine des mutations est comparée exactement au `Host`, sans comparaison par suffixe.
- Le jeton reste présent dans la page servie, mais uniquement pour un `Host` autorisé.
- La politique de contenu autorise encore les styles et le gestionnaire inline de l'interface actuelle (`'unsafe-inline'`) ; son resserrage reste à faire.

## Ce qui n'a pas été testé

- Aucune action n'a été appliquée à une machine réelle dans le cadre des contrôles consignés : les parcours d'application et de restauration ont été exercés en mode démonstration.
- La collecte réelle a été exécutée le 2026-11-14 en lecture seule (`--no-admin`). La mémoire totale, les 289 services et les 201 tâches planifiées correspondent aux API Windows (`Get-Service`, `Get-ScheduledTask`) ; les 7 entrées de démarrage sont plus complètes que les 6 de `Win32_StartupCommand`, car RamScope lit aussi le registre 32 bits. Le chemin administrateur qui lit directement les XML des tâches n'a pas été comparé au repli `schtasks`, et le repli ajoute environ 0,6 à 0,8 s à la collecte en lecture seule.
- Les 38 tests unitaires couvrent la validation des actions, l'identité simulée d'un processus, l'agrégation des gains par hôte, la nature des gains dans le rapport, l'audit des 601 règles (unicité, casse, modes, sources, verrous), le faux positif `php`/`hp` et l'absence d'action des heuristiques sans source. Les refus d'identité réelle (`win.rs`) restent non exercés sur un processus vivant ; le delta de mémoire mesuré a été vérifié en démonstration, où la mémoire simulée ne varie pas. La revue visuelle de l'étape 5 a ouvert les panneaux de détails et les modales dans Chromium ; elle n'a pas fait de parcours clavier avec de vraies frappes ni rouvert les fichiers exportés. Le test `decode_une_tache_xml` couvre le parseur des tâches, pas la lecture réelle des ACL. Le détail des commandes et des codes retour figure dans `PLAN.md`.

Voir `SAFETY.md` pour les limites d'usage et `PLAN.md` pour le plan de travail et les défauts connus.
