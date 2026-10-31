import { Injectable, inject, signal } from '@angular/core';

import { BackendClient } from './backend.client';
import { SessionSummary } from './models';

/** Session data from the Rust backend, exposed as read-only signals. */
@Injectable({ providedIn: 'root' })
export class SessionService {
  private readonly backend = inject(BackendClient);

  private readonly sessionsState = signal<readonly SessionSummary[]>([]);
  private readonly loadingState = signal(false);
  private readonly errorState = signal<string | null>(null);

  readonly sessions = this.sessionsState.asReadonly();
  readonly loading = this.loadingState.asReadonly();
  readonly error = this.errorState.asReadonly();

  async refresh(): Promise<void> {
    this.loadingState.set(true);
    this.errorState.set(null);
    try {
      this.sessionsState.set(await this.backend.call<SessionSummary[]>('list_sessions'));
    } catch (error) {
      this.errorState.set(String(error));
    } finally {
      this.loadingState.set(false);
    }
  }
}
