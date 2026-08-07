use std::fmt;
use std::fs;
use std::path::Path;
use std::process::Command;

/// 版本号 bump 类型，按优先级从高到低排列
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BumpLevel {
    /// 不需要 bump
    None,
    /// 修复、性能优化、重构 → patch bump
    Patch,
    /// 新功能 → minor bump
    Minor,
    /// 破坏性变更 → major bump
    Major,
}

/// Conventional Commit 的分类结果
#[derive(Debug, Clone)]
struct ParsedCommit {
    /// 原始 commit 消息
    message: String,
    /// CHANGELOG 中的分组类别
    category: CommitCategory,
    /// 版本 bump 级别
    bump: BumpLevel,
}

/// CHANGELOG 分组类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitCategory {
    Added,
    Fixed,
    Changed,
    Other,
    /// 不计入 CHANGELOG（chore/docs/ci 等）
    Skip,
}

impl fmt::Display for CommitCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitCategory::Added => write!(f, "Added"),
            CommitCategory::Fixed => write!(f, "Fixed"),
            CommitCategory::Changed => write!(f, "Changed"),
            CommitCategory::Other => write!(f, "Other"),
            CommitCategory::Skip => write!(f, "Skip"),
        }
    }
}

/// 语义版本号
#[derive(Debug, Clone)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

