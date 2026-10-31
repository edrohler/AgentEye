import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { SearchHit } from './models';
import { SearchService } from './search.service';
import { FakeBackend } from './testing/fake-backend';

function hit(path: string): SearchHit {
  return { agent: 'claude', path, role: 'user', timestamp: null, byteOffset: 0, snippet: 'x' };
}

function setup(responses: Record<string, unknown>): { service: SearchService; backend: FakeBackend } {
  const backend = new FakeBackend(responses);
  TestBed.configureTestingModule({
    providers: [provideZonelessChangeDetection(), backend.provide()],
  });
  return { service: TestBed.inject(SearchService), backend };
}

describe('SearchService', () => {
  it('searches the backend and groups hits by session', async () => {
    const { service, backend } = setup({ search: [hit('a'), hit('b'), hit('a')] });

    await service.search('needle');

    expect(backend.calls).toEqual([{ command: 'search', args: { query: 'needle' } }]);
    expect(service.hasQuery()).toBeTrue();
    expect(service.hitCount()).toBe(3);
    expect(service.groups().map((g) => g.path)).toEqual(['a', 'b']);
    expect(service.loading()).toBeFalse();
  });

  it('clears results without calling the backend for a blank query', async () => {
    const { service, backend } = setup({ search: [hit('a')] });
    await service.search('needle');

    await service.search('   ');

    expect(backend.calls.length).toBe(1);
    expect(service.hasQuery()).toBeFalse();
    expect(service.hitCount()).toBe(0);
  });

  it('keeps the newest result when responses arrive out of order', async () => {
    const pending: ((hits: SearchHit[]) => void)[] = [];
    const backend = new FakeBackend();
    backend.call = <T>() => new Promise<T>((resolve) => pending.push(resolve as never));
    TestBed.configureTestingModule({
      providers: [provideZonelessChangeDetection(), backend.provide()],
    });
    const service = TestBed.inject(SearchService);

    const first = service.search('old');
    const second = service.search('new');
    pending[1]([hit('new-result')]);
    await second;
    pending[0]([hit('old-result')]);
    await first;

    expect(service.groups().map((g) => g.path)).toEqual(['new-result']);
    expect(service.loading()).toBeFalse();
  });

  it('reports a backend error', async () => {
    const { service } = setup({ search: new Error('index not ready') });

    await service.search('needle');

    expect(service.error()).toContain('index not ready');
    expect(service.loading()).toBeFalse();
  });
});
