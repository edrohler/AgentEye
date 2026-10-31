import { Provider } from '@angular/core';

import { BackendClient, Unlisten } from '../backend.client';

type Handler = (payload: unknown) => void;

/** A stand-in for {@link BackendClient} that serves canned command results and lets tests emit events. */
export class FakeBackend {
  readonly calls: { command: string; args?: Record<string, unknown> }[] = [];
  private readonly handlers = new Map<string, Handler[]>();

  private readonly responses: Record<string, unknown>;

  constructor(responses: Record<string, unknown> = {}) {
    this.responses = responses;
  }

  call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    this.calls.push({ command, args });
    const response = this.responses[command];
    const value = typeof response === 'function' ? (response as () => unknown)() : response;
    return value instanceof Error ? Promise.reject(value) : Promise.resolve(value as T);
  }

  listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
    const handlers = this.handlers.get(event) ?? [];
    handlers.push(handler as Handler);
    this.handlers.set(event, handlers);
    return Promise.resolve(() => {
      this.handlers.set(
        event,
        (this.handlers.get(event) ?? []).filter((h) => h !== (handler as Handler)),
      );
    });
  }

  /** Delivers an event to everything listening to it, as the backend would. */
  emit(event: string, payload: unknown): void {
    (this.handlers.get(event) ?? []).forEach((handler) => handler(payload));
  }

  /** Registers this fake in place of the real {@link BackendClient}. */
  provide(): Provider {
    return { provide: BackendClient, useValue: this };
  }
}
