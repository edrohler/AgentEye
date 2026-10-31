import { AgentKind, SearchHit } from './models';

/** All hits from one session file, kept in the order the backend ranked them. */
export interface SearchGroup {
  readonly path: string;
  readonly agent: AgentKind;
  readonly hits: readonly SearchHit[];
}

/** Groups ranked hits by session. Groups appear in the order their best hit was ranked. */
export function groupHitsByPath(hits: readonly SearchHit[]): readonly SearchGroup[] {
  const groups = new Map<string, { agent: AgentKind; hits: SearchHit[] }>();
  for (const hit of hits) {
    const group = groups.get(hit.path);
    if (group) {
      group.hits.push(hit);
    } else {
      groups.set(hit.path, { agent: hit.agent, hits: [hit] });
    }
  }
  return [...groups].map(([path, group]) => ({ path, ...group }));
}
