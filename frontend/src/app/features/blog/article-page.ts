import { DatePipe } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  RESPONSE_INIT,
  effect,
  inject,
  input,
} from '@angular/core';
import { RouterLink } from '@angular/router';

import { SITE } from '../../core/config';
import { PublicArticle } from '../../core/models';
import { SeoService } from '../../core/seo/seo.service';
import { NewsletterForm } from '../../shared/newsletter-form';

@Component({
  selector: 'app-article-page',
  imports: [RouterLink, DatePipe, NewsletterForm],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './article-page.html',
  styleUrl: './article-page.scss',
})
export class ArticlePage {
  private readonly seo = inject(SeoService);
  private readonly responseInit = inject(RESPONSE_INIT, { optional: true });

  readonly article = input.required<PublicArticle | null>();

  constructor() {
    effect(() => {
      const a = this.article();
      if (!a) {
        // Côté serveur : vrai code HTTP 404 pour les moteurs de recherche.
        if (this.responseInit) this.responseInit.status = 404;
        this.seo.set({ title: 'Article introuvable', path: '/blog', noindex: true });
        return;
      }
      const url = `${SITE.url}/blog/${a.slug}`;
      this.seo.set({
        title: a.seo_title || a.title,
        description: a.seo_description || a.excerpt,
        path: `/blog/${a.slug}`,
        image: a.cover_image,
        type: 'article',
        publishedAt: a.published_at,
        modifiedAt: a.updated_at,
        tags: a.tags,
        jsonLd: {
          '@context': 'https://schema.org',
          '@type': 'BlogPosting',
          headline: a.title,
          description: a.seo_description || a.excerpt,
          image: a.cover_image ? this.seo.absolute(a.cover_image) : undefined,
          datePublished: a.published_at,
          dateModified: a.updated_at,
          inLanguage: 'fr-FR',
          keywords: a.tags.join(', '),
          timeRequired: `PT${a.reading_minutes}M`,
          mainEntityOfPage: url,
          url,
          author: { '@type': 'Person', name: SITE.author, url: SITE.url },
          publisher: {
            '@type': 'Organization',
            name: SITE.name,
            logo: { '@type': 'ImageObject', url: `${SITE.url}/apple-touch-icon.png` },
          },
        },
      });
    });
  }
}
