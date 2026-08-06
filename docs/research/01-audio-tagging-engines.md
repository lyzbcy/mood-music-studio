# 调研报告 01：音乐自动打标签 / 情绪识别 / 库管理

> **元数据**
> - 调研时间：2026-08-06
> - 调研者：ZCode subagent（用 GitHub API 实时核实 star 数与活跃度）
> - 调研范围：GitHub + 全网
> - 用途：为「打标签引擎」模块（`modules/tagging-engine.md`）选型提供依据
> - 结论：已写入 ADR-0001 决策 3，选用 **Essentia**

---

## 核心结论

要做一个「自动打情绪标签的本地音乐库」桌面软件，**最值得复用的组件**：

1. **Essentia（MTG/essentia）** —— 唯一一个同时提供「音频特征提取 + 内置预训练 mood/genre 模型 + 离线运行」的一体化库。自带 MTG-Jamendo 56 个 mood/theme 标签、Discogs 400 genre 标签、Russell 情绪环的 valence/arousal 回归。是整个项目的「引擎」。
2. **WB2024/Essentia-to-Metadata** —— 已把 Essentia 模型接到「写回音频文件标签」的现成参考实现（MP3/FLAC/M4A 全覆盖），正好验证目标场景，可 fork 改造。
3. **beets（beetbox/beets）** —— Python 音乐库管理 + MusicBrainz 自动标签器，插件系统成熟。作为「库管理 + 文件组织 + 元数据补全」的底座候选。

辅助数据源：MusicBrainz + AcoustID（免费，识别「这首歌是什么」）+ Last.fm（免费，补充众包 mood tag）。

---

## 一、音乐自动打标签 / 情绪识别开源项目

### 1. Essentia —— 本项目核心引擎 ⭐⭐⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/MTG/essentia |
| Star | 3,680（实时核实） |
| 最近推送 | 2026-07-22（**活跃维护**） |
| 技术栈 | C++ 核心库 + Python 绑定，TensorFlow 集成 |
| License | **AGPL-3.0**（注意：传染性 copyleft，商业闭源分发需谨慎） |
| 出品方 | MTG（Music Technology Group，UPF 巴塞罗那庞培法布拉大学） |

**核心能力**：音乐信息检索（MIR）领域最权威的开源库，既提供传统 DSP 特征（MFCC、频谱、节奏、调性），又内置一整套 TensorFlow 预训练模型。关键：已把模型和标签体系打包好，`pip install essentia` 即用。

**内置的情绪/genre 模型**（来自官方 models 页）：
- **MTG-Jamendo 情绪/主题**：56 个 mood/theme 标签（happy、sad、dark、aggressive、relaxing、uplifting、emotional、energetic、atmospheric、cinematic 等）—— 正是要的「情绪/场景分类」
- **MTG-Jamendo top50tags**：genre + mood + instrument 混合 50 标签
- **Discogs-Electronica**：400 个细粒度 genre 标签
- **MagnaTagATune (MTT)**：188 标签，含 mood
- **MSD (Million Song Dataset)**：自动打标签分类器
- **moods_mirex**：5 个情绪簇（Cluster 1-5，MIREX 标准）
- **mood_happy / mood_sad / mood_aggressive / mood_relaxed / mood_party**：5 个独立二元情绪分类器
- **Arousal/Valence 回归**：基于 DEAM / EmoMusic / MuSe，直接输出 Russell 情绪环坐标（0~1 唤醒度 + 效价度）—— 学术界最通用的二维情绪模型，能精细定位「欢快-平静-紧张-悲伤」四象限

**复用建议**：直接用 Essentia 的 `TensorflowPredictorMusiCNN` / `TensorflowPredictorMood` 系列算法加载预训练模型，一行 Python 拿情绪预测。情绪标签首选 MTG-Jamendo mood/theme（最丰富），配合 valence/arousal 做连续情绪值。**注意 AGPL 协议**，桌面软件若要闭源商业发行，需评估 SaaS/隔离调用或商用授权。

