+++
title = "L'art de la simplicité numérique"
slug = "l-art-de-la-simplicite-numerique"
excerpt = "Comment créer des expériences utilisateur efficaces avec moins d'éléments."
tags = ["UX", "Angular", "Architecture", "Frontend"]
cover_image = "/images/blog/simplicite.png"
seo_description = "Concevoir des interfaces et des applications Angular plus simples : moins d'éléments à l'écran, moins d'état, moins de dépendances, pour une meilleure expérience utilisateur."
status = "published"
published_at = "2023-05-15T08:00:00Z"
+++

« La perfection est atteinte non quand il n'y a plus rien à ajouter, mais quand il n'y a plus rien à retirer. » La phrase de Saint-Exupéry est citée dans tous les ateliers de design, et pourtant nos interfaces n'ont jamais été aussi chargées. Bannières, pop-ups de consentement, carrousels, notifications, menus à trois niveaux… Chaque élément a été ajouté pour une bonne raison, et c'est précisément le problème : personne n'a jamais la bonne raison de retirer quoi que ce soit.

Après plus de quinze ans à construire des interfaces, de sites vitrines en PHP jusqu'à des plateformes e-santé en Angular, je suis convaincu d'une chose : la simplicité n'est pas un style graphique, c'est une discipline. Elle s'applique autant à l'écran qu'au code qui le produit.

## La simplicité côté utilisateur

### Chaque élément a un coût

Un bouton, un lien, une icône : chaque élément affiché demande un effort à l'utilisateur. Il doit le percevoir, comprendre ce qu'il fait, décider s'il le concerne. La **loi de Hick** le formalise : le temps de décision augmente avec le nombre d'options. Un menu de douze entrées n'est pas « plus complet » qu'un menu de cinq, il est plus lent à utiliser.

L'exercice le plus utile que je connaisse consiste à prendre un écran existant et à se demander, pour chaque élément : **que se passerait-il si on le supprimait ?** Souvent, la réponse est « rien » ou « quelques utilisateurs chercheraient ailleurs ». Ces éléments sont des candidats au retrait, ou au moins au déplacement vers un niveau secondaire.

### Une action principale par écran

Un écran efficace a un objectif clair. Sur une page de connexion, c'est se connecter. Sur une fiche article, c'est lire. Tout ce qui détourne de cet objectif doit être justifié.

Concrètement, cela se traduit par une hiérarchie visuelle franche : **un seul bouton principal** mis en valeur, les actions secondaires en style discret, les actions rares ou destructrices reléguées dans un menu. Quand trois boutons de même couleur se font concurrence, l'utilisateur hésite, et l'hésitation est la première source de friction.

### L'espace blanc n'est pas du vide

Les maquettes denses donnent l'impression d'en offrir plus. En réalité, l'espace blanc structure l'information : il regroupe ce qui va ensemble (loi de proximité), sépare ce qui est distinct et laisse respirer le contenu. Augmenter les marges d'une interface est souvent le moyen le plus rapide de la rendre plus lisible, sans toucher à une seule fonctionnalité.

### La divulgation progressive

Simplifier ne veut pas dire amputer. Les utilisateurs avancés ont besoin d'options avancées ; ils n'ont simplement pas besoin de les voir en permanence. La **divulgation progressive** consiste à afficher d'abord l'essentiel, puis à révéler le reste à la demande : un lien « Options avancées », un panneau repliable, une étape supplémentaire dans un formulaire.

Sur une plateforme de gestion de parc informatique sur laquelle j'ai travaillé, le formulaire d'import d'utilisateurs comportait une vingtaine de champs de configuration. En ne gardant visibles que le fichier et le mode d'import, et en regroupant le reste sous des valeurs par défaut sensées, le nombre de tickets de support liés à cette fonctionnalité a nettement diminué.

### Des formulaires qui en demandent moins

Les formulaires sont l'endroit où la complexité fait le plus de dégâts. Quelques principes :

- Ne demander que ce qui est **réellement nécessaire maintenant**. Le numéro de téléphone peut attendre.
- Déduire ce qui peut l'être : la ville à partir du code postal, le type de carte à partir du numéro.
- Valider au bon moment, avec des messages qui expliquent comment corriger, pas seulement ce qui ne va pas.
- Préférer un champ unique « Nom complet » à la combinaison prénom / nom / deuxième prénom quand le métier le permet.

## La simplicité côté code

Une interface simple produite par un code compliqué finit toujours par se complexifier : chaque évolution coûte cher, les bugs s'accumulent, et les compromis visuels se multiplient. La simplicité doit donc descendre jusque dans l'architecture.

### Moins de modules, plus de composants autonomes

Pendant des années, une application Angular imposait de déclarer chaque composant dans un `NgModule`, d'importer les bons modules au bon endroit et de démêler des erreurs de dépendances parfois obscures. Les **composants standalone**, stables depuis Angular 15, suppriment cette couche d'indirection :

