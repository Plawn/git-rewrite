use git2::{Oid, Repository, Sort, StatusOptions, ResetType, BranchType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

fn check_working_directory_clean(repo: &Repository) -> Result<(), String> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(false);

    let statuses = repo.statuses(Some(&mut opts))
        .map_err(|e| format!("Failed to get status: {}", e))?;

    if !statuses.is_empty() {
        return Err("Working directory has uncommitted changes. Please commit or stash them first.".to_string());
    }
    Ok(())
}

fn stash_changes(repo: &mut Repository) -> Result<bool, String> {
    let signature = repo.signature()
        .map_err(|e| format!("Failed to get signature: {}", e))?;

    let stash_result = repo.stash_save(
        &signature,
        "git-rewrite-auto-stash",
        None
    );

    match stash_result {
        Ok(_) => Ok(true),
        Err(e) if e.message().contains("nothing to stash") => Ok(false),
        Err(e) => Err(format!("Failed to stash: {}", e))
    }
}

fn unstash_changes(repo: &mut Repository) -> Result<(), String> {
    repo.stash_pop(0, None)
        .map_err(|e| format!("Failed to restore stashed changes: {}", e))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub date: i64,
    pub parent_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitPage {
    pub commits: Vec<CommitInfo>,
    pub has_more: bool,
    pub total_count: usize,
}

#[tauri::command]
fn get_commits(repo_path: String, offset: usize, limit: usize) -> Result<CommitPage, String> {
    let repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let mut revwalk = repo.revwalk().map_err(|e| format!("Failed to create revwalk: {}", e))?;
    revwalk.set_sorting(Sort::TIME).map_err(|e| format!("Failed to set sorting: {}", e))?;
    revwalk.push_head().map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let all_oids: Vec<Oid> = revwalk.filter_map(|r| r.ok()).collect();
    let total_count = all_oids.len();

    let commits: Vec<CommitInfo> = all_oids
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|oid| {
            let commit = repo.find_commit(oid).ok()?;
            let author = commit.author();
            let parent_ids: Vec<String> = (0..commit.parent_count())
                .filter_map(|i| commit.parent_id(i).ok())
                .map(|id| id.to_string())
                .collect();
            Some(CommitInfo {
                hash: oid.to_string(),
                short_hash: oid.to_string()[..7].to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: author.name().unwrap_or("Unknown").to_string(),
                email: author.email().unwrap_or("").to_string(),
                date: commit.time().seconds(),
                parent_ids,
            })
        })
        .collect();

    let has_more = offset + commits.len() < total_count;

    Ok(CommitPage {
        commits,
        has_more,
        total_count,
    })
}

