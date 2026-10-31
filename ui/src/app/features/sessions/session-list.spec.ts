import { provideZonelessChangeDetection, signal } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';

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

  function render(sessions: SessionSummary[]): ComponentFixture<SessionList> {
    TestBed.configureTestingModule({
      imports: [SessionList],
      providers: [
        provideZonelessChangeDetection(),
        {
          provide: SessionService,
          useValue: {
            sessions: signal(sessions),
            loading: signal(false),
            error: signal(null),
            refresh,
          },
        },
      ],
    });
    return TestBed.createComponent(SessionList);
  }

  /** The virtual scroller measures and renders rows after the first change detection, so let it settle. */
  async function settle(fixture: ComponentFixture<SessionList>): Promise<void> {
    fixture.detectChanges();
    await new Promise((resolve) => setTimeout(resolve, 50));
    await fixture.whenStable();
    fixture.detectChanges();
  }

  const rows = (fixture: ComponentFixture<SessionList>): HTMLElement[] => [
    ...(fixture.nativeElement as HTMLElement).querySelectorAll<HTMLElement>('[data-testid=session]'),
  ];

  const filterButtons = (fixture: ComponentFixture<SessionList>): HTMLButtonElement[] => [
    ...(fixture.nativeElement as HTMLElement).querySelectorAll<HTMLButtonElement>('[role=group] button'),
  ];

  beforeEach(() => refresh.calls.reset());

  it('loads sessions on init and lists them', async () => {
    const fixture = render([session('claude', 'fix build'), session('codex', 'add flag')]);
    await settle(fixture);

    expect(refresh).toHaveBeenCalledTimes(1);
    expect(rows(fixture).length).toBe(2);
  });

  it('filters by agent and exposes the selected filter to assistive technology', async () => {
    const fixture = render([session('claude', 'fix build'), session('codex', 'add flag')]);
    await settle(fixture);
    expect(filterButtons(fixture).map((b) => b.getAttribute('aria-pressed'))).toEqual([
      'true',
      'false',
      'false',
    ]);

    filterButtons(fixture)
      .find((b) => b.textContent?.trim() === 'codex')
      ?.click();
    await settle(fixture);

    expect(rows(fixture).length).toBe(1);
    expect(rows(fixture)[0].textContent).toContain('add flag');
    expect(filterButtons(fixture).map((b) => b.getAttribute('aria-pressed'))).toEqual([
      'false',
      'false',
      'true',
    ]);
  });

  it('renders only the rows near the window however many sessions there are', async () => {
    const many = Array.from({ length: 5000 }, (_, i) => session('claude', `session ${i}`));
    const fixture = render(many);
    await settle(fixture);

    const rendered = rows(fixture).length;
    expect(rendered).toBeGreaterThan(0);
    expect(rendered).toBeLessThan(150);
    expect(rows(fixture)[0].textContent).toContain('session 0');
  });

  it('renders the rows at the new position after the window is scrolled', async () => {
    const many = Array.from({ length: 5000 }, (_, i) => session('claude', `session ${i}`));
    const fixture = render(many);
    document.body.appendChild(fixture.nativeElement);
    await settle(fixture);

    const targetRow = 2000;
    window.scrollTo(0, targetRow * 56 + (fixture.nativeElement as HTMLElement).offsetTop);

    // The scroller reacts to real scroll events on animation frames, so poll until it catches up.
    let firstRow = -1;
    for (let attempt = 0; attempt < 40 && firstRow < targetRow - 30; attempt++) {
      await new Promise((resolve) => setTimeout(resolve, 50));
      fixture.detectChanges();
      const match = /session (\d+)/.exec(rows(fixture)[0]?.textContent ?? '');
      firstRow = match ? Number(match[1]) : -1;
    }

    window.scrollTo(0, 0);
    expect(firstRow).toBeGreaterThanOrEqual(targetRow - 30);
    expect(firstRow).toBeLessThanOrEqual(targetRow);
    expect(rows(fixture).length).toBeLessThan(150);
  });
});
