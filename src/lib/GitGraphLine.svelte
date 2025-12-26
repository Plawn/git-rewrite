<script lang="ts">
  import type { GraphNode } from './graphUtils';

  interface Props {
    node: GraphNode;
    maxRails: number;
    rowHeight: number;
    isSelected?: boolean;
  }

  let { node, maxRails, rowHeight, isSelected = false }: Props = $props();

  // Color palette for branches
  const colors = [
    '#6366f1', // indigo
    '#8b5cf6', // violet
    '#ec4899', // pink
    '#f97316', // orange
    '#22c55e', // green
    '#06b6d4', // cyan
    '#3b82f6', // blue
    '#eab308', // yellow
  ];

  const railWidth = 14;
  const nodeRadius = 4;
  const strokeWidth = 2;

  // Calculate SVG dimensions
  let svgWidth = $derived(Math.max((maxRails + 1) * railWidth, railWidth * 2));
  let centerY = $derived(rowHeight / 2);

  function getRailX(rail: number): number {
    return rail * railWidth + railWidth / 2 + 2;
  }

  function getColor(colorIndex: number): string {
    return colors[colorIndex % colors.length];
  }
</script>

<svg
  width={svgWidth}
  height={rowHeight}
  class="git-graph-line"
  style="min-width: {svgWidth}px"
>
  <!-- Pass-through rails (vertical lines that continue through this row) -->
  {#each node.passThroughRails as pt}
    <line
      x1={getRailX(pt.rail)}
      y1={0}
      x2={getRailX(pt.rail)}
      y2={rowHeight}
      stroke={getColor(pt.colorIndex)}
      stroke-width={strokeWidth}
      opacity="0.5"
    />
  {/each}

  <!-- Connections -->
  {#each node.connections as conn}
    {#if conn.type === 'straight'}
      <!-- Vertical line on this rail -->
      <line
        x1={getRailX(conn.fromRail)}
        y1={0}
        x2={getRailX(conn.toRail)}
        y2={rowHeight}
        stroke={getColor(conn.colorIndex)}
        stroke-width={strokeWidth}
      />
    {:else if conn.type === 'merge'}
      <!-- Line coming from another rail (above) merging into this commit -->
      <path
        d="M {getRailX(conn.fromRail)} 0
           L {getRailX(conn.fromRail)} {centerY - 4}
           Q {getRailX(conn.fromRail)} {centerY}, {getRailX(conn.toRail)} {centerY}"
        fill="none"
        stroke={getColor(conn.colorIndex)}
        stroke-width={strokeWidth}
      />
    {:else if conn.type === 'branch'}
      <!-- Line branching from this commit to another rail (below) -->
      <path
        d="M {getRailX(conn.fromRail)} {centerY}
           Q {getRailX(conn.toRail)} {centerY}, {getRailX(conn.toRail)} {centerY + 4}
           L {getRailX(conn.toRail)} {rowHeight}"
        fill="none"
        stroke={getColor(conn.colorIndex)}
        stroke-width={strokeWidth}
      />
    {/if}
  {/each}

  <!-- The commit node -->
  <circle
    cx={getRailX(node.rail)}
    cy={centerY}
    r={nodeRadius}
    fill={isSelected ? '#ffffff' : getColor(node.colorIndex)}
    stroke={getColor(node.colorIndex)}
    stroke-width={isSelected ? 2.5 : 1.5}
    class="commit-node"
    class:selected={isSelected}
  />
</svg>

<style>
  .git-graph-line {
    flex-shrink: 0;
    display: block;
  }

  .commit-node {
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.3));
  }

  .commit-node.selected {
    filter: drop-shadow(0 0 4px var(--accent-color, #6366f1));
  }
</style>
