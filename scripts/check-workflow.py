#!/usr/bin/env python3
"""Exercise familiar commands against an isolated copy of intro, including panic logs."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def main():
    work = ROOT / '.tools'
    work.mkdir(exist_ok=True)
    contest = Path(tempfile.mkdtemp(prefix='workflow-check-', dir=work))
    source = ROOT / 'contests/intro-heuristics'
    for name in ['Cargo.toml', 'Cargo.lock', 'ahc.toml', 'ahc']:
        shutil.copy2(source / name, contest / name)
    shutil.copytree(source / 'src', contest / 'src')
    shutil.copytree(ROOT / 'templates/contest/scripts', contest / 'scripts')
    shutil.copytree(source / 'tools', contest / 'tools', ignore=shutil.ignore_patterns('target', 'in', '.git'))
    environment = dict(os.environ, CARGO_NET_OFFLINE='true')

    def run(*args, check=True):
        return subprocess.run(args, cwd=contest, env=environment, check=check)

    run('./scripts/build.sh', 'beam_search')
    run('./scripts/run-one.sh', 'beam_search', '0')
    run('./scripts/debug.sh', 'beam_search', '0')
    run('./scripts/run-all.sh', 'beam_search', '2')
    result = json.loads((contest / 'results/latest.json').read_text())
    assert result['cases'] == 2 and result['accepted_cases'] == 2
    run('./ahc', 'save', 'beam-baseline', '--solver', 'beam_search', '--cases', '1', '--builtin')
    run('./ahc', 'list')
    run('./ahc', 'export', '--solver', 'beam_search')
    assert (contest / 'submit/beam_search.rs').stat().st_size > 0
    assert list((contest / 'visualizations').rglob('*.html'))
    (contest / 'src/bin/panic_check.rs').write_text('fn main() { panic!("workflow panic check"); }\n')
    assert run('./scripts/debug.sh', 'panic_check', '0', check=False).returncode != 0
    assert 'workflow panic check' in (contest / 'results/logs/panic_check/debug/0000.log').read_text()
    print(f'Workflow verification passed: {contest}')


if __name__ == '__main__':
    main()
