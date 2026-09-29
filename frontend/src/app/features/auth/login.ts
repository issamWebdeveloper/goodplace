import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';
import { Logo } from '../../shared/logo';

type Mode = 'password' | 'magic';

@Component({
  selector: 'app-login',
  imports: [FormsModule, RouterLink, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="auth-page">
      <div class="card auth-card">
        <div class="card-body">
          <app-logo [size]="48" />
          <h1>Connexion</h1>

          <div class="tabs" role="tablist">
            <button type="button" role="tab" [attr.aria-selected]="mode() === 'password'" (click)="switch('password')">
              Mot de passe
            </button>
            <button type="button" role="tab" [attr.aria-selected]="mode() === 'magic'" (click)="switch('magic')">
              Abonné (lien email)
            </button>
          </div>

          @if (mode() === 'password') {
            <form class="form-stack" (ngSubmit)="login()">
              <div class="field">
                <label for="email">Email</label>
                <input id="email" class="input" type="email" name="email" [(ngModel)]="email" autocomplete="email" required />
              </div>
              <div class="field">
                <label for="password">Mot de passe</label>
                <input id="password" class="input" type="password" name="password" [(ngModel)]="password" autocomplete="current-password" required />
              </div>
              @if (error(); as err) {
                <p class="alert alert-error" role="alert">{{ err }}</p>
              }
              <button class="btn btn-primary" [disabled]="pending()">Se connecter</button>
            </form>
            <div class="links">
              <a routerLink="/mot-de-passe-oublie">Mot de passe oublié ?</a>
              <a routerLink="/inscription">Créer un compte</a>
            </div>
          } @else {
            @if (message(); as msg) {
              <p class="alert alert-success" role="status">{{ msg }}</p>
            } @else {
              <p class="intro">
                Abonné à la newsletter ? Recevez un lien de connexion valable 10 minutes, sans mot de passe.
              </p>
              <form class="form-stack" (ngSubmit)="sendMagicLink()">
                <div class="field">
                  <label for="magic-email">Email</label>
                  <input id="magic-email" class="input" type="email" name="email" [(ngModel)]="email" autocomplete="email" required />
                </div>
                @if (error(); as err) {
                  <p class="alert alert-error" role="alert">{{ err }}</p>
                }
                <button class="btn btn-primary" [disabled]="pending()">Recevoir mon lien</button>
              </form>
            }
            <div class="links">
              <a routerLink="/inscription" [queryParams]="{ type: 'newsletter' }">S'abonner à la newsletter</a>
            </div>
          }
        </div>
      </div>
    </div>
  `,
  styleUrl: './auth-shared.scss',
})
export class Login {
  private readonly auth = inject(AuthStore);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);

  protected readonly mode = signal<Mode>('password');
  protected readonly pending = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly message = signal<string | null>(null);
  protected email = '';
  protected password = '';

  constructor() {
    inject(SeoService).set({ title: 'Connexion', path: '/connexion', noindex: true });
  }

  switch(mode: Mode): void {
    this.mode.set(mode);
    this.error.set(null);
    this.message.set(null);
  }

  async login(): Promise<void> {
    await this.run(async () => {
      const user = await this.auth.login(this.email, this.password);
      const back = this.route.snapshot.queryParamMap.get('retour');
      const target = back?.startsWith('/') ? back : user.role === 'admin' ? '/admin' : '/compte';
      await this.router.navigateByUrl(target);
    });
  }

  async sendMagicLink(): Promise<void> {
    await this.run(async () => {
      const res = await this.auth.requestMagicLink(this.email);
      this.message.set(res.message);
    });
  }

  private async run(action: () => Promise<void>): Promise<void> {
    this.pending.set(true);
    this.error.set(null);
    try {
      await action();
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.pending.set(false);
    }
  }
}
