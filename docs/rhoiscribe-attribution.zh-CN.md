<div align="center">

<a href="https://github.com/czxieddan/RHoiScribe"><img align="left" src="https://i.imgur.com/o72xNas.png" alt="RHoiScribe" width="95"></a>
<a href="https://www.gnu.org/licenses/agpl-3.0.html"><img align="right" src="https://i.imgur.com/9hl34Lt.png" alt="GNU AGPLv3" width="95"></a>

<p align="center">
<h3><strong>基于 RHoiScribe 与秋起图书馆的 AGPL-3.0 改版</strong></h3>
本仓库是 RHoiScribe 的<strong>改版（fork）</strong>，不是上游原项目，也不代表上游作者的立场。<br>
它以 <strong>GNU Affero 通用公共许可证第 3 版或更高版本（AGPL-3.0-or-later）</strong> 发布，与上游一致。
</p>

<br clear="both">

</div>

## 上游项目

| 项目 | 说明 |
| --- | --- |
| 名称 | RHoiScribe —— 面向 Hearts of Iron IV modding agents 的本地 MCP 服务器与 SKILL |
| 作者 | CzXieDdan |
| 仓库 | https://github.com/czxieddan/RHoiScribe |
| 许可证 | GNU AGPL-3.0-or-later |
| 版权 | Copyright (C) 2026 CzXieDdan |

本改版保留上游的许可证与版权声明，`LICENSE` 文件未作修改。

## 内容来源：秋起图书馆

本改版新增的知识条目，改写自 **秋起图书馆 / QIUQI LIBRARY（霜泽图书馆）** —— 一个面向中文本地化 HOI4 模组制作的资料库，以 Steam 创意工坊条目形式发布：

- Steam 创意工坊条目 ID：**3445449478**
- 游戏 App ID：394360（Hearts of Iron IV）
- 链接：https://steamcommunity.com/sharedfiles/filedetails/?id=3445449478

该资料库汇集了社区教程、模板、代码提词与工具。本仓库在 `resources/knowledge/hoi4/**` 下，把每个主题所依据的原始文档路径记录在 `source_refs` 字段中，因此每一条结论都能回溯到具体文件。

**这里不转载游戏本体文件、不转载二进制资源、也不转载长篇教程原文**：条目正文是按原文核对后重新撰写的英文摘要，中文只保留在 `tags` 中作为检索同义词（搜索为子串匹配，中文关键词可直接命中）。

## 本次改动

日期：2026-11-14

1. **知识库扩充**：66 篇 / 117 KB → 85 篇 / 275 KB。新增阵营（goal / rule / template / initiative）、权力平衡 BOP、collections、循环与数组操作、event_target、脚本化 GUI 超事件与窗口框架、版本迁移（1.13 / 1.15 / 1.16.9 → 1.17.3）、defines 覆盖、特殊工程、装备模块与设计、建筑槽位、人物特质、国家精神与修正、OOB 与开局设置、剧本选择、逆向 AI 策略、AI 区域与模板。
2. **新增 MCP 工具** `register_hoi4_gfx_sprites`：扫描模组工作区内已有的图片目录，先读取全部 `interface/**/*.gfx`，只把缺失的 `spriteType` 注册追加到 `.gfx` 文件。按纹理路径去重，因此重复执行结果一致；同时报告已注册项、纹理文件已丢失的孤儿项与命名冲突跳过项。附单元测试与端到端测试。
3. **构建修复** `build.rs`：Windows 图标 / 版本资源改为可选。找不到资源编译器（`llvm-rc`、Windows SDK 的 `rc.exe` 或 `RC` 环境变量）时只打印 warning 并跳过，不再 panic，使纯 Rust 工具链也能编译；若编译器存在但执行失败，仍然报错。
4. **文档**：`docs/features.*`（英 / 中 / 日 / 俄）补充新工具与知识库章节；`CONTRIBUTING.md` 说明资源编译器可选；README 与各语言 README 增加改版声明。

## 许可证义务

本改版依据 AGPL-3.0-or-later 分发：

- 完整的对应源码即本仓库，任何人获取二进制的同时都有权获取源码。
- 若你修改本改版并以网络服务形式提供，必须向该服务的使用者提供你修改后的源码。
- 分发时必须保留 `LICENSE`、本署名页与 `NOTICE` 中的版权与来源声明。

## 商标

Hearts of Iron IV 是 Paradox Interactive AB 的商标。本仓库不包含任何游戏内容，仅包含文档、提示词与源代码。
