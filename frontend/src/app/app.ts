import { ChangeDetectionStrategy, Component, OnInit, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';

import { AuthStore } from './core/auth/auth.store';
import { Footer } from './layout/footer';
import { Header } from './layout/header';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, Header, Footer],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <a class="skip-link" href="#contenu">Aller au contenu</a>
    <app-header />
    <main id="contenu">
      <router-outlet />
    </main>
    <app-footer />
  `,
  styles: `
    .skip-link {
      position: absolute;
      left: 16px;
      top: -60px;
      z-index: 200;
      padding: 10px 16px;
      background: #fff;
      border-radius: var(--gp-radius);
      box-shadow: var(--gp-shadow);
      &:focus {
        top: 16px;
      }
    }
    main {
      display: block;
      min-height: 60vh;
    }
  `,
})
export class App implements OnInit {
  private readonly auth = inject(AuthStore);

  ngOnInit(): void {
    // La session n'est lue que dans le navigateur (le cookie n'est pas transmis au rendu SSR).
    void this.auth.ensureLoaded();
  }
}
