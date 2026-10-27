"""Build a portable ZIP (or a macOS .app ZIP) without local development files."""
import argparse
import hashlib
import json
from pathlib import Path
import plistlib
import shutil
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BINARY = "orcs-from-the-neighborhood"
PLATFORMS = ("linux-x86_64", "windows-x86_64", "macos-aarch64", "macos-x86_64")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=PLATFORMS, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    if not args.binary.is_file():
        parser.error(f"binary does not exist: {args.binary}")
    args.output.mkdir(parents=True, exist_ok=True)
    name = f"{BINARY}-{version}-{args.platform}"
    archive = args.output / f"{name}.zip"
    with tempfile.TemporaryDirectory() as tmp:
        package = Path(tmp) / name
        package.mkdir()
        mac = args.platform.startswith("macos-")
        if mac:
            contents = package / "Orcs from the Neighborhood.app" / "Contents"
            executable = contents / "MacOS" / BINARY
            resources = contents / "Resources"
            contents.mkdir(parents=True)
            (contents / "Info.plist").write_bytes(plistlib.dumps({
                "CFBundleExecutable": BINARY,
                "CFBundleIdentifier": "com.villetacore.orcs-from-the-neighborhood",
                "CFBundleName": "Orcs from the Neighborhood",
                "CFBundlePackageType": "APPL",
                "CFBundleShortVersionString": version,
                "CFBundleVersion": version,
                "LSMinimumSystemVersion": "12.0",
                "NSHighResolutionCapable": True,
                "NSHumanReadableCopyright": "Copyright 2026 villetacore. MIT License.",
            }))
        else:
            executable = package / (BINARY + (".exe" if args.platform.startswith("windows-") else ""))
            resources = package
        executable.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(args.binary, executable)
        executable.chmod(0o755)
        # Copy exactly the artwork referenced by the compiled catalogue, plus the back.
        cards = json.loads((ROOT / "data/cards.json").read_text())
        images = {c["image"] for c in cards} | {"assets/art/card_back.png"}
        for image in sorted(images):
            destination = resources / image
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / image, destination)
        shutil.copytree(ROOT / "rules", package / "rules")
        for doc in ("LICENSE", "CHANGELOG.md", "INSTALL.md"):
            shutil.copyfile(ROOT / doc, package / doc)
        (package / "README.md").write_text(
            "# Орки с района\n\nЛокальная карточная RPG для 2–4 игроков. Автор: villetacore.\n\n"
            "Распакуйте весь архив. Инструкции: [INSTALL.md](INSTALL.md). "
            "Правила: [quickstart.md](rules/quickstart.md).\n\n"
            "Исходники и обратная связь: https://github.com/villetacore/orcs-from-the-neighborhood\n",
            encoding="utf-8",
        )
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as output:
            for path in sorted(package.rglob("*")):
                if path.is_file():
                    info = zipfile.ZipInfo(path.relative_to(package.parent).as_posix(), (2026, 1, 1, 0, 0, 0))
                    info.create_system = 3
                    info.external_attr = (0o100755 if path == executable else 0o100644) << 16
                    info.compress_type = zipfile.ZIP_DEFLATED
                    output.writestr(info, path.read_bytes())
        with zipfile.ZipFile(archive) as result:
            assert result.testzip() is None, "corrupt release archive"
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(".zip.sha256").write_text(f"{digest}  {archive.name}\n", encoding="ascii")
    print(archive)


if __name__ == "__main__":
    main()
