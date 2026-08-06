# 模块：存储与同步（storage-sync）

> **负责**：抽象音乐文件的物理存储，支持本地、云端、剪映对接三种来源
> **所属路线**：MVP 线 C（来源管理）
> **依赖**：可插拔后端（LocalStorage / R2Storage / GithubReleaseStorage）
> **对接**：library-manager（提供逻辑路径 → 物理路径解析）
> **调研依据**：`research/03-storage-and-ai-music.md` §问题一

---

## 当前状态：⬜ 未开始（MVP 只做 LocalStorage）

| 子任务 | 状态 | 说明 |
|---|---|---|
| StorageBackend 抽象接口 | ⬜ | 定义 get/put/stream 三个方法 |
| LocalStorage 实现 | ⬜ | 直接读写本地文件系统 |
| 剪映音乐 ID 映射 | ⬜ | 静态 JSON 数据 + 搜索 |
| R2Storage 后端 | ⬜ Phase 2 | Cloudflare R2 + presigned URL |
| GithubReleaseStorage | ⬜ 备选 | 仅小规模 demo |

---

## 1. 职责边界

**做什么**：
- 定义统一 `StorageBackend` 抽象接口
- 实现各后端（本地/R2/GitHub Release）
- 管理音乐来源元数据（source 字段）
- 提供剪映音乐库查询/ID 显示

**不做什么**：
- 音乐元数据/标签管理 → library-manager
- 决定文件存哪 → 由用户配置选择后端

---

## 2. 抽象接口

```python
from abc import ABC, abstractmethod

class StorageBackend(ABC):
    @abstractmethod
    def put(self, key: str, file_path: str) -> str:
        """存储文件，返回逻辑路径"""

    @abstractmethod
    def get_local_path(self, logical_path: str) -> str | None:
        """返回本地可播放路径（云端后端可能先下载到 cache）"""

    @abstractmethod
    def get_stream_url(self, logical_path: str, expires_in: int = 3600) -> str:
        """返回可流式播放的 URL（本地=file://，云端=presigned）"""

    @abstractmethod
    def delete(self, logical_path: str) -> None:
        """删除文件"""

    @abstractmethod
    def exists(self, logical_path: str) -> bool:
        """是否存在"""
```

---

## 3. 各后端实现

### 3.1 LocalStorage（MVP）
```python
class LocalStorage(StorageBackend):
    def get_local_path(self, logical_path): return logical_path  # 直接是本地路径
    def get_stream_url(self, logical_path, **_):
        return f"http://127.0.0.1:{PORT}/api/library/tracks/{id}/audio"
```
逻辑路径 = 本地绝对路径，零转换。

### 3.2 R2Storage（Phase 2）
```python
import boto3
class R2Storage(StorageBackend):
    def __init__(self, config):
        self.s3 = boto3.client("s3",
            endpoint_url=config["r2_endpoint"],
            aws_access_key_id=config["r2_key"],
            aws_secret_access_key=config["r2_secret"])
        self.bucket = config["r2_bucket"]

    def put(self, key, file_path):
        self.s3.upload_file(file_path, self.bucket, key)
        return f"r2://{self.bucket}/{key}"

    def get_stream_url(self, logical_path, expires_in=3600):
        key = parse_key(logical_path)
        return self.s3.generate_presigned_url(
            'get_object', Params={'Bucket': self.bucket, 'Key': key},
            ExpiresIn=expires_in)
```

### 3.3 GithubReleaseStorage（备选/hack）
仅适合极小规模 demo，不推荐主存储。

---

## 4. 剪映音乐对接

### 需求（用户原始要求）
> 「最次的方案那就是显示这个剪映里面对应音乐的 ID 或者说是他的名称」

### 实现策略（C3）
1. **数据源**：剪映音乐库是动态的，无官方 API。MVP 用**静态 JSON 映射**（手工采集常见音乐）
2. **匹配**：用户库音乐按标题/音频指纹 → 匹配剪映库 → 记录 `source_ref`（剪映音乐 ID）
3. **展示**：前端在曲目详情显示「剪映对应：[音乐名 / ID]」，用户可在剪映内搜索该 ID

```python
# GET /api/source/jianying/search?q=雨夜
→ {
  "ok": true,
  "data": [
    { "jianying_id": "C_7_xxx", "title": "雨夜独行", "duration": 180, "category": "忧伤" }
  ]
}
```

### 采集方式（待定）
- 手工整理常见音乐的 JSON（MVP）
- 社区贡献维护一个公开清单
- 或逆向剪映客户端（灰色，不推荐）

---

## 5. 配置

```json
// ~/Library/Application Support/mood-music-studio/config.json
{
  "storage": {
    "backend": "local",          // local | r2 | github
    "local_root": "/Users/zeen/Music",
    "r2": {                       // backend=r2 时启用
      "endpoint": "...",
      "bucket": "...",
      "key": "...",
      "secret": "..."
    }
  }
}
```

---

## 6. 待决项
- [ ] R2 凭证存储方式（明文 config？keychain？）
- [ ] 云端音乐「按需下载到本地缓存」策略
- [ ] 多设备同步冲突解决
- [ ] 剪映音乐数据源是否合规（版权考量）
