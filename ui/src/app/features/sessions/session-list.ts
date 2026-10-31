import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';

import { AgentKind } from '../../core/models';
import { SessionService } from '../../core/session.service';
import { SettingsService } from '../../core/settings.service';

type AgentFilter = AgentKind | 'all';

@Component({
  selector: 'app-session-list',
  imports: [DatePipe],
  templateUrl: './session-list.html',
  host: { class: 'block px-6 py-4' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SessionList implements OnInit {
  private readonly service = inject(SessionService);
  private readonly pageSize = inject(SettingsService).sessionPageSize;

  protected readonly filters: readonly AgentFilter[] = ['all', 'claude', 'codex'];
  protected readonly filter = signal<AgentFilter>('all');

  /** How many pages of sessions are showing. Kept as a count so changing the page size setting resizes the list. */
  private readonly pagesShown = signal(1);

  protected readonly loading = this.service.loading;
  protected readonly error = this.service.error;

  private readonly matchingSessions = computed(() => {
    const filter = this.filter();
    const sessions = this.service.sessions();
    return filter === 'all' ? sessions : sessions.filter((s) => s.agent === filter);
  });

  protected readonly totalCount = computed(() => this.matchingSessions().length);

  /** Only this slice is rendered, so the page stays fast however many sessions exist. */
  protected readonly visibleSessions = computed(() =>
    this.matchingSessions().slice(0, this.pageSize() * this.pagesShown()),
  );

  protected readonly remainingCount = computed(() => this.totalCount() - this.visibleSessions().length);

  ngOnInit(): void {
    void this.service.refresh();
  }

  protected refresh(): void {
    void this.service.refresh();
  }

  protected setFilter(filter: AgentFilter): void {
    this.filter.set(filter);
    this.pagesShown.set(1);
  }

  protected viewMore(): void {
    this.pagesShown.update((pages) => pages + 1);
  }

  protected formatSize(bytes: number): string {
    return bytes < 1024 * 1024
      ? `${Math.max(1, Math.round(bytes / 1024))} KB`
      : `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
}
