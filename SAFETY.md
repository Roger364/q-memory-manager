# Prudence d'usage

RamScope est un outil d'observation et de décision, pas un nettoyeur. Il ne supprime aucun fichier et ne modifie aucun réglage sans confirmation explicite.

## Trois choses à ne pas confondre

- **Mesure** : ce que le système rapporte maintenant (total, disponible, working set, mémoire privée).
- **Estimation** : un gain calculé à partir d'une mesure, ou l'attribution d'une part de mémoire partagée. Toujours approximatif.
- **Recommandation** : un jugement sur l'utilité d'un élément. Ce n'est pas une mesure, et cela peut être faux pour ton usage.

Un chiffre élevé n'est pas une preuve qu'un élément est inutile.

## Ce que RamScope refuse

- Modifier ou arrêter un service de la liste `PROTECTED_SERVICES` (`src/rules.rs`).
- Terminer un processus de la liste `PROTECTED_PROCESSES`, un PID inférieur ou égal à 8, ou RamScope lui-même.
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

## Effets différés

- Passer un service en manuel ne libère pas sa mémoire immédiatement : cela change ce qui se passe au prochain démarrage, et une dépendance peut le relancer à la demande.
- Désactiver un démarrage n'arrête pas le programme déjà lancé.
- Arrêter un service n'équivaut pas à le désactiver : il peut redémarrer seul.

## Limites des chiffres

- La mémoire d'un service hébergé dans `svchost.exe` est partagée : RamScope affiche une part, pas une mesure isolée. Ces parts ne s'additionnent pas.
- La mémoire « compressée » affichée est le working set du processus Memory Compression : approximation.
- Les gains affichés sont des estimations.
- Défauts connus, non corrigés à ce stade : un gain différé peut être présenté comme immédiat quand l'option « Libérer la RAM maintenant » est cochée (D3) ; une application tierce sans fenêtre au-dessus de 150 Mo peut être proposée à la fermeture sur ce seul critère (D4) ; des gains de 60 ou 40 Mo peuvent être estimés sans mesure (D5) ; l'addition des parts d'un même hôte reste approximative (D6). Voir `PLAN.md`.

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
