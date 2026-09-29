import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { RouterLink } from '@angular/router';

import { ArticleSummary } from '../core/models';

@Component({
  selector: 'app-article-card',
  imports: [RouterLink, DatePipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @let a = article();
    <article class="card card-hover">
      <a [routerLink]="['/blog', a.slug]" class="cover" tabindex="-1" aria-hidden="true">
        @if (a.cover_image) {
          <img [src]="a.cover_image" alt="" width="1200" height="630" loading="lazy" />
        }
      </a>
      <div class="card-body">
        <p class="meta">
          @if (a.published_at) {
            <time [attr.datetime]="a.published_at">{{ a.published_at | date: 'd MMMM y' }}</time>
            <span aria-hidden="true">•</span>
          }
          <span>{{ a.reading_minutes }} min de lecture</span>
        </p>
        <h3>
          <a [routerLink]="['/blog', a.slug]">{{ a.title }}</a>
        </h3>
        <p class="excerpt">{{ a.excerpt }}</p>
        <div class="tags">
          @for (tag of a.tags; track tag) {
            <span class="tag">{{ tag }}</span>
          }
        </div>
      </div>
    </article>
  `,
  styles: `
    :host {
      display: block;
    }
    article {
      height: 100%;
      display: flex;
      flex-direction: column;
    }
    .cover {
      display: block;
      aspect-ratio: 1200 / 630;
      background: var(--gp-gradient);
      overflow: hidden;
      img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        transition: transform 0.4s ease;
      }
    }
    article:hover .cover img {
      transform: scale(1.04);
    }
    .card-body {
      display: flex;
      flex-direction: column;
      flex: 1;
    }
    .meta {
      display: flex;
      gap: 8px;
      font-size: 0.78rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--gp-muted);
      margin-bottom: 10px;
    }
    h3 {
      font-size: 1.2rem;
      margin-bottom: 10px;
      a {
        color: var(--gp-darker);
      }
      a:hover {
        color: var(--gp-primary-strong);
      }
    }
    .excerpt {
      color: var(--gp-muted);
      font-size: 0.95rem;
      flex: 1;
    }
    .tags {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
    }
  `,
})
export class ArticleCard {
  readonly article = input.required<ArticleSummary>();
}
