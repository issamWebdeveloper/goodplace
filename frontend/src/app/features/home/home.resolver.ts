import { HttpClient } from '@angular/common/http';
import { inject } from '@angular/core';
import { ResolveFn } from '@angular/router';
import { forkJoin } from 'rxjs';

import { ArticlePage, Profile } from '../../core/models';
import { ProfileService } from '../../core/profile/profile.service';

export interface HomeData {
  profile: Profile;
  latest: ArticlePage;
}

export const homeResolver: ResolveFn<HomeData> = () =>
  forkJoin({
    profile: inject(ProfileService).get(),
    latest: inject(HttpClient).get<ArticlePage>('/api/articles', { params: { per_page: 3 } }),
  });
