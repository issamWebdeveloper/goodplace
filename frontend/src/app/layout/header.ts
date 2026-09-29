import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { NavigationEnd, Router, RouterLink, RouterLinkActive } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { filter } from 'rxjs';

import { AuthStore } from '../core/auth/auth.store';
import { Logo } from '../shared/logo';

@Component({
  selector: 'app-header',
  imports: [RouterLink, RouterLinkActive, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '(window:scroll)': 'onScroll()',
    '[class.scrolled]': 'scrolled()',
  },
  template: `
    <header>
      <div class="container bar">
        <a routerLink="/" class="brand" aria-label="GoodPlace, accueil">
          <app-logo [size]="42" />
          <span class="brand-name">Good<strong>Place</strong></span>
        </a>

        <button
          class="burger"
          type="button"
          [attr.aria-expanded]="open()"
          aria-controls="main-nav"
          (click)="open.set(!open())"
        >
          <span class="visually-hidden">Menu</span>
          <span></span><span></span><span></span>
        </button>

        <nav id="main-nav" [class.open]="open()" aria-label="Navigation principale">
          <ul>
            <li><a routerLink="/" fragment="competences">Compétences</a></li>
            <li><a routerLink="/" fragment="parcours">Parcours</a></li>
            <li><a routerLink="/" fragment="projets">Projets</a></li>
            <li><a routerLink="/blog" routerLinkActive="active">Blog</a></li>
            <li><a routerLink="/" fragment="contact">Contact</a></li>
          </ul>
          <div class="actions">
            @if (auth.user(); as user) {
              @if (user.role === 'admin') {
                <a class="btn btn-primary btn-small" routerLink="/admin">Administration</a>
              } @else {
                <a class="btn btn-primary btn-small" routerLink="/compte">Mon compte</a>
              }
            } @else {
              <a class="btn btn-primary btn-small" routerLink="/connexion">Connexion</a>
            }
          </div>
        </nav>
      </div>
    </header>
  `,
  styles: `
    :host {
      position: sticky;
      top: 0;
      z-index: 100;
      display: block;
      background: rgba(255, 255, 255, 0.96);
      backdrop-filter: blur(8px);
      transition: box-shadow 0.2s ease;
    }
    :host(.scrolled) {
      box-shadow: 0 2px 18px rgba(0, 0, 0, 0.08);
    }
    .bar {
      height: var(--gp-header-height);
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 24px;
    }
    .brand {
      display: inline-flex;
      align-items: center;
      gap: 12px;
      color: var(--gp-darker);
    }
    .brand-name {
      font-family: var(--gp-font-heading);
      font-size: 1.35rem;
      font-weight: 300;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      strong {
        font-weight: 900;
        color: var(--gp-primary-strong);
      }
    }
    nav {
      display: flex;
      align-items: center;
      gap: 32px;
    }
    ul {
      display: flex;
      gap: 28px;
      list-style: none;
      margin: 0;
      padding: 0;
    }
    ul a {
      position: relative;
      font-family: var(--gp-font-heading);
      font-size: 0.82rem;
      font-weight: 700;
      letter-spacing: 0.14em;
      text-transform: uppercase;
      color: var(--gp-text);
      padding: 8px 0;
      &::after {
        content: '';
        position: absolute;
        left: 0;
        right: 100%;
        bottom: 0;
        height: 2px;
        background: var(--gp-gradient);
        transition: right 0.2s ease;
      }
      &:hover,
      &.active {
        color: var(--gp-primary-strong);
      }
      &:hover::after,
      &.active::after {
        right: 0;
      }
    }
    .burger {
      display: none;
      flex-direction: column;
      gap: 5px;
      padding: 10px;
      background: none;
      border: 0;
      cursor: pointer;
      span:not(.visually-hidden) {
        width: 24px;
        height: 2px;
        background: var(--gp-dark);
      }
    }
    @media (max-width: 960px) {
      .burger {
        display: inline-flex;
      }
      nav {
        position: absolute;
        top: var(--gp-header-height);
        left: 0;
        right: 0;
        flex-direction: column;
        align-items: stretch;
        gap: 16px;
        padding: 20px clamp(16px, 4vw, 40px) 28px;
        background: #fff;
        box-shadow: 0 12px 20px rgba(0, 0, 0, 0.08);
        display: none;
        &.open {
          display: flex;
        }
      }
      ul {
        flex-direction: column;
        gap: 4px;
      }
    }
  `,
})
export class Header {
  protected readonly auth = inject(AuthStore);
  protected readonly open = signal(false);
  protected readonly scrolled = signal(false);

  constructor() {
    inject(Router)
      .events.pipe(
        filter((e) => e instanceof NavigationEnd),
        takeUntilDestroyed(),
      )
      .subscribe(() => this.open.set(false));
  }

  onScroll(): void {
    this.scrolled.set(window.scrollY > 10);
  }
}
