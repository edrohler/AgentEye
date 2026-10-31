import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/** A function that stops listening to an event. */
export type Unlisten = () => void;

/**
 * The only place the UI talks to the Tauri backend. Services depend on this class instead of
 * `@tauri-apps/api`, so tests replace one object rather than mocking a module.
 */
@Injectable({ providedIn: 'root' })
export class BackendClient {
  /** Calls a Rust command and resolves with its result. */
  call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(command, args);
  }

  /** Subscribes to a backend event; resolves with a function that unsubscribes. */
  async listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
    return listen<T>(event, (e) => handler(e.payload));
  }
}
