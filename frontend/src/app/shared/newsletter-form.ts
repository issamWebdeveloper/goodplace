import { ChangeDetectionStrategy, Component, inject, input, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { AuthStore } from '../core/auth/auth.store';
import { errorMessage } from '../core/http-errors';

/** Inscription à la newsletter : email uniquement, confirmation par lien. */
@Component({
  selector: 'app-newsletter-form',
  imports: [FormsModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (message(); as msg) {
      <p class="alert alert-success" role="status">{{ msg }}</p>
    } @else {
      <form (ngSubmit)="submit()" class="newsletter" [class.inverse]="inverse()">
        <label class="visually-hidden" for="nl-email-{{ uid }}">Adresse email</label>
        <input
          id="nl-email-{{ uid }}"
          class="input"
          type="email"
          name="email"
          [(ngModel)]="email"
          placeholder="votre@email.fr"
          autocomplete="email"
          required
        />
        <button class="btn" [class.btn-light]="inverse()" [class.btn-primary]="!inverse()" [disabled]="pending()">
          {{ pending() ? 'Envoi…' : "S'abonner" }}
        </button>
      </form>
      @if (error(); as err) {
        <p class="field-error" role="alert">{{ err }}</p>
      }
    }
  `,
  styles: `
    .newsletter {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      .input {
        flex: 1 1 220px;
        border-radius: 500px;
        padding-inline: 22px;
      }
    }
    .inverse .input {
      border-color: transparent;
    }
    .alert {
      color: inherit;
    }
    .field-error {
      margin-top: 8px;
    }
  `,
})
export class NewsletterForm {
  private static counter = 0;
  private readonly auth = inject(AuthStore);

  readonly inverse = input(false);
  protected readonly uid = NewsletterForm.counter++;
  protected email = '';
  protected readonly pending = signal(false);
  protected readonly message = signal<string | null>(null);
  protected readonly error = signal<string | null>(null);

  async submit(): Promise<void> {
    if (!this.email.trim()) return;
    this.pending.set(true);
    this.error.set(null);
    try {
      const res = await this.auth.subscribe(this.email);
      this.message.set(res.message);
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.pending.set(false);
    }
  }
}
