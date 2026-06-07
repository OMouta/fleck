/**
 * Tiny subsequence fuzzy matcher for the command palette.
 *
 * Returns a score (higher = better) when every character of `query` appears in
 * `text` in order, or null when there's no match. Consecutive matches and
 * word-boundary / start-of-string hits score higher so the obvious result ranks
 * first. Case-insensitive.
 */
export function fuzzyScore(query: string, text: string): number | null {
  const q = query.toLowerCase();
  const t = text.toLowerCase();
  if (q.length === 0) return 0;

  let score = 0;
  let ti = 0;
  let prevMatched = false;

  for (let qi = 0; qi < q.length; qi++) {
    const ch = q[qi];
    let found = -1;
    for (let i = ti; i < t.length; i++) {
      if (t[i] === ch) {
        found = i;
        break;
      }
    }
    if (found === -1) return null;

    // Base point for the match.
    score += 1;
    // Bonus for matching at the start or after a separator (word boundary).
    if (found === 0 || /[\s\-_./]/.test(t[found - 1])) score += 3;
    // Bonus for consecutive matches.
    if (prevMatched && found === ti) score += 2;

    prevMatched = found === ti;
    ti = found + 1;
  }

  // Prefer shorter targets (tighter matches) and full prefix matches.
  if (t.startsWith(q)) score += 5;
  score -= t.length * 0.02;
  return score;
}

/** Best fuzzy score across several candidate strings (e.g. label + aliases). */
export function bestFuzzyScore(query: string, candidates: string[]): number | null {
  let best: number | null = null;
  for (const candidate of candidates) {
    const s = fuzzyScore(query, candidate);
    if (s !== null && (best === null || s > best)) best = s;
  }
  return best;
}
