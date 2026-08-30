use axum::extract::Path;
use axum::http::{HeaderValue, header};
use axum::response::{Html, IntoResponse, Response};
use pulldown_cmark::{Options, Parser};

/// Documentation in reading order: (slug, title, markdown).
/// The sidebar, prev/next navigation, and `/docs` index all derive from this.
pub const DOCS: &[(&str, &str, &str)] = &[
    (
        "quickstart",
        "Quick start",
        include_str!("../../../docs/quickstart.md"),
    ),
    (
        "api",
        "API and signing",
        include_str!("../../../docs/api.md"),
    ),
    (
        "mailbox",
        "Agent mailbox & MCP",
        include_str!("../../../docs/mailbox.md"),
    ),
    (
        "configuration",
        "Configuration",
        include_str!("../../../docs/configuration.md"),
    ),
    (
        "observability",
        "OpenTelemetry",
        include_str!("../../../docs/observability.md"),
    ),
    (
        "operations",
        "Operations",
        include_str!("../../../docs/operations.md"),
    ),
    (
        "security",
        "Security",
        include_str!("../../../docs/security.md"),
    ),
    (
        "migration-v01-v02",
        "Migration v0.1 → v0.2",
        include_str!("../../../docs/migration-v01-v02.md"),
    ),
    (
        "migration-v02-v03",
        "Migration v0.2 → v0.3",
        include_str!("../../../docs/migration-v02-v03.md"),
    ),
];

pub fn lookup(name: &str) -> Option<&'static str> {
    DOCS.iter()
        .find(|(slug, _, _)| *slug == name)
        .map(|(_, _, markdown)| *markdown)
}

fn title_for(name: &str) -> &'static str {
    DOCS.iter()
        .find(|(slug, _, _)| *slug == name)
        .map(|(_, title, _)| *title)
        .unwrap_or("Documentation")
}

pub fn render(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);
    let mut html = String::with_capacity(markdown.len() * 2);
    pulldown_cmark::html::push_html(&mut html, parser);
    rewrite_doc_links(&html)
}

fn rewrite_doc_links(html: &str) -> String {
    let mut result = html.to_string();
    for (slug, _, _) in DOCS {
        for suffix in [".md)", ".md\""] {
            let from = format!("{slug}{suffix}");
            let to = match suffix {
                ".md)" => format!("/docs/{slug})"),
                _ => format!("/docs/{slug}\""),
            };
            result = result.replace(&from, &to);
        }
    }
    result
}

type DocLink = (&'static str, &'static str);

fn navigation_for(name: &str) -> (Option<DocLink>, Option<DocLink>) {
    let position = DOCS.iter().position(|(slug, _, _)| *slug == name);
    let position = match position {
        Some(index) => index,
        None => return (None, None),
    };
    let previous = if position > 0 {
        let (slug, title, _) = DOCS[position - 1];
        Some((slug, title))
    } else {
        None
    };
    let next = DOCS
        .get(position + 1)
        .map(|&(slug, title, _)| (slug, title));
    (previous, next)
}

