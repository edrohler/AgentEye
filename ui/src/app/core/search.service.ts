import { Injectable, computed, inject, signal } from '@angular/core';

import { BackendClient } from './backend.client';
import { SearchHit } from './models';
import { groupHitsByPath } from './search-groups';

/** Full-text search state: the query, the grouped results, and whether a request is in flight. */
@Injectable({ providedIn: 'root' })
export class SearchService {
  private readonly backend = inject(BackendClient);

  private readonly queryState = signal('');
  private readonly hitsState = signal<readonly SearchHit[]>([]);
  private readonly loadingState = signal(false);
  private readonly errorState = signal<string | null>(null);

  /** Id of the newest request, so a slow older response cannot overwrite a newer one. */
  private latestRequest = 0;

  readonly query = this.queryState.asReadonly();
  readonly loading = this.loadingState.asReadonly();
  readonly error = this.errorState.asReadonly();
  readonly hitCount = computed(() => this.hitsState().length);
  readonly groups = computed(() => groupHitsByPath(this.hitsState()));
  /** True once the user has typed something worth searching for. */
  readonly hasQuery = computed(() => this.queryState().trim().length > 0);

  async search(query: string): Promise<void> {
    this.queryState.set(query);
    const request = ++this.latestRequest;

    if (query.trim().length === 0) {
      this.hitsState.set([]);
      this.errorState.set(null);
      this.loadingState.set(false);
      return;
    }

    this.loadingState.set(true);
    try {
      const hits = await this.backend.call<SearchHit[]>('search', { query });
      if (request === this.latestRequest) {
        this.hitsState.set(hits);
        this.errorState.set(null);
      }
    } catch (error) {
      if (request === this.latestRequest) {
        // The old hits belong to an earlier query; showing them under the new one would mislead.
        this.hitsState.set([]);
        this.errorState.set(String(error));
      }
    } finally {
      if (request === this.latestRequest) {
        this.loadingState.set(false);
      }
    }
  }
}
