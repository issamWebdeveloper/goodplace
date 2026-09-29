+++
title = "devkit : des devcontainers sécurisés pour coder avec les agents IA"
slug = "devkit-devcontainers-ia"
excerpt = "Présentation de devkit, un CLI qui génère en une commande un devcontainer isolé, avec pare-feu sortant, mappage des ports et intégration de GitHub Spec-Kit pour une vingtaine d'assistants IA."
tags = ["IA", "DevOps", "Docker", "Projet"]
cover_image = "/images/blog/devkit.png"
seo_description = "devkit, CLI Node.js pour générer des devcontainers sécurisés : pare-feu iptables par liste blanche, ports par framework, Spec-Kit et agents IA comme Claude Code."
status = "published"
published_at = "2026-03-02T08:00:00Z"
+++

Les agents de développement IA — Claude Code, Copilot, Gemini CLI, Codex et bien d'autres — ont changé ma façon de travailler. Ils lisent le code, exécutent des commandes, installent des dépendances, lancent les tests. C'est précisément ce qui les rend utiles… et ce qui pose question : **que peut faire un agent qui exécute des commandes sur ma machine ?**

La réponse raisonnable est de le faire travailler dans un environnement isolé. Les **devcontainers** s'y prêtent parfaitement, mais leur configuration est fastidieuse à répéter pour chaque projet. J'ai donc écrit **devkit**, un petit CLI qui génère tout cela en une commande. Le code est sur [GitHub](https://github.com/issamWebdeveloper/devkit).

```bash
npm install -g git+https://github.com/issamWebdeveloper/devkit.git
devkit init
```

## Ce que fait `devkit init`

Le CLI pose quelques questions, puis génère un dossier `.devcontainer/` complet :

1. **Choix des technologies** : Angular, React, Next.js, Vue, Svelte, Express, NestJS, Nuxt, Astro, ou un port personnalisé.
2. **Configuration des ports** : chaque technologie propose son port par défaut (4200 pour Angular, 5173 pour Vite, 4321 pour Astro…), modifiable.
3. **Emplacement du projet** : dossier courant ou nouveau dossier.
4. **Génération** du `Dockerfile`, du `devcontainer.json` et d'un script de pare-feu.
5. **Intégration optionnelle de GitHub Spec-Kit**, avec l'assistant IA de votre choix.

Le résultat s'ouvre directement dans VS Code (« Reopen in Container »), dans les IDE JetBrains, ou en ligne de commande avec la Dev Container CLI officielle — pratique pour Neovim.

## Un catalogue de technologies déclaratif

Toute la connaissance « métier » du CLI tient dans des constantes, sans logique cachée :

```js
export const TECHNOLOGIES = [
  { name: 'Angular', value: 'angular', defaultPort: 4200 },
  { name: 'React', value: 'react', defaultPort: 5173 },
  { name: 'Next.js', value: 'nextjs', defaultPort: 3000 },
  { name: 'Astro', value: 'astro', defaultPort: 4321 },
  // …
  { name: 'Autre (port personnalise)', value: 'custom', defaultPort: null },
];
```

Ajouter un framework revient à ajouter une ligne. Les questions interactives s'appuient sur `@inquirer/prompts`, les couleurs sur `chalk` et les indicateurs de progression sur `ora` : quatre dépendances au total, commander compris.

Quand deux technologies partagent le même port par défaut (React et Vue sur 5173, par exemple), l'utilisateur est invité à en changer un : on évite ainsi un conflit que l'on découvrirait seulement au lancement des serveurs.

## Le cœur du sujet : un pare-feu en liste blanche

Isoler le système de fichiers ne suffit pas. Un agent qui a accès à Internet sans restriction peut télécharger n'importe quoi ou envoyer du code n'importe où. Le devcontainer généré applique donc une politique réseau stricte au démarrage : **tout est bloqué, sauf une liste explicite de destinations**.

Le script `init-firewall.sh` procède en plusieurs étapes :

```bash
# 1. Préserver la résolution DNS interne de Docker avant de tout vider
DOCKER_DNS_RULES=$(iptables-save -t nat | grep "127\.0\.0\.11" || true)
iptables -F && iptables -X

# 2. Autoriser DNS, SSH et la boucle locale
iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
iptables -A INPUT -i lo -j ACCEPT

# 3. Construire un ipset des plages autorisées
ipset create allowed-domains hash:net
gh_ranges=$(curl -s https://api.github.com/meta)
```

Les plages IP de GitHub sont récupérées depuis l'API officielle `api.github.com/meta`, **validées une par une** (le script refuse toute plage qui n'a pas la forme d'un CIDR), agrégées puis ajoutées à l'ipset. Viennent ensuite les domaines indispensables : le registre npm, l'API d'Anthropic pour Claude Code, la marketplace et les mises à jour de VS Code. Enfin, la politique par défaut passe à `DROP`, et le script **vérifie lui-même** que la configuration fonctionne : une requête vers un domaine non autorisé doit échouer, une requête vers GitHub doit réussir.

Le conteneur reçoit uniquement les capacités `NET_ADMIN` et `NET_RAW` nécessaires à ce script. L'agent peut ainsi travailler en mode autonome, sans demander de confirmation à chaque commande, avec un risque maîtrisé : dans le pire des cas, il casse le conteneur, pas la machine.

## Un environnement confortable

Sécurisé ne veut pas dire spartiate. L'image de base Node.js inclut `git`, `gh`, `fzf`, `zsh`, `jq`, `git-delta` pour des diffs lisibles, et l'historique du shell est persisté dans un volume Docker : il survit à la reconstruction du conteneur. Côté VS Code, ESLint, Prettier et GitLens sont préinstallés, avec le formatage à l'enregistrement.

La configuration de l'assistant IA est elle aussi stockée dans un volume nommé par conteneur : on ne se reconnecte pas à chaque reconstruction.

## Spec-Kit : du prompt à la spécification

La seconde moitié de devkit concerne la méthode. Donner des instructions vagues à un agent produit du code vague. **GitHub Spec-Kit** propose une approche de *spec-driven development* : on rédige d'abord une spécification, puis un plan technique, puis une liste de tâches, et l'agent implémente tâche par tâche en s'y référant.

devkit propose d'initialiser Spec-Kit dans le projet avec l'assistant de votre choix. Le catalogue en compte une vingtaine : Claude Code, GitHub Copilot, Gemini CLI, Cursor, Windsurf, Amazon Q Developer, Codex CLI, Qwen Code, Roo Code, opencode, ou encore SHAI d'OVHcloud.

Spec-Kit s'installe via `uvx`, l'exécuteur de l'outil Python `uv`. Si `uvx` est absent, devkit le détecte et propose de l'installer, plutôt que d'échouer avec un message cryptique :

```js
export async function ensureUvx() {
  if (checkUvx()) return true;
  const install = await confirm({ message: 'Voulez-vous installer uv (inclut uvx) via pip ?', default: true });
  if (!install) return false;
  // installation avec un indicateur de progression…
}
```

## Ce que j'en retiens

- **L'isolation est la condition de l'autonomie.** Un agent IA est bien plus utile quand on peut le laisser travailler sans valider chaque commande ; c'est acceptable uniquement dans un environnement cloisonné.
- **La liste blanche réseau** est la mesure la plus efficace : elle limite à la fois les téléchargements hasardeux et les fuites de code.
- **Un bon CLI se fait oublier** : quelques questions, des valeurs par défaut sensées, des erreurs qui expliquent quoi faire.
- **La spécification avant le code** reste la meilleure façon de guider un agent, comme on guiderait un collègue.

Ce site lui-même a d'ailleurs été développé dans un environnement de ce type, avec un agent IA travaillant à partir d'une spécification détaillée.
