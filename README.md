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
- **Gains** : un gain « tout de suite » correspond à une fermeture ou à un arrêt ; un gain « au prochain démarrage » correspond à un service passé en manuel ou à un démarrage désactivé. Ce sont des estimations, pas des mesures.

## Niveaux

| Niveau | Sens |
|---|---|
| Vital | Composant dont l'arrêt casse Windows ou la session. Verrouillé. |
| Système | Composant Windows normal, à laisser tel quel. |
| À garder | Utile pour ton usage (pilote, sécurité, usage quotidien). |
| Selon usage | Le bon réglage dépend de ce que tu fais. |
| À alléger | Démarrage automatique ou agent d'arrière-plan superflu. |
| Superflu | Inutile dans la quasi-totalité des cas. |
| Non répertorié | RamScope ne sait pas : aucune action proposée. |

Les services et processus verrouillés sont refusés par le moteur, quel que soit le point d'entrée, et pas seulement masqués dans l'interface.

## Actions

- Type de démarrage d'un service : automatique, automatique différé, manuel, désactivé.
- Entrée de démarrage : activée ou désactivée via le mécanisme `StartupApproved`, le même que le Gestionnaire des tâches.
- Tâche planifiée : activée ou désactivée.
- Arrêt immédiat d'un service.
- Terminaison d'un processus : irréversible, sans restauration possible.

Chaque lot appliqué qui contient au moins une action réussie et réversible crée un point de restauration JSON. La restauration rejoue les actions inverses ; elle ne recrée pas un processus terminé.

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
- La politique de contenu autorise encore les styles et le gestionnaire inline de l'interface actuelle (`'unsafe-inline'`) ; elle sera resserrée avec la refonte de l'interface.

## Ce qui n'a pas été testé

- Aucune action n'a été appliquée à une machine réelle dans le cadre des contrôles consignés : les parcours d'application et de restauration ont été exercés en mode démonstration.
- La collecte réelle a été compilée (`cargo test`, `cargo build --release`) mais n'a pas été comparée ici au Gestionnaire des tâches ni à `services.msc`.
- Les tests unitaires couvrent la validation des actions, la totalisation des gains et les invariants des tables de règles. Le détail des commandes et des codes retour figure dans `PLAN.md`.

Voir `SAFETY.md` pour les limites d'usage et `PLAN.md` pour le plan de travail et les défauts connus.