impl SemVer {
    /// 从字符串解析版本号（如 "0.1.5" 或 "v0.1.5"）
    fn parse(s: &str) -> Option<SemVer> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(SemVer {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    /// 根据 bump 级别计算下一个版本
    fn bump(&self, level: BumpLevel) -> SemVer {
        match level {
            BumpLevel::Major => SemVer {
                major: self.major + 1,
                minor: 0,
                patch: 0,
            },
            BumpLevel::Minor => SemVer {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
            },
            BumpLevel::Patch => SemVer {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
            },
            BumpLevel::None => self.clone(),
        }
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 解析单条 commit 消息，返回分类和 bump 级别
fn parse_commit(message: &str) -> ParsedCommit {
    let msg = message.trim();

    // 检测 breaking change
    if msg.contains("BREAKING CHANGE") || msg.contains("BREAKING-CHANGE") {
        return ParsedCommit {
            message: msg.to_string(),
            category: CommitCategory::Changed,
            bump: BumpLevel::Major,
        };
    }

    // 提取前缀（支持 scope），检测 ! 标记
    let has_bang = msg.contains("!:");
    if has_bang {
        return ParsedCommit {
            message: msg.to_string(),
            category: CommitCategory::Changed,
            bump: BumpLevel::Major,
        };
    }

    // 按 conventional commit 前缀分类
    let prefix = extract_prefix(msg);
    match prefix {
        Some("feat") => ParsedCommit {
            message: extract_description(msg),
            category: CommitCategory::Added,
            bump: BumpLevel::Minor,
        },
        Some("fix") => ParsedCommit {
            message: extract_description(msg),
            category: CommitCategory::Fixed,
            bump: BumpLevel::Patch,
        },
        Some("perf") => ParsedCommit {
            message: extract_description(msg),
            category: CommitCategory::Fixed,
            bump: BumpLevel::Patch,
        },
        Some("refactor") => ParsedCommit {
            message: extract_description(msg),
            category: CommitCategory::Changed,
            bump: BumpLevel::Patch,
        },
        Some("chore" | "docs" | "ci" | "style" | "test" | "build") => ParsedCommit {
            message: extract_description(msg),
            category: CommitCategory::Skip,
            bump: BumpLevel::None,
        },
        _ => ParsedCommit {
            message: msg.to_string(),
            category: CommitCategory::Other,
            bump: BumpLevel::Patch,
        },
    }
}

/// 提取 conventional commit 前缀（如 "feat"、"fix(scope)" → "fix"）
fn extract_prefix(msg: &str) -> Option<&str> {
    // 查找第一个 `:` 或 `(` 的位置
    let colon_pos = msg.find(':')?;
    let prefix_part = &msg[..colon_pos];
    // 去掉可能的 scope 部分，如 "fix(core)" → "fix"
    let prefix = if let Some(paren_pos) = prefix_part.find('(') {
        &prefix_part[..paren_pos]
    } else {
        prefix_part
    };
    // 去掉可能的 ! 标记
    let prefix = prefix.strip_suffix('!').unwrap_or(prefix);
    Some(prefix.trim())
}

/// 从 commit 消息中提取描述部分（去掉前缀）
fn extract_description(msg: &str) -> String {
    if let Some(pos) = msg.find(':') {
        msg[pos + 1..].trim().to_string()
    } else {
        msg.to_string()
    }
}

/// 运行 git 命令并返回 stdout
fn git_cmd(args: &[&str], workspace_root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace_root)
        .output()
        .map_err(|e| format!("无法执行 git 命令: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git {} 失败: {}", args.join(" "), stderr.trim()))
    }
}

/// 获取最新的 v* tag
fn get_latest_tag(workspace_root: &Path) -> Option<String> {
    git_cmd(
        &["describe", "--tags", "--abbrev=0", "--match", "v*"],
        workspace_root,
    )
    .ok()
}

/// 获取从指定 tag（或从项目开始）到 HEAD 的所有 commit 消息
fn get_commits_since(tag: Option<&str>, workspace_root: &Path) -> Vec<String> {
    let range = match tag {
        Some(t) => format!("{}..HEAD", t),
        None => "HEAD".to_string(),
    };

    let args = vec!["log", &range, "--format=%s"];
    let output = git_cmd(&args, workspace_root).unwrap_or_default();

    if output.is_empty() {
        return Vec::new();
    }

    output.lines().map(|s| s.to_string()).collect()
}

/// 更新根 Cargo.toml 中的所有版本号
///
/// 使用逐行字符串替换，避免 toml_edit 解析多行内联表的兼容性问题。
/// 更新策略：
/// - `[workspace.package]` 区段中的 `version = "x.y.z"` → 直接替换
/// - `[workspace.dependencies]` 区段中同时包含 `path =` 和 `version =` 的行 → 替换 version
fn update_cargo_toml(
    workspace_root: &Path,
    new_version: &str,
    old_version: &str,
) -> Result<(), String> {
    let cargo_path = workspace_root.join("Cargo.toml");
    let content =
        fs::read_to_string(&cargo_path).map_err(|e| format!("无法读取 Cargo.toml: {}", e))?;

    let old_ver_quoted = format!("\"{}\"", old_version);
    let new_ver_quoted = format!("\"{}\"", new_version);

    let mut result = Vec::new();
    let mut in_workspace_package = false;
    let mut in_workspace_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 追踪当前所在的 TOML 区段
        if trimmed.starts_with('[') {
            in_workspace_package = trimmed == "[workspace.package]";
            in_workspace_deps = trimmed == "[workspace.dependencies]";
        }

        if in_workspace_package
            && trimmed.starts_with("version")
            && trimmed.contains(&old_ver_quoted)
        {
            // [workspace.package] 中的 version 行
            result.push(line.replace(&old_ver_quoted, &new_ver_quoted));
        } else if in_workspace_deps && trimmed.contains("path") && trimmed.contains(&old_ver_quoted)
        {
            // [workspace.dependencies] 中带 path 的内部依赖行
            result.push(line.replace(&old_ver_quoted, &new_ver_quoted));
        } else {
            result.push(line.to_string());
        }
    }

    // 保持原文件的换行风格
    let mut output = result.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }

    fs::write(&cargo_path, output).map_err(|e| format!("无法写入 Cargo.toml: {}", e))?;

    Ok(())
}

