"""允许 `python -m app` 启动 sidecar。

`-m 包名` 会执行包下的 __main__.py（而非 __init__.py），
所以这里转发到 app.main()。
"""
from . import main

if __name__ == "__main__":
    main()
