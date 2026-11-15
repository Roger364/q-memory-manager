# Contribuer à RamScope

RamScope est un projet open source de Bisse Studio. Les contributions sont
bienvenues : corrections, tests, documentation, remarques sur la sécurité ou
améliorations de l'interface.

## Licence des contributions

En soumettant une contribution, vous certifiez que vous avez le droit de la
fournir et vous acceptez qu'elle soit distribuée sous la licence MIT du projet.
Ajoutez un sign-off à vos commits :

```text
Signed-off-by: Prénom Nom <adresse@exemple.ch>
```

Le projet utilise le [Developer Certificate of Origin](https://developercertificate.org/)
(DCO). Il n'y a pas de CLA pour l'instant. Si le modèle de licence évolue, un
accord séparé pourrait être demandé ; les contributions déjà publiées restent
sous MIT.

## Construire et tester

Prérequis : Rust stable, cible Windows, et Node.js 24 pour les tests de
l'interface.

```text
cd ramscope
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Les tests Node se lancent via `cargo test` ou directement :

```text
node --test src/web/core.test.mjs
```

## Règles de sécurité

- Ne testez jamais une action réelle sur votre machine quotidienne : utilisez
  `--demo` ou un Windows Sandbox jetable.
- Ne committez pas de secrets, de jetons, de rapports contenant des données
  personnelles ni le contenu de l'archive `sandbox-test-*.zip`.
- Ne contournez pas les garde-fous (`Host`, jeton, `Origin`, validation serveur,
  verrous moteur) pour faire passer un test.
- Pour une vulnérabilité, suivez [`SECURITY.md`](SECURITY.md) au lieu d'ouvrir
  une issue publique.

## Proposer une modification

1. Ouvrez une issue pour discuter d'un changement important.
2. Gardez les changements ciblés et ajoutez un test de réversion nommé lorsque
   le comportement change.
3. Vérifiez `cargo fmt --check`, `cargo test` et Clippy avant la pull request.
4. Décrivez le problème, la solution et les limites de test dans la description.

Les échanges sont en français ou en anglais. Les désaccords techniques sont
bienvenus ; les attaques personnelles ne le sont pas.