```ts
@Component({
  selector: 'app-newsletter-form',
  standalone: true,
  imports: [ReactiveFormsModule, NgIf],
  templateUrl: './newsletter-form.component.html',
})
export class NewsletterFormComponent {
  private fb = inject(FormBuilder);
  form = this.fb.nonNullable.group({ email: ['', [Validators.required, Validators.email]] });
}
```

Le composant déclare lui-même ce dont il a besoin. On le lit, on le comprend, on le déplace sans casser le reste. Angular 16 fournit même une schematic pour migrer progressivement un projet existant : `ng generate @angular/core:standalone`.

### Moins d'état

La majorité des bugs d'interface viennent d'un état dupliqué ou désynchronisé : une liste filtrée stockée à côté de la liste source, un compteur mis à jour manuellement, un indicateur de chargement oublié dans une branche d'erreur. La règle : **stocker le minimum, dériver le reste**.

Les signals, arrivés en developer preview avec Angular 16, rendent ce principe naturel :

```ts
readonly articles = signal<Article[]>([]);
readonly query = signal('');

readonly filtered = computed(() => {
  const q = this.query().toLowerCase();
  return this.articles().filter(a => a.title.toLowerCase().includes(q));
});
```

`filtered` n'est jamais stocké, jamais mis à jour à la main : il ne peut donc pas être faux. Avant d'introduire un store global, posez-vous la question : cet état est-il vraiment partagé par des écrans éloignés ? Souvent, un service avec quelques signals, ou même l'état local du composant, suffit largement.

### Moins de dépendances

Chaque dépendance ajoutée à `package.json` est une dette : du poids dans le bundle, des mises à jour à suivre, des failles potentielles, des incompatibilités lors de la prochaine montée de version d'Angular. Avant d'installer une bibliothèque, je me pose trois questions :

1. La plateforme le fait-elle déjà ? `Intl.DateTimeFormat` remplace bien des bibliothèques de dates, les pipes Angular couvrent la plupart des formatages.
2. Combien de lignes me faudrait-il pour l'écrire moi-même ? Si c'est moins de cinquante, la dépendance est rarement rentable.
3. Est-elle maintenue, et par qui ?

### Moins d'abstraction prématurée

L'abstraction est utile quand elle capture un motif réellement répété. Créée trop tôt, elle fige des hypothèses fausses. Un composant « générique » avec quinze entrées de configuration est plus difficile à utiliser que trois composants simples et explicites. Je préfère attendre la troisième occurrence d'un motif avant de le factoriser : à ce moment-là, on sait enfin ce qui varie vraiment.

### Des composants petits et nommés par leur rôle

Un composant qui fait une chose, avec un nom qui dit laquelle, n'a presque pas besoin de documentation. `ArticleCardComponent`, `ReadingTimePipe`, `NewsletterFormComponent` : on sait où chercher. À l'inverse, un `SharedComponent` de 800 lignes est le signe qu'il faut découper.

## Supprimer est une fonctionnalité

La plupart des feuilles de route ne contiennent que des ajouts. Pourtant, retirer une fonctionnalité peu utilisée est souvent la meilleure amélioration possible : moins de code à maintenir, moins de tests, moins de confusion pour les utilisateurs. Les outils d'analyse d'usage sont là pour ça : si un écran n'a été ouvert que trois fois dans le trimestre, il mérite une discussion.

Côté code, la même logique s'applique. Supprimer du code mort, des options de configuration jamais utilisées ou des branches conditionnelles héritées d'un ancien client est l'une des tâches les plus rentables qu'un développeur puisse faire. Et l'une des plus satisfaisantes.

## Simple ne veut pas dire simpliste

Il y a un piège : confondre simplicité et pauvreté. Une interface simple n'est pas une interface vide, c'est une interface où **la complexité a été prise en charge par les concepteurs plutôt que par les utilisateurs**. Derrière un bouton « Importer » qui fonctionne du premier coup, il y a de la détection de format, de la gestion d'erreurs, des valeurs par défaut intelligentes. Le travail n'a pas disparu, il a changé de camp.

C'est aussi pour cela que la simplicité demande du temps. Faire compliqué est facile : il suffit d'ajouter. Faire simple exige de comprendre le besoin, de trancher, de dire non, et de recommencer.

## En pratique

- Pour chaque élément d'écran, demandez-vous ce qui se passerait sans lui.
- Une action principale par écran, et une hiérarchie visuelle qui la met en évidence.
- Utilisez l'espace blanc et la divulgation progressive plutôt que la densité.
- Côté Angular : composants standalone, état minimal et dérivé, dépendances choisies avec parcimonie.
- Inscrivez des suppressions dans la feuille de route, pas seulement des ajouts.

La simplicité est rarement le premier jet. C'est le résultat d'une série de retraits, et c'est ce qui la rend si précieuse.
