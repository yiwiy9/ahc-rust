"""Isolated workspace for exercising the real generator and public scripts."""
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def example_workspace(prefix):
    work = ROOT / '.tools'
    work.mkdir(exist_ok=True)
    workspace = Path(tempfile.mkdtemp(prefix=prefix, dir=work))
    (workspace / '.ahc-root').write_text('verification workspace\n')
    shutil.copytree(ROOT / 'templates', workspace / 'templates')
    shutil.copytree(ROOT / 'docs', workspace / 'docs')
    shutil.copyfile(ROOT / 'README.md', workspace / 'README.md')
    shutil.copytree(ROOT / 'scripts', workspace / 'scripts', ignore=shutil.ignore_patterns('__pycache__'))
    tools = workspace / 'contests/intro-heuristics/tools'
    shutil.copytree(ROOT / 'contests/intro-heuristics/tools', tools,
                    ignore=shutil.ignore_patterns('target', 'in', '.git'))
    (workspace / 'ahc').write_text('#!/bin/sh\nexec ' + shlex.quote(str(ROOT / 'ahc')) + ' "$@"\n')
    (workspace / 'ahc').chmod(0o755)
    environment = dict(os.environ, CARGO_NET_OFFLINE='true',
                       CARGO_TARGET_DIR=str(work / 'verification-target'))
    environment.pop('AHC_SAMPLE_DELTAS', None)
    environment.pop('AHC_ITERATIONS', None)
    subprocess.run(['./scripts/new.sh', 'intro-example', '--example'],
                   cwd=workspace, env=environment, check=True)
    contest = workspace / 'contests/intro-example'
    # New README links must resolve from the generated layout, not the template tree.
    for readme in [contest / 'README.md', contest / 'samples/README.md']:
        for link in re.findall(r'\]\(([^)]+)\)', readme.read_text()):
            if '://' not in link and not link.startswith('#'):
                target = link.split('#')[0]
                assert (readme.parent / target).exists(), (readme, link)
    return contest, environment
