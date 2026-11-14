//------------------------------------------------------------------------------------
// gfx_registration.rs -- Part of RHoiScribe
//
// Copyright (C) 2026 CzXieDdan. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// https://github.com/czxieddan/RHoiScribe
//------------------------------------------------------------------------------------

//! Register existing interface images as HOI4 `spriteType` entries.
//!
//! The procedural `generate_gui_gfx_asset` pipeline invents new art. This module
//! solves the opposite, far more common problem: a mod already has a folder of
//! PNG/DDS/TGA icons (focus icons, idea icons, decision icons, event pictures)
//! and needs a matching `interface/*.gfx` registration block for every one of
//! them. It scans the workspace for existing registrations first, so it is safe
//! to re-run after adding new art: only genuinely missing sprites are emitted.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const DEFAULT_GFX_FILE: &str = "interface/rhoiscribe_sprites.gfx";
const DEFAULT_SPRITE_PREFIX: &str = "GFX_";
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "dds", "tga"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterHoi4GfxSpritesRequest {
    pub workspace_root: String,
    pub image_directory: String,
    pub gfx_file: Option<String>,
    pub sprite_prefix: Option<String>,
    pub name_style: Option<String>,
    pub uppercase: Option<bool>,
    pub recursive: Option<bool>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GfxSpriteRegistration {
    pub sprite_name: String,
    pub image_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GfxSkippedImage {
    pub image_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterHoi4GfxSpritesResult {
    pub dry_run: bool,
    pub applied: bool,
    pub gfx_file: String,
    pub image_directory: String,
    pub scanned_images: usize,
    pub scanned_gfx_files: usize,
    pub already_registered: Vec<GfxSpriteRegistration>,
    pub generated: Vec<GfxSpriteRegistration>,
    pub orphaned_sprites: Vec<GfxSpriteRegistration>,
    pub skipped: Vec<GfxSkippedImage>,
    pub appended_block: String,
    pub messages: Vec<String>,
}

pub fn register_hoi4_gfx_sprites(
    request: RegisterHoi4GfxSpritesRequest,
) -> Result<RegisterHoi4GfxSpritesResult, String> {
    let root = Path::new(&request.workspace_root);
    if !root.is_dir() {
        return Err(format!(
            "workspace_root `{}` is not an existing directory",
            request.workspace_root
        ));
    }

    let image_directory = normalize_relative_directory(&request.image_directory)?;
    let image_root = root.join(&image_directory);
    if !image_root.is_dir() {
        return Err(format!(
            "image_directory `{}` does not exist under workspace_root",
            image_directory
        ));
    }

    let gfx_file = request
        .gfx_file
        .as_deref()
        .map(normalize_relative_path)
        .transpose()?
        .unwrap_or_else(|| DEFAULT_GFX_FILE.to_string());
    let prefix = request
        .sprite_prefix
        .clone()
        .unwrap_or_else(|| DEFAULT_SPRITE_PREFIX.to_string());
    if prefix.contains(['"', '\\', '/']) {
        return Err("sprite_prefix must not contain quotes or path separators".to_string());
    }

    let name_style = request
        .name_style
        .clone()
        .unwrap_or_else(|| "file_stem".to_string());
    if name_style != "file_stem" && name_style != "relative_path" {
        return Err("name_style must be \"file_stem\" or \"relative_path\"".to_string());
    }
    let uppercase = request.uppercase.unwrap_or(false);
    let recursive = request.recursive.unwrap_or(true);

    let existing = scan_workspace_registrations(root)?;
    let images = collect_images(root, &image_root, recursive)?;

    let mut generated = Vec::new();
    let mut already_registered = Vec::new();
    let mut skipped = Vec::new();
    let mut batch_names = BTreeSet::new();

    for image in &images {
        let relative = to_manifest_path(
            image
                .strip_prefix(root)
                .map_err(|error| format!("failed to relativize {}: {}", image.display(), error))?,
        );
        let lookup = relative.to_ascii_lowercase();

        if let Some(sprite_name) = existing.by_texture.get(&lookup) {
            already_registered.push(GfxSpriteRegistration {
                sprite_name: sprite_name.clone(),
                image_path: relative,
            });
            continue;
        }

        let raw_stem = if name_style == "relative_path" {
            relative
                .trim_start_matches(&format!("{}/", image_directory))
                .to_string()
        } else {
            image
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string()
        };
        let sanitized = sanitize_sprite_token(&raw_stem);
        if sanitized.is_empty() {
            skipped.push(GfxSkippedImage {
                image_path: relative,
                reason:
                    "file name has no ASCII alphanumeric characters to build a sprite name from"
                        .to_string(),
            });
            continue;
        }

        let mut sprite_name = format!("{}{}", prefix, sanitized);
        if uppercase {
            sprite_name = sprite_name.to_ascii_uppercase();
        }

        if existing.by_name.contains(&sprite_name) || !batch_names.insert(sprite_name.clone()) {
            skipped.push(GfxSkippedImage {
                image_path: relative,
                reason: format!(
                    "sprite name `{}` is already registered; pass a different sprite_prefix or rename the file",
                    sprite_name
                ),
            });
            continue;
        }

        generated.push(GfxSpriteRegistration {
            sprite_name,
            image_path: relative,
        });
    }

    let orphaned_sprites = collect_orphans(root, &image_directory, &existing);

    let appended_block = render_sprite_types(&generated);
    let mut messages = Vec::new();
    let mut applied = false;

    if generated.is_empty() {
        messages.push(
            "Every image in image_directory already has a spriteType registration.".to_string(),
        );
    } else if request.dry_run {
        messages.push(format!(
            "Dry run: {} spriteType entries would be written to {}.",
            generated.len(),
            gfx_file
        ));
    } else {
        let target = root.join(&gfx_file);
        write_registrations(&target, &generated)?;
        applied = true;
        messages.push(format!(
            "Wrote {} spriteType entries to {}.",
            generated.len(),
            gfx_file
        ));
    }

    if !orphaned_sprites.is_empty() {
        messages.push(format!(
            "{} registered sprite(s) point at files that are missing on disk; check whether art was deleted or moved.",
            orphaned_sprites.len()
        ));
    }
    if !skipped.is_empty() {
        messages.push(format!(
            "{} image(s) were skipped; see the skipped list for the reason.",
            skipped.len()
        ));
    }

    Ok(RegisterHoi4GfxSpritesResult {
        dry_run: request.dry_run,
        applied,
        gfx_file,
        image_directory,
        scanned_images: images.len(),
        scanned_gfx_files: existing.files,
        already_registered,
        generated,
        orphaned_sprites,
        skipped,
        appended_block,
        messages,
    })
}

#[derive(Debug, Default)]
struct ExistingRegistrations {
    files: usize,
    by_texture: BTreeMap<String, String>,
    by_name: BTreeSet<String>,
}

fn scan_workspace_registrations(root: &Path) -> Result<ExistingRegistrations, String> {
    let mut existing = ExistingRegistrations::default();
    let interface_root = root.join("interface");
    let mut gfx_files = Vec::new();
    if interface_root.is_dir() {
        collect_files_with_extension(&interface_root, "gfx", true, &mut gfx_files)?;
    }

    for file in &gfx_files {
        existing.files += 1;
        let text = fs::read_to_string(file)
            .map_err(|error| format!("failed to read {}: {}", file.display(), error))?;
        for (sprite_name, texture) in parse_sprite_registrations(&text) {
            existing.by_name.insert(sprite_name.clone());
            existing
                .by_texture
                .insert(texture.to_ascii_lowercase(), sprite_name);
        }
    }
    Ok(existing)
}

fn collect_orphans(
    root: &Path,
    image_directory: &str,
    existing: &ExistingRegistrations,
) -> Vec<GfxSpriteRegistration> {
    let prefix = format!("{}/", image_directory.to_ascii_lowercase());
    let mut orphans = Vec::new();
    for (texture, sprite_name) in &existing.by_texture {
        if !texture.starts_with(&prefix) {
            continue;
        }
        if !root.join(texture).is_file() {
            orphans.push(GfxSpriteRegistration {
                sprite_name: sprite_name.clone(),
                image_path: texture.clone(),
            });
        }
    }
    orphans
}

fn collect_images(root: &Path, image_root: &Path, recursive: bool) -> Result<Vec<PathBuf>, String> {
    let mut images = Vec::new();
    for extension in SUPPORTED_IMAGE_EXTENSIONS {
        collect_files_with_extension(image_root, extension, recursive, &mut images)?;
    }
    images.sort();
    let mut unique = Vec::new();
    let mut seen = BTreeSet::new();
    for image in images {
        let key = to_manifest_path(image.strip_prefix(root).unwrap_or(&image));
        if seen.insert(key) {
            unique.push(image);
        }
    }
    Ok(unique)
}

fn collect_files_with_extension(
    directory: &Path,
    extension: &str,
    recursive: bool,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {}", directory.display(), error))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {}", error))?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_files_with_extension(&path, extension, recursive, output)?;
            }
            continue;
        }
        let matches = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(extension));
        if matches {
            output.push(path);
        }
    }
    Ok(())
}

