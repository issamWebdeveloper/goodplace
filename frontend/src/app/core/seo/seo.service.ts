import { DOCUMENT, Injectable, inject } from '@angular/core';
import { Meta, Title } from '@angular/platform-browser';

import { SITE } from '../config';

export interface SeoData {
  title: string;
  description?: string;
  /** Chemin de la page (ex. `/blog/mon-article`). */
  path: string;
  image?: string | null;
  type?: 'website' | 'article' | 'profile';
  publishedAt?: string | null;
  modifiedAt?: string | null;
  tags?: string[];
  noindex?: boolean;
  jsonLd?: Record<string, unknown> | Record<string, unknown>[];
}

/**
 * Centralise les balises SEO : titre, description, URL canonique, Open Graph,
 * Twitter Card et données structurées JSON-LD. Fonctionne côté serveur (SSR),
 * ce qui garantit que les robots reçoivent des métadonnées complètes.
 */
@Injectable({ providedIn: 'root' })
export class SeoService {
  private readonly title = inject(Title);
  private readonly meta = inject(Meta);
  private readonly document = inject(DOCUMENT);

  set(data: SeoData): void {
    const fullTitle = data.title === SITE.name ? SITE.name : `${data.title} | ${SITE.name}`;
    const description = data.description ?? SITE.description;
    const url = SITE.url + data.path;
    const image = this.absolute(data.image ?? SITE.defaultImage);

    this.title.setTitle(fullTitle);
    this.tag('name', 'description', description);
    this.tag('name', 'robots', data.noindex ? 'noindex, nofollow' : 'index, follow');

    this.tag('property', 'og:site_name', SITE.name);
    this.tag('property', 'og:locale', 'fr_FR');
    this.tag('property', 'og:type', data.type ?? 'website');
    this.tag('property', 'og:title', data.title);
    this.tag('property', 'og:description', description);
    this.tag('property', 'og:url', url);
    this.tag('property', 'og:image', image);

    this.tag('name', 'twitter:card', 'summary_large_image');
    this.tag('name', 'twitter:title', data.title);
    this.tag('name', 'twitter:description', description);
    this.tag('name', 'twitter:image', image);

    this.meta.removeTag("property='article:published_time'");
    this.meta.removeTag("property='article:modified_time'");
    this.meta.removeTag("property='article:tag'");
    if (data.type === 'article') {
      if (data.publishedAt) this.tag('property', 'article:published_time', data.publishedAt);
      if (data.modifiedAt) this.tag('property', 'article:modified_time', data.modifiedAt);
      for (const tag of data.tags ?? []) {
        this.meta.addTag({ property: 'article:tag', content: tag });
      }
    }

    this.canonical(url);
    this.jsonLd(data.jsonLd);
  }

  absolute(pathOrUrl: string): string {
    return /^https?:\/\//.test(pathOrUrl) ? pathOrUrl : SITE.url + pathOrUrl;
  }

  private tag(attr: 'name' | 'property', key: string, content: string): void {
    this.meta.updateTag({ [attr]: key, content }, `${attr}='${key}'`);
  }

  private canonical(url: string): void {
    let link = this.document.head.querySelector<HTMLLinkElement>('link[rel="canonical"]');
    if (!link) {
      link = this.document.createElement('link');
      link.rel = 'canonical';
      this.document.head.appendChild(link);
    }
    link.href = url;
  }

  private jsonLd(data: SeoData['jsonLd']): void {
    const id = 'gp-jsonld';
    this.document.getElementById(id)?.remove();
    if (!data) return;
    const script = this.document.createElement('script');
    script.id = id;
    script.type = 'application/ld+json';
    // Échappe « < » pour empêcher toute fermeture prématurée de la balise script.
    script.textContent = JSON.stringify(data).replace(/</g, '\\u003c');
    this.document.head.appendChild(script);
  }
}