---

### 2. librosa —— 特征提取基础库 ⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/librosa/librosa |
| Star | 8,543（实时核实） |
| 最近推送 | 2026-08-03（**非常活跃**） |
| 技术栈 | 纯 Python（NumPy/SciPy） |
| License | ISC（宽松，可商用） |

**核心能力**：Python 生态最流行的音频/MIR 信号处理库，提供 MFCC、mel 频谱、色度（chroma）、onset、tempo 等全套特征提取。**不自带预训练模型**，只是「造砖的工具」，需要自己训练分类器。

**复用建议**：走「自训练情绪模型」路线时是特征提取首选（比 Essentia 轻、纯 Python、无编译）。但如果直接用现成模型，Essentia 已包含同等特征能力，可不必再引。

---

### 3. madmom —— 节奏/音符检测专家 ⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/CPJKU/madmom |
| Star | 1,689（实时核实） |
| 最近推送 | 2026-03-20（维护偏缓） |
| 技术栈 | Python + Cython |
| License | 代码 BSD；**模型 CC BY-NC-SA 4.0（禁止商用）** |

**核心能力**：维也纳 JKU 出品，专注 beat/onset/chord/音符 onset 检测，深度学习模型质量高。

**复用建议**：与情绪/genre 关系不大，**情绪项目基本不需要**。除非要做 BPM/节奏稳定性这类节拍派生情绪特征。注意 NC（非商业）模型协议。

---

### 4. minzwon/sota-music-tagging-models —— SOTA 打标签模型集 ⭐⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/minzwon/sota-music-tagging-models |
| Star | 439 |
| 最近推送 | 2023-11（代码冻结，论文模型仍是基准） |
| 技术栈 | PyTorch |
| License | **MIT**（宽松） |

**核心能力**：作者 Minz Won（Essentia 模型论文作者）把业内 9 个 SOTA 自动打标签模型用 PyTorch 统一实现：FCN、**Musicnn**、Sample-level CNN、Sample-level CNN+SE、CRNN、Self-attention、**Harmonic CNN**、**Short-chunk CNN**、Short-chunk CNN+Residual。

论文（Won et al. 2020）在 692 个标签（含 mood/theme）上评测，作者建议：
- 数据少 → 用 **Musicnn**
- 求简单且效果好 → **Short-chunk CNN + Residual**
- 求泛化最佳 → **Harmonic CNN**

**复用建议**：若不想被 Essentia 的 AGPL 绑定，这套 PyTorch + MIT 是更干净的替代。但需要自己接数据训练或下载 checkpoint。**实际上 Essentia 内置模型就是这套的训练产物**，所以更省事仍是直接用 Essentia。

---

### 5. WB2024/Essentia-to-Metadata —— 本地自动打标签的现成参考实现 ⭐⭐⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/WB2024/Essentia-to-Metadata |
| Star | 109 |
| 最近推送 | 2026-04-04（**近期活跃**） |
| 技术栈 | Python（Essentia + Mutagen + NumPy） |
| License | 代码 MIT；依赖 Essentia AGPL，预训练模型 CC BY-NC-ND 4.0 |

**核心能力**：**这正是目标场景的开源雏形** —— 用 Essentia Discogs 模型预测 400 个 genre、用 MTG-Jamendo 模型检测 mood（happy/dark/energetic 等），然后直接写入音频文件标签（Vorbis Comments for FLAC/OGG、ID3v2 for MP3/WAV、iTunes atoms for M4A）。**完全离线运行**，初始下载模型后无需联网。

**复用建议**：**强烈建议 fork 作为起点**。已解决「模型预测 → 标签写入 → 多格式适配」工程问题，只需补「批量扫描 + 按情绪筛选/播放的 UI」。省下数周工程活。

---

