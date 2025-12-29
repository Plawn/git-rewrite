import type { CommitInfo } from './types';
import { GRAPH } from './constants';

export interface GraphConnection {
  fromRail: number;
  toRail: number;
  type: 'straight' | 'merge' | 'branch';
  colorIndex: number;
}

export interface GraphNode {
  rail: number;
  colorIndex: number;
  connections: GraphConnection[];
  passThroughRails: { rail: number; colorIndex: number }[];
}

export interface GraphLayout {
  nodes: Map<string, GraphNode>;
  maxRails: number;
}

/**
 * Calculate git graph layout from commits.
 * Commits should be sorted from newest to oldest (HEAD first).
 *
 * Optimized with O(1) lookups using Maps instead of array searches.
 */
export function calculateGraphLayout(commits: CommitInfo[]): GraphLayout {
  if (commits.length === 0) {
    return { nodes: new Map(), maxRails: 0 };
  }

  const nodes = new Map<string, GraphNode>();

  // Track active rails: each entry is [expectedCommitHash, colorIndex] or null
  const activeRails: (readonly [string, number] | null)[] = [];

  // O(1) lookup: which rails are expecting a given commit hash
  const hashToRails = new Map<string, number[]>();

  // O(1) lookup: track empty rail indices for reuse
  const emptyRails: number[] = [];

  // Track active rail count to avoid filtering
  let activeRailCount = 0;
  let maxRails = 0;
  let colorCounter = 0;

  // Build a set of commits we have for quick lookup
  const commitSet = new Set(commits.map(c => c.hash));

  // Helper: register a rail as expecting a commit hash
  function setRailExpecting(railIdx: number, hash: string, color: number) {
    activeRails[railIdx] = [hash, color] as const;
    const existing = hashToRails.get(hash);
    if (existing) {
      existing.push(railIdx);
    } else {
      hashToRails.set(hash, [railIdx]);
    }
  }

  // Helper: clear a rail
  function clearRail(railIdx: number) {
    const rail = activeRails[railIdx];
    if (rail) {
      const hash = rail[0];
      const rails = hashToRails.get(hash);
      if (rails) {
        const idx = rails.indexOf(railIdx);
        if (idx !== -1) rails.splice(idx, 1);
        if (rails.length === 0) hashToRails.delete(hash);
      }
      activeRails[railIdx] = null;
      emptyRails.push(railIdx);
      activeRailCount--;
    }
  }

  // Helper: allocate a rail (reuse empty or create new)
  function allocateRail(): number {
    activeRailCount++;
    if (emptyRails.length > 0) {
      return emptyRails.pop()!;
    }
    const newIdx = activeRails.length;
    activeRails.push(null);
    return newIdx;
  }

  for (const commit of commits) {
    // O(1) lookup: find rails expecting this commit
    const railsExpectingMe = hashToRails.get(commit.hash) ?? [];

    // Determine which rail this commit should be on
    let myRail: number;
    let myColor: number;

    if (railsExpectingMe.length > 0) {
      // Use the first rail that expects us
      myRail = railsExpectingMe[0];
      myColor = activeRails[myRail]![1];
    } else {
      // Allocate a new rail
      myRail = allocateRail();
      myColor = colorCounter++ % GRAPH.COLOR_COUNT;
    }

    const connections: GraphConnection[] = [];

    // Clear all rails that were expecting this commit (and draw merge lines)
    // Copy the array since we'll modify hashToRails during iteration
    const railsToClear = [...railsExpectingMe];
    for (const railIdx of railsToClear) {
      if (railIdx !== myRail) {
        // Draw a merge line from that rail to our rail
        connections.push({
          fromRail: railIdx,
          toRail: myRail,
          type: 'merge',
          colorIndex: activeRails[railIdx]![1],
        });
      }
      clearRail(railIdx);
    }

    // Set up rails for parents
    const parentCount = commit.parent_ids.length;

    if (parentCount === 0) {
      // Initial commit - rail ends here, already cleared above or not allocated
      if (railsExpectingMe.length === 0) {
        // We allocated a rail but don't need it
        activeRailCount--;
        emptyRails.push(myRail);
      }
    } else if (parentCount === 1) {
      // Single parent - continue on same rail
      const parentHash = commit.parent_ids[0];

      if (railsExpectingMe.length === 0) {
        // New rail was allocated, count is already incremented
      } else {
        // Reusing existing rail, need to increment since we cleared it
        activeRailCount++;
        // Remove from empty rails if it was added
        const emptyIdx = emptyRails.indexOf(myRail);
        if (emptyIdx !== -1) emptyRails.splice(emptyIdx, 1);
      }

      setRailExpecting(myRail, parentHash, myColor);

      // Draw straight line down if parent is in our commits
      if (commitSet.has(parentHash)) {
        connections.push({
          fromRail: myRail,
          toRail: myRail,
          type: 'straight',
          colorIndex: myColor,
        });
      }
    } else {
      // Merge commit - multiple parents
      for (let i = 0; i < commit.parent_ids.length; i++) {
        const parentHash = commit.parent_ids[i];

        if (i === 0) {
          // First parent continues on our rail
          if (railsExpectingMe.length === 0) {
            // New rail was allocated
          } else {
            activeRailCount++;
            const emptyIdx = emptyRails.indexOf(myRail);
            if (emptyIdx !== -1) emptyRails.splice(emptyIdx, 1);
          }

          setRailExpecting(myRail, parentHash, myColor);

          if (commitSet.has(parentHash)) {
            connections.push({
              fromRail: myRail,
              toRail: myRail,
              type: 'straight',
              colorIndex: myColor,
            });
          }
        } else {
          // Other parents: check if already expected on a rail (O(1) lookup)
          const existingRails = hashToRails.get(parentHash);

          if (existingRails && existingRails.length > 0) {
            // Parent already on a rail, draw branch line to it
            const existingRail = existingRails[0];
            connections.push({
              fromRail: myRail,
              toRail: existingRail,
              type: 'branch',
              colorIndex: activeRails[existingRail]![1],
            });
          } else {
            // Allocate a new rail for this parent
            const parentRail = allocateRail();
            const newColor = colorCounter++ % GRAPH.COLOR_COUNT;
            setRailExpecting(parentRail, parentHash, newColor);

            connections.push({
              fromRail: myRail,
              toRail: parentRail,
              type: 'branch',
              colorIndex: newColor,
            });
          }
        }
      }
    }

    // Track max rails (O(1) instead of filtering)
    maxRails = Math.max(maxRails, activeRailCount, myRail + 1);

    // Calculate pass-through rails (rails not involved in this commit's connections)
    const involvedRails = new Set<number>([myRail]);
    for (const conn of connections) {
      involvedRails.add(conn.fromRail);
      involvedRails.add(conn.toRail);
    }

    const passThroughRails: { rail: number; colorIndex: number }[] = [];
    for (let i = 0; i < activeRails.length; i++) {
      const rail = activeRails[i];
      if (rail !== null && !involvedRails.has(i)) {
        passThroughRails.push({ rail: i, colorIndex: rail[1] });
      }
    }

    nodes.set(commit.hash, {
      rail: myRail,
      colorIndex: myColor,
      connections,
      passThroughRails,
    });
  }

  return { nodes, maxRails };
}
