# 调研报告 03：免费音乐云存储 & AI 分镜配乐技术路线

> **元数据**
> - 调研时间：2026-08-06
> - 调研者：ZCode subagent（用 WebSearch/WebFetch 实际检索）
> - 用途：① 音乐存储方案选型 ② AI「分镜→配乐」技术路线
> - 结论：已写入 ADR-0001 决策 2（本地优先，R2 备选）和决策 4（LAION CLAP）

---

## 问题一：免费云端存储音乐文件

### 1. GitHub 存音乐的可行性评估

#### 1.1 GitHub 硬性限制

| 限制类型 | 数值 | 说明 |
|---|---|---|
| 单文件 push（警告） | 50 MiB | Git 发警告 |
| 单文件 push（硬限） | **100 MB** | 直接拒绝推送 |
| 仓库推荐大小 | **1 GB**（软限） | 超过 5 GB 可能被 GitHub 联系 |
| Git LFS 单文件（Free/Pro） | 2 GB | |
| Git LFS 单文件（Team / Enterprise） | 4 GB / 5 GB | |
| Git LFS **免费额度** | **1 GB 存储 + 1 GB 带宽/月** | 超额后 ~$5/月买 50GB 包 |
| GitHub Releases 附件 | **单文件 2 GB**（无总量上限） | 唯一真正适合大文件的官方途径 |

#### 1.2 三种「在 GitHub 上放音乐」的玩法
- **直接 commit 进仓库**：单文件 100MB 硬限 → 普通有损 MP3（3-8MB）能放，但 FLAC/无损（30-100MB+）易踩线；仓库很快超 1GB，每个文件进 Git 历史，克隆爆炸。**不推荐**。
- **Git LFS**：能存大文件，但免费仅 1GB 存储 + 1GB 带宽/月，放几十首无损就爆，且 LFS 流式播放需鉴权，不能当 CDN。**不推荐做主存储**。
- **GitHub Releases 附件**：单文件 2GB，每个 release 可放多个附件，**总量基本不限**，公开仓库 release 资产支持 **CDN 直链下载**（`https://github.com/.../releases/download/...`），能被 `<audio>` 直接播放。**这是 GitHub 上唯一靠谱的玩法**。

#### 1.3 结论：GitHub 到底适不适合？
**适合**：个人开发者、音乐总量不大（< 2GB）、想要零成本 + CDN 直链 + 自动化上传（`gh release upload` 一行）、做开源 demo 的背景音乐托管。
**不适合**：音乐库 > 几 GB、需批量管理/检索、需目录结构/版本回滚（release 不支持目录）、商业/正式产品（违反 ToS 精神，可能被封仓库）、国内访问（`github.com` 抽风，`objects.githubusercontent.com` 国内**经常超时/被墙**）。

> **一句话**：GitHub Releases 当「小型免费音乐 CDN + 自动上传」是可行的 hack，但绝不是音乐管理软件的正经存储方案，尤其国内用户访问不稳定。

---

### 2. 更好的免费音乐云存储替代方案（按适合度排序）

| # | 方案 | 免费额度 | 有 API | 支持直链/流式 | 会被封 | 国内访问 | 综合评价 |
|---|---|---|---|---|---|---|---|
| **1** | **Cloudflare R2** | **10 GB 存储**；Class A 100万次/月、Class B 1000万次/月；**流量完全免费** | ✅ S3 兼容 + presigned URL | ✅ 完美支持，零流量费 | 不会 | 较好 | ⭐⭐⭐⭐⭐ **首选** |
| **2** | **Backblaze B2** | **10 GB 存储**；下载 1GB/天免费；API 调用免费 | ✅ S3 兼容 | ✅（配 Cloudflare 走 SF 连接免流量费） | 不会 | 一般 | ⭐⭐⭐⭐ |
| **3** | **AWS S3 + CloudFront** | S3 5GB（仅前 12 个月）；CloudFront **1TB/月永久免费**；S3→CF 流量免费 | ✅ S3 + SDK 全套 | ✅ 完美 | 不会 | 差 | ⭐⭐⭐ 12 个月后要钱 |
| **4** | **Google Drive API** | **15 GB**（与 Gmail/Photos 共享） | ✅ Drive API v3 | ⚠️ 可生成直链但**ToS 禁止当 CDN/媒体托管**，滥用封号 | ⚠️ 封号风险 | 差（被墙） | ⭐⭐ 不建议做生产存储 |
| **5** | **OneDrive（Graph API）** | 5 GB | ✅ Microsoft Graph | ⚠️ 直链复杂，不适合公开流式 | 较宽松 | 国内可达 | ⭐⭐ |
| **6** | **Dropbox API** | 2 GB | ✅ | ⚠️ 直链有带宽限制 | 较宽松 | 差（被墙） | ⭐⭐ |

