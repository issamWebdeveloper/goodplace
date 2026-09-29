import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { SeoService } from '../../core/seo/seo.service';

@Component({
  selector: 'app-admin-layout',
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="admin">
      <aside>
        <p class="text-meta">Administration</p>
        <nav aria-label="Administration">
          <a routerLink="/admin" routerLinkActive="active" [routerLinkActiveOptions]="{ exact: true }">Tableau de bord</a>
          <a routerLink="/admin/articles" routerLinkActive="active">Articles</a>
          <a routerLink="/admin/articles/nouveau">+ Nouvel article</a>
          <a routerLink="/admin/utilisateurs" routerLinkActive="active">Utilisateurs</a>
        </nav>
        <button class="btn btn-default btn-small" (click)="logout()">Déconnexion</button>
      </aside>
      <main>
        <router-outlet />
      </main>
    </div>
  `,
  styles: `
    .admin {
      display: grid;
      grid-template-columns: 240px 1fr;
      min-height: calc(100vh - var(--gp-header-height));
      background: var(--gp-surface-muted);
    }
    aside {
      padding: 32px 24px;
      background: var(--gp-dark);
      color: #fff;
      display: flex;
      flex-direction: column;
      gap: 24px;
    }
    nav {
      display: grid;
      gap: 4px;
      a {
        padding: 10px 14px;
        border-radius: var(--gp-radius);
        color: rgba(255, 255, 255, 0.75);
        font-size: 0.92rem;
        &:hover {
          color: #fff;
          background: rgba(255, 255, 255, 0.06);
        }
        &.active {
          color: #fff;
          background: var(--gp-gradient-button);
        }
      }
    }
    main {
      padding: clamp(20px, 4vw, 40px);
      min-width: 0;
    }
    @media (max-width: 860px) {
      .admin {
        grid-template-columns: 1fr;
      }
      aside {
        padding: 20px;
      }
      nav {
        grid-auto-flow: column;
        overflow-x: auto;
      }
    }
  `,
})
export class AdminLayout {
  private readonly auth = inject(AuthStore);
  private readonly router = inject(Router);

  constructor() {
    inject(SeoService).set({ title: 'Administration', path: '/admin', noindex: true });
  }

  async logout(): Promise<void> {
    await this.auth.logout();
    await this.router.navigateByUrl('/');
  }
}
