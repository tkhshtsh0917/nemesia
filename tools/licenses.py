"""Collect dependency notices without packaging Cargo's absolute source paths."""
import json
import shutil
import sys
from pathlib import Path


def collect(metadata: Path, destination: Path) -> None:
    destination.mkdir()
    for own in ("LICENSE-MIT", "LICENSE-APACHE", "NOTICE"):
        shutil.copyfile(own, destination / own)

    packages = json.loads(metadata.read_text())["packages"]
    for package in packages:
        if package["source"] is None:
            continue
        root = Path(package["manifest_path"]).parent
        package_dir = destination / f'{package["name"]}-{package["version"]}'
        package_dir.mkdir()
        notices = sorted(path for path in root.iterdir() if path.is_file() and
                         path.name.upper().startswith(("LICENSE", "NOTICE", "COPYING", "COPYRIGHT", "UNLICENSE")))
        if not notices:
            # Workspaces can carry the license at their repository root.
            notices = sorted(path for path in root.parent.iterdir() if path.is_file() and
                             path.name.upper().startswith(("LICENSE", "NOTICE", "COPYING")))
        if not notices:
            raise RuntimeError(f'Missing dependency notice: {package["name"]}')
        for notice in notices:
            shutil.copyfile(notice, package_dir / notice.name)


if __name__ == "__main__":
    collect(Path(sys.argv[1]), Path(sys.argv[2]))
