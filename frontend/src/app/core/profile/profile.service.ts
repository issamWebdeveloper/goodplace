import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, shareReplay } from 'rxjs';

import { Profile } from '../models';

@Injectable({ providedIn: 'root' })
export class ProfileService {
  private readonly http = inject(HttpClient);
  private readonly profile$ = this.http.get<Profile>('/api/profile').pipe(shareReplay(1));

  get(): Observable<Profile> {
    return this.profile$;
  }
}