### 6. AMAAI-Lab/awesome-MER —— 情绪识别资源索引 ⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/AMAAI-Lab/awesome-MER |
| Star | 79 |
| 最近推送 | 2025-04 |

**核心能力**：Music Emotion Recognition 的 awesome 列表，汇总数据集（DEAM 2058、PMEmo 794、EMOPIA、MuSe 9 万、MTG-Jamendo 1.8 万、MSD Last.fm 子集 50 万等）和方法。

**复用建议**：不作为生产工具，而是「想自研情绪模型时找数据集/论文」的入口。用 Essentia 现成模型基本用不上。

---

### 7. 其他情绪识别仓库（可复用价值：低）

仅作了解，不建议直接复用：
- `rxng8/Music-Emotion-Recognition-Algorithm`
- `seungheondoh/audio-lyrics-emotion-recognition`（音频+歌词双模态）
- `saeedehj/mood-audio-classification`（VGGish）

---

## 二、音乐库管理 + 标签系统

### 1. beets —— 最值得集成的库管底座 ⭐⭐⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/beetbox/beets |
| Star | 15,502（实时核实） |
| 最近推送 | 2026-08-05（**昨天刚更新，极其活跃**） |
| 技术栈 | Python |
| License | **MIT**（宽松，可商用） |

**核心能力**："为强迫症音乐爱好者而生的媒体库管理系统"。基于 MusicBrainz 自动打标 + 重命名 + 整理 + 去重。**插件生态是核心卖点**：官方插件包括 `chroma`（AcoustID 指纹识别）、`lastgenre`（从 Last.fm 拉 genre）、`lyrics`、`replaygain`、`duplicates`、`fetchart`（封面）等，支持「inline 插件」。

**标签体系**：基于 MusicBrainz 标准 tag schema。**原生不支持 mood 标签筛选**，但插件系统完全开放，可写 beets 插件调 Essentia 打 mood 标签写入 `mood` 字段，然后用 `beet ls mood:happy` 筛选。

**复用建议**：**作为桌面软件的「后端引擎」候选**。beets 已有命令行 + Web UI（`beet web`），只需：① 写 beets 插件调 Essentia 注入 mood 标签；② 在它之上套桌面 GUI。MIT 协议无障碍。

---

### 2. MusicBrainz Picard —— 桌面端标杆 ⭐⭐⭐

| 项 | 内容 |
|---|---|
| GitHub | https://github.com/metabrainz/picard |
| Star | 5,077（实时核实） |
| 最近推送 | 2026-08-05（**今天刚更新**） |
| 技术栈 | Python（PyQt 桌面 GUI） |
| License | GPL-2.0+（copyleft） |

**核心能力**：MusicBrainz 官方桌面标签器，跨平台 GUI。「album-oriented」+ AcoustID 声纹识别 + CD lookup 三路匹配，批量回填标签、嵌入封面。

**复用建议**：作为同类产品对标和参考（尤其 AcoustID 集成、tag 写回、多格式支持代码）。**不建议直接 fork** —— 完整 GUI 应用而非库，GPL-2 协议较严。

---

### 3. 其他（可复用价值：低）

- **Quod Libet**（Python/GTK 桌面播放器）：以「灵活的任意 tag 查询」著称，原生支持自定义 mood tag 筛选，适合做交互参考
- **Navidrome / SwingMusic / sensibleHub**：自托管 Web 音乐服务器，偏向流媒体播放而非打标签

---

## 三、音乐元数据 API / 标签数据源

