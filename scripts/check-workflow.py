#!/usr/bin/env python3
"""Exercise public scripts against a generated single-file intro workspace."""
import json
import subprocess

from verification import example_workspace


def main():
    contest, environment = example_workspace('workflow-check-')

    def run(*args, check=True, **kwargs):
        return subprocess.run(args, cwd=contest, env=environment, check=check, **kwargs)

    source = contest / 'src/bin/a.rs'
    source.write_text(source.read_text() + '\n// handwritten contest change\n')
    original = source.read_bytes()
    beam = contest / 'src/bin/beam_practice.rs'
    beam.write_text(source.read_text().replace(
        'let state = solve_greedy(&input, State::new(&input));',
        'let state = beam::solve_beam(&input, State::new(&input), 10);'
    ) + '\n' + (contest / 'samples/beam.rs').read_text())
    run('./scripts/build.sh', 'beam_practice')
    run('./scripts/run-one.sh', 'beam_practice', '0')
    run('./scripts/debug.sh', 'beam_practice', '0')
    run('./scripts/run-all.sh', 'beam_practice', '2')
    result = json.loads((contest / 'results/latest.json').read_text())
    assert result['solver'] == 'beam_practice' and result['accepted_cases'] == result['cases'] == 2
    assert source.read_bytes() == original

    run('./scripts/save.sh', 'beam_practice', 'beam-baseline', '1')
    snapshots = [json.loads(line) for line in (contest / 'snapshots/index.jsonl').read_text().splitlines()]
    assert len(snapshots) == 1 and snapshots[0]['solver'] == 'beam_practice'
    listed = run('./scripts/list.sh', capture_output=True, text=True)
    assert 'beam-baseline' in listed.stdout
    run('./scripts/compare.sh', '1', '1')

    # Verify clipboard behavior without changing the user's real clipboard.
    fake_bin = contest / 'fake-bin'
    fake_bin.mkdir()
    clipboard = contest / 'clipboard.txt'
    (fake_bin / 'pbcopy').write_text('#!/bin/sh\ncat > "$AHC_TEST_CLIPBOARD"\n')
    (fake_bin / 'pbcopy').chmod(0o755)
    environment['PATH'] = str(fake_bin) + ':' + environment['PATH']
    environment['AHC_TEST_CLIPBOARD'] = str(clipboard)
    run('./scripts/export.sh', 'beam_practice')
    exported = contest / 'submit/beam_practice.rs'
    assert exported.stat().st_size > 0 and clipboard.read_bytes() == exported.read_bytes()
    assert list((contest / 'visualizations').rglob('*.html'))

    # Restoring uses the saved source and keeps a backup of subsequent edits.
    source.write_text(source.read_text() + '\n// change after saving\n')
    run('./scripts/restore.sh', '1')
    assert source.read_bytes() == original
    run('./scripts/run-one.sh', 'beam_practice', '0')

    (contest / 'src/bin/panic_check.rs').write_text('fn main() { panic!("workflow panic check"); }\n')
    assert run('./scripts/debug.sh', 'panic_check', '0', check=False).returncode != 0
    assert 'workflow panic check' in (contest / 'results/logs/panic_check/debug/0000.log').read_text()
    print(f'Workflow verification passed: {contest}')


if __name__ == '__main__':
    main()
