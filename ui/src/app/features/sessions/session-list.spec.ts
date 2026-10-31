import { provideZonelessChangeDetection, signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { SessionSummary } from '../../core/models';
import { SessionService } from '../../core/session.service';
import { SessionList } from './session-list';

function session(agent: SessionSummary['agent'], title: string): SessionSummary {
  return {
    agent,
    path: `/sessions/${title}.jsonl`,
    sessionId: null,
    title,
    cwd: null,
    model: null,
    sizeBytes: 2048,
    lastActivityMs: 0,
  };
}

describe('SessionList', () => {
  const refresh = jasmine.createSpy('refresh').and.resolveTo();

  beforeEach(() => {
    refresh.calls.reset();
    TestBed.configureTestingModule({
      imports: [SessionList],
      providers: [
        provideZonelessChangeDetection(),
        {
          provide: SessionService,
          useValue: {
            sessions: signal([session('claude', 'fix build'), session('codex', 'add flag')]),
            loading: signal(false),
            error: signal(null),
            refresh,
          },
        },
      ],
    });
  });

  it('loads sessions on init and lists them', () => {
    const fixture = TestBed.createComponent(SessionList);
    fixture.detectChanges();

    expect(refresh).toHaveBeenCalledTimes(1);
    expect(fixture.nativeElement.querySelectorAll('.sessions li').length).toBe(2);
  });

  it('filters by agent', async () => {
    const fixture = TestBed.createComponent(SessionList);
    fixture.detectChanges();

    const codexButton = [...fixture.nativeElement.querySelectorAll('.filters button')].find(
      (b) => (b as HTMLElement).textContent?.trim() === 'codex',
    ) as HTMLButtonElement;
    codexButton.click();
    await fixture.whenStable();

    const rows = fixture.nativeElement.querySelectorAll('.sessions li');
    expect(rows.length).toBe(1);
    expect(rows[0].textContent).toContain('add flag');
  });
});
