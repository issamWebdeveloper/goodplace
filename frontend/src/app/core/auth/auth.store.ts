import { HttpClient } from '@angular/common/http';
import { Injectable, PLATFORM_ID, computed, inject, signal } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { firstValueFrom } from 'rxjs';

import { MessageResponse, Role, User } from '../models';

export interface VerifyEmailResult {
  role: Role;
  set_password_token?: string;
  user?: User;
}

/**
 * État d'authentification de l'application.
 * La session est portée par un cookie HttpOnly : le front ne manipule jamais de jeton.
 */
@Injectable({ providedIn: 'root' })
export class AuthStore {
  private readonly http = inject(HttpClient);
  private readonly isBrowser = isPlatformBrowser(inject(PLATFORM_ID));

  private readonly _user = signal<User | null>(null);
  private loading: Promise<User | null> | null = null;

  readonly user = this._user.asReadonly();
  readonly isLoggedIn = computed(() => this._user() !== null);
  readonly isAdmin = computed(() => this._user()?.role === 'admin');

  /** Charge la session courante une seule fois (uniquement dans le navigateur). */
  ensureLoaded(): Promise<User | null> {
    if (!this.isBrowser) return Promise.resolve(null);
    this.loading ??= firstValueFrom(this.http.get<User | null>('/api/auth/me'))
      .then((user) => {
        this._user.set(user);
        return user;
      })
      .catch(() => {
        this._user.set(null);
        return null;
      });
    return this.loading;
  }

  async login(email: string, password: string): Promise<User> {
    const user = await firstValueFrom(
      this.http.post<User>('/api/auth/login', { email, password }),
    );
    this.setUser(user);
    return user;
  }

  register(name: string, email: string): Promise<MessageResponse> {
    return firstValueFrom(this.http.post<MessageResponse>('/api/auth/register', { name, email }));
  }

  subscribe(email: string): Promise<MessageResponse> {
    return firstValueFrom(this.http.post<MessageResponse>('/api/auth/subscribe', { email }));
  }

  async verifyEmail(token: string): Promise<VerifyEmailResult> {
    const result = await firstValueFrom(
      this.http.post<VerifyEmailResult>('/api/auth/verify-email', { token }),
    );
    if (result.user) this.setUser(result.user);
    return result;
  }

  async setPassword(token: string, password: string): Promise<User> {
    const user = await firstValueFrom(
      this.http.post<User>('/api/auth/password', { token, password }),
    );
    this.setUser(user);
    return user;
  }

  forgotPassword(email: string): Promise<MessageResponse> {
    return firstValueFrom(
      this.http.post<MessageResponse>('/api/auth/password/forgot', { email }),
    );
  }

  requestPasswordChange(): Promise<MessageResponse> {
    return firstValueFrom(
      this.http.post<MessageResponse>('/api/auth/password/change-request', {}),
    );
  }

  requestMagicLink(email: string): Promise<MessageResponse> {
    return firstValueFrom(this.http.post<MessageResponse>('/api/auth/magic-link', { email }));
  }

  async consumeMagicLink(token: string): Promise<User> {
    const user = await firstValueFrom(
      this.http.post<User>('/api/auth/magic-link/consume', { token }),
    );
    this.setUser(user);
    return user;
  }

  async updateName(name: string): Promise<User> {
    const user = await firstValueFrom(this.http.patch<User>('/api/account', { name }));
    this.setUser(user);
    return user;
  }

  async deleteAccount(): Promise<void> {
    await firstValueFrom(this.http.delete('/api/account'));
    this.setUser(null);
  }

  async logout(): Promise<void> {
    await firstValueFrom(this.http.post('/api/auth/logout', {}));
    this.setUser(null);
  }

  private setUser(user: User | null): void {
    this._user.set(user);
    this.loading = Promise.resolve(user);
  }
}
