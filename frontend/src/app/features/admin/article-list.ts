import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, input, resource } from '@angular/core';
import { RouterLink } from '@angular/router';

import { ArticleStatus } from '../../core/models';
import { AdminService, STATUS_LABELS } from './admin.service';

@Component({
  selector: 'app-article-list',
  imports: [RouterLink, DatePipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="head">
      <h1>Articles</h1>
      <a class="btn btn-primary" routerLink="/admin/articles/nouveau">Nouvel article</a>
    </div>

    <nav class="filters" aria-label="Filtrer par statut">
      <a routerLink="/admin/articles" [class.active]="!statut()">Tous</a>
      @for (s of statuses; track s) {
        <a routerLink="/admin/articles" [queryParams]="{ statut: s }" [class.active]="statut() === s">{{ labels[s] }}</a>
      }
    </nav>

    @if (articles.isLoading()) {
      <p class="muted">Chargement…</p>
    } @else if (articles.error()) {
      <p class="alert alert-error">Impossible de charger les articles.</p>
    } @else {
      <div class="table-wrap card">
        <table>
          <thead>
            <tr>
              <th>Titre</th>
              <th>Statut</th>
              <th>Date</th>
              <th>Lecture</th>
            </tr>
          </thead>
          <tbody>
            @for (a of articles.value(); track a.id) {
              <tr>
                <td>
                  <a [routerLink]="['/admin/articles', a.id]">{{ a.title }}</a>
                  <small>/blog/{{ a.slug }}</small>
                </td>
                <td><span class="badge badge-{{ a.status }}">{{ labels[a.status] }}</span></td>
                <td>
                  @switch (a.status) {
                    @case ('published') {
                      {{ a.published_at | date: 'dd/MM/y HH:mm' }}
                    }
                    @case ('validated') {
                      @if (a.scheduled_at) {
                        ⏱ {{ a.scheduled_at | date: 'dd/MM/y HH:mm' }}
                      } @else {
                        <span class="muted">non programmé</span>
                      }
                    }
                    @default {
                      <span class="muted">modifié le {{ a.updated_at | date: 'dd/MM/y' }}</span>
                    }
                  }
                </td>
                <td>{{ a.reading_minutes }} min</td>
              </tr>
            } @empty {
              <tr><td colspan="4" class="muted">Aucun article.</td></tr>
            }
          </tbody>
        </table>
      </div>
    }
  `,
  styles: `
    .head {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
      flex-wrap: wrap;
    }
    .filters {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
      margin: 16px 0 24px;
      a {
        padding: 5px 14px;
        border-radius: 500px;
        border: 1px solid var(--gp-border);
        background: #fff;
        font-size: 0.82rem;
        color: var(--gp-text);
        &.active {
          color: #fff;
          background: var(--gp-gradient-button);
          border-color: transparent;
        }
      }
    }
    .table-wrap {
      overflow-x: auto;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.92rem;
    }
    th,
    td {
      padding: 14px 18px;
      text-align: left;
      border-bottom: 1px solid var(--gp-border);
      vertical-align: top;
    }
    th {
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--gp-muted);
    }
    td a {
      font-weight: 600;
    }
    td small {
      display: block;
      color: var(--gp-muted);
    }
  `,
})
export class ArticleList {
  private readonly admin = inject(AdminService);

  /** Paramètre de requête `?statut=` lié automatiquement. */
  readonly statut = input<ArticleStatus | undefined>();

  protected readonly labels = STATUS_LABELS;
  protected readonly statuses: ArticleStatus[] = ['draft', 'validated', 'published'];
  protected readonly articles = resource({
    params: () => ({ status: this.statut() }),
    loader: ({ params }) => this.admin.articles(params.status),
  });
}
