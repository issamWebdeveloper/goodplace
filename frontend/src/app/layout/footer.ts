import { ChangeDetectionStrategy, Component } from '@angular/core';
import { RouterLink } from '@angular/router';

import { Logo } from '../shared/logo';
import { NewsletterForm } from '../shared/newsletter-form';

@Component({
  selector: 'app-footer',
  imports: [RouterLink, Logo, NewsletterForm],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <footer class="section-dark">
      <div class="container grid">
        <div>
          <a routerLink="/" class="brand">
            <app-logo [size]="44" />
            <span>GoodPlace</span>
          </a>
          <p>
            Développement web front-end Angular, full-stack et intelligence artificielle appliquée
            au développement.
          </p>
        </div>
        <nav aria-label="Liens du pied de page">
          <h2>Navigation</h2>
          <ul>
            <li><a routerLink="/">Accueil</a></li>
            <li><a routerLink="/blog">Blog</a></li>
            <li><a href="/rss.xml">Flux RSS</a></li>
            <li><a routerLink="/inscription">Créer un compte</a></li>
          </ul>
        </nav>
        <div>
          <h2>Newsletter</h2>
          <p>Recevez les nouveaux articles par email. Désinscription en un clic.</p>
          <app-newsletter-form [inverse]="true" />
        </div>
      </div>
      <div class="container bottom">
        <span>© {{ year }} GoodPlace — Issam Gharsallah</span>
        <span>
          <a href="https://gharsallah.fr" rel="me noopener" target="_blank" hreflang="en">Portfolio</a>
          ·
          <a href="https://github.com/issamWebdeveloper" rel="noopener" target="_blank">GitHub</a>
          ·
          <a href="https://www.linkedin.com/in/issam-gharsallah/" rel="noopener" target="_blank">LinkedIn</a>
        </span>
      </div>
    </footer>
  `,
  styles: `
    footer {
      padding: 72px 0 28px;
    }
    .grid {
      display: grid;
      grid-template-columns: 1.3fr 0.7fr 1.3fr;
      gap: 48px;
    }
    .brand {
      display: inline-flex;
      align-items: center;
      gap: 12px;
      color: #fff;
      font-family: var(--gp-font-heading);
      font-weight: 300;
      font-size: 1.3rem;
      letter-spacing: 0.14em;
      text-transform: uppercase;
      margin-bottom: 18px;
    }
    h2 {
      font-size: 0.85rem;
      font-weight: 700;
      letter-spacing: 0.18em;
      text-transform: uppercase;
      margin-bottom: 18px;
    }
    ul {
      list-style: none;
      padding: 0;
      margin: 0;
      display: grid;
      gap: 8px;
    }
    a {
      color: rgba(255, 255, 255, 0.8);
      &:hover {
        color: var(--gp-primary);
      }
    }
    .bottom {
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      gap: 12px;
      margin-top: 56px;
      padding-top: 24px;
      border-top: 1px solid rgba(255, 255, 255, 0.1);
      font-size: 0.85rem;
      color: rgba(255, 255, 255, 0.55);
    }
    @media (max-width: 860px) {
      .grid {
        grid-template-columns: 1fr;
        gap: 36px;
      }
    }
  `,
})
export class Footer {
  protected readonly year = new Date().getFullYear();
}