| API | 用途 | 免费? | 是否需要 Key | 关键说明 |
|---|---|---|---|---|
| **MusicBrainz** | 曲目元数据权威来源 | 完全免费 | 不需 key，要求 User-Agent，限速 1 req/秒 | 开放数据 CC0。**打标签管线核心数据源** |
| **AcoustID + Chromaprint** | 声纹识别「这首歌是什么」 | 完全免费 | 需 client key（免费注册） | 本地算指纹 → 查 AcoustID 拿 MBID。无文件名/乱命名文件的救命稻草 |
| **Last.fm API** | 众包 tag（含大量 mood/情绪标签） | 免费 | 需免费 API key | 限速约 5 req/秒。`track.getTopTags` 补充情绪标签的好来源 |
| **Spotify Audio Features** | valence、energy、danceability 等 | ⚠️ **已对大多数新应用停用/弃用！** | 需 OAuth token | **重大变化**：已下架对新应用访问。**不能作为稳定数据源**。这反而让「本地用 Essentia 自己算情绪」更有必要 |
| **Discogs API** | genre/style（电子乐细分 400+） | 免费 | 需 token | Essentia Discogs-Electronica 模型的标签来源 |

**关键结论**：情绪/场景标签**不要依赖在线 API**（Spotify 已废、Last.fm 有限速且不稳）。正确架构是**本地 Essentia 模型算情绪**（离线、免费、无限速），MusicBrainz/AcoustID/Last.fm 只用来补「这是什么歌」的身份元数据。

---

## 四、给项目落地的架构建议

基于以上调研，推荐的复用栈：

```
桌面 GUI（Tauri/Electron 或直接 beets web）
        │
beets（库管理 + 文件组织 + 查询筛选）MIT  ← 底座候选
  └─ mood 插件（调下面的引擎写 mood tag）
        │
Essentia（情绪/genre 模型推理）AGPL  ← 引擎
  · MTG-Jamendo mood/theme（56 标签）
  · valence/arousal 回归（二维情绪）
  · Discogs genre（400 标签）
        │（身份补全，可选）
MusicBrainz + AcoustID + Last.fm（免费）  ← 元数据
```

**落地三步走**：
1. **MVP**：直接 fork `WB2024/Essentia-to-Metadata`，加批量扫描目录 + 简单「按 mood 列表筛选」CLI/网页。1-2 周出 demo。
2. **产品化**：引入 beets 作为库管底座，把 Essentia 打标写成 beets 插件，套桌面 GUI。
3. **差异化**：在 valence/arousal 二维情绪平面做「情绪地图」可视化（Spotify Audio Features 停用后本地软件能提供的独特体验），或结合时间/场景做智能歌单。

**合规注意**：Essentia 是 AGPL-3.0，模型部分标签 CC BY-NC-ND（部分非商用）。闭源商业产品需：① 把 Essentia 作为独立进程/服务调用（网络隔离可规避 AGPL 传染）；② 或联系 MTG 谈商用授权；③ 或改用 `sota-music-tagging-models`（MIT）+ 自训/下载 NC 之外的模型。

---

## 关键参考链接

**核心引擎与参考实现**：
- Essentia: https://github.com/MTG/essentia (3.7k star, AGPL, 活跃)
- Essentia 模型页: https://essentia.upf.edu/models.html
- Essentia-to-Metadata（本地打情绪标签现成方案）: https://github.com/WB2024/Essentia-to-Metadata
- sota-music-tagging-models: https://github.com/minzwon/sota-music-tagging-models

**库管理底座**：
- beets: https://github.com/beetbox/beets (15.5k star, MIT, 活跃)
- MusicBrainz Picard: https://github.com/metabrainz/picard (5.1k star, GPL-2)

**数据/资源**：
- MTG-Jamendo 数据集（55k 曲，195 标签含 56 mood）: https://mtg.github.io/mtg-jamendo-dataset/
- awesome-MER（情绪识别论文/数据集索引）: https://github.com/AMAAI-Lab/awesome-MER

**特征提取**：
- librosa: https://github.com/librosa/librosa (8.5k star, ISC)
- madmom: https://github.com/CPJKU/madmom (1.7k star, 节拍相关，情绪项目可不用)

**元数据 API**：
- MusicBrainz: https://musicbrainz.org
- AcoustID（指纹，免费）: https://acoustid.org/
- Last.fm API（众包 tag）: https://www.last.fm/api
