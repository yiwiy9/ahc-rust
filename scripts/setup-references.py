#!/usr/bin/env python3
"""Link existing repositories for browsing; never edit or pull their contents."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def setup(destination, sources, backup_root):
    if destination.is_symlink():
        raise SystemExit(f'Reference directory itself must not be a link: {destination}')
    destination = destination.parent.resolve() / destination.name
    sources = {label: source.resolve() for label, source in sources.items()}
    for label, source in sources.items():
        if Path(label).name != label or label in {'.', '..'}:
            raise SystemExit(f'Invalid reference name: {label}')
        if not source.is_dir():
            raise SystemExit(f'Reference source not found: {source}')
        if source == destination or source in destination.parents or destination in source.parents:
            raise SystemExit(f'Reference would create a cycle: {source}')

    manifest_path = destination / 'manifest.json'
    if manifest_path.is_file() and not manifest_path.is_symlink():
        # Migrate only untouched generated copies; retain the entire old directory.
        manifest = json.loads(manifest_path.read_text())
        actual = {p.relative_to(destination).as_posix() for p in destination.rglob('*') if p.is_file()}
        if any(p.is_symlink() for p in destination.rglob('*')) or actual != set(manifest) | {'manifest.json'}:
            raise SystemExit('Unmanaged/missing reference files. Migration stopped without changes.')
        for name, entry in manifest.items():
            relative = Path(name)
            if relative.is_absolute() or '..' in relative.parts:
                raise SystemExit('Invalid reference manifest path')
            if hashlib.sha256((destination / relative).read_bytes()).hexdigest() != entry['sha256']:
                raise SystemExit(f'Edited reference: {name}. Migration stopped without changes.')
        backup_root.mkdir(parents=True, exist_ok=True)
        backup = Path(tempfile.mkdtemp(prefix='reference-copy-backup-', dir=backup_root)) / 'references'
        destination.rename(backup)
        print(f'Previous copies preserved: {backup}')
    elif destination.exists():
        # Check every existing entry before creating even one link.
        for path in destination.iterdir():
            if path.name not in sources or not path.is_symlink() or path.resolve() != sources[path.name]:
                raise SystemExit(f'Unexpected reference: {path}. No files were changed.')

    destination.mkdir(parents=True, exist_ok=True)
    for label, source in sources.items():
        link = destination / label
        if not link.is_symlink():
            link.symlink_to(os.path.relpath(source, destination), target_is_directory=True)
        print(f'{link} -> {os.readlink(link)}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lib', type=Path, default=ROOT.parent.parent / 'atcoder-rust/src/lib/src')
    parser.add_argument('--abc', type=Path, default=ROOT.parent.parent / 'atcoder-rust/src/contest')
    args = parser.parse_args()
    setup(ROOT / 'references', {'atcoder-lib': args.lib, 'abc': args.abc}, ROOT / '.tools')


if __name__ == '__main__':
    main()
