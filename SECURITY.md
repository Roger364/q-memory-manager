# Sécurité

## Signaler une vulnérabilité

Merci de ne pas ouvrir une issue publique pour une vulnérabilité non corrigée.
Écrivez à **contact@bissestudio.ch** avec :

- la version de RamScope et de Windows ;
- les options de lancement (`--demo`, `--no-admin`, `--admin`, élévation) ;
- une description du problème et, si possible, une reproduction minimale ;
- l'impact observé ou attendu.

N'incluez pas de secrets, de jetons de session, de rapports exportés contenant
des données personnelles ou de captures de votre machine.

## Périmètre

Sont concernés les problèmes du code de RamScope, notamment :

- le serveur HTTP local, le jeton de session, la validation de `Host` et
  d'`Origin` ;
- les chemins de mutation et de restauration ;
- l'élévation UAC et la relance de l'instance élevée ;
- l'échappement HTML et l'injection dans l'interface ou le rapport exporté ;
- la lecture de données système qui pourrait exposer des informations locales.

Les vulnérabilités de Windows, du navigateur ou de services tiers ne relèvent
pas de ce dépôt ; signalez-les à leurs éditeurs.

## Versions

La branche `main` et la version `1.x` sont maintenues. Les versions plus
anciennes ne reçoivent pas de correctif dédié.

## Engagement

RamScope est maintenu par un développeur indépendant, sans SLA et sans
programme de prime. Les rapports sérieux sont examinés et une correction est
publiée dès que possible. Une divulgation coordonnée est préférée ; un délai de
90 jours avant publication est un point de départ raisonnable, à ajuster selon
la gravité et la disponibilité d'un correctif.

## Réduction du risque

Pour un usage d'inspection, préférez `--no-admin` : RamScope reste en lecture
seule et ne peut pas demander l'élévation. N'exécutez pas RamScope sur une
machine de production sans avoir lu `SAFETY.md` et testé les actions dans un
Windows Sandbox jetable.
