import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';

import { AuthStore } from './auth.store';

/** Réservé aux personnes connectées. */
export const authGuard: CanActivateFn = async (_route, state) => {
  const user = await inject(AuthStore).ensureLoaded();
  return user
    ? true
    : inject(Router).createUrlTree(['/connexion'], { queryParams: { retour: state.url } });
};

/** Réservé à l'administrateur. */
export const adminGuard: CanActivateFn = async (_route, state) => {
  const user = await inject(AuthStore).ensureLoaded();
  if (user?.role === 'admin') return true;
  return inject(Router).createUrlTree(user ? ['/'] : ['/connexion'], {
    queryParams: user ? {} : { retour: state.url },
  });
};

/** Redirige une personne déjà connectée hors des pages de connexion/inscription. */
export const guestGuard: CanActivateFn = async () => {
  const user = await inject(AuthStore).ensureLoaded();
  return user ? inject(Router).createUrlTree([user.role === 'admin' ? '/admin' : '/compte']) : true;
};
