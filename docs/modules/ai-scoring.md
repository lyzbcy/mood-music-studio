# 模块：AI 分镜配乐（ai-scoring）

> **负责**：分镜提示词 → 语义检索用户本地库 → 返回最匹配的音乐片段 + 音效
> **所属路线**：MVP 线 B（AI 分镜配乐）
> **依赖**：LAION CLAP、LanceDB（向量库）、FFmpeg（音频切片）
> **对接**：library-manager（取音乐文件）、tagging-engine（标签补充）、mcp-api（对外暴露）
> **调研依据**：`research/03-storage-and-ai-music.md` §问题二

---

## 当前状态：⬜ 未开始

| 子任务 | 状态 | 说明 |
|---|---|---|
| B1. CLAP 单首 embedding | ⬜ | 加载 clap-htsat-fused |
| B2. LanceDB 建表 + 批量入库 | ⬜ | 切片策略 |
| B3. 文本→向量→Top-K 检索 | ⬜ | 核心能力 |
| B4. 前端提示词面板 + 结果展示 | ⬜ | 片段时间戳 + 播放 |
| B5. 文档完整化 | ⬜ | 本文件 |

---

## 1. 职责边界

**做什么**：
- 把音乐库每首歌切成片段（10-30s），用 CLAP 提取音频向量入 LanceDB
- 接收分镜提示词，用 CLAP 文本编码器转成向量
- 在向量库做 ANN 检索，返回 Top-K 匹配片段
- 可选：用 LLM 把中文/口语提示词归一化为 CLAP 友好英文描述
- 可选：检索 Freesound / Epidemic Sound 网络库作补充

**不做什么**：
- 音乐文件管理 → library-manager
- 标签打标（情绪分类）→ tagging-engine（CLAP 是语义向量，互补）
- 时间线编辑 UI → desktop-gui

---

## 2. 核心流程

```
分镜提示词："雨夜都市，主角独行，忧伤悬疑"
    │
    ▼
① 提示词归一化（可选，LLM 调用）
   → "rainy night city, lonely protagonist, melancholic suspense"
    │
    ▼
② CLAP 文本编码器
   text_features = clap_model.get_text_features([normalized_prompt])
   → query_vector [512]
    │
    ▼
③ LanceDB ANN 检索
   table.search(query_vector).limit(top_k).where("source = 'user_library'")
   → Top-K 片段 [{track_id, start_sec, end_sec, score}]
    │
    ▼
④ 标签补充（从 auto_tags 拿命中的标签）
    │
    ▼
⑤ 网络库补充（可选，scope 含 'network' 时）
   Freesound / Epidemic Sound 文本搜索
    │
    ▼
返回：matches[] + sfx[]（音效）
```

---

## 3. 关键实现

### 3.1 音乐切片入库（离线，B1+B2）
```python
# 策略：每 30s 切一片，50% 重叠
import librosa
from transformers import ClapModel

model = ClapModel.from_pretrained("laion/clap-htsat-fused")
audio, sr = librosa.load("song.mp3", sr=48000, mono=True)

clip_len = 30 * sr  # 30 秒
hop = clip_len // 2  # 50% overlap
for start in range(0, len(audio) - clip_len, hop):
    clip = audio[start:start+clip_len]
    inputs = processor(audios=clip, return_tensors="pt", sampling_rate=sr)
    vec = model.get_audio_features(**inputs)[0]
    lance_table.add([{
        "track_id": track_id,
        "start_sec": start/sr,
        "end_sec": (start+clip_len)/sr,
        "vector": vec.numpy()
    }])
```

### 3.2 文本检索（在线，B3）
```python
inputs = processor(text="melancholic suspense", return_tensors="pt")
query_vec = model.get_text_features(**inputs)[0]
results = lance_table.search(query_vec.numpy()).limit(5).to_list()
# results: [{track_id, start_sec, end_sec, _distance, vector}, ...]
```

---

## 4. 向量库选型

| 候选 | 优势 | 劣势 | MVP 决策 |
|---|---|---|---|
| **LanceDB** | 单机嵌入式、零配置、列式存储、支持 metadata 过滤 | 社区较小 | **MVP 选定** |
| Chroma | 纯 Python、易用 | 内存占用大、不适合 10万+ 向量 | 备选 |
| Qdrant | 高性能、Rust 内核 | 需起服务或用 embedded 模式 | 量大时升级 |

> 待 B1 完成后做基准测试（1k/10k 片段检索延迟）最终确认。

---

## 5. 中文提示词处理

⚠️ **已知风险**：LAION CLAP 原版文本编码器（RoBERTa）偏英文，中文检索效果差。

**应对方案**（按优先级）：
1. 用 LLM（GLM/GPT）把中文提示词翻译成 CLAP 友好英文音乐描述
2. 用 WavCaps 数据集微调（提升中文，工程量大，Phase 2+）
3. 评估 CLaMP 3（ACL 2025，跨语言 MIR 模型）

---

## 6. API

见 `architecture.md` §4.3。

### MCP Tool
- `score_for_prompt(prompt, scope, top_k, include_sfx)` → 配乐方案

---

## 7. 待决项
- [ ] 切片长度（30s 固定 vs 自适应）
- [ ] 是否对每个片段单独入库（存储/检索成本 vs 召回质量）
- [ ] LLM 归一化用什么模型（GLM-4 Flash？本地小模型？）
- [ ] 网络库 API key 由用户提供还是预置