/// Extract `(sprite_name, texturefile)` pairs from a `.gfx` document.
///
/// HOI4 sprite registrations put `name` and `texturefile` on their own lines
/// inside `SpriteType`/`frameAnimatedSpriteType` blocks, so a line-oriented scan
/// pairs them reliably without a full Paradox grammar.
fn parse_sprite_registrations(text: &str) -> Vec<(String, String)> {
    let mut registrations = Vec::new();
    let mut pending_name: Option<String> = None;

    for line in text.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, raw_value)) = split_assignment(line) else {
            continue;
        };
        let key = key.to_ascii_lowercase();
        if is_sprite_definition_keyword(&key) {
            // A new sprite block starts here; a dangling `name` belongs to the
            // previous, already-closed block and must not leak into this one.
            pending_name = None;
            continue;
        }
        let value = raw_value.trim_matches('"').trim();
        if value.is_empty() || value == "{" || value == "}" {
            continue;
        }
        if key == "name" {
            pending_name = Some(value.to_string());
        } else if key == "texturefile" {
            if let Some(name) = pending_name.take() {
                registrations.push((name, value.to_string()));
            }
        }
    }

    registrations
}

fn is_sprite_definition_keyword(key: &str) -> bool {
    matches!(
        key,
        "spritetype"
            | "frameanimatedspritetype"
            | "corneredtilespritetype"
            | "textspritetype"
            | "progressbartype"
            | "piecharttype"
            | "linetype"
    )
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, character) in line.char_indices() {
        match character {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn split_assignment(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value.trim()))
}