pub fn page(active: Option<&str>, title: &str, body: &str, nonce: &str) -> String {
    let sidebar = DOCS
        .iter()
        .map(|&(slug, doc_title, _)| {
            let class = if Some(slug) == active {
                " class=\"active\""
            } else {
                ""
            };
            format!("<a{class} href=\"/docs/{slug}\">{doc_title}</a>")
        })
        .collect::<String>();
    let (previous, next) = active.map_or((None, None), navigation_for);
    let footer = match (previous, next) {
        (Some((p_slug, p_title)), Some((n_slug, n_title))) => format!(
            "<div class=\"pager\"><a class=\"pager-link\" href=\"/docs/{p_slug}\"><span>← Previous</span><strong>{p_title}</strong></a><a class=\"pager-link next\" href=\"/docs/{n_slug}\"><span>Next →</span><strong>{n_title}</strong></a></div>"
        ),
        (Some((p_slug, p_title)), None) => format!(
            "<div class=\"pager\"><a class=\"pager-link\" href=\"/docs/{p_slug}\"><span>← Previous</span><strong>{p_title}</strong></a></div>"
        ),
        (None, Some((n_slug, n_title))) => format!(
            "<div class=\"pager\"><a class=\"pager-link next\" href=\"/docs/{n_slug}\"><span>Next →</span><strong>{n_title}</strong></a></div>"
        ),
        (None, None) => String::new(),
    };
    let breadcrumb = active.map_or_else(
        || "Documentation".to_string(),
        |name| format!("Documentation <span>/</span> {}", title_for(name)),
    );
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="color-scheme" content="dark light"><title>{title} · PromptJang Relay</title><style nonce="{nonce}">
:root {{ font-family:"IBM Plex Sans",ui-sans-serif,system-ui,sans-serif; color-scheme:dark; --bg:#080d16; --surface:#0e1624; --elevated:#152033; --border:#263349; --text:#f4f7fb; --muted:#9aa8ba; --accent:#00d4aa; }}
@media (prefers-color-scheme: light) {{ :root {{ color-scheme:light; --bg:#f5f8f7; --surface:#fff; --elevated:#eef3f1; --border:#d8e2df; --text:#101820; --muted:#5c6975; }} }}
* {{ box-sizing:border-box }} body {{ margin:0; background:var(--bg); color:var(--text); line-height:1.7; font-size:15.5px }}
.layout {{ display:grid; grid-template-columns:250px minmax(0,1fr); min-height:100vh }}
aside {{ position:sticky; top:0; height:100vh; padding:26px 16px; border-right:1px solid var(--border); background:var(--surface); display:flex; flex-direction:column }}
.brand {{ font-weight:700; margin:0 0 2px; font-size:16px }} .brand span {{ color:var(--accent) }}
.eyebrow {{ color:var(--muted); font-size:10.5px; letter-spacing:.09em; margin:0 0 20px }}
aside nav {{ display:grid; gap:3px }}
aside nav a {{ padding:8px 11px; border-radius:8px; color:var(--muted); text-decoration:none; font-size:13.5px; border-left:2px solid transparent }}
aside nav a:hover {{ background:color-mix(in srgb,var(--accent) 9%,transparent); color:var(--accent) }}
aside nav a.active {{ color:var(--accent); background:color-mix(in srgb,var(--accent) 11%,transparent); border-left-color:var(--accent); font-weight:600 }}
.backlink {{ margin-top:auto; color:var(--muted); font-size:12.5px; text-decoration:none; padding:8px 11px; border-radius:8px; border:1px solid var(--border) }}
.backlink:hover {{ color:var(--accent); border-color:var(--accent) }}
main {{ max-width:840px; padding:38px 40px 60px; margin:0 auto; width:100% }}
.crumbs {{ color:var(--muted); font-size:12.5px; margin-bottom:6px }} .crumbs span {{ margin:0 6px; color:var(--border) }}
h1 {{ font-size:27px; margin:0 0 22px; padding-bottom:14px; border-bottom:1px solid var(--border) }}
h2 {{ font-size:19px; margin:36px 0 12px }} h3 {{ font-size:16px; margin:26px 0 10px }}
p {{ margin:12px 0 }} a {{ color:var(--accent) }}
code {{ font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:.87em; background:var(--elevated); padding:2px 6px; border-radius:5px }}
.codeblock {{ position:relative; margin:16px 0 }}
.codeblock .copy {{ position:absolute; top:9px; right:9px; padding:4px 10px; font-size:11.5px; border-radius:6px; border:1px solid var(--border); background:var(--surface); color:var(--muted); cursor:pointer; font-family:inherit }}
.codeblock .copy:hover {{ color:var(--accent); border-color:var(--accent) }}
.codeblock .copy.done {{ color:var(--accent); border-color:var(--accent) }}
.codeblock pre {{ margin:0; background:var(--elevated); border:1px solid var(--border); border-radius:10px; padding:16px; overflow-x:auto }}
pre code {{ background:none; padding:0; font-size:13px; line-height:1.6 }}
table {{ border-collapse:collapse; width:100%; margin:16px 0; font-size:13.5px }} th,td {{ border:1px solid var(--border); padding:8px 12px; text-align:left; vertical-align:top }} th {{ background:var(--surface); font-weight:650 }}
blockquote {{ border-left:3px solid var(--accent); margin:16px 0; padding:4px 18px; color:var(--muted); background:var(--surface); border-radius:0 8px 8px 0 }}
hr {{ border:none; border-top:1px solid var(--border); margin:32px 0 }}
ul,ol {{ padding-left:24px }} li {{ margin:5px 0 }}
.pager {{ display:grid; grid-template-columns:1fr 1fr; gap:14px; margin-top:46px; padding-top:22px; border-top:1px solid var(--border) }}
.pager-link {{ display:grid; gap:2px; padding:13px 16px; border:1px solid var(--border); border-radius:10px; text-decoration:none; color:var(--text) }}
.pager-link span {{ color:var(--muted); font-size:11.5px }} .pager-link strong {{ font-size:14px }}
.pager-link:hover {{ border-color:var(--accent) }} .pager-link.next {{ text-align:right }}
@media(max-width:780px) {{ .layout {{ grid-template-columns:1fr }} aside {{ position:static; height:auto; border-right:none; border-bottom:1px solid var(--border) }} .backlink {{ display:none }} main {{ padding:26px 18px 50px }} .pager {{ grid-template-columns:1fr }} }}
@media (prefers-reduced-motion: no-preference) {{ .pager-link,aside nav a {{ transition:border-color .15s,background .15s,color .15s }} }}
</style></head><body><div class="layout"><aside><p class="eyebrow">RELAY · SELF-HOSTED</p><p class="brand">PromptJang <span>Relay</span></p><nav aria-label="Documentation">{sidebar}</nav><a class="backlink" href="/">← Open the UI</a></aside><main><p class="crumbs">{breadcrumb}</p>{body}{footer}</main></div><script nonce="{nonce}">
document.querySelectorAll("pre").forEach(function (pre) {{
  var wrapper = document.createElement("div");
  wrapper.className = "codeblock";
  pre.parentNode.insertBefore(wrapper, pre);
  wrapper.appendChild(pre);
  var button = document.createElement("button");
  button.className = "copy";
  button.type = "button";
  button.setAttribute("aria-label", "Copy code to clipboard");
  button.textContent = "Copy";
  button.addEventListener("click", function () {{
    navigator.clipboard.writeText(pre.textContent.trim()).then(function () {{
      button.textContent = "Copied ✓";
      button.classList.add("done");
      setTimeout(function () {{ button.textContent = "Copy"; button.classList.remove("done"); }}, 1400);
    }});
  }});
  wrapper.appendChild(button);
}});
</script></body></html>"#
    )
}

fn docs_response(active: Option<&str>, title: &str, body: &str) -> Response {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let csp = format!(
        "default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'nonce-{nonce}'; script-src 'nonce-{nonce}'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
    );
    let mut response = Html(page(active, title, body, &nonce)).into_response();
    if let Ok(value) = HeaderValue::from_str(&csp) {
        response
            .headers_mut()
            .insert(header::CONTENT_SECURITY_POLICY, value);
    }
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

pub async fn index() -> Response {
    let body = render(
        "# PromptJang Relay documentation\n\nRun signed webhook delivery or durable agent mailboxes on your PostgreSQL. Start with the quick start, then open only the guide you need.\n",
    );
    docs_response(None, "Documentation", &body)
}

pub async fn article(Path(name): Path<String>) -> Response {
    match lookup(&name) {
        Some(markdown) => {
            let title = title_for(&name);
            docs_response(Some(&name), title, &render(markdown))
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_documents_resolve_and_unknown_reject() {
        // Arrange
        let names = DOCS.iter().map(|(slug, _, _)| *slug);

        // Act + Assert
        for name in names {
            assert!(lookup(name).is_some(), "{name} must resolve");
        }
        assert!(lookup("../../etc/passwd").is_none());
        assert!(lookup("missing").is_none());
    }

    #[test]
    fn markdown_renders_headings_code_and_tables() {
        // Arrange
        let markdown =
            "# Title\n\n```bash\ncurl -s http://x\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";

        // Act
        let html = render(markdown);

        // Assert
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<pre><code class=\"language-bash\">"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn links_to_known_documents_are_rewritten_and_others_survive() {
        // Arrange
        let markdown = "[mailbox](mailbox.md) and [external](../examples/observability/README.md)";

        // Act
        let html = render(markdown);

        // Assert
        assert!(html.contains("href=\"/docs/mailbox\""));
        assert!(html.contains("href=\"../examples/observability/README.md\""));
    }

    #[test]
    fn sidebar_marks_the_active_document() {
        // Arrange
        let page = page(
            Some("mailbox"),
            "Agent mailbox & MCP",
            "<h1>x</h1>",
            "test-nonce",
        );

        // Assert
        assert!(page.contains(r#"class="active" href="/docs/mailbox""#));
        assert!(!page.contains("href=\"/docs/api\" class=\"active\""));
    }

    #[test]
    fn middle_pages_link_both_ways_and_edges_link_one_way() {
        // Arrange
        let first = page(
            Some("quickstart"),
            "Quick start",
            "<h1>x</h1>",
            "test-nonce",
        );
        let middle = page(Some("api"), "API and signing", "<h1>x</h1>", "test-nonce");
        let last_slug = DOCS[DOCS.len() - 1].0;
        let last = page(Some(last_slug), "Last", "<h1>x</h1>", "test-nonce");

        // Act + Assert
        assert!(first.contains("Next →") && !first.contains("← Previous"));
        assert!(middle.contains("← Previous") && middle.contains("Next →"));
        assert!(last.contains("← Previous") && !last.contains("Next →"));
    }

    #[test]
    fn every_page_ships_copy_buttons_over_code_blocks() {
        // Arrange
        let page = page(
            Some("api"),
            "API and signing",
            "<pre><code>curl</code></pre>",
            "test-nonce",
        );

        // Act + Assert
        assert!(page.contains(r#"button.className = "copy""#));
        assert!(page.contains("navigator.clipboard.writeText"));
    }

    #[test]
    fn page_shell_keeps_the_ui_exit_and_brand() {
        // Arrange
        let page = page(None, "Documentation", "<h1>x</h1>", "test-nonce");

        // Assert
        assert!(page.contains("PromptJang <span>Relay</span>"));
        assert!(page.contains("href=\"/\""));
        assert!(page.contains("color-scheme:dark"));
        assert!(page.contains(r#"style nonce="test-nonce""#));
        assert!(page.contains(r#"script nonce="test-nonce""#));
    }

    #[test]
    fn documentation_responses_are_public_content_not_cached_spa_shells() {
        let response = docs_response(None, "Documentation", "<h1>x</h1>");

        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store"))
        );
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
