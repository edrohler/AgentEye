import { SearchHit } from './models';
import { groupHitsByPath } from './search-groups';

function hit(path: string, byteOffset: number): SearchHit {
  return { agent: 'claude', path, role: 'user', timestamp: null, byteOffset, snippet: '' };
}

describe('groupHitsByPath', () => {
  it('returns no groups for no hits', () => {
    expect(groupHitsByPath([])).toEqual([]);
  });

  it('groups by path, ordered by first appearance, preserving hit order', () => {
    const groups = groupHitsByPath([hit('a', 1), hit('b', 2), hit('a', 3)]);

    expect(groups.map((g) => g.path)).toEqual(['a', 'b']);
    expect(groups[0].hits.map((h) => h.byteOffset)).toEqual([1, 3]);
    expect(groups[1].hits.map((h) => h.byteOffset)).toEqual([2]);
  });
});
