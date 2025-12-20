export interface CommitInfo {
  hash: string;
  short_hash: string;
  message: string;
  author: string;
  email: string;
  date: number;
  parent_ids: string[];
}

export interface CommitPage {
  commits: CommitInfo[];
  has_more: boolean;
  total_count: number;
}

export interface RepoInfo {
  name: string;
  path: string;
  branch: string;
}

export interface BranchInfo {
  name: string;
  is_current: boolean;
}

export interface DiffLine {
  line_type: string;
  content: string;
  old_line_no: number | null;
  new_line_no: number | null;
}

export interface FileDiff {
  path: string;
  status: string;
  old_path: string | null;
  insertions: number;
  deletions: number;
  lines: DiffLine[];
}

export interface DiffStats {
  insertions: number;
  deletions: number;
  files_changed: number;
}

export interface CommitDiff {
  hash: string;
  message: string;
  author: string;
  date: number;
  stats: DiffStats;
  files: FileDiff[];
}
