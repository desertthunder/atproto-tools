use atp_tools_bsky::{
    AuthorFeedLinkOptions, ExternalLinkPost, FollowLastPost, FollowSyncOptions, FollowsOptions, FollowsProgress,
    FollowsSort, FollowsSortDirection, FollowsSortField, fetch_follow_sync_with_progress,
    fetch_follows_external_link_posts_with_progress, fetch_follows_report_with_progress,
};
use atp_tools_core::{
    ActorRepoInfo, AppConfig, AtprotoClient, LexiconSyncSpec, ParallelProgress, generate_serde_models, sync_lexicons,
};
use atp_tools_margin::{SourceNotesDocument, export_notes, export_source_notes};
use clap::Parser;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod args;
mod echo;

use args::{BskyCommands, Cli, Commands, ConfigCommands, FollowsSortFieldArg, LexiconCommands, MarginCommands, Tool};

impl From<FollowsSortFieldArg> for FollowsSortField {
    fn from(field: FollowsSortFieldArg) -> Self {
        match field {
            FollowsSortFieldArg::Handle => Self::Handle,
            FollowsSortFieldArg::Did => Self::Did,
            FollowsSortFieldArg::ProfileUrl => Self::ProfileUrl,
            FollowsSortFieldArg::LastPostAt => Self::LastPostAt,
            FollowsSortFieldArg::LastPostRkey => Self::LastPostRkey,
            FollowsSortFieldArg::LastPostUrl => Self::LastPostUrl,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.clone();
    let mut config = AppConfig::load(config_path.clone())?;

    match cli.command {
        Commands::Info { actor, json } => {
            let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
            let client = AtprotoClient::new(config.services)?;
            let info = client.actor_repo_info(&actor).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                print_info(&info);
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::Set { field, value } => {
                config.set_field(&field, value)?;
                let path = config.save(config_path)?;
                echo::pair("config", path.display());
                echo::pair(&field, config.get_field(&field)?);
            }
        },
        Commands::Lexicons { command } => match command {
            LexiconCommands::Sync { tool, repo, commit, source_path, dest, files } => {
                let mut spec = default_lexicon_sync_spec(tool, commit);
                if let Some(repo) = repo {
                    spec.repo = repo;
                }
                if let Some(source_path) = source_path {
                    spec.source_path = source_path;
                }
                if let Some(dest) = dest {
                    spec.dest_dir = dest;
                }
                if !files.is_empty() {
                    spec.files = files;
                }
                let report = sync_lexicons(spec).await?;

                echo::pair("commit", report.commit);
                let written = report
                    .written
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>();
                echo::list("written", &written);
            }
            LexiconCommands::Generate { tool, input, output } => {
                let input = input.unwrap_or_else(|| default_lexicon_input(tool));
                let output = output.unwrap_or_else(|| default_generated_output(tool));
                let report = generate_serde_models(input, output)?;
                echo::pair("output", report.output.display());
                echo::list("structs", &report.structs);
            }
        },
        Commands::Margin { command } => match command {
            MarginCommands::Export { source, actor, output_dir } => {
                let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
                let client = AtprotoClient::new(config.services)?;

                let documents = if let Some(source) = source {
                    if let Some(document) = export_source_notes(&client, &actor, &source).await? {
                        vec![document]
                    } else {
                        Vec::new()
                    }
                } else {
                    export_notes(&client, &actor).await?
                };

                std::fs::create_dir_all(&output_dir)?;
                let written = write_margin_documents(&output_dir, &documents)?;
                echo::pair("documents", written.len());
                echo::list("written", &written);
            }
        },
        Commands::Bsky { command } => match command {
            BskyCommands::LinkDigest {
                actor,
                since,
                until,
                limit,
                min_score,
                min_shares,
                feed_limit,
                max_pages,
                refresh_follows,
            } => {
                let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
                let limit = limit.unwrap_or(config.link_digest.limit);
                let min_score = min_score.unwrap_or(config.link_digest.min_score);
                let min_shares = min_shares.unwrap_or(config.link_digest.min_shares);
                anyhow::ensure!(limit > 0, "limit must be greater than 0");
                anyhow::ensure!(min_score >= 0, "minimum score must be greater than or equal to 0");
                anyhow::ensure!(min_shares > 0, "minimum shares must be greater than 0");
                let client = AtprotoClient::new(config.services)?;
                let follows_options = FollowSyncOptions::default();
                let sync = fetch_follow_sync_with_progress(
                    &client,
                    &actor,
                    refresh_follows,
                    follows_options,
                    print_follows_progress,
                )
                .await?;
                let link_options = AuthorFeedLinkOptions { since, until, limit: feed_limit, max_pages };
                echo::status(format!("fetching author feeds for {} follows", sync.follows.len()));
                let links = fetch_follows_external_link_posts_with_progress(
                    client,
                    sync.follows.clone(),
                    link_options,
                    print_author_feed_progress,
                )
                .await?;
                echo::clear_status();
                print_link_digest_markdown(links, limit, min_score, min_shares);
            }
            BskyCommands::Follows { actor, limit, sort, asc, desc, sort_ascending, sort_descending, refresh, json } => {
                let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
                let client = AtprotoClient::new(config.services)?;
                let options = follows_options(limit, sort, asc, desc, sort_ascending, sort_descending);
                let report =
                    fetch_follows_report_with_progress(&client, &actor, refresh, options, print_follows_progress)
                        .await?;
                echo::clear_status();

                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    echo::pair("actor", &report.actor);
                    echo::pair("did", &report.actor_did);
                    echo::pair("follows", report.follows.len());
                    echo::pair("followsHash", &report.follows_hash);
                    echo::pair("cache", report.cache_path.display());
                    print_follows(&report.follows);
                }
            }
        },
    }

    Ok(())
}

fn follows_options(
    limit: Option<usize>, sort: Option<FollowsSortFieldArg>, asc: bool, desc: bool,
    sort_ascending: Option<FollowsSortFieldArg>, sort_descending: Option<FollowsSortFieldArg>,
) -> FollowsOptions {
    let (field, direction) = if let Some(field) = sort_ascending {
        (field, FollowsSortDirection::Asc)
    } else if let Some(field) = sort_descending {
        (field, FollowsSortDirection::Desc)
    } else {
        (
            sort.unwrap_or(FollowsSortFieldArg::LastPostAt),
            if desc && !asc { FollowsSortDirection::Desc } else { FollowsSortDirection::Asc },
        )
    };

    FollowsOptions { limit, sort: FollowsSort { field: field.into(), direction }, ..FollowsOptions::default() }
}

fn print_follows(follows: &[FollowLastPost]) {
    let rows = follows
        .iter()
        .map(|follow| {
            [
                follow.handle.as_str(),
                follow.did.as_str(),
                follow.profile_url.as_str(),
                follow.last_post_at.as_deref().unwrap_or(""),
                follow.last_post_url.as_deref().unwrap_or(""),
            ]
        })
        .collect::<Vec<_>>();
    let widths = column_widths(&["handle", "did", "profile", "lastPostAt"], &rows);

    println!(
        "{:<handle_width$}  {:<did_width$}  {:<profile_width$}  {:<last_post_at_width$}  lastPost",
        "handle",
        "did",
        "profile",
        "lastPostAt",
        handle_width = widths[0],
        did_width = widths[1],
        profile_width = widths[2],
        last_post_at_width = widths[3],
    );

    for row in rows {
        println!(
            "{:<handle_width$}  {:<did_width$}  {:<profile_width$}  {:<last_post_at_width$}  {}",
            row[0],
            row[1],
            row[2],
            row[3],
            row[4],
            handle_width = widths[0],
            did_width = widths[1],
            profile_width = widths[2],
            last_post_at_width = widths[3],
        );
    }
}

fn column_widths(headers: &[&str; 4], rows: &[[&str; 5]]) -> [usize; 4] {
    let mut widths = headers.map(str::len);

    for row in rows {
        for index in 0..widths.len() {
            widths[index] = widths[index].max(row[index].len());
        }
    }

    widths
}

fn print_follows_progress(progress: FollowsProgress) {
    match progress {
        FollowsProgress::ResolvingActor => echo::status("resolving actor"),
        FollowsProgress::CheckingCache { path } => echo::status(format!("checking cache {}", path.display())),
        FollowsProgress::CacheHit { path, count } => {
            echo::status(format!("cache hit: {count} follows from {}", path.display()));
        }
        FollowsProgress::CacheStale { path, generated_at_unix } => {
            echo::status(format!(
                "cache stale: {} generated at {generated_at_unix}",
                path.display()
            ));
        }
        FollowsProgress::ApplyingLimit { limit } => echo::status(format!("limiting to {limit} follows")),
        FollowsProgress::FetchingFollowsPage { page } => {
            echo::status(format!("fetching follows page {page}"));
        }
        FollowsProgress::FetchedFollowsPage { page: _, total } => {
            echo::status(format!("fetched {total} follows"));
        }
        FollowsProgress::FetchingLastPosts { completed, total } => {
            if completed == 1 || completed == total || completed % 25 == 0 {
                echo::progress("latest posts", completed, total);
            }
        }
        FollowsProgress::WritingCache { path } => echo::status(format!("writing cache {}", path.display())),
        FollowsProgress::WroteCache { path } => echo::status(format!("wrote cache {}", path.display())),
    }
}

fn print_author_feed_progress(progress: ParallelProgress) {
    if progress.completed == 1 || progress.completed == progress.total || progress.completed % 25 == 0 {
        echo::progress("author feeds", progress.completed, progress.total);
    }
}

#[derive(Debug, Clone)]
struct DigestLink {
    uri: String,
    title: String,
    description: String,
    bookmark_count: i64,
    repost_count: i64,
    like_count: i64,
    shares: Vec<ExternalLinkPost>,
}

impl DigestLink {
    fn score(&self) -> i64 {
        self.bookmark_count + self.repost_count + self.like_count
    }

