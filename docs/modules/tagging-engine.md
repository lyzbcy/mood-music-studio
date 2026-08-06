# 模块：打标签引擎（tagging-engine）

> **负责**：把音频文件转换为多维度标签（情绪/场景/genre/BPM/valence-arousal）
> **所属路线**：MVP 线 A（自动打标签）
> **依赖**：Essentia、librosa
> **对接**：library-manager（写入 auto_tags 表）、ai-scoring（提供标签数据）
> **调研依据**：`research/01-audio-tagging-engines.md`

---

## 当前状态：⬜ 未开始

| 子任务 | 状态 | 说明 |
|---|---|---|
| A1. Essentia 单首打标 | ⬜ | MTG-Jamendo mood/theme 56 标签 |
| A2. 标签写入 SQLite | ⬜ | auto_tags 表 |
| A3. 批量扫描 + 异步任务 | ⬜ | 10 首起，进度上报 |
| A4. 前端联动 | ⬜ | 导入按钮 + 筛选侧栏 |
| A5. 文档完整化 | ⬜ | 本文件 |

---

## 1. 职责边界

**做什么**：
- 接收音频文件路径 → 调 Essentia 模型 → 输出多维度标签 + 置信度
- 提供「单首分析」「批量分析」「重新分析」三个 API
- 把结果写入 `auto_tags` 表（见 `architecture.md` §3.1）
- 管理模型文件下载与缓存

**不做什么**（其他模块负责）：
- 文件导入/去重/索引 → library-manager
- 向量化（CLAP embedding） → ai-scoring
- UI 展示 → desktop-gui

---

## 2. 标签体系

采用**多源融合标签**，每条标签记录 `source` 字段以区分来源。

### 2.1 情绪标签（核心）
| 来源模型 | 标签集 | 字段 |
|---|---|---|
| MTG-Jamendo mood/theme | 56 个：happy, sad, dark, aggressive, relaxing, uplifting, emotional, energetic, atmospheric, cinematic, ... | tag_type='mood' |
| moods_mirex | 5 簇（Cluster 1-5） | tag_type='mood_cluster' |
| mood_happy/sad/aggressive/relaxed/party | 5 个二元分类器 | tag_type='mood_binary' |
| **valence/arousal 回归** | 连续值 0-1，二维情绪坐标 | tag_type='valence_arousal', metadata={valence, arousal} |

### 2.2 其他维度
| 维度 | 模型 | 标签集 |
|---|---|---|
| genre | Discogs-Electronica | 400 个细粒度 genre |
| genre (粗) | MTG-Jamendo genre | top50tags 混合 |
| 乐器 | MTG-Jamendo instrument | piano, guitar, drums, strings, ... |
| BPM | Essentia RhythmExtractor | 数值，metadata={bpm} |
| 调性 | KeyExtractor | C major, A minor, ... |

---

## 3. API 接口

```python
# POST /api/tag/analyze
{
  "track_ids": ["uuid1", "uuid2"],   # 或
  "force": false                      # true 则忽略已分析状态，重跑
}
# 返回 task_id，异步执行
→ { "ok": true, "data": { "task_id": "..." } }

# GET /api/tag/task/{task_id}
→ { "ok": true, "data": { "status": "running", "progress": 0.6, "done": 6, "total": 10 } }
```

---

## 4. 实现要点

### 4.1 Essentia 调用示例（待 A1 实现）
```python
from essentia.standard import TensorflowPredictorMusiCNN, TensorflowPredictorMood

# 情绪标签
predictor = TensorflowPredictorMood(graphFilename='models/msd-musicnn-1.pb')
audio = es.MonoLoader(filename='song.mp3')()
activations = predictor(audio)
# activations → 56 个 mood/theme 标签的激活值
```

### 4.2 模型下载策略
- 首次启动按需下载到 `~/Library/Application Support/mood-music-studio/models/`
- 基础集（mood + genre + valence）约 500MB
- 国内可加镜像源（HuggingFace 镜像 / ghproxy）

### 4.3 性能考量
- 单首 3-4 分钟歌曲分析约 5-15 秒（取决于 CPU）
- 批量任务串行跑（Essentia 模型加载重，并行收益低）
- 长任务支持中断 + 断点续传（analysis_status 记录到 SQLite）

---

## 5. 待决项
- [ ] 标签同义词合并（happy / happiness / joyful → 统一？）
- [ ] 置信度阈值（< 0.3 是否丢弃？）
- [ ] 用户手动标签如何与自动标签融合查询
- [ ] beets 插件形式 vs 直接调用（Phase 2 评估）
