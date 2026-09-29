import { ChangeDetectionStrategy, Component, inject, resource } from '@angular/core';
import { RouterLink } from '@angular/router';

import { AdminService } from './admin.service';

@Component({
  selector: 'app-dashboard',
  imports: [RouterLink],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <h1>Tableau de bord</h1>
    @if (stats.value(); as s) {
      <div class="grid">
        <a class="card stat" routerLink="/admin/articles" [queryParams]="{ statut: 'draft' }">
          <strong>{{ s.drafts }}</strong><span>Brouillons</span>
        </a>
        <a class="card stat" routerLink="/admin/articles" [queryParams]="{ statut: 'validated' }">
          <strong>{{ s.validated }}</strong><span>Validés / programmés</span>
        </a>
        <a class="card stat" routerLink="/admin/articles" [queryParams]="{ statut: 'published' }">
          <strong>{{ s.published }}</strong><span>Publiés</span>
        </a>
        <a class="card stat" routerLink="/admin/utilisateurs">
          <strong>{{ s.users }}</strong><span>Utilisateurs</span>
        </a>
        <a class="card stat" routerLink="/admin/utilisateurs" [queryParams]="{ role: 'subscriber' }">
          <strong>{{ s.subscribers }}</strong><span>Abonnés newsletter</span>
        </a>
      </div>
    } @else if (stats.error()) {
      <p class="alert alert-error">Impossible de charger les statistiques.</p>
    }
    <div class="card help">
      <div class="card-body">
        <h2>Cycle de publication</h2>
        <ol>
          <li><span class="badge badge-draft">Brouillon</span> : l'article est en cours de rédaction, invisible sur le site.</li>
          <li><span class="badge badge-validated">Validé</span> : relu et prêt. Avec une date de publication, le cron le met en ligne automatiquement (vérification chaque minute).</li>
          <li><span class="badge badge-published">Publié</span> : visible en ligne, dans le sitemap et le flux RSS ; les abonnés sont notifiés par email.</li>
        </ol>
        <a class="btn btn-primary" routerLink="/admin/articles/nouveau">Écrire un article</a>
      </div>
    </div>
  `,
  styles: `
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
      gap: 18px;
      margin-bottom: 32px;
    }
    .stat {
      padding: 24px;
      color: var(--gp-text);
      strong {
        display: block;
        font-family: var(--gp-font-heading);
        font-size: 2.4rem;
        font-weight: 900;
        color: var(--gp-primary-strong);
      }
      span {
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: var(--gp-muted);
      }
      &:hover {
        box-shadow: var(--gp-shadow-hover);
      }
    }
    .help li {
      margin-bottom: 10px;
    }
  `,
})
export class Dashboard {
  private readonly admin = inject(AdminService);
  protected readonly stats = resource({ loader: () => this.admin.stats() });
}
