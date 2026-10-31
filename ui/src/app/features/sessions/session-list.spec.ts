import { provideZonelessChangeDetection, signal } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SessionSummary } from '../../core/models';
import { SessionService } from '../../core/session.service';
import { SettingsService } from '../../core/settings.service';
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
          useValue: { sessions: signal(sessions), loading: signal(false), error: signal(null), refresh },
        },
      ],
    });
    const fixture = TestBed.createComponent(SessionList);
    fixture.detectChanges();
    return fixture;
  }

  async function update(fixture: ComponentFixture<SessionList>): Promise<void> {
    await fixture.whenStable();
    fixture.detectChanges();
  }

  const element = (fixture: ComponentFixture<SessionList>) => fixture.nativeElement as HTMLElement;
  const rows = (fixture: ComponentFixture<SessionList>) =>
    [...element(fixture).querySelectorAll<HTMLElement>('[data-testid=session]')];
  const filterButtons = (fixture: ComponentFixture<SessionList>) =>
    [...element(fixture).querySelectorAll<HTMLButtonElement>('[role=group] button')];
  const viewMoreButton = (fixture: ComponentFixture<SessionList>) =>
    [...element(fixture).querySelectorAll<HTMLButtonElement>('footer button')].find(
      (b) => b.textContent?.trim() === 'View more',
    );
  const manySessions = (count: number) =>
    Array.from({ length: count }, (_, i) => session(i % 2 === 0 ? 'claude' : 'codex', `session ${i}`));

  beforeEach(() => {
    refresh.calls.reset();
    localStorage.removeItem('agenteye.settings');
  });

  afterEach(() => localStorage.removeItem('agenteye.settings'));

  it('loads sessions on init and lists them without a View more button when they all fit', () => {
    const fixture = render([session('claude', 'fix build'), session('codex', 'add flag')]);

    expect(refresh).toHaveBeenCalledTimes(1);
    expect(rows(fixture).length).toBe(2);
    expect(element(fixture).textContent).toContain('Showing 2 of 2');
    expect(viewMoreButton(fixture)).toBeUndefined();
  });

  it('filters by agent and exposes the selected filter to assistive technology', async () => {
    const fixture = render([session('claude', 'fix build'), session('codex', 'add flag')]);
    expect(filterButtons(fixture).map((b) => b.getAttribute('aria-pressed'))).toEqual([
      'true',
      'false',
      'false',
    ]);

    filterButtons(fixture)
      .find((b) => b.textContent?.trim() === 'codex')
      ?.click();
    await update(fixture);

    expect(rows(fixture).length).toBe(1);
    expect(rows(fixture)[0].textContent).toContain('add flag');
    expect(filterButtons(fixture).map((b) => b.getAttribute('aria-pressed'))).toEqual([
      'false',
      'false',
      'true',
    ]);
  });

  it('shows only one page of a long list and reports how many exist', () => {
    const fixture = render(manySessions(5000));

    expect(rows(fixture).length).toBe(100);
    expect(rows(fixture)[0].textContent).toContain('session 0');
    expect(element(fixture).textContent).toContain('Showing 100 of 5000');
    expect(viewMoreButton(fixture)).toBeDefined();
  });

  it('adds another page each time View more is pressed, until everything is showing', async () => {
    const fixture = render(manySessions(250));

    viewMoreButton(fixture)?.click();
    await update(fixture);
    expect(rows(fixture).length).toBe(200);

    viewMoreButton(fixture)?.click();
    await update(fixture);
    expect(rows(fixture).length).toBe(250);
    expect(element(fixture).textContent).toContain('Showing 250 of 250');
    expect(viewMoreButton(fixture)).toBeUndefined();
  });

  it('starts over at one page when the filter changes', async () => {
    const fixture = render(manySessions(1000));
    viewMoreButton(fixture)?.click();
    await update(fixture);
    expect(rows(fixture).length).toBe(200);

    filterButtons(fixture)
      .find((b) => b.textContent?.trim() === 'codex')
      ?.click();
    await update(fixture);

    expect(rows(fixture).length).toBe(100);
    expect(element(fixture).textContent).toContain('Showing 100 of 500');
  });

  it('uses the page size from settings, including when it changes', async () => {
    const fixture = render(manySessions(1000));
    expect(rows(fixture).length).toBe(100);

    TestBed.inject(SettingsService).setSessionPageSize(250);
    await update(fixture);

    expect(rows(fixture).length).toBe(250);
    viewMoreButton(fixture)?.click();
    await update(fixture);
    expect(rows(fixture).length).toBe(500);
  });
});
