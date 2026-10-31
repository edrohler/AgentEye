import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';

import { AgentKind } from '../../core/models';
import { SessionService } from '../../core/session.service';

type AgentFilter = AgentKind | 'all';

@Component({
  selector: 'app-session-list',
  imports: [DatePipe],
  templateUrl: './session-list.html',
  styleUrl: './session-list.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SessionList implements OnInit {
  private readonly service = inject(SessionService);

  protected readonly filters: readonly AgentFilter[] = ['all', 'claude', 'codex'];
  protected readonly filter = signal<AgentFilter>('all');

  protected readonly loading = this.service.loading;
  protected readonly error = this.service.error;
  protected readonly visibleSessions = computed(() => {
    const filter = this.filter();
    const sessions = this.service.sessions();
    return filter === 'all' ? sessions : sessions.filter((s) => s.agent === filter);
  });

  ngOnInit(): void {
    void this.service.refresh();
  }

  protected refresh(): void {
    void this.service.refresh();
  }

  protected setFilter(filter: AgentFilter): void {
    this.filter.set(filter);
  }

  protected formatSize(bytes: number): string {
    return bytes < 1024 * 1024
      ? `${Math.max(1, Math.round(bytes / 1024))} KB`
      : `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
}
