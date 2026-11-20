#!/usr/bin/env python3
"""Package the repository's native bundle as a Horizon .hnmod archive."""
from __future__ import annotations

import json
import re
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "dist" / "horizon_replay.hnmod"
NATIVE = ROOT / "horizon-native.json"
MIXIN = ROOT / "horizon-mixin.toml"
API_CARGO = ROOT / "api" / "Cargo.toml"
API_LIB = ROOT / "api" / "src" / "lib.rs"


def main() -> None:
    meta = json.loads(NATIVE.read_text(encoding="utf-8"))
    manifest = tomllib.loads(MIXIN.read_text(encoding="utf-8"))
    mod = manifest["mod"]
    revision = manifest["compatibility"]["horizon_revision"]

    if meta.get("schema") != 1:
        raise SystemExit("unsupported horizon-native.json schema")
    if not re.fullmatch(r"[a-z0-9_]{1,64}", meta.get("id", "")):
        raise SystemExit("invalid native mod id")
    if meta["id"] != mod["id"] or meta["version"] != mod["version"]:
        raise SystemExit("native metadata and horizon-mixin.toml disagree")
    if meta["horizon"] != revision or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise SystemExit("Horizon revision mismatch or invalid commit")
    if not re.fullmatch(r"[0-9a-f]{40}", meta.get("mixins", "")):
        raise SystemExit("invalid horizon-mixins revision")
    if meta.get("hooks") != 1:
        raise SystemExit("camera_pose requires native hooks API 1")

    for path in (MIXIN, API_CARGO, API_LIB):
        if not path.is_file():
            raise SystemExit(f"missing bundle file: {path.relative_to(ROOT)}")

    entries = {
        "horizon-native.json": NATIVE,
        f"mixins/{meta['id']}/horizon-mixin.toml": MIXIN,
        f"mixins/{meta['id']}/api/Cargo.toml": API_CARGO,
        f"mixins/{meta['id']}/api/src/lib.rs": API_LIB,
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    partial = OUT.with_suffix(".hnmod.partial")
    with zipfile.ZipFile(partial, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for archive_name, source in entries.items():
            archive.write(source, archive_name)
    partial.replace(OUT)

    with zipfile.ZipFile(OUT) as archive:
        names = set(archive.namelist())
        if names != set(entries):
            raise SystemExit("archive contents failed verification")
        bad = archive.testzip()
        if bad:
            raise SystemExit(f"archive CRC check failed: {bad}")

    print(f"Built {OUT.relative_to(ROOT)} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