**对象存储 vs 网盘的核心差别**：对象存储（R2/B2/S3）天然为「API + 直链 + 流式」设计，可生成带签名临时 URL 让 `<audio>`/`<video>` 直接播放，不会被当「滥用网盘」封号；网盘类本质是给人用的，做公开媒体分发违反 ToS。

### 3. 最终推荐

**首选：Cloudflare R2**

理由：
1. **10GB 免费**且永久，对个人音乐库（几百到上千首有损格式）足够
2. **流量完全免费**——音乐流式播放最费流量，这点碾压 S3/B2/所有网盘
3. **S3 兼容 API**，所有主流 SDK（aws-sdk、boto3、minio）直接能用，迁移成本低
4. **presigned URL** 天然支持客户端直传/直链播放，不用自己搭鉴权服务
5. 国内可达性在海外对象存储里算好的（CF 节点多）

**架构建议**：
```
App  ──(presigned PUT URL)──>  R2 存音乐
     <──(presigned GET URL)──   R2 直接给 <audio> 流式播放
后端用 Workers/轻服务 生成签名 + 管理元数据（歌名/标签/向量）
```

**备选**：库再大或要冗余 → **R2 + Backblaze B2 双写**（B2 配 Cloudflare 走 Backbone 连接免流量费）。

**坚决别选**：把 GitHub 当正经存储（封仓+国内访问问题）、Google Drive 当流式 CDN（封号风险）。

---

## 问题二：AI「分镜转配乐 / 视频自动配乐」工具与 API

> 需求：用户输入「分镜提示词」→ 系统从（用户本地库优先 + 网络库）智能筛选每段适合的音乐/音效。
> 本质是 **「文本 → 音乐检索（text-to-music retrieval）」** 任务，生成型只是补充。

### 1. 生成型 AI 音乐工具（适合「找不到就现生成」）

| 名称 | 类型 | API | 免费 | 现状 | 可复用价值 |
|---|---|---|---|---|---|
| **Suno** | 生成 | ❌ 无官方 API（第三方代理 musicapi.ai / UdioAPI.pro 等，$0.014–0.11/首） | 有限免费 | 商业最强 | 低（无官方 API，代理不稳） |
| **Udio** | 生成 | ❌ 同上 | 有限免费 | 质量高 | 低 |
| **Google MusicFX / Lyria** | 生成 | ❌ 无公开 API；2026 收购 ProducerAI，转型「Google Flow Music」 | 免费网页工具 | 实验性，随时可能下线；带 SynthID 水印 | 低 |
| **OpenAI MuseNet** | 生成 | ❌ | — | **已停服（2022-12-12）** | 无 |
| **Beatoven.ai / Mubert / Loudly** | 生成（BGM 定位） | ✅ 部分有 | $3–14/月起 | 中 | 中（BGM 场景可考虑） |

**结论**：生成型 API 全部不免费/无官方 API，且对「从已有库检索」核心需求**帮助不大**。更适合「库里实在没有，临时造一段」。

### 2. 音乐检索 / 推荐 API（适合「从网络库挑」）

