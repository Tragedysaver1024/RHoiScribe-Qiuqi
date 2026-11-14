<div align="center">

<a href="https://github.com/czxieddan/RHoiScribe"><img align="left" src="https://i.imgur.com/o72xNas.png" alt="RHoiScribe" width="95"></a>
<a href="https://www.gnu.org/licenses/agpl-3.0.html"><img align="right" src="https://i.imgur.com/9hl34Lt.png" alt="GNU AGPLv3" width="95"></a>

<p align="center">
<h3><strong>An AGPL-3.0 edition based on RHoiScribe and the Qiuqi Library</strong></h3>
This repository is a <strong>modified edition (fork)</strong> of RHoiScribe, not the upstream project, and it does not speak for the upstream author.<br>
It is distributed under the <strong>GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)</strong>, the same license as upstream.
</p>

<br clear="both">

</div>

## Upstream project

| Field | Value |
| --- | --- |
| Name | RHoiScribe — local MCP server and SKILL for Hearts of Iron IV modding agents |
| Author | CzXieDdan |
| Repository | https://github.com/czxieddan/RHoiScribe |
| License | GNU AGPL-3.0-or-later |
| Copyright | Copyright (C) 2026 CzXieDdan |

This edition keeps the upstream license and copyright notices; `LICENSE` is unmodified.

## Knowledge source: the Qiuqi Library

The knowledge topics added by this edition are rewritten from **秋起图书馆 / QIUQI LIBRARY** ("Autumn Library"), a Chinese-language Hearts of Iron IV modding library distributed as a Steam Workshop item:

- Steam Workshop item ID: **3445449478**
- Game app ID: 394360 (Hearts of Iron IV)
- Link: https://steamcommunity.com/sharedfiles/filedetails/?id=3445449478

The library is a curated collection of community tutorials, templates, code prompt sheets and tools. Every topic under `resources/knowledge/hoi4/**` records the document it was condensed from in its `source_refs` field, so each claim can be traced back to a specific file.

**No game files, no binary assets and no verbatim long-form tutorial text are redistributed here.** Bodies are English rewrites checked against the source; Chinese is kept in `tags` as search synonyms, because knowledge search is a case-insensitive substring match and therefore matches Chinese keywords directly.

## Changes in this edition

Date: 2026-11-14

1. **Knowledge catalog expanded** from 66 topics / 117 KB to 85 topics / 275 KB. Added: faction goals, rules, templates and initiative; power balance (BOP); collections; loops and array operations; event targets; scripted GUI super-events and window framework; version migration (1.13 / 1.15 / 1.16.9 → 1.17.3); define overriding; special projects; equipment modules and designs; building slots and construction; character traits; ideas and modifier reference; OOB and start-of-game setup; bookmark scenario selection; reversed AI strategy; AI areas and AI templates.
2. **New MCP tool** `register_hoi4_gfx_sprites`: scans an existing image directory inside a mod workspace, reads every `interface/**/*.gfx` first, and appends only the missing `spriteType` registrations. Idempotent by texture path, and it reports already-registered sprites, orphaned registrations whose texture file is gone, and skipped name collisions. Ships with unit and end-to-end tests.
3. **Build fix** in `build.rs`: the Windows icon/version resource is now optional. With no resource compiler available (`llvm-rc`, a Windows SDK `rc.exe`, or `RC`), the build prints a warning and links no resource instead of panicking, so a plain Rust toolchain can compile the project; a compiler that is found but fails is still a hard error.
4. **Documentation**: `docs/features.*` (en, zh-CN, ja, ru) cover the new tool and the knowledge catalog; `CONTRIBUTING.md` explains the optional resource compiler; the README and the localized READMEs carry the modified-edition notice.

## Implementation assistance

The changes listed above were implemented with assistance from a DeepSeek Harness agent, running locally on the maintainer's machine:

- Runtime: [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness)
- Model: `deepseek-flash`
- Date: 2026-11-14
- Scope: knowledge catalog expansion, the `register_hoi4_gfx_sprites` tool, the cross-platform `build.rs` fix, documentation updates.

This is a credit line, not a GitHub account: every commit in this repository is authored by the human maintainer, and no authorship, warranty or support for these changes is claimed by DeepSeek or by DeepSeek Harness.

## License obligations

This edition is distributed under AGPL-3.0-or-later:

- The complete corresponding source is this repository; whoever receives a binary is entitled to the source.
- If you modify this edition and run it as a network service, you must offer your modified source to the users of that service.
- Redistribution must keep `LICENSE`, this attribution page and the copyright/source statements in `NOTICE`.

## Trademarks

Hearts of Iron IV is a trademark of Paradox Interactive AB. This repository ships no game content; it contains documentation, prompts and source code only.
