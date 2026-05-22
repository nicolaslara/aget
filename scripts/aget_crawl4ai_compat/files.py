import os
from pathlib import Path


def write_private_text(path: Path, content: str) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    try:
        if hasattr(os, "fchmod"):
            os.fchmod(fd, 0o600)
        else:
            os.chmod(path, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as file:
            fd = -1
            file.write(content)
    finally:
        if fd >= 0:
            os.close(fd)
