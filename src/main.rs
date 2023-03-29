use clap::Parser;
use color_eyre::Result;
use git2::Repository;

#[derive(Parser)]
struct Args {
    #[clap()]
    repo_path: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let repo = Repository::open(args.repo_path)?;

    // - If 2 or more tags exist, get all commits between 2 most-recent tags
    // - If 1 tag exists, get all commits between it and HEAD
    // - If no tags exist, get all commits between HEAD and the first commit
    let mut revwalk = repo.revwalk()?;
    let tags = repo.tag_names(None)?;
    let tags: Vec<&str> = tags.into_iter().flatten().collect();

    if tags.len() >= 2 {
        let tag1 = tags[tags.len() - 2];
        let tag2 = tags[tags.len() - 1];

        revwalk.push_range(&format!("{}..{}~1", tag1, tag2))?;
    } else if tags.len() == 1 {
        let tag = repo.revparse_single(tags[0])?;
        revwalk.push(tag.id())?;
        revwalk.push(repo.head()?.target().expect("no OID at HEAD!?"))?;
    } else {
        revwalk.push(repo.head()?.target().expect("no OID at HEAD!?"))?;

        // Read the OID of the first commit from the repo
        let mut revwalk2 = repo.revwalk()?;
        revwalk2.push(repo.head()?.target().expect("no OID at HEAD!?"))?;
        revwalk2.set_sorting(git2::Sort::TIME)?;
        let oid = revwalk2
            .next()
            .expect("no commits in repo!?")
            .expect("no OID for commit!?");
        revwalk.push(oid)?;
    }

    revwalk.set_sorting(git2::Sort::REVERSE)?;

    let mut buffer = String::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let summary = commit.summary().unwrap_or("no summary");
        let shorthash = commit.id().to_string()[..7].to_string();
        let author = commit.author();
        let author = author.name().unwrap_or("no author");

        buffer.push_str(&format!("`{shorthash}`: {summary} ({author})\n"));
    }

    println!("{buffer}");

    Ok(())
}
