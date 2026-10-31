import { DestroyRef, Injectable, computed, inject, signal } from '@angular/core';

import { BackendClient, Unlisten } from './backend.client';
import { IndexStatus } from './models';

/** Name of the backend event that carries {@link IndexStatus} updates. */
const STATUS_EVENT = 'index-status';

/** The background indexer's status, kept current from backend events. */
@Injectable({ providedIn: 'root' })
export class IndexStatusService {
  private readonly backend = inject(BackendClient);

  private readonly statusState = signal<IndexStatus>({ state: 'idle' });
  private unlisten: Unlisten | null = null;
  private destroyed = false;

  readonly status = this.statusState.asReadonly();
  readonly isRunning = computed(() => this.statusState().state === 'running');
  /** Indexing progress as 0-100, or null when nothing is running. */
  readonly percent = computed(() => {
    const status = this.statusState();
    if (status.state !== 'running' || status.filesTotal === 0) {
      return null;
    }
    return Math.round((status.filesDone / status.filesTotal) * 100);
  });

  constructor() {
    inject(DestroyRef).onDestroy(() => {
      this.destroyed = true;
      this.unlisten?.();
    });
    void this.connect();
  }

  /** Asks the backend to bring the search index up to date. Does nothing if it is already running. */
  async reindex(): Promise<void> {
    await this.backend.call('start_indexing');
  }

  private async connect(): Promise<void> {
    // Listen before asking for the current status, so no update falls in the gap.
    const unlisten = await this.backend.listen<IndexStatus>(STATUS_EVENT, (status) =>
      this.statusState.set(status),
    );
    if (this.destroyed) {
      unlisten();
      return;
    }
    this.unlisten = unlisten;
    this.statusState.set(await this.backend.call<IndexStatus>('index_status'));
  }
}