    fn distinct_sharer_count(&self) -> usize {
        self.distinct_sharers().len()
    }

    fn distinct_sharers(&self) -> Vec<String> {
        let mut sharers = self
            .shares
            .iter()
            .map(|share| share.shared_by.clone())
            .collect::<Vec<_>>();
        sharers.sort();
        sharers.dedup();
        sharers
    }

    fn first_seen(&self) -> &str {
        self.shares
            .iter()
            .map(|share| share.shared_at.as_str())
            .min()
            .unwrap_or_default()
    }

    fn last_seen(&self) -> &str {
        self.shares
            .iter()
            .map(|share| share.shared_at.as_str())
            .max()
            .unwrap_or_default()
    }
}

fn print_link_digest_markdown(links: Vec<ExternalLinkPost>, limit: usize, min_score: i64, min_shares: usize) {
    let mut digest_links = aggregate_digest_links(links)
        .into_iter()
        .filter(|link| link.score() >= min_score)
        .filter(|link| link.distinct_sharer_count() >= min_shares)
        .collect::<Vec<_>>();
    digest_links.sort_by(|left, right| {
        right
            .distinct_sharer_count()
            .cmp(&left.distinct_sharer_count())
            .then_with(|| right.score().cmp(&left.score()))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.uri.cmp(&right.uri))
    });
    digest_links.truncate(limit);