/// 生成 CHANGELOG 条目并更新文件
fn update_changelog(
    workspace_root: &Path,
    version: &str,
    commits: &[ParsedCommit],
    repo_url: &str,
) -> Result<(), String> {
    let changelog_path = workspace_root.join("CHANGELOG.md");

    // 按类别分组
    let added: Vec<_> = commits
        .iter()
        .filter(|c| c.category == CommitCategory::Added)
        .collect();
    let fixed: Vec<_> = commits
        .iter()
        .filter(|c| c.category == CommitCategory::Fixed)
        .collect();
    let changed: Vec<_> = commits
        .iter()
        .filter(|c| c.category == CommitCategory::Changed)
        .collect();
    let other: Vec<_> = commits
        .iter()
        .filter(|c| c.category == CommitCategory::Other)
        .collect();

    // 构建新条目
    let today = get_today_date();
    let mut entry = format!("\n## [{version}]({repo_url}/releases/tag/v{version}) - {today}\n");

    if !added.is_empty() {
        entry.push_str("\n### Added\n\n");
        for c in &added {
            entry.push_str(&format!("- {}\n", c.message));
        }
    }
    if !fixed.is_empty() {
        entry.push_str("\n### Fixed\n\n");
        for c in &fixed {
            entry.push_str(&format!("- {}\n", c.message));
        }
    }
    if !changed.is_empty() {
        entry.push_str("\n### Changed\n\n");
        for c in &changed {
            entry.push_str(&format!("- {}\n", c.message));
        }
    }
    if !other.is_empty() {
        entry.push_str("\n### Other\n\n");
        for c in &other {
            entry.push_str(&format!("- {}\n", c.message));
        }
    }

    // 读取现有 CHANGELOG 并插入新条目
    let existing = if changelog_path.exists() {
        fs::read_to_string(&changelog_path).map_err(|e| format!("无法读取 CHANGELOG.md: {}", e))?
    } else {
        "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n\
             The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\n\
             and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n\
             ## [Unreleased]\n".to_string()
    };

    // 在 ## [Unreleased] 之后插入新版本条目
    let new_content = if let Some(pos) = existing.find("## [Unreleased]") {
        let after_unreleased = pos + "## [Unreleased]".len();
        // 跳过 [Unreleased] 后面可能的换行
        let insert_pos = existing[after_unreleased..]
            .find('\n')
            .map(|p| after_unreleased + p)
            .unwrap_or(after_unreleased);
        format!(
            "{}{}{}",
            &existing[..insert_pos],
            entry,
            &existing[insert_pos..]
        )
    } else {
        // 没找到 [Unreleased] 标记，直接追加到文件开头后面
        format!("{}\n{}", existing.trim(), entry)
    };

    fs::write(&changelog_path, new_content).map_err(|e| format!("无法写入 CHANGELOG.md: {}", e))?;

    Ok(())
}

/// 获取今天的日期（YYYY-MM-DD 格式）
fn get_today_date() -> String {
    // 通过 git 获取当前日期，避免引入额外依赖
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=short"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let date = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if date.len() == 10 {
                return date;
            }
            // 回退：用系统时间
            fallback_date()
        }
        _ => fallback_date(),
    }
}

/// 回退日期获取方式
fn fallback_date() -> String {
    // Windows 和 Unix 通用
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    // 如果 git 也不可用，返回占位符
    match output {
        Ok(_) => {
            // 使用 PowerShell 获取日期
            let ps = Command::new("powershell")
                .args(["-Command", "Get-Date -Format yyyy-MM-dd"])
                .output();
            match ps {
                Ok(o) if o.status.success() => {
                    String::from_utf8_lossy(&o.stdout).trim().to_string()
                }
                _ => "unknown".to_string(),
            }
        }
        _ => "unknown".to_string(),
    }
}

/// 从 Cargo.toml 中提取 repository URL
fn get_repo_url(workspace_root: &Path) -> String {
    let cargo_path = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path).unwrap_or_default();

    // 简单字符串搜索提取 repository 值，避免 TOML 解析
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("repository") && trimmed.contains('=') {
            if let Some(val) = trimmed.split('=').nth(1) {
                let url = val.trim().trim_matches('"').trim_end_matches(".git");
                if url.starts_with("http") {
                    return url.to_string();
                }
            }
        }
    }

    "https://github.com/tierlabx/widget-rs".to_string()
}

