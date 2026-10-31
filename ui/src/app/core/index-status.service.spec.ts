import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { IndexStatusService } from './index-status.service';
import { IndexStatus } from './models';
import { FakeBackend } from './testing/fake-backend';

async function setup(initial: IndexStatus): Promise<{ service: IndexStatusService; backend: FakeBackend }> {
  const backend = new FakeBackend({ index_status: initial, start_indexing: undefined });
  TestBed.configureTestingModule({
    providers: [provideZonelessChangeDetection(), backend.provide()],
  });
  const service = TestBed.inject(IndexStatusService);
  // The service connects asynchronously; a timer tick lets every pending promise settle.
  await new Promise((resolve) => setTimeout(resolve));
  return { service, backend };
}

describe('IndexStatusService', () => {
  it('starts from the status the backend reports', async () => {
    const { service } = await setup({
      state: 'running',
      filesDone: 5,
      filesTotal: 10,
      messagesAdded: 3,
    });

    expect(service.isRunning()).toBeTrue();
    expect(service.percent()).toBe(50);
  });

  it('follows status events', async () => {
    const { service, backend } = await setup({ state: 'idle' });
    expect(service.percent()).toBeNull();

    backend.emit('index-status', { state: 'failed', message: 'disk full' });

    expect(service.status()).toEqual({ state: 'failed', message: 'disk full' });
    expect(service.isRunning()).toBeFalse();
  });

  it('ignores a status snapshot that arrives after a newer live update', async () => {
    const running: IndexStatus = { state: 'running', filesDone: 1, filesTotal: 2, messagesAdded: 0 };
    let resolveSnapshot: (status: IndexStatus) => void = () => undefined;
    const backend = new FakeBackend();
    backend.call = <T>() => new Promise<T>((resolve) => (resolveSnapshot = resolve as never));
    TestBed.configureTestingModule({
      providers: [provideZonelessChangeDetection(), backend.provide()],
    });
    const service = TestBed.inject(IndexStatusService);
    await new Promise((resolve) => setTimeout(resolve));

    // Indexing finishes (live event) before the earlier "running" snapshot is delivered.
    backend.emit('index-status', { state: 'idle' });
    resolveSnapshot(running);
    await new Promise((resolve) => setTimeout(resolve));

    expect(service.status()).toEqual({ state: 'idle' });
    expect(service.isRunning()).toBeFalse();
  });

  it('asks the backend to reindex', async () => {
    const { service, backend } = await setup({ state: 'idle' });

    await service.reindex();

    expect(backend.calls.map((c) => c.command)).toContain('start_indexing');
  });
});
