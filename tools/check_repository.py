"""Check publishable files for private machine paths, secrets and image metadata.

Uses git's ignore rules so local editor settings, saves and build outputs stay local.
Never prints a matching value: diagnostics contain only the file and check name.
"""
import re
import struct
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATTERNS = {
    "private home path": re.compile(rb"/(?:Users|home)/[A-Za-z0-9_.-]+/|[A-Za-z]:\\Users\\[A-Za-z0-9_.-]+\\"),
    "private key": re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    "access token": re.compile(rb"(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{50,}|AKIA[A-Z0-9]{16})"),
}


def main():
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT, check=True, capture_output=True,
    )
    errors = []
    files = sorted(set(result.stdout.decode().split("\0")) - {""})
    for name in files:
        path = ROOT / name
        if not path.is_file():
            continue
        data = path.read_bytes()
        for label, pattern in PATTERNS.items():
            if pattern.search(data):
                errors.append((name, label))
        if data.startswith(b"\x89PNG\r\n\x1a\n"):
            offset = 8
            while offset + 12 <= len(data):
                size = struct.unpack_from(">I", data, offset)[0]
                kind = data[offset + 4:offset + 8]
                if kind in (b"tEXt", b"zTXt", b"iTXt", b"eXIf"):
                    errors.append((name, "PNG text/EXIF metadata"))
                offset += size + 12
        if data.startswith(b"\xff\xd8") and b"Exif\0\0" in data:
            errors.append((name, "JPEG EXIF metadata"))
    for name, label in errors:
        print(f"{name}: {label}")
    if errors:
        raise SystemExit(1)
    print(f"Privacy checks passed: {len(files)} publishable files.")


if __name__ == "__main__":
    main()
