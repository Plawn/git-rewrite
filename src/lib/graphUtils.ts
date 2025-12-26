import type { CommitInfo } from './types';

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
 */
export function calculateGraphLayout(commits: CommitInfo[]): GraphLayout {
  if (commits.length === 0) {
    return { nodes: new Map(), maxRails: 0 };
  }

  const nodes = new Map<string, GraphNode>();

  // Track active rails: each entry is [expectedCommitHash, colorIndex] or null
  const activeRails: (readonly [string, number] | null)[] = [];
  let maxRails = 0;
  let colorCounter = 0;

  // Build a set of commits we have for quick lookup
  const commitSet = new Set(commits.map(c => c.hash));

  for (const commit of commits) {
    // Find rails that are expecting this commit
    const railsExpectingMe: number[] = [];
    for (let i = 0; i < activeRails.length; i++) {
      const rail = activeRails[i];
      if (rail && rail[0] === commit.hash) {
        railsExpectingMe.push(i);
      }
    }

    // Determine which rail this commit should be on
    let myRail: number;
    let myColor: number;

    if (railsExpectingMe.length > 0) {
      // Use the first rail that expects us
      myRail = railsExpectingMe[0];
      myColor = activeRails[myRail]![1];
    } else {
      // Find an empty rail or create a new one
      const emptyRail = activeRails.findIndex(r => r === null);
      myColor = colorCounter++ % 8;
      if (emptyRail !== -1) {
        myRail = emptyRail;
      } else {
        myRail = activeRails.length;
        activeRails.push(null);
      }
    }

    const connections: GraphConnection[] = [];

    // Clear all rails that were expecting this commit
    for (const railIdx of railsExpectingMe) {
      if (railIdx !== myRail) {
        // Draw a merge line from that rail to our rail
        connections.push({
          fromRail: railIdx,
          toRail: myRail,
          type: 'merge',
          colorIndex: activeRails[railIdx]![1],
        });
      }
      activeRails[railIdx] = null;
    }

    // Set up rails for parents
    const parentCount = commit.parent_ids.length;

    if (parentCount === 0) {
      // Initial commit - rail ends here
      activeRails[myRail] = null;
    } else if (parentCount === 1) {
      // Single parent - continue on same rail
      const parentHash = commit.parent_ids[0];
      activeRails[myRail] = [parentHash, myColor] as const;

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
          activeRails[myRail] = [parentHash, myColor] as const;
          if (commitSet.has(parentHash)) {
            connections.push({
              fromRail: myRail,
              toRail: myRail,
              type: 'straight',
              colorIndex: myColor,
            });
          }
        } else {
          // Other parents: check if already expected on a rail
          const existingRail = activeRails.findIndex(
            r => r !== null && r[0] === parentHash
          );

          if (existingRail !== -1) {
            // Parent already on a rail, draw branch line to it
            connections.push({
              fromRail: myRail,
              toRail: existingRail,
              type: 'branch',
              colorIndex: activeRails[existingRail]![1],
            });
          } else {
            // Find or create a new rail for this parent
            const emptyRail = activeRails.findIndex(r => r === null);
            const newColor = colorCounter++ % 8;
            let parentRail: number;

            if (emptyRail !== -1) {
              parentRail = emptyRail;
              activeRails[emptyRail] = [parentHash, newColor] as const;
            } else {
              parentRail = activeRails.length;
              activeRails.push([parentHash, newColor] as const);
            }

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

    // Track max rails
    const activeCount = activeRails.filter(r => r !== null).length;
    maxRails = Math.max(maxRails, activeCount, myRail + 1);

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
