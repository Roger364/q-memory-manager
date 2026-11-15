# Signature de code — RamScope

## État actuel

- **Pas de signature** : le `.exe` n'est pas signé. SmartScreen affiche un avertissement au premier lancement.
- **Pas de publication automatique** : pas de release GitHub avec binaire.
- La page site renvoie vers GitHub + instructions de compilation (pas de promesse de `.exe` signé).

## Piste choisie : SignPath.io

SignPath.io est gratuit pour les projets open source. Il signe les binaires via un workflow GitHub Actions.

### Prérequis

1. **Compte SignPath.io** : créer un compte sur https://signpath.io (gratuit pour open source).
2. **Organisation** : créer une organisation SignPath (ex. `bisse-studio`).
3. **Projet** : créer un projet `ramscope` dans SignPath.
4. **Politique de signature** : créer une politique `release-signing` de type `release`.
5. **Configuration d'artefact** : créer `windows-exe` (type `File`, extension `.exe`).
6. **Certificat** : SignPath fournit un certificat OV gratuit pour les projets open source.

### Configuration GitHub

Dans le dépôt `Roger364/q-memory-manager` :

1. **Variables** (Settings → Variables → Actions) :
   - `SIGNPATH_ORGANIZATION_ID` = ID de l'organisation SignPath

2. **Secrets** (Settings → Secrets → Actions) :
   - `SIGNPATH_API_TOKEN` = jeton API SignPath (généré dans SignPath → API tokens)

### Workflow

Le fichier `.github/workflows/release.yml` est déjà en place. Il :
1. Build le `.exe` en release.
2. Lance les tests + clippy + fmt.
3. Soumet le binaire à SignPath pour signature (si `SIGNPATH_ORGANIZATION_ID` est défini).
4. Crée une release GitHub draft avec le binaire (signé ou non) + SHA-256.

### Déclenchement

- **Tag** : `git tag v1.0.1 && git push origin v1.0.1`
- **Manuel** : GitHub → Actions → Release → Run workflow → entrer la version.

### Fallback (sans SignPath)

Si `SIGNPATH_ORGANIZATION_ID` n'est pas défini, le workflow produit un binaire non signé et crée la release quand même. Le site ne proposera pas de téléchargement direct tant que le binaire n'est pas signé.

### Mise à jour du site

Une fois un binaire signé disponible :
1. Mettre à jour `site-v2/content/site.ts` → `downloadUrl` dans `openSourceTools[0]`.
2. Mettre à jour la page Open source pour afficher le bouton de téléchargement.
3. Mettre à jour `ramscope.md` dans le vault.

## Alternatives évaluées

| Option | Coût | SmartScreen | Verdict |
|---|---|---|---|
| SignPath.io | Gratuit (open source) | Évité après réputation | **Choisi** |
| Microsoft Store (MSIX) | ~19 $ one-time | Évité | Packaging MSIX incompatible avec UAC/%LOCALAPPDATA% |
| winget | Gratuit | S'applique quand même | Canal secondaire possible |
| Certificat OV/EV payant | 200-500 $/an | EV évite immédiatement | Trop cher pour l'instant |
