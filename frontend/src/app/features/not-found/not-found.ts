import { ChangeDetectionStrategy, Component, RESPONSE_INIT, inject } from '@angular/core';
import { RouterLink } from '@angular/router';

import { SeoService } from '../../core/seo/seo.service';

@Component({
  selector: 'app-not-found',
  imports: [RouterLink],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <section class="section">
      <div class="container container-narrow center">
        <p class="code text-gradient">404</p>
        <h1 class="heading-section">Page introuvable</h1>
        <p class="lead">La page demandée n'existe pas ou a été déplacée.</p>
        <a class="btn btn-primary" routerLink="/">Retour à l'accueil</a>
      </div>
    </section>
  `,
  styles: `
    .center {
      text-align: center;
    }
    .code {
      font-family: var(--gp-font-heading);
      font-weight: 100;
      font-size: clamp(6rem, 20vw, 12rem);
      line-height: 1;
      margin: 0;
    }
  `,
})
export class NotFound {
  constructor() {
    const response = inject(RESPONSE_INIT, { optional: true });
    if (response) response.status = 404;
    inject(SeoService).set({ title: 'Page introuvable', path: '/404', noindex: true });
  }
}
