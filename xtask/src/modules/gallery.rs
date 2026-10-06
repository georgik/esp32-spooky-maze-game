/// Assemble the ESPBrew snapshot gallery site.
///
/// It reads the per-board snapshot zips from `deploy/site/zips/` and the optional
/// WASM browser build from `deploy/wasm/`, then writes:
///
///   deploy/site/index.html      — a per-board card grid
///   deploy/site/screenshots/<board>/ — expanded snapshot frames
///   deploy/site/wasm/           — the WASM browser build (if present)
///
/// Only boards that actually produced a snapshot get an `<img>` card; the rest
/// get a placeholder. The script is robust to a missing/empty zip (it simply
/// yields no frame → placeholder). It must be run from the repo root, where the
/// `deploy/` tree is found relative to the current working directory.
///
/// All file assembly lives here (on the GitHub-hosted deploy runner), never on
/// the self-hosted hardware runner.
use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Directories relative to the repo root (the working directory).
fn site_dir() -> PathBuf {
    Path::new("deploy").join("site")
}

fn zips_dir() -> PathBuf {
    site_dir().join("zips")
}

fn screenshots_dir() -> PathBuf {
    site_dir().join("screenshots")
}

fn wasm_src() -> PathBuf {
    Path::new("deploy").join("wasm")
}

fn wasm_dst() -> PathBuf {
    site_dir().join("wasm")
}

/// File extensions treated as a hardware snapshot frame. `Path::extension()`
/// returns the extension WITHOUT the leading dot, so these have none too.
const IMG_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

/// Assemble the gallery. Returns Ok even when nothing was snapshotted (an empty
/// gallery page is still valid output).
pub fn assemble_gallery(_verbose: bool) -> Result<()> {
    println!("[GALLERY] Assembling ESPBrew snapshot gallery");
    println!("{}", "=".repeat(60));

    let screenshots = screenshots_dir();
    fs::create_dir_all(&screenshots).context("create screenshots dir")?;

    // Copy the optional WASM browser build into the site (kept live, never
    // flashed). Absent when the wasm job failed or was skipped.
    copy_wasm()?;

    // Expand each board's snapshot zip and remember its first frame.
    let mut cards: Vec<(String, Option<PathBuf>)> = Vec::new();
    for zip in collect_zips() {
        let board = board_name(&zip);
        let board_dir = screenshots.join(&board);
        fs::create_dir_all(&board_dir).context("create board screenshot dir")?;
        let frame = expand_zip(&zip, &board_dir)?;
        cards.push((board, frame));
    }

    write_index(&cards)?;

    println!("[GALLERY] {} board(s)", cards.len());
    Ok(())
}

/// Collect `deploy/site/zips/*.zip`, sorted by name for deterministic output.
fn collect_zips() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(zips_dir()) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("zip") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

/// The board name is the snapshot zip filename without the `.zip` extension.
fn board_name(zip: &Path) -> String {
    zip.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string()
}

/// Recursively copy `src/` into `dst/`.
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).context("read wasm src dir")? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).with_context(|| format!("copy {}", from.display()))?;
        }
    }
    Ok(())
}

/// Copy the optional WASM browser build into the site.
fn copy_wasm() -> Result<()> {
    if !wasm_src().exists() {
        return Ok(());
    }
    let dst = wasm_dst();
    if dst.exists() {
        fs::remove_dir_all(&dst).ok();
    }
    copy_dir_all(&wasm_src(), &dst).context("copy wasm build")?;
    Ok(())
}

/// Expand a snapshot zip into `board_dir/`, returning the first image found.
fn expand_zip(zip: &Path, board_dir: &Path) -> Result<Option<PathBuf>> {
    let file = fs::File::open(zip).with_context(|| format!("open {}", zip.display()))?;
    let mut archive =
        zip::ZipArchive::new(file).with_context(|| format!("read zip {}", zip.display()))?;

    for i in 0..archive.len() {
        let mut f = archive
            .by_index(i)
            .with_context(|| format!("entry {} of {}", i, zip.display()))?;

        // Guard against path traversal (zip-slip): only accept entries that
        // stay inside board_dir.
        let outpath = board_dir.join(f.name().to_string());
        if !outpath.starts_with(board_dir) {
            continue;
        }

        if f.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = fs::File::create(&outpath)
                .with_context(|| format!("create {}", outpath.display()))?;
            io::copy(&mut f, &mut out)?;
        }
    }

    Ok(first_image(board_dir))
}

/// Return the first image file under `dir` (sorted), or None.
fn first_image(dir: &Path) -> Option<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|ext| IMG_EXTS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files.into_iter().next()
}

/// Render one gallery card (an `<img>` when a frame exists, else a placeholder).
fn card_html(board: &str, rel: Option<&str>) -> String {
    let safe_board = board.replace('&', "&amp;").replace('"', "&quot;");
    match rel {
        Some(src) => {
            format!(
                "  <div class=\"card\">\n    <h2>{board}</h2>\n    <img class=\"screenshot\" src=\"{src}\" alt=\"{board} hardware snapshot\">\n  </div>\n",
                board = safe_board,
                src = src.replace('&', "&amp;").replace('"', "&quot;")
            )
        }
        None => {
            format!(
                "  <div class=\"card\">\n    <h2>{board}</h2>\n    <div class=\"placeholder\">No hardware snapshot captured</div>\n  </div>\n",
                board = safe_board
            )
        }
    }
}

/// Write `deploy/site/index.html` from the per-board cards.
fn write_index(cards: &[(String, Option<PathBuf>)]) -> Result<()> {
    let body = cards
        .iter()
        .map(|(board, frame)| {
            let rel = frame.as_deref().and_then(|p| p.to_str());
            card_html(board, rel)
        })
        .collect::<String>();

    let body = if body.is_empty() {
        "  <div class=\"card\"><h2>Nothing snapshotted</h2>\n  <div class=\"placeholder\">No hardware snapshots were captured this run</div></div>\n"
            .to_string()
    } else {
        body
    };

    let html = format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>Spooky Maze Game — ESPBrew Snapshots</title>\n\
         <style>\n\
         body {{ margin: 0; padding: 24px; background: #000; color: #eee; font-family: Arial, sans-serif; }}\n\
         h1 {{ text-align: center; font-weight: normal; letter-spacing: 1px; }}\n\
         .gallery {{ display: flex; flex-wrap: wrap; gap: 24px; justify-content: center; align-items: flex-start; }}\n\
         .card {{ background: #111; border: 1px solid #333; border-radius: 10px; padding: 14px; width: 300px; text-align: center; }}\n\
         .card h2 {{ font-size: 15px; margin: 0 0 10px 0; color: #fff; }}\n\
         .card img.screenshot {{ width: 260px; height: 260px; image-rendering: pixelated; border: 1px solid #444; background: #1a1a1a; border-radius: 4px; }}\n\
         .placeholder {{ width: 260px; height: 260px; display: flex; align-items: center; justify-content: center; color: #777; border: 1px dashed #555; border-radius: 4px; }}\n\
         </style>\n\
         </head>\n\
         <body>\n\
         <h1>Spooky Maze Game — ESPBrew Hardware Snapshots</h1>\n\
         <div class=\"gallery\">\n\
         {body}\n\
         </div>\n\
         </body>\n\
         </html>\n"
    );

    fs::write(site_dir().join("index.html"), html).context("write index.html")?;
    Ok(())
}
