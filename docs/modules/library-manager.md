# 模块：音乐库管理器（library-manager）

> **负责**：音乐文件的导入、去重、索引、查询、筛选、流式播放
> **所属路线**：MVP 线 A（与打标签协同）
> **依赖**：SQLite、FFmpeg（音频解码）
> **对接**：tagging-engine（触发分析）、ai-scoring（提供数据）、storage-sync（抽象存储）、desktop-gui（前端）
> **数据模型**：见 `architecture.md` §3.1

---

## 当前状态：⬜ 未开始

| 子任务 | 状态 | 说明 |
|---|---|---|
| SQLite schema 建立 | ⬜ | tracks + auto_tags 表 |
| 目录扫描导入 | ⬜ | 与 A3 协同 |
| 去重（file_hash） | ⬜ | sha256 前 16 位 |
| 查询筛选（按标签） | ⬜ | `?tag=mood:happy` |
| 流式播放端点 | ⬜ | `/api/library/tracks/{id}/audio` Range 支持 |

---

## 1. 职责边界

**做什么**：
- 扫描用户指定目录，发现音频文件（mp3/flac/m4a/wav/ogg）
- 计算 file_hash 去重
- 解析音频基础元数据（时长、采样率、现有 ID3 标签）
- 维护 SQLite 索引（tracks 表）
- 提供多维筛选查询（按标签、情绪、genre、用户标签、自由文本）
- 流式返回音频给前端 `<audio>` 播放

**不做什么**：
- 自动打标签 → tagging-engine
- 向量化 → ai-scoring
- 云同步 → storage-sync

---

## 2. 导入流程

```
POST /api/library/scan { "path": "/Users/zeen/Music", "recursive": true }
    │
    ▼
遍历目录，匹配音频扩展名
    │
    ▼
对每个文件：
  1. 计算 sha256(file_content)[:16] → file_hash
  2. 查 tracks 表，若已存在 → skip（去重）
  3. 用 mutagen 读现有 ID3/Vorbis 标签（title/artist/album）
  4. 用 ffprobe / librosa 读 duration_sec、sample_rate
  5. INSERT tracks，analysis_status='pending'
    │
    ▼
返回新增列表 + 任务 ID（供前端轮询）
    │
    ▼
（前端触发 tagging-engine 批量打标）
```

---

## 3. 查询能力

### SQL 查询模板
```sql
-- 按情绪标签筛选
SELECT t.* FROM tracks t
JOIN auto_tags a ON a.track_id = t.id
WHERE a.tag_type = 'mood' AND a.tag_value = 'happy'
  AND a.confidence > 0.5
GROUP BY t.id;

-- 多标签 AND 查询（同时含 happy 和 energetic）
SELECT t.* FROM tracks t
WHERE t.id IN (
  SELECT track_id FROM auto_tags WHERE tag_type='mood' AND tag_value='happy'
) AND t.id IN (
  SELECT track_id FROM auto_tags WHERE tag_type='mood' AND tag_value='energetic'
);
```

### REST 筛选语法
```
GET /api/library/tracks?tag=mood:happy,mood:energetic&limit=50&offset=0
GET /api/library/tracks?genre=electronic&bpm_min=120&bpm_max=140
GET /api/library/tracks?q=love (自由文本搜 title/artist)
```

---

## 4. 流式播放

```python
# GET /api/library/tracks/{id}/audio
# 返回 StreamingResponse，支持 Range 请求（前端 <audio> 拖动进度条）
from fastapi.responses import StreamingResponse

@app.get("/api/library/tracks/{id}/audio")
def stream_audio(id: str, range_header: str = Header(None)):
    track = get_track(id)
    return stream_file_with_range(track.file_path, range_header)
```

---

## 5. 与 storage-sync 的关系

library-manager 只认**逻辑路径**（track.file_path），物理位置由 storage-sync 决定：
- LocalStorage：file_path 是本地绝对路径
- R2Storage：file_path 是 R2 key，播放时生成 presigned URL

这让 library-manager 不用关心音乐在哪台机器/云端。

---

## 6. 待决项
- [ ] 是否引入 beets 做文件组织（重命名/移动）？MVP 先不引入，保持文件原位
- [ ] 增量扫描：文件变更监听（fsevents/inotify）
- [ ] 播放列表、收藏
- [ ] 封面图提取（mutagen 可读 embedded cover）