/// 执行版本发布流程
pub fn run_release(workspace_root: &Path, manual_version: Option<&str>, dry_run: bool) {
    println!("正在分析提交历史...\n");

    // 1. 获取最新 tag 和当前版本
    let latest_tag = get_latest_tag(workspace_root);
    let current_version = match &latest_tag {
        Some(tag) => SemVer::parse(tag).unwrap_or_else(|| {
            eprintln!("无法解析 tag '{}' 为语义版本号", tag);
            std::process::exit(1);
        }),
        None => {
            println!("未找到任何 v* tag，将从 0.0.0 开始");
            SemVer {
                major: 0,
                minor: 0,
                patch: 0,
            }
        }
    };

    println!("当前版本: v{}", current_version);
    if let Some(tag) = &latest_tag {
        println!("最新 tag:  {}", tag);
    }

    // 2. 获取 commits
    let raw_commits = get_commits_since(latest_tag.as_deref(), workspace_root);
    if raw_commits.is_empty() {
        println!("\n自上次 tag 以来没有新的提交，无需发布。");
        return;
    }

    println!("发现 {} 条新提交\n", raw_commits.len());

    // 3. 解析 commits
    let parsed: Vec<ParsedCommit> = raw_commits.iter().map(|m| parse_commit(m)).collect();

    // 4. 计算最高 bump 级别
    let max_bump = parsed
        .iter()
        .map(|c| c.bump)
        .max()
        .unwrap_or(BumpLevel::None);

    if max_bump == BumpLevel::None && manual_version.is_none() {
        println!("所有提交均为非功能性变更 (chore/docs/ci 等)，无需发布。");
        println!("如需强制发布，请使用 --version 手动指定版本号。");
        return;
    }

    // 5. 确定新版本号
    let new_version = match manual_version {
        Some(v) => SemVer::parse(v).unwrap_or_else(|| {
            eprintln!("无效的版本号格式: '{}'，期望格式: x.y.z", v);
            std::process::exit(1);
        }),
        None => current_version.bump(max_bump),
    };

    println!("下一个版本: v{}", new_version);

    // 显示 commit 分类摘要
    let changelog_commits: Vec<_> = parsed
        .iter()
        .filter(|c| c.category != CommitCategory::Skip)
        .collect();

    if !changelog_commits.is_empty() {
        println!("\nCHANGELOG 条目:");
        for c in &changelog_commits {
            println!("  [{}] {}", c.category, c.message);
        }
    }

    // dry-run 模式到此结束
    if dry_run {
        println!("\n(dry-run 模式，未做任何修改)");
        return;
    }

    let version_str = new_version.to_string();

    // 6. 更新 Cargo.toml
    println!("\n正在更新 Cargo.toml...");
    if let Err(e) = update_cargo_toml(workspace_root, &version_str, &current_version.to_string()) {
        eprintln!("更新 Cargo.toml 失败: {}", e);
        std::process::exit(1);
    }
    println!("Cargo.toml 已更新");

    // 7. 更新 CHANGELOG
    println!("正在更新 CHANGELOG.md...");
    let repo_url = get_repo_url(workspace_root);
    let changelog_entries: Vec<_> = parsed
        .iter()
        .filter(|c| c.category != CommitCategory::Skip)
        .cloned()
        .collect();
    if let Err(e) = update_changelog(workspace_root, &version_str, &changelog_entries, &repo_url) {
        eprintln!("更新 CHANGELOG.md 失败: {}", e);
        std::process::exit(1);
    }
    println!("CHANGELOG.md 已更新");

    // 8. Git commit + tag
    println!("\n正在提交变更...");
    let commit_msg = format!("chore: release v{}", version_str);

    if let Err(e) = git_cmd(&["add", "Cargo.toml", "CHANGELOG.md"], workspace_root) {
        eprintln!("git add 失败: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = git_cmd(&["commit", "-m", &commit_msg], workspace_root) {
        eprintln!("git commit 失败: {}", e);
        std::process::exit(1);
    }

    let tag_name = format!("v{}", version_str);
    if let Err(e) = git_cmd(&["tag", &tag_name], workspace_root) {
        eprintln!("git tag 失败: {}", e);
        std::process::exit(1);
    }
    println!("已创建 tag: {}", tag_name);

    // 9. 自动推送
    println!("\n正在推送到远程仓库...");
    if let Err(e) = git_cmd(&["push"], workspace_root) {
        eprintln!("git push 失败: {}", e);
        eprintln!("请手动执行: git push && git push --tags");
        std::process::exit(1);
    }
    if let Err(e) = git_cmd(&["push", "--tags"], workspace_root) {
        eprintln!("git push --tags 失败: {}", e);
        eprintln!("请手动执行: git push --tags");
        std::process::exit(1);
    }

    println!("\n发布完成! v{} 已推送到远程仓库。", version_str);
    println!("GitHub Actions 将自动触发打包流程。");
}