#[tauri::command]
fn search_commits(repo_path: String, query: String, offset: usize, limit: usize) -> Result<CommitPage, String> {
    let repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let mut revwalk = repo.revwalk().map_err(|e| format!("Failed to create revwalk: {}", e))?;
    revwalk.set_sorting(Sort::TIME).map_err(|e| format!("Failed to set sorting: {}", e))?;
    revwalk.push_head().map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let query_lower = query.to_lowercase();

    // Filter commits matching the query
    let matching_commits: Vec<CommitInfo> = revwalk
        .filter_map(|r| r.ok())
        .filter_map(|oid| {
            let commit = repo.find_commit(oid).ok()?;
            let author = commit.author();
            let hash_str = oid.to_string();
            let message = commit.message().unwrap_or("").to_string();
            let author_name = author.name().unwrap_or("Unknown").to_string();
            let email = author.email().unwrap_or("").to_string();

            // Match against hash, message, or author
            let matches = hash_str.to_lowercase().starts_with(&query_lower)
                || message.to_lowercase().contains(&query_lower)
                || author_name.to_lowercase().contains(&query_lower)
                || email.to_lowercase().contains(&query_lower);

            if matches {
                let parent_ids: Vec<String> = (0..commit.parent_count())
                    .filter_map(|i| commit.parent_id(i).ok())
                    .map(|id| id.to_string())
                    .collect();
                Some(CommitInfo {
                    hash: hash_str,
                    short_hash: oid.to_string()[..7].to_string(),
                    message,
                    author: author_name,
                    email,
                    date: commit.time().seconds(),
                    parent_ids,
                })
            } else {
                None
            }
        })
        .collect();

    let total_count = matching_commits.len();
    let commits: Vec<CommitInfo> = matching_commits
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let has_more = offset + commits.len() < total_count;

    Ok(CommitPage {
        commits,
        has_more,
        total_count,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewriteResult {
    pub new_hash: String,
    pub updated_branches: Vec<String>,
}

#[tauri::command]
fn edit_commit_message(
    repo_path: String,
    commit_hash: String,
    new_message: String,
    auto_stash: bool,
) -> Result<RewriteResult, String> {
    let mut repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    // Check for uncommitted changes
    let did_stash = if auto_stash {
        stash_changes(&mut repo)?
    } else {
        check_working_directory_clean(&repo)?;
        false
    };

    let result = rewrite_commit_message(&repo, &commit_hash, &new_message);

    // Restore stashed changes if we stashed them
    if did_stash {
        let _ = unstash_changes(&mut repo);
    }

    result
}

fn rewrite_commit_message(
    repo: &Repository,
    commit_hash: &str,
    new_message: &str,
) -> Result<RewriteResult, String> {
    let target_oid = Oid::from_str(commit_hash)
        .map_err(|e| format!("Invalid commit hash: {}", e))?;

    let target_commit = repo.find_commit(target_oid)
        .map_err(|e| format!("Failed to find target commit: {}", e))?;

    let base_parent_oid = if target_commit.parent_count() > 0 {
        target_commit.parent_id(0).map_err(|e| format!("Failed to get parent: {}", e))?
    } else {
        return Err("Cannot rewrite the initial commit".to_string());
    };

    // Get current branch name
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let current_branch = head.shorthand().unwrap_or("HEAD").to_string();
    let head_oid = head.target().ok_or("HEAD has no target")?;

    // Verify commit is in current branch
    if !repo.graph_descendant_of(head_oid, target_oid).unwrap_or(false) && head_oid != target_oid {
        return Err("Commit not found in current branch history".to_string());
    }

    // Collect commits from HEAD to target
    let mut commits_to_rewrite: Vec<Oid> = Vec::new();
    let mut revwalk = repo.revwalk().map_err(|e| format!("Failed to create revwalk: {}", e))?;
    revwalk.push_head().map_err(|e| format!("Failed to push HEAD: {}", e))?;

    for oid in revwalk {
        let oid = oid.map_err(|e| format!("Failed to walk: {}", e))?;
        commits_to_rewrite.push(oid);
        if oid == target_oid {
            break;
        }
    }

    commits_to_rewrite.reverse();

    // Build new commit chain
    let mut current_parent_oid = base_parent_oid;

    for oid in &commits_to_rewrite {
        let old_commit = repo.find_commit(*oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let message = if *oid == target_oid {
            new_message.to_string()
        } else {
            old_commit.message().unwrap_or("").to_string()
        };

        let tree = old_commit.tree()
            .map_err(|e| format!("Failed to get tree: {}", e))?;

        let parent = repo.find_commit(current_parent_oid)
            .map_err(|e| format!("Failed to find parent: {}", e))?;

        let new_oid = repo.commit(
            None,
            &old_commit.author(),
            &old_commit.committer(),
            &message,
            &tree,
            &[&parent],
        ).map_err(|e| format!("Failed to create commit: {}", e))?;

        current_parent_oid = new_oid;
    }

    let new_head_oid = current_parent_oid;

    // Update HEAD
    let new_head = repo.find_commit(new_head_oid)
        .map_err(|e| format!("Failed to find new HEAD: {}", e))?;
    repo.reset(new_head.as_object(), ResetType::Hard, None)
        .map_err(|e| format!("Failed to reset HEAD: {}", e))?;

    Ok(RewriteResult {
        new_hash: new_head_oid.to_string(),
        updated_branches: vec![current_branch],
    })
}

#[tauri::command]
fn squash_commits(
    repo_path: String,
    commit_hashes: Vec<String>,
    new_message: String,
    auto_stash: bool,
) -> Result<RewriteResult, String> {
    if commit_hashes.is_empty() {
        return Err("No commits selected".to_string());
    }

    if commit_hashes.len() == 1 {
        return Err("Need at least 2 commits to squash".to_string());
    }

    let mut repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    // Check for uncommitted changes
    let did_stash = if auto_stash {
        stash_changes(&mut repo)?
    } else {
        check_working_directory_clean(&repo)?;
        false
    };

    let result = squash_commits_impl(&repo, &commit_hashes, &new_message);

    // Restore stashed changes if we stashed them
    if did_stash {
        let _ = unstash_changes(&mut repo);
    }

    result
}

fn squash_commits_impl(
    repo: &Repository,
    commit_hashes: &[String],
    new_message: &str,
) -> Result<RewriteResult, String> {
    // Convert hashes to Oids and get commit data
    let mut commits_to_squash: Vec<(Oid, i64)> = commit_hashes
        .iter()
        .filter_map(|h| {
            let oid = Oid::from_str(h).ok()?;
            let commit = repo.find_commit(oid).ok()?;
            Some((oid, commit.time().seconds()))
        })
        .collect();

    if commits_to_squash.len() < 2 {
        return Err("Need at least 2 valid commits to squash".to_string());
    }

    // Sort by time, oldest first
    commits_to_squash.sort_by_key(|(_, time)| *time);

    let squash_oids: std::collections::HashSet<Oid> = commits_to_squash.iter().map(|(oid, _)| *oid).collect();
    let oldest_oid = commits_to_squash.first().unwrap().0;
    let newest_oid = commits_to_squash.last().unwrap().0;

    // Get the parent of the oldest commit to squash
    let oldest_commit = repo.find_commit(oldest_oid)
        .map_err(|e| format!("Failed to find oldest commit: {}", e))?;

    let base_parent_oid = if oldest_commit.parent_count() > 0 {
        oldest_commit.parent_id(0).map_err(|e| format!("Failed to get parent: {}", e))?
    } else {
        return Err("Cannot squash the initial commit".to_string());
    };

    // Get current branch info
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let current_branch = head.shorthand().unwrap_or("HEAD").to_string();

    // Walk from HEAD to find all commits we need to replay
    let mut all_commits: Vec<Oid> = Vec::new();
    let mut revwalk = repo.revwalk().map_err(|e| format!("Failed to create revwalk: {}", e))?;
    revwalk.push_head().map_err(|e| format!("Failed to push HEAD: {}", e))?;

    for oid in revwalk {
        let oid = oid.map_err(|e| format!("Failed to walk: {}", e))?;
        all_commits.push(oid);
        if oid == oldest_oid {
            break;
        }
    }

    if !all_commits.contains(&oldest_oid) {
        return Err("Commits not found in current branch history".to_string());
    }

    // Reverse to go from oldest to newest
    all_commits.reverse();

    // Rewrite commits
    let mut current_parent_oid = base_parent_oid;
    let mut squash_tree: Option<git2::Tree> = None;
    let mut first_squash_author: Option<git2::Signature> = None;
    let mut first_squash_committer: Option<git2::Signature> = None;

    for oid in &all_commits {
        let old_commit = repo.find_commit(*oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        if squash_oids.contains(oid) {
            // This commit is part of the squash - we'll use the tree from the newest one
            squash_tree = Some(old_commit.tree()
                .map_err(|e| format!("Failed to get tree: {}", e))?);

            // Keep the author/committer from the first (oldest) commit in squash
            if first_squash_author.is_none() {
                first_squash_author = Some(old_commit.author().to_owned());
                first_squash_committer = Some(old_commit.committer().to_owned());
            }

            // Check if this is the last commit to squash (newest)
            if *oid == newest_oid {
                // Create the squashed commit
                let parent = repo.find_commit(current_parent_oid)
                    .map_err(|e| format!("Failed to find parent: {}", e))?;

                let new_oid = repo.commit(
                    None,
                    first_squash_author.as_ref().unwrap(),
                    first_squash_committer.as_ref().unwrap(),
                    new_message,
                    squash_tree.as_ref().unwrap(),
                    &[&parent],
                ).map_err(|e| format!("Failed to create squashed commit: {}", e))?;

                current_parent_oid = new_oid;
            }
        } else {
            // Regular commit - just replay it
            let tree = old_commit.tree()
                .map_err(|e| format!("Failed to get tree: {}", e))?;

            let parent = repo.find_commit(current_parent_oid)
                .map_err(|e| format!("Failed to find parent: {}", e))?;

            let new_oid = repo.commit(
                None,
                &old_commit.author(),
                &old_commit.committer(),
                old_commit.message().unwrap_or(""),
                &tree,
                &[&parent],
            ).map_err(|e| format!("Failed to create commit: {}", e))?;

            current_parent_oid = new_oid;
        }
    }

    let new_head_oid = current_parent_oid;

    // Update HEAD
    let new_head = repo.find_commit(new_head_oid)
        .map_err(|e| format!("Failed to find new HEAD: {}", e))?;
    repo.reset(new_head.as_object(), ResetType::Hard, None)
        .map_err(|e| format!("Failed to reset HEAD: {}", e))?;

    Ok(RewriteResult {
        new_hash: new_head_oid.to_string(),
        updated_branches: vec![current_branch],
    })
}

#[tauri::command]
fn validate_repo(path: String) -> Result<bool, String> {
    match Repository::open(&path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
fn get_repo_info(repo_path: String) -> Result<RepoInfo, String> {
    let repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let branch_name = head.shorthand().unwrap_or("HEAD").to_string();

    let path = PathBuf::from(&repo_path);
    let repo_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| repo_path.clone());

    Ok(RepoInfo {
        name: repo_name,
        path: repo_path,
        branch: branch_name,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: String,
    pub content: String,
    pub old_line_no: Option<usize>,
    pub new_line_no: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub old_path: Option<String>,
    pub insertions: usize,
    pub deletions: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffStats {
    pub insertions: usize,
    pub deletions: usize,
    pub files_changed: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitDiff {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: i64,
    pub stats: DiffStats,
    pub files: Vec<FileDiff>,
}

#[tauri::command]
fn get_branches(repo_path: String) -> Result<Vec<BranchInfo>, String> {
    let repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let head = repo.head().ok();
    let current_branch = head.as_ref()
        .and_then(|h| h.shorthand())
        .unwrap_or("");

    let branches = repo.branches(Some(BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    let mut result: Vec<BranchInfo> = branches
        .filter_map(|b| {
            let (branch, _) = b.ok()?;
            let name = branch.name().ok()??.to_string();
            Some(BranchInfo {
                is_current: name == current_branch,
                name,
            })
        })
        .collect();

    // Sort: current branch first, then alphabetically
    result.sort_by(|a, b| {
        if a.is_current && !b.is_current {
            std::cmp::Ordering::Less
        } else if !a.is_current && b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(result)
}

#[tauri::command]
fn get_commit_diff(repo_path: String, commit_hash: String) -> Result<CommitDiff, String> {
    let repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let oid = Oid::from_str(&commit_hash)
        .map_err(|e| format!("Invalid commit hash: {}", e))?;

    let commit = repo.find_commit(oid)
        .map_err(|e| format!("Failed to find commit: {}", e))?;

    let commit_tree = commit.tree()
        .map_err(|e| format!("Failed to get commit tree: {}", e))?;

    // Get parent tree (or empty tree for initial commit)
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)
            .map_err(|e| format!("Failed to get parent: {}", e))?
            .tree()
            .map_err(|e| format!("Failed to get parent tree: {}", e))?)
    } else {
        None
    };

    // Create diff
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
        .map_err(|e| format!("Failed to create diff: {}", e))?;

    let diff_stats = diff.stats()
        .map_err(|e| format!("Failed to get diff stats: {}", e))?;

    let mut files: Vec<FileDiff> = Vec::new();

    // Iterate through deltas (files)
    for delta_idx in 0..diff.deltas().len() {
        let delta = diff.get_delta(delta_idx).unwrap();

        let status = match delta.status() {
            git2::Delta::Added => "Added",
            git2::Delta::Deleted => "Deleted",
            git2::Delta::Modified => "Modified",
            git2::Delta::Renamed => "Renamed",
            git2::Delta::Copied => "Copied",
            _ => "Unknown",
        }.to_string();

        let new_file = delta.new_file();
        let old_file = delta.old_file();

        let path = new_file.path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let old_path = if delta.status() == git2::Delta::Renamed {
            old_file.path().map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let mut lines: Vec<DiffLine> = Vec::new();
        let mut insertions: usize = 0;
        let mut deletions: usize = 0;

        // Get patch for this file
        if let Ok(patch) = git2::Patch::from_diff(&diff, delta_idx) {
            if let Some(patch) = patch {
                for hunk_idx in 0..patch.num_hunks() {
                    if let Ok((hunk, _)) = patch.hunk(hunk_idx) {
                        // Add hunk header
                        let header = format!(
                            "@@ -{},{} +{},{} @@",
                            hunk.old_start(),
                            hunk.old_lines(),
                            hunk.new_start(),
                            hunk.new_lines()
                        );
                        lines.push(DiffLine {
                            line_type: "header".to_string(),
                            content: header,
                            old_line_no: None,
                            new_line_no: None,
                        });

                        // Get lines in this hunk
                        let num_lines = patch.num_lines_in_hunk(hunk_idx).unwrap_or(0);
                        for line_idx in 0..num_lines {
                            if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
                                let (line_type, old_no, new_no) = match line.origin() {
                                    '+' => {
                                        insertions += 1;
                                        ("add".to_string(), None, line.new_lineno().map(|n| n as usize))
                                    }
                                    '-' => {
                                        deletions += 1;
                                        ("delete".to_string(), line.old_lineno().map(|n| n as usize), None)
                                    }
                                    ' ' => {
                                        ("context".to_string(),
                                         line.old_lineno().map(|n| n as usize),
                                         line.new_lineno().map(|n| n as usize))
                                    }
                                    _ => continue,
                                };

                                let content = String::from_utf8_lossy(line.content()).to_string();
                                lines.push(DiffLine {
                                    line_type,
                                    content,
                                    old_line_no: old_no,
                                    new_line_no: new_no,
                                });
                            }
                        }
                    }
                }
            }
        }

        files.push(FileDiff {
            path,
            status,
            old_path,
            insertions,
            deletions,
            lines,
        });
    }

    let author = commit.author();

    Ok(CommitDiff {
        hash: commit_hash,
        message: commit.message().unwrap_or("").to_string(),
        author: author.name().unwrap_or("Unknown").to_string(),
        date: commit.time().seconds(),
        stats: DiffStats {
            insertions: diff_stats.insertions(),
            deletions: diff_stats.deletions(),
            files_changed: diff_stats.files_changed(),
        },
        files,
    })
}

#[tauri::command]
fn switch_branch(repo_path: String, branch_name: String, auto_stash: bool) -> Result<(), String> {
    let mut repo = Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    // Handle uncommitted changes
    let did_stash = if auto_stash {
        stash_changes(&mut repo)?
    } else {
        check_working_directory_clean(&repo)?;
        false
    };

    // Verify the branch exists
    {
        let _branch = repo.find_branch(&branch_name, BranchType::Local)
            .map_err(|e| format!("Failed to find branch '{}': {}", branch_name, e))?;
    }

    // Checkout the branch
    repo.set_head(&format!("refs/heads/{}", branch_name))
        .map_err(|e| format!("Failed to set HEAD: {}", e))?;

    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .map_err(|e| format!("Failed to checkout: {}", e))?;

    // Restore stashed changes
    if did_stash {
        let _ = unstash_changes(&mut repo);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::UnderWindowBackground, None, None)
                .expect("Failed to apply vibrancy");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_commits,
            search_commits,
            edit_commit_message,
            squash_commits,
            validate_repo,
            get_repo_info,
            get_branches,
            switch_branch,
            get_commit_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
