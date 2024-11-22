use clap::Parser;
use normalize_path::NormalizePath;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use walkdir::WalkDir;
use slugify::slugify;

#[derive(Parser, Debug)]
struct Commands {
  /// The directory to search within
  scan_dir: Option<PathBuf>,

  #[arg(long = "no-slugify", default_value_t = false)]
  no_slugify: bool,

  #[arg(short = 'y', default_value_t = false)]
  force: bool,
}

fn main() -> anyhow::Result<()> {
  let cmd = Commands::parse();
  let scan_dir: PathBuf;
  if let Some(target) = cmd.scan_dir {
    if target.is_relative() {
      scan_dir = std::env::current_dir()?
        .join(target)
        .normalize();
    } else {
      scan_dir = target;
    }
  } else {
    scan_dir = std::env::current_dir()?
        .normalize();
  }

  println!("Config:");
  println!("  Scanning: {}/**/*", scan_dir.to_str().unwrap());
  println!("  Move to:  {}", scan_dir.to_str().unwrap());
  println!("  Slugify:  {}", !cmd.no_slugify);

  let mut matches = HashMap::<PathBuf, PathBuf>::new();
  let mut delete = HashSet::<PathBuf>::new();

  println!("");

  for entry in WalkDir::new(&scan_dir) {
    let entry = entry?;
    let entry_path = entry.path();

    if entry_path.is_dir() && entry_path != &scan_dir {
      delete.insert(entry_path.to_path_buf());
      continue;
    }

    if !entry_path.is_file() {
      continue;
    }

    let file_ext = entry_path.extension().unwrap().to_str().unwrap().to_string();
    let file_name = entry_path.file_name().unwrap().to_str().unwrap().to_string();
    let file_stem = entry_path.file_stem().unwrap().to_str().unwrap().to_string();
    let file_name_slug: String;

    if cmd.no_slugify {
      file_name_slug = format!("{}.{}", &file_stem, &file_ext);
    } else {
      file_name_slug = format!("{}.{}", slugify!(entry_path.file_stem().unwrap().to_str().unwrap()), file_ext);
    }

    if entry_path.parent().unwrap() == &scan_dir && file_name == file_name_slug {
      continue;
    };

    let target = scan_dir.join(&file_name_slug);
    matches.insert(entry_path.to_path_buf(), target);

    let src = pathdiff::diff_paths(&entry_path, &scan_dir).unwrap();
    println!("  From: {}\n  To:   {}\n", src.to_str().unwrap(), file_name_slug);
  }

  if delete.is_empty() && matches.is_empty() {
    println!("Nothing to do");    
    return Ok(());
  }

  println!("Delete Directories:");
  for entry in &delete {
    println!("  {}", entry.to_str().unwrap());
  }
  
  if !cmd.force {
    println!("");
    print!("Continue? ({} found) [y/N] ", matches.len());
    let mut line = String::new();
    let _ = std::io::stdout().flush();
    std::io::stdin().read_line(&mut line).unwrap();
    line = line.trim().to_string();

    if line != "y" && line != "Y" {
      println!("Nothing changed");
      return Ok(());
    }
  } else {
    println!("");
  }

  for (from, to) in matches.iter() {
    println!("  Move: {}", to.to_str().unwrap());
    fs::rename(from, to)?;
  }

  for entry in &delete {
    println!("  Del:  {}", entry.to_str().unwrap());
    fs::remove_dir_all(&entry).ok();
  }

  return Ok(())
}
