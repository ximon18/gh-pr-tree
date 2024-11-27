use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use jiff::{Unit, Zoned};
use rocket::http::ContentType;
use rocket::{get, routes, State};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time;

#[derive(Deserialize)]
struct GhCliAuthor {
    #[serde(default = "bot")]
    name: String,
}

fn bot() -> String {
    "bot".to_string()
}

#[derive(Deserialize)]
struct GhCliLabel {
    name: String,

    color: String,
}

#[derive(Deserialize)]
struct GhCliPr {
    author: GhCliAuthor,

    #[serde(alias = "baseRefName")]
    base: String,

    #[serde(alias = "headRefName")]
    head: String,

    number: usize,

    title: String,

    url: String,

    labels: Vec<GhCliLabel>,
}

#[get("/")]
async fn index(
    html: &State<Arc<RwLock<HashMap<String, String>>>>,
) -> (ContentType, String) {
    let mut out_html = "<html><body><h1>Monitored GitHub Repositories</h1><hr/><ul>\n".to_string();
    for repo in html.read().await.keys() {
        out_html.push_str(&format!("<li><a href=\"/{repo}\">{repo}</a></li>\n"));
    }
    out_html.push_str("</ul></body></html>");
    (ContentType::HTML, out_html)
}

#[get("/<org>/<repo>")]
async fn repo_prs(
    org: &str,
    repo: &str,
    html: &State<Arc<RwLock<HashMap<String, String>>>>,
) -> (ContentType, String) {
    if let Some(found_html) = html.read().await.get(&format!("{org}/{repo}")) {
        (ContentType::HTML, found_html.clone())
    } else {
        (ContentType::Plain, "Unknown repository".to_string())
    }
}

#[tokio::main]
async fn main() {
    let html = Arc::new(RwLock::new(HashMap::<String, String>::new()));
    let mut args = std::env::args();

    if args.len() < 1 {
        eprintln!("Expected org/repo command line argument");
        std::process::exit(1);
    };

    args.next();

    for repo in args {
        eprintln!("{repo}");
        let bg_html = html.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(time::Duration::from_secs(60));
            loop {
                let start = Instant::now();
                let repo_clone = repo.clone();
                if let Ok(Ok(res)) = tokio::task::spawn_blocking(|| {
                    Command::new("gh")
                        .arg("pr")
                        .arg("list")
                        .arg("--repo")
                        .arg(repo_clone)
                        .arg("--limit")
                        .arg("100")
                        .arg("--json")
                        .arg("author,number,title,baseRefName,headRefName,url,labels")
                        .output()
                })
                .await
                {
                    if !res.status.success() {
                        bg_html.write().await.insert(
                            repo.clone(),
                            format!("gh cli command exited with code {}", res.status),
                        );
                        std::io::stderr().write_all(&res.stderr).unwrap();
                    } else {
                        if let Ok(prs) = serde_json::from_slice(&res.stdout) {
                            let duration = start.elapsed();
                            let now = Zoned::now().round(Unit::Second).unwrap();
                            let new_html =
                                format!("<html><body><h1>{repo}</h1></hr>{}<hr/>Last updated: {}<br/>Generated in {}ms</body><html>", pr_to_html(&prs, "main", 0), now, duration.as_millis());
                            bg_html.write().await.insert(repo.clone(), new_html);
                        }
                    }
                }

                interval.tick().await;
            }
        });
    }

    rocket::build()
        .mount("/", routes![index, repo_prs])
        .manage(html)
        .launch()
        .await
        .unwrap();
}

fn pr_to_html(prs: &Vec<GhCliPr>, parent: &str, depth: u8) -> String {
    let mut out = String::new();
    out.push_str("<ul>\n");
    for GhCliPr {
        number,
        author: GhCliAuthor { name },
        title,
        url,
        head,
        labels,
        ..
    } in prs.iter().filter(move |pr| pr.base == parent)
    {
        out.push_str(&format!(
            "<li>PR #<a href=\"{url}\">{number}</a> by {name}: {title}"
        ));
        for GhCliLabel { name, color } in labels {
            out.push_str(&format!("&nbsp;<span style=\"font-family: arial; font-weight: bold; color: {color}\">{name}</span>"));
        }
        out.push('\n');
        out.push_str(&pr_to_html(prs, head, depth + 1));
        out.push_str("</li>\n");
    }
    out.push_str("</ul>\n");
    out
}