| 名称 | 类型 | API | 免费 | 关键点 |
|---|---|---|---|---|
| **Last.fm API** | 检索/推荐 | ✅ [last.fm/api](https://www.last.fm/api) | ✅ 完全免费 | `track.getSimilar` / `artist.getSimilar` / `tag.getSimilar` 做相似推荐；海量标签数据，可复用度高 |
| **Spotify Recommendations API** | 检索 | ❌ | — | **已废弃**，返回 404；不可依赖 |
| **网易云 / QQ 音乐** | 检索 | ⚠️ 非官方 | 灰色 | 无官方开放 API，逆向不稳，版权风险大 |
| **Epidemic Sound API** | 检索（商用版权音乐） | ✅ [developers.epidemicsound.com](https://developers.epidemicsound.com/) | ✅ 免费 API key | **55 万曲 + 25 万音效**，支持「用自然语言描述声音」做 text-to-sound 搜索，自带「自动配乐工作流」可按时间线放置音乐和音效 —— **和需求高度契合** |
| **Freesound API** | 检索（CC 音效） | ✅ [freesound.org/docs/api](https://freesound.org/docs/api/) | ✅ 免费 | 80 万+ CC 音效/采样，适合做音效库（脚步、环境、冲击），含基于内容的相似搜索 |

> **强烈建议先看 Epidemic Sound API**：专为「视频自动配乐」设计，自带文本搜索 + 时间线放置，能省掉一大半自研工作。

### 3. 开源「音频-文本」跨模态模型（核心引擎，自建本地库检索）

实现「**输入文字提示词 → 从用户本地音乐库语义检索最匹配片段**」的关键。原理：用模型把音频和文本都映射到同一个向量空间，算余弦相似度。

| 模型 | GitHub | 训练数据 | 能力 | 可复用度 | 备注 |
|---|---|---|---|---|---|
| **LAION CLAP** ⭐ | [LAION-AI/CLAP](https://github.com/LAION-AI/CLAP) | LAION-Audio-630K（63万音频-文本对） | 文本↔音频检索、零样本分类 | ⭐⭐⭐⭐⭐ **首选** | 已集成进 HuggingFace Transformers（`ClapModel`），有 `clap-htsat-fused` 预训练权重，Python 几行提特征；社区最活跃 |
| **AudioCLIP** | [AndreyGuzhov/AudioCLIP](https://github.com/AndreyGuzhov/AudioCLIP) | AudioSet + ESC-50 | 文本/图像/音频三模态对齐 | ⭐⭐⭐ | 2021 年，偏环境音分类，音乐专精度不如 CLAP；仓库较老 |
| **WavCaps** | [XinhaoMei/WavCaps](https://github.com/XinhaoMei/WavCaps) | 40 万弱标注音频-文本对 | **数据集**（不是模型） | ⭐⭐⭐ | 主要价值是拿来微调 CLAP，提升中文/音乐描述效果 |
| **CLaMP 3**（ACL 2025） | 论文 [aclanthology.org/2025.findings-acl.133](https://aclanthology.org/2025.findings-acl.133.pdf) | — | 专为**音乐信息检索**设计，跨模态+跨语言 | ⭐⭐⭐⭐ | 最新，针对 MIR 任务，值得跟踪 |
| **MuQ-MuLan** | — | — | 音乐专用文本-音频对齐 | ⭐⭐⭐ | 音乐领域专用，可作 CLAP 替补 |

**LAION CLAP 为什么首选**：
- 预训练权重直接可用，无需自训
- 文本编码器（RoBERTa）+ 音频编码器（HTSAT）已对齐
- 典型用法：
  ```
  1. 离线：遍历本地音乐库 → 切片 → CLAP 音频编码器 → 向量 → 入向量库
  2. 在线：分镜提示词 → CLAP 文本编码器 → 向量 → 向量库 ANN 检索 → Top-K 片段
  ```
- 中文支持注意：原版文本编码器偏英文，中文提示词建议先用翻译模型或换中文友好的文本编码器微调

### 4. 推荐技术路线：文字提示词 → 智能检索用户本地音乐库

```
分镜提示词（如："雨夜都市，主角独行，忧伤悬疑"）
        │
        ▼
① 提示词归一化（可选）
   - LLM 把口语分镜 → 标准音乐描述
   - 提取情绪/节奏/乐器/BPM 标签
        ▼
② 语义检索核心：LAION CLAP
   提示词 → CLAP 文本编码器 → query 向量
        ▼
③ 向量库 ANN 检索（用户本地库优先）
   引擎：Chroma / Qdrant / Milvus / LanceDB
   每首音乐预切成 10-30s 片段入库
   返回 Top-K 片段 + 时间戳 + 相似度
        ▼
④ 兜底/补充：网络库
   - Epidemic Sound API（商用音乐+音效）
   - Freesound API（CC 音效）
   - Last.fm getSimilar（风格扩展）
   库里实在没有 → Suno/Udio 生成（可选）
        ▼
⑤ 音效层叠加（对白外的环境音/冲击音）
   Freesound + CLAP 同一套向量检索
        ▼
分镜 → 配乐 + 音效 时间线
```

**组件清单（可直接组合）**：
1. **语义引擎**：[LAION-AI/CLAP](https://github.com/LAION-AI/CLAP)（HuggingFace `ClapModel`）
2. **向量库**：LanceDB（单机轻量，适合本地音乐库）或 Qdrant
3. **音频切片/解码**：`librosa` / `pydub` / `ffmpeg`
4. **网络库补充**：[Epidemic Sound API](https://developers.epidemicsound.com/) + [Freesound API](https://freesound.org/docs/api/)
5. **风格扩展**：[Last.fm API](https://www.last.fm/api) `getSimilar`
6. **提示词归一化**：任意 LLM（GLM/GPT）把分镜口语转成 CLAP 友好的英文音乐描述

**MVP 路径**：
- 第一周：CLAP + 本地 50 首音乐入库，跑通「文字 → 片段」
- 第二周：接 Freesound 音效层
- 第三周：接 Epidemic Sound 做商用兜底
- 生成型（Suno/Udio）作为「最后兜底」按需加，**不要当主力**（贵、无官方 API、慢）

---

## 关键结论速查

| 问题 | 结论 |
|---|---|
| GitHub 适合存音乐吗？ | **Releases 附件（2GB/文件）能当小规模免费 CDN 用，但不适合做正经音乐库**；国内访问不稳、有封仓风险 |
| 音乐文件存哪最好？ | **Cloudflare R2**（10GB 免费 + 零流量费 + S3 API + presigned 直链播放）。MVP 阶段本地优先 |
| AI 自动配乐怎么做？ | **LAION CLAP 语义检索 + Epidemic Sound/Freesound 网络库 + LLM 归一化提示词**；生成型只兜底 |
| 用 Spotify API 吗？ | **别用，推荐接口已废弃返回 404**，改用 Last.fm |
| 用 Suno/Udio API 吗？ | 无官方 API，第三方代理不稳；仅作「找不到就生成」的最后兜底 |

---

## 主要参考来源

- GitHub 限制：[About large files on GitHub](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)、[GitProtect GitHub Storage Limits](https://gitprotect.io/blog/github-storage-limits/)
- R2：[Cloudflare R2 Pricing](https://developers.cloudflare.com/r2/pricing/)
- B2：[Backblaze B2 Pricing](https://www.backblaze.com/cloud-storage/pricing)
- S3：[AWS S3 Pricing](https://aws.amazon.com/s3/pricing/)、[AWS Free Tier 1TB CloudFront](https://aws.amazon.com/blogs/aws/aws-free-tier-data-transfer-expansion-100-gb-from-regions-and-1-tb-from-amazon-cloudfront-per-month/)
- Google Drive：[Drive API Limits](https://developers.google.com/workspace/drive/api/guides/limits)
- LAION CLAP：[GitHub](https://github.com/LAION-AI/CLAP)、[HuggingFace docs](https://huggingface.co/docs/transformers/en/model_doc/clap)
- AudioCLIP：[GitHub](https://github.com/AndreyGuzhov/AudioCLIP)、[arXiv 2106.13043](https://arxiv.org/abs/2106.13043)
- WavCaps：[GitHub](https://github.com/XinhaoMei/WavCaps)、[HuggingFace dataset](https://huggingface.co/datasets/cvssp/WavCaps)
- CLaMP 3：[ACL 2025 paper](https://aclanthology.org/2025.findings-acl.133.pdf)
- Epidemic Sound：[developers.epidemicsound.com](https://developers.epidemicsound.com/)
- Freesound：[API docs](https://freesound.org/docs/api/)
- Last.fm：[API portal](https://www.last.fm/api)
- Spotify 推荐接口废弃：[社区讨论](https://community.spotify.com/t5/Spotify-for-Developers/Recent-api-changes/td-p/6596884)
- MuseNet 停服 / MusicFX 现状：[openai.com/index/musenet](https://openai.com/index/musenet/)
