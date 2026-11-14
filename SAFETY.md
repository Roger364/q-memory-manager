# Prudence d'usage

RamScope est un outil d'observation et de décision, pas un nettoyeur. Il ne supprime aucun fichier et ne modifie aucun réglage sans confirmation explicite.

## Trois choses à ne pas confondre

- **Mesure** : ce que le système rapporte maintenant (total, disponible, working set, mémoire privée).
- **Estimation** : un gain calculé à partir d'une mesure, ou l'attribution d'une part de mémoire partagée. Toujours approximatif.
- **Recommandation** : un jugement sur l'utilité d'un élément. Ce n'est pas une mesure, et cela peut être faux pour ton usage.

Un gain porte désormais sa nature : **mesuré** (delta de mémoire disponible avant/après un arrêt), **estimé** (calcul à partir d'une mesure) ou **non chiffrable** (aucune mesure disponible). « Non chiffrable » ne veut pas dire zéro.

Un chiffre élevé n'est pas une preuve qu'un élément est inutile.

## Ce que RamScope refuse

- Modifier ou arrêter un service de la liste `PROTECTED_SERVICES` (`src/rules.rs`).
- Terminer un processus de la liste `PROTECTED_PROCESSES`, un PID inférieur ou égal à 8, ou RamScope lui-même.
- Terminer un processus dont le chemin d'image ou l'heure de création ne correspond plus à ce qui a été analysé : le PID est rouvert et son identité revérifiée avant la terminaison.
- Appliquer une action sans droits administrateur.

Ces refus existent à deux niveaux : validation dans `src/plan.rs`, garde-fous dans `src/win.rs`. Ils restent actifs même si l'interface est contournée.

## Réversible, partiellement réversible, irréversible

| Action | Retour possible |
|---|---|
| Changer le type de démarrage d'un service | Oui : le type précédent est enregistré dans le point de restauration |
| Désactiver une entrée de démarrage | Oui : l'état précédent est enregistré |
| Désactiver une tâche planifiée | Oui : l'état précédent est enregistré |
| Arrêter un service maintenant | Partiel : il peut être redémarré, mais son état, ses connexions et son cache sont perdus |
| Terminer un processus | **Non** : le travail non enregistré est perdu, et le point de restauration ne recrée pas le processus |

Un point de restauration est écrit pour un lot qui contient au moins une action réussie et réversible. Il ne couvre pas les terminaisons de processus.

L'interface ne présélectionne aucune action risquée : la sélection de masse ne coche que des changements de configuration réversibles à gain différé, jamais un processus. « Terminer » reste une action directe, hors de tout lot, avec une confirmation qui nomme le processus et rappelle la perte du travail non enregistré.

## Effets différés

- Passer un service en manuel ne libère pas sa mémoire immédiatement : cela change ce qui se passe au prochain démarrage, et une dépendance peut le relancer à la demande.
- Désactiver un démarrage n'arrête pas le programme déjà lancé.
- Arrêter un service n'équivaut pas à le désactiver : il peut redémarrer seul.

## Limites des chiffres

- La mémoire d'un service hébergé dans `svchost.exe` est partagée : RamScope affiche une part, pas une mesure isolée. Ces parts ne s'additionnent pas.
- La mémoire « compressée » affichée est le working set du processus Memory Compression : approximation.
- Les gains de configuration sont des estimations ; un arrêt immédiat affiche en plus un delta de mémoire disponible mesuré après coup. Le reste n'est jamais présenté comme mesuré.
- Un service hébergé dans un `svchost.exe` partagé ne compte que si **tous** les services de cet hôte sont dans le lot : le gain vaut alors la RAM de l'hôte, une seule fois. Sinon il est « non chiffrable ». Les parts `host_ram / shared` ne sont jamais additionnées.

## Limites des règles

- Depuis l'étape 4, le verrou et la gravité sont séparés : `Advice.locked` porte le refus, `Level` décrit la gravité. Un service verrouillé mais non vital (par exemple Windows Update) reste « À garder » ou « Système », avec un cadenas ; « Vital » est réservé aux composants dont l'arrêt casse Windows.
- Les 601 règles portent une provenance de **famille** (table RamScope héritée, avec une référence Microsoft Learn, éditeurs, Gestionnaire des tâches ou Planificateur de tâches), pas une URL vérifiée règle par règle. Aucune règle n'est sans source, mais aucune n'a de date de revue individuelle : la valeur est `non datée`. Une justification peut donc être fausse pour ton usage ; vérifie l'éditeur et le chemin avant d'agir.
- Les correspondances sont exactes : nom de service, nom d'exécutable, nom d'entrée ou composant/suffixe de chemin selon la table. Une entrée `php` ne déclenche plus la règle `hp`. Hors table, les anciennes heuristiques de nom (« update », « helper », « telemetry ») ne proposent plus d'action : le conseil reste `Unknown`.
- La liste des services hébergés par un `svchost.exe` est reprise dans le conseil du processus ; la branche qui la rendait inatteignable a été supprimée.
- Le total réel est de 601 règles, et non 602 : `eventsystem` et `eventsysevent` désignaient le même service COM+ Event System et ont été fusionnés. Voir `PLAN.md`, journal 7.6.

## Droits et surface locale

- L'élévation UAC est demandée au lancement, sauf avec `--demo` ou `--no-admin`.
- L'interface écoute uniquement sur la boucle locale ; toute autre adresse passée à `--host` est refusée au démarrage.
- L'en-tête `Host` est validé contre les adresses de la session, l'origine est comparée exactement, et toute route `/api/` exige le jeton. Le jeton reste présent dans la page, mais seulement pour un `Host` autorisé.
- Fermez l'application quand vous ne l'utilisez pas : une page ouverte dans le même navigateur reste sans effet, mais la surface n'existe que pendant l'exécution.

## Avant une première utilisation réelle

1. Lancer en `--demo` pour voir les écrans et le type d'actions proposées.
2. Lancer en `--no-admin` pour comparer l'inventaire avec le Gestionnaire des tâches et `services.msc`.
3. Pour une application réelle, préférer une machine virtuelle ou un instantané système.
4. Relire la liste des changements dans la confirmation, en particulier les actions irréversibles.

## Éléments non identifiés

Si RamScope ne reconnaît pas un élément, il ne propose rien. La bonne réaction est de vérifier l'éditeur et le chemin avant d'agir, pas de faire confiance à un seuil de mémoire.
