// Types mirrored by hand from the Rust crates. Keep them in lockstep with the serde attributes
// in `agenteye-core`, `agenteye-index` and `agenteye-engine`, and change both in the same commit.

/** Mirrors `agenteye_core::AgentKind` (serialized lowercase). */
export type AgentKind = 'claude' | 'codex';

/** Mirrors `agenteye_core::SessionSummary` (serialized camelCase). */
export interface SessionSummary {
  readonly agent: AgentKind;
  readonly path: string;
  readonly sessionId: string | null;
  readonly title: string | null;
  readonly cwd: string | null;
  readonly model: string | null;
  readonly sizeBytes: number;
  /** Milliseconds since the Unix epoch. */
  readonly lastActivityMs: number;
}

/** Mirrors `agenteye_index::SearchHit` (serialized camelCase). */
export interface SearchHit {
  readonly agent: AgentKind;
  readonly path: string;
  readonly role: string;
  readonly timestamp: string | null;
  /** Byte offset of the matching line in `path`. */
  readonly byteOffset: number;
  /** Excerpt with each match wrapped in the highlight markers; see `snippet.ts`. */
  readonly snippet: string;
}

/** Mirrors `agenteye_engine::IndexStatus` (serialized with a `state` tag). */
export type IndexStatus =
  | { readonly state: 'idle' }
  | {
      readonly state: 'running';
      readonly filesDone: number;
      readonly filesTotal: number;
      readonly messagesAdded: number;
    }
  | { readonly state: 'failed'; readonly message: string };