fn render_sprite_entries(entries: &[GfxSpriteRegistration]) -> String {
    let mut body = String::new();
    for entry in entries {
        body.push_str("\tspriteType = {\n");
        body.push_str(&format!("\t\tname = \"{}\"\n", entry.sprite_name));
        body.push_str(&format!("\t\ttexturefile = \"{}\"\n", entry.image_path));
        body.push_str("\t}\n");
    }
    body
}

fn render_sprite_types(entries: &[GfxSpriteRegistration]) -> String {
    format!("spriteTypes = {{\n{}}}\n", render_sprite_entries(entries))
}

fn write_registrations(target: &Path, entries: &[GfxSpriteRegistration]) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {}", parent.display(), error))?;
    }

    let block = render_sprite_types(entries);
    let body = render_sprite_entries(entries);

    if !target.exists() {
        let header = "## Generated by RHoiScribe register_hoi4_gfx_sprites. Re-running this tool only adds missing sprites.\n";
        fs::write(target, format!("{}{}", header, block))
            .map_err(|error| format!("failed to write {}: {}", target.display(), error))?;
        return Ok(());
    }

    let existing = fs::read_to_string(target)
        .map_err(|error| format!("failed to read {}: {}", target.display(), error))?;

    match sprite_types_block(&existing) {
        Some((open, close)) => {
            let indented = indent_block(&body);
            let prefix = &existing[..open];
            let suffix = &existing[close..];
            let mut merged = String::with_capacity(existing.len() + body.len());
            merged.push_str(prefix);
            merged.push('\n');
            merged.push_str(&indented);
            merged.push('\n');
            merged.push_str(suffix);
            fs::write(target, merged)
                .map_err(|error| format!("failed to write {}: {}", target.display(), error))?;
        }
        None => {
            let mut merged = existing;
            if !merged.ends_with('\n') {
                merged.push('\n');
            }
            merged.push('\n');
            merged.push_str(&block);
            fs::write(target, merged)
                .map_err(|error| format!("failed to write {}: {}", target.display(), error))?;
        }
    }
    Ok(())
}

/// Locate the last `spriteTypes = { ... }` block and return
/// `(index_after_opening_brace, index_of_closing_brace)`.
fn sprite_types_block(text: &str) -> Option<(usize, usize)> {
    let mut search_from = 0;
    let mut found = None;
    while let Some(offset) = text[search_from..].find("spriteTypes") {
        let start = search_from + offset;
        let after_keyword = start + "spriteTypes".len();
        let open = text[after_keyword..]
            .find('{')
            .map(|index| after_keyword + index)?;
        let close = matching_brace(text, open);
        found = close.map(|close| (open + 1, close));
        search_from = after_keyword;
    }
    found
}

