import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { FakeBackend } from '../../core/testing/fake-backend';
import { IndexStatusIndicator } from './index-status';

describe('IndexStatusIndicator', () => {
  async function render(status: unknown) {
    const backend = new FakeBackend({ index_status: status, start_indexing: undefined });
    TestBed.configureTestingModule({
      imports: [IndexStatusIndicator],
      providers: [provideZonelessChangeDetection(), backend.provide()],
    });
    const fixture = TestBed.createComponent(IndexStatusIndicator);
    await new Promise((resolve) => setTimeout(resolve));
    await fixture.whenStable();
    fixture.detectChanges();
    return { fixture, element: fixture.nativeElement as HTMLElement, backend };
  }

  it('shows progress and disables reindex while running', async () => {
    const { element } = await render({
      state: 'running',
      filesDone: 1,
      filesTotal: 4,
      messagesAdded: 0,
    });

    expect(element.textContent).toContain('Indexing 25%');
    expect(element.querySelector('button')?.disabled).toBeTrue();
  });

  it('shows ready when idle and starts indexing on click', async () => {
    const { element, backend } = await render({ state: 'idle' });

    expect(element.textContent).toContain('Search index ready');
    element.querySelector('button')?.click();

    expect(backend.calls.some((c) => c.command === 'start_indexing')).toBeTrue();
  });

  it('shows a failure', async () => {
    const { element } = await render({ state: 'failed', message: '3 of 10 sessions could not be indexed' });

    expect(element.textContent).toContain('Indexing failed: 3 of 10 sessions could not be indexed');
  });
});
