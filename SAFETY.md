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

Ces refus existent à deux niveaux pour les services et les processus : validation dans `src/plan.rs`, garde-fous dans `src/win.rs`. Ils restent actifs même si l'interface est contournée. Pour les entrées de démarrage et les tâches, il n'existe pas de liste de protection distincte : `plan::validate` valide l'action, puis le moteur revérifie l'état courant avant l'écriture ; la formulation « deux verrous » ne s'applique donc pas à ces deux types.

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

## Confiance locale (décision D9-1)

Les processus locaux sont considérés comme **non fiables**. Un programme exécuté sous le même compte utilisateur peut lire la page servie sur la boucle locale, récupérer le jeton de session et fabriquer un en-tête `Origin` : le jeton n'authentifie donc pas un processus local. Il protège la session contre une page ou une origine qui n'a pas reçu cette page, pas contre un programme malveillant déjà présent sur la machine.

Conséquences dans cette version :

- `/api/apply` recollecte l'état courant et refuse toute action de service, de démarrage ou de tâche sans état observé (`from` / `was_enabled`), ou dont l'état ne correspond plus au snapshot.
- L'action doit être exactement celle que le snapshot propose pour cette cible ; une cible réelle mais une action arbitraire est refusée. L'arrêt immédiat n'est accepté que pour un service que l'analyse propose de passer en manuel ou de désactiver.
- Le démarrage d'un service (`StartService`) est réservé au chemin interne de restauration. Les actions inverses enregistrées avec `from: None` ne passent pas par la validation des actions utilisateur.
- Le jeton est généré par `BCryptGenRandom` (32 octets), avec échec explicite du démarrage si le CSPRNG est indisponible.

**Risque résiduel assumé pour la v1 :** un processus local hostile du même compte peut encore lire le jeton, consulter le snapshot et demander une action que RamScope propose. Le canal named pipe/DACL ou un helper élevé avec consentement par lot n'est pas implémenté et fera l'objet d'un lot séparé. Ne pas présenter cette version comme une protection contre un malware local.

## Droits et surface locale

- Le lancement par défaut ne demande **pas** l'élévation UAC. S'il n'est pas déjà élevé, RamScope démarre en lecture seule et propose « Passer en administrateur » dans l'interface ; depuis un terminal déjà administrateur, la session reste élevée. L'UAC n'est demandé qu'après ce clic, ou au lancement avec `--admin`.
- `--no-admin` est strict pour l'élévation : le bouton est masqué et `/api/elevate` répond `403`. Il ne transforme pas une session déjà élevée en session non administrateur ; ne pas lancer `--no-admin` depuis un terminal administrateur si l'on veut garantir un audit sans droits.
- Une fois l'instance élevée, elle reste administrateur jusqu'à sa fermeture. La réduction structurelle de cette durée (helper one-shot) n'est pas implémentée ; elle est prévue dans le lot séparé D9-2b.
- L'interface écoute uniquement sur la boucle locale ; toute autre adresse passée à `--host` est refusée au démarrage. Avec `--host localhost`, la liaison suit la résolution locale (souvent `::1`) : l'alias `127.0.0.1` n'est pas garanti ; utiliser l'adresse affichée au démarrage.
- L'en-tête `Host` est validé contre les adresses de la session, l'origine est comparée exactement, et toute route `/api/` exige le jeton. Toute mutation exige un `Origin` présent et exact ; une origine absente ou invalide reçoit `403`. Le jeton reste présent dans la page, mais seulement pour un `Host` autorisé.
- Le corps d'une mutation est limité à **1 Mio**. `Content-Length` est vérifié avant lecture ; la lecture réelle est aussi bornée, ce qui couvre l'absence de longueur et les corps chunked. Un dépassement reçoit `413`.
- En lecture seule, les tâches planifiées sont lues via `schtasks /query /xml` quand `System32\Tasks` est inaccessible : cela n'élève pas les droits et n'exécute aucune tâche. Le repli appelle l'outil système par chemin absolu et fusionne les lectures partielles par chemin, sans doublon. Le 2026-11-14, les 201 tâches et 167 tâches activées retrouvées correspondaient à `Get-ScheduledTask`.
- Fermez l'application quand vous ne l'utilisez pas : une page ouverte dans le même navigateur reste sans effet, mais la surface n'existe que pendant l'exécution.

## Avant une première utilisation réelle

1. Lancer en `--demo` pour voir les écrans et le type d'actions proposées.
2. Lancer sans option, ou en `--no-admin` strict, pour comparer l'inventaire avec le Gestionnaire des tâches et `services.msc` sans demander l'UAC.
3. Pour une application réelle, préférer une machine virtuelle ou un instantané système.
4. Relire la liste des changements dans la confirmation, en particulier les actions irréversibles.

## Éléments non identifiés

Si RamScope ne reconnaît pas un élément, il ne propose rien. La bonne réaction est de vérifier l'éditeur et le chemin avant d'agir, pas de faire confiance à un seuil de mémoire.
