import { ChangeDetectionStrategy, Component, computed, effect, inject, input } from '@angular/core';
import { RouterLink } from '@angular/router';

import { SITE } from '../../core/config';
import { SeoService } from '../../core/seo/seo.service';
import { ArticleCard } from '../../shared/article-card';
import { BlogListData } from './blog.resolvers';

@Component({
  selector: 'app-blog-list',
  imports: [RouterLink, ArticleCard],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <section class="section-primary page-hero">
      <div class="container">
        <span class="text-meta">Blog</span>
        <h1 class="heading-display">{{ tag() ?? 'Articles' }}</h1>
        <p class="lead-inverse">
          Angular, performance, architecture front-end, Rust et IA appliquée au développement :
          retours d'expérience et projets.
        </p>
      </div>
    </section>

    <section class="section">
      <div class="container">
        <nav class="tags" aria-label="Filtrer par thème">
          <a routerLink="/blog" [class.active]="!tag()">Tous</a>
          @for (t of data().tags; track t.tag) {
            <a routerLink="/blog" [queryParams]="{ tag: t.tag }" [class.active]="tag() === t.tag">
              {{ t.tag }} <small>{{ t.count }}</small>
            </a>
          }
        </nav>

        @if (page().items.length) {
          <div class="grid">
            @for (a of page().items; track a.slug) {
              <app-article-card [article]="a" />
            }
          </div>
        } @else {
          <p class="empty">Aucun article pour le moment.</p>
        }

        @if (page().total_pages > 1) {
          <nav class="pagination" aria-label="Pagination">
            @for (p of pages(); track p) {
              <a
                routerLink="/blog"
                [queryParams]="{ page: p === 1 ? null : p, tag: tag() }"
                [class.active]="p === page().page"
                [attr.aria-current]="p === page().page ? 'page' : null"
                >{{ p }}</a
              >
            }
          </nav>
        }
      </div>
    </section>
  `,
  styles: `
    .page-hero {
      padding-block: clamp(56px, 8vw, 100px);
    }
    .lead-inverse {
      max-width: 640px;
      font-size: 1.1rem;
      color: rgba(255, 255, 255, 0.9);
    }
    .tags {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-bottom: 40px;
      a {
        padding: 6px 16px;
        border-radius: 500px;
        border: 1px solid var(--gp-border);
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--gp-text);
        small {
          color: var(--gp-muted);
          margin-left: 4px;
        }
        &:hover {
          border-color: var(--gp-primary);
        }
        &.active {
          color: #fff;
          border-color: transparent;
          background: var(--gp-gradient-button);
          small {
            color: rgba(255, 255, 255, 0.8);
          }
        }
      }
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
      gap: 28px;
    }
    .empty {
      text-align: center;
      color: var(--gp-muted);
    }
    .pagination {
      display: flex;
      justify-content: center;
      gap: 8px;
      margin-top: 48px;
      a {
        display: grid;
        place-items: center;
        width: 42px;
        height: 42px;
        border-radius: 50%;
        border: 1px solid var(--gp-border);
        color: var(--gp-text);
        &.active {
          color: #fff;
          background: var(--gp-gradient-button);
          border-color: transparent;
        }
      }
    }
  `,
})
export class BlogList {
  private readonly seo = inject(SeoService);
  readonly data = input.required<BlogListData>();

  protected readonly page = computed(() => this.data().page);
  protected readonly tag = computed(() => this.data().tag);
  protected readonly pages = computed(() =>
    Array.from({ length: this.page().total_pages }, (_, i) => i + 1),
  );

  constructor() {
    effect(() => {
      const tag = this.tag();
      const p = this.page().page;
      const query = new URLSearchParams();
      if (tag) query.set('tag', tag);
      if (p > 1) query.set('page', String(p));
      const qs = query.toString();
      this.seo.set({
        title: tag ? `Articles « ${tag} »` : 'Blog',
        description: tag
          ? `Tous les articles du blog GoodPlace sur le thème ${tag}.`
          : 'Articles sur Angular, la performance web, l’architecture front-end, Rust et l’IA appliquée au développement.',
        path: '/blog' + (qs ? `?${qs}` : ''),
        jsonLd: {
          '@context': 'https://schema.org',
          '@type': 'Blog',
          name: `${SITE.name} — Blog`,
          url: `${SITE.url}/blog`,
          inLanguage: 'fr-FR',
          author: { '@type': 'Person', name: SITE.author },
          blogPost: this.page().items.map((a) => ({
            '@type': 'BlogPosting',
            headline: a.title,
            url: `${SITE.url}/blog/${a.slug}`,
            datePublished: a.published_at,
          })),
        },
      });
    });
  }
}