    println!("---");
    println!("title: Social Links");
    println!("date: {}", current_utc_date());
    println!("---");
    println!();

    for link in &digest_links {
        let title = if link.title.is_empty() { link.uri.as_str() } else { link.title.as_str() };
        println!("## {}", markdown_plain_text(title));
        println!();
        println!("URL: <{}>", link.uri);
        println!();
        println!("Shared by:");
        println!();
        for sharer in link.distinct_sharers() {
            println!("- @{sharer}");
        }
        println!();
        println!("First seen: {}", digest_seen_time(link.first_seen()));
        println!("Last seen: {}", digest_seen_time(link.last_seen()));
        println!();
    }
}

fn aggregate_digest_links(links: Vec<ExternalLinkPost>) -> Vec<DigestLink> {
    let mut by_uri = BTreeMap::<String, DigestLink>::new();

    for link in links {
        let entry = by_uri.entry(link.external_uri.clone()).or_insert_with(|| DigestLink {
            uri: link.external_uri.clone(),
            title: link.title.clone(),
            description: link.description.clone(),
            bookmark_count: 0,
            repost_count: 0,
            like_count: 0,
            shares: Vec::new(),
        });

        let already_counted_post = entry.shares.iter().any(|share| share.post_uri == link.post_uri);
        let already_recorded_share = entry
            .shares
            .iter()
            .any(|share| share.post_uri == link.post_uri && share.shared_by_did == link.shared_by_did);
        if already_recorded_share {
            continue;
        }

        if entry.title.is_empty() && !link.title.is_empty() {
            entry.title = link.title.clone();
        }
        if entry.description.is_empty() && !link.description.is_empty() {
            entry.description = link.description.clone();
        }

        if !already_counted_post {
            entry.bookmark_count += link.bookmark_count;
            entry.repost_count += link.repost_count;
            entry.like_count += link.like_count;
        }
        entry.shares.push(link);
    }

    by_uri.into_values().collect()
}