fn matching_brace(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut characters = text[open..].char_indices();
    while let Some((offset, character)) = characters.next() {
        match character {
            '"' => in_string = !in_string,
            '#' if !in_string => {
                for (_, next) in characters.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn indent_block(body: &str) -> String {
    body.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("\t{}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn sanitize_sprite_token(value: &str) -> String {
    let mut token = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            token.push(character);
        } else if character == '_'
            || character == '-'
            || character == ' '
            || character == '/'
            || character == '\\'
        {
            token.push('_');
        }
    }
    while token.contains("__") {
        token = token.replace("__", "_");
    }
    token.trim_matches('_').to_string()
}

fn normalize_relative_directory(value: &str) -> Result<String, String> {
    let normalized = normalize_relative_path(value)?;
    Ok(normalized.trim_end_matches('/').to_string())
}

fn normalize_relative_path(value: &str) -> Result<String, String> {
    let candidate = value.trim().replace('\\', "/");
    if candidate.is_empty()
        || candidate.starts_with('/')
        || candidate.contains(':')
        || candidate
            .chars()
            .any(|character| character.is_control() || character == '"')
        || candidate
            .split('/')
            .any(|segment| segment == ".." || segment == ".")
    {
        return Err(format!("unsafe relative path `{}`", value));
    }
    Ok(candidate)
}

fn to_manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sprite_pairs() {
        let text = "spriteTypes = {\n\tspriteType = {\n\t\tname = \"GFX_a\"\n\t\ttexturefile = \"gfx/interface/a.png\"\n\t}\n\tframeAnimatedSpriteType = {\n\t\tname = \"GFX_b\"\n\t\ttexturefile = \"gfx/interface/b.png\"\n\t}\n}\n";
        let parsed = parse_sprite_registrations(text);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].0, "GFX_a");
        assert_eq!(parsed[1].1, "gfx/interface/b.png");
    }

    #[test]
    fn ignores_commented_entries() {
        let text = "spriteTypes = {\n\t# name = \"GFX_gone\"\n\t# texturefile = \"gfx/interface/gone.png\"\n}\n";
        assert!(parse_sprite_registrations(text).is_empty());
    }

    #[test]
    fn finds_last_sprite_types_block() {
        let text = "spriteTypes = {\n}\n\nspriteTypes = {\n\tspriteType = {\n\t}\n}\n";
        let (open, close) = sprite_types_block(text).expect("block");
        assert!(open < close);
        assert_eq!(&text[close..close + 1], "}");
    }

    #[test]
    fn sanitizes_tokens() {
        assert_eq!(sanitize_sprite_token("my icon-1"), "my_icon_1");
        assert_eq!(sanitize_sprite_token("图标"), "");
    }

    #[test]
    fn rejects_traversal() {
        assert!(normalize_relative_path("../etc").is_err());
        assert!(normalize_relative_path("/abs").is_err());
        assert!(normalize_relative_path("gfx/interface").is_ok());
    }

    #[test]
    fn registers_missing_images_and_is_idempotent() {
        let root = temp_workspace("gfx-register");
        fs::create_dir_all(root.join("interface")).unwrap();
        fs::create_dir_all(root.join("gfx/interface/icons/nested")).unwrap();
        fs::write(
            root.join("interface/existing.gfx"),
            "spriteTypes = {\n\tspriteType = {\n\t\tname = \"GFX_existing\"\n\t\ttexturefile = \"gfx/interface/icons/existing.png\"\n\t}\n}\n",
        )
        .unwrap();
        for image in [
            "gfx/interface/icons/existing.png",
            "gfx/interface/icons/new_focus.png",
            "gfx/interface/icons/nested/deep.dds",
        ] {
            fs::write(root.join(image), [0u8; 4]).unwrap();
        }

        let request = RegisterHoi4GfxSpritesRequest {
            workspace_root: root.to_string_lossy().to_string(),
            image_directory: "gfx/interface/icons".to_string(),
            gfx_file: None,
            sprite_prefix: None,
            name_style: None,
            uppercase: None,
            recursive: None,
            dry_run: true,
        };
        let preview = register_hoi4_gfx_sprites(request.clone()).expect("dry run");
        assert_eq!(preview.scanned_images, 3);
        assert_eq!(preview.already_registered.len(), 1);
        assert_eq!(preview.generated.len(), 2);
        assert_eq!(preview.generated[0].sprite_name, "GFX_deep");
        assert!(!preview.applied);
        assert!(!root.join("interface/rhoiscribe_sprites.gfx").exists());

        let applied = register_hoi4_gfx_sprites(RegisterHoi4GfxSpritesRequest {
            dry_run: false,
            ..request.clone()
        })
        .expect("apply");
        assert!(applied.applied);
        let written = fs::read_to_string(root.join("interface/rhoiscribe_sprites.gfx")).unwrap();
        assert!(written.contains("name = \"GFX_new_focus\""));
        assert!(written.contains("texturefile = \"gfx/interface/icons/nested/deep.dds\""));
        assert!(written.contains("spriteTypes = {"));

        let second = register_hoi4_gfx_sprites(RegisterHoi4GfxSpritesRequest {
            dry_run: false,
            ..request
        })
        .expect("second run");
        assert!(second.generated.is_empty());
        assert_eq!(second.already_registered.len(), 3);

        let _ = fs::remove_dir_all(&root);
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "rhoiscribe-{}-{}-{}",
            label,
            std::process::id(),
            stamp
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }
}
