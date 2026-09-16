#!/usr/bin/env python3
"""Generate all solvers in an isolated directory, compile, test and export offline."""
from pathlib import Path
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def main():
    work = ROOT / '.tools'
    work.mkdir(exist_ok=True)
    # Retain the directory so failures and generated submission files can be inspected.
    destination = Path(tempfile.mkdtemp(prefix='template-check-', dir=work))
    shutil.copytree(ROOT / 'templates/contest', destination, dirs_exist_ok=True)
    for path in destination.rglob('*'):
        if path.is_file():
            path.write_text(path.read_text().replace('__CONTEST_ID__', 'ahc_template_check'))
    environment = dict(os.environ, CARGO_NET_OFFLINE='true', CARGO_TARGET_DIR=str(work / 'verification-target'))
    environment.pop('AHC_SAMPLE_DELTAS', None)
    environment.pop('AHC_ITERATIONS', None)

    def run(*args):
        subprocess.run(args, cwd=ROOT, env=environment, check=True)

    for name in ['beam', 'beam-fast', 'random-search', 'local-search-direct', 'local-search-rebuild']:
        run(str(ROOT / 'ahc'), '--contest', str(destination), 'add', name)
    # The generated states must also work when problem::Input cannot be cloned.
    problem = destination / 'src/problem.rs'
    problem.write_text(problem.read_text().replace('#[derive(Debug, Clone, PartialEq, Eq)]', '#[derive(Debug)]', 1))
    (destination / 'tests').mkdir()
    shutil.copyfile(ROOT / 'tests/engines.rs', destination / 'tests/engines.rs')
    run('cargo', 'test', '--offline', '--manifest-path', str(destination / 'Cargo.toml'), '--all-targets')
    for solver in ['a', 'greedy', 'valid', 'beam', 'beam_fast', 'random_search', 'local_search_direct', 'local_search_rebuild']:
        run(str(ROOT / 'ahc'), '--contest', str(destination), 'export', '--solver', solver)
        # Compile the actual exported source, not only its include!-based original.
        exported = destination / 'src/bin/exported_check.rs'
        shutil.copyfile(destination / f'submit/{solver}.rs', exported)
        run('cargo', 'check', '--offline', '--manifest-path', str(destination / 'Cargo.toml'), '--bin', 'exported_check')
    print(f'Template verification passed: {destination}')


if __name__ == '__main__':
    main()