fn digest_seen_time(value: &str) -> &str {
    value
        .get(11..16)
        .filter(|time| time.as_bytes().get(2) == Some(&b':'))
        .unwrap_or(value)
}

fn current_utc_date() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or_default();
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year, month as u32, day as u32)
}

fn markdown_plain_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn write_margin_documents(output_dir: &PathBuf, documents: &[SourceNotesDocument]) -> anyhow::Result<Vec<String>> {
    let mut written = Vec::with_capacity(documents.len());

    for document in documents {
        let path = output_dir.join(document.filename());
        std::fs::write(&path, document.to_markdown()?)?;
        written.push(path.display().to_string());
    }

    Ok(written)
}

fn default_lexicon_input(tool: Tool) -> PathBuf {
    match tool {
        Tool::Bsky => PathBuf::from("lexicons/app/bsky"),
        Tool::Margin => PathBuf::from("lexicons/at/margin"),
        Tool::Tangled => PathBuf::from("lexicons/sh/tangled"),
    }
}

fn default_generated_output(tool: Tool) -> PathBuf {
    match tool {
        Tool::Bsky => PathBuf::from("crates/bsky/src/generated.rs"),
        Tool::Margin => PathBuf::from("crates/margin/src/generated.rs"),
        Tool::Tangled => PathBuf::from("crates/tngl/src/generated.rs"),
    }
}

fn default_lexicon_sync_spec(tool: Tool, commit: String) -> LexiconSyncSpec {
    match tool {
        Tool::Bsky => LexiconSyncSpec {
            repo: "bluesky-social/atproto".to_string(),
            commit,
            source_path: "lexicons/app/bsky".to_string(),
            dest_dir: PathBuf::from("lexicons/app/bsky"),
            files: vec![
                "actor/defs.json".to_string(),
                "feed/defs.json".to_string(),
                "feed/getAuthorFeed.json".to_string(),
                "feed/post.json".to_string(),
                "graph/getFollows.json".to_string(),
            ],
            preserve_paths: true,
        },
        Tool::Margin => LexiconSyncSpec {
            repo: "margin-at/margin".to_string(),
            commit,
            source_path: "lexicons/at/margin".to_string(),
            dest_dir: PathBuf::from("lexicons/at/margin"),
            files: vec![
                "collection.json".to_string(),
                "collectionItem.json".to_string(),
                "like.json".to_string(),
                "note.json".to_string(),
            ],
            preserve_paths: false,
        },
        Tool::Tangled => LexiconSyncSpec {
            repo: "tangled.org/tangled.org/core".to_string(),
            commit,
            source_path: "lexicons".to_string(),
            dest_dir: PathBuf::from("lexicons/sh/tangled"),
            files: vec![
                "string/string.json".to_string(),
                "repo/repo.json".to_string(),
                "issue/issue.json".to_string(),
            ],
            preserve_paths: false,
        },
    }
}

fn print_info(info: &ActorRepoInfo) {
    echo::pair("did", &info.profile.did);
    echo::pair("handle", &info.profile.handle);

    echo::opt("displayName", info.profile.display_name.as_deref());
    echo::opt("description", info.profile.description.as_deref());
    echo::opt("pronouns", info.profile.pronouns.as_deref());
    echo::opt("website", info.profile.website.as_deref());
    echo::opt("avatar", info.profile.avatar.as_deref());
    echo::opt("banner", info.profile.banner.as_deref());
    echo::opt("followersCount", info.profile.followers_count);
    echo::opt("followsCount", info.profile.follows_count);
    echo::opt("postsCount", info.profile.posts_count);
    echo::opt("indexedAt", info.profile.indexed_at.as_deref());
    echo::opt("createdAt", info.profile.created_at.as_deref());

    echo::pair("repoHandle", &info.repo.handle);
    echo::pair("repoDid", &info.repo.did);
    echo::pair("handleIsCorrect", info.repo.handle_is_correct);
    echo::pair("collectionsCount", info.repo.collections.len());
    echo::list("collections", &info.repo.collections);
}
