#!/usr/bin/env python3
"""Generate, paste, compile and score the actual single-file examples offline."""
from pathlib import Path
import json
import re
import shutil
import subprocess

from verification import ROOT, example_workspace


def main():
    destination, environment = example_workspace('template-check-')
    source = destination / 'src/bin/a.rs'
    initial = source.read_text()
    assert 'include!(' not in initial
    assert 'proconio::input' in initial and 'pub raw: String' not in initial
    assert list((destination / 'src').rglob('*.rs')) == [source]

    def run(*args, **kwargs):
        return subprocess.run(args, cwd=destination, env=environment, check=True, **kwargs)

    # Copying a file under src/bin must work without registering a Cargo target.
    shutil.copyfile(source, destination / 'src/bin/manual_copy.rs')
    # Removing the optional preview must leave a working clone-and-advance fallback.
    fallback, removed = re.subn(
        r'(?m)^        fn evaluated_value_after\(&self, input: &Input, action: &Action\) -> i64 \{\n'
        r'            self\.value_after\(input, action\)\n        \}\n',
        '', initial,
    )
    assert removed == 1
    (destination / 'src/bin/fallback_copy.rs').write_text(fallback)
    solvers = ['beam', 'beam_fast', 'random_search', 'local_search_direct', 'local_search_rebuild']
    for name in solvers:
        shutil.copyfile(destination / f'samples/complete/{name}.rs', destination / f'src/bin/{name}.rs')
        # Also test the user's intended workflow: keep their State, paste only the addition.
        pasted = initial
        if name.startswith('local_search'):
            pasted = pasted.replace('fn main() {', 'fn main() {\n    let started_at = std::time::Instant::now();')
            pasted = pasted.replace('let output = state.into_output();',
                                    f'let output = state.into_output();\n    let output = {name}::improve(&input, output, started_at);')
        else:
            call = {
                'beam': 'beam::solve_beam(&input, State::new(&input), 100)',
                'beam_fast': 'beam_fast::solve_beam_fast(&input, State::new(&input), 100)',
                'random_search': 'random_search::solve_seeded(&input, &State::new(&input), 20, 3, 42)',
            }[name]
            original = 'let state = solve_greedy(&input, State::new(&input));'
            assert pasted.count(original) == 1
            pasted = pasted.replace(original, f'let state = {call};')
        pasted += '\n' + (destination / f'samples/{name}.rs').read_text()
        assert 'include!(' not in pasted
        (destination / f'src/bin/pasted_{name}.rs').write_text(pasted)

    tests = destination / 'tests'
    tests.mkdir()
    for name in ['engines.rs', 'intro_samples.rs']:
        shutil.copyfile(ROOT / 'tests' / name, tests / name)
    shutil.copytree(ROOT / 'templates/framework', tests / 'framework')
    shutil.copyfile(ROOT / 'templates/contest/src/framework/constructive.rs', tests / 'framework/constructive.rs')
    fragments = tests / 'fragments'
    fragments.mkdir()
    for name in ['problem.rs', 'constructive_state.rs']:
        shutil.copyfile(ROOT / 'templates/contest/src' / name, fragments / name)
    for name in ['local_search_direct.rs', 'local_search_rebuild.rs']:
        shutil.copyfile(ROOT / 'templates/states' / name, fragments / name)
    run('cargo', 'test', '--offline', '--all-targets')
    run('cargo', 'test', '--offline', '--release', '--test', 'engines', '--test', 'intro_samples')

    # Build once, then score every complete / manually pasted binary with official tools.
    run('cargo', 'build', '--offline', '--release', '--bins')
    tools_environment = dict(environment, CARGO_TARGET_DIR=str(destination / 'tools/target'))
    subprocess.run(['cargo', 'build', '--offline', '--release', '--manifest-path', 'tools/Cargo.toml'],
                   cwd=destination, env=tools_environment, check=True)
    seeds = destination / 'seeds.txt'
    seeds.write_text('0\n1\n')
    run(str(destination / 'tools/target/release/gen'), str(seeds))
    environment['AHC_ITERATIONS'] = '20'
    names = ['a', 'manual_copy', 'fallback_copy', *solvers, *(f'pasted_{name}' for name in solvers)]
    results = {}
    # Direct cargo invocations above share a cache; the CLI uses each contest's own target.
    target = Path(environment['CARGO_TARGET_DIR']) / 'release'
    for name in names:
        scores = []
        for seed in range(2):
            input_path = destination / f'in/{seed:04}.txt'
            output = run(str(target / name), input=input_path.read_text(), text=True, capture_output=True)
            lines = output.stdout.split()
            assert len(lines) == 365 and all(1 <= int(value) <= 26 for value in lines), name
            output_path = destination / f'{name}-{seed}.txt'
            output_path.write_text(output.stdout)
            scored = run(str(destination / 'tools/target/release/vis'), str(input_path), str(output_path),
                         text=True, capture_output=True)
            assert 'Score = ' in scored.stdout, scored
            scores.append(scored.stdout.strip())
        results[name] = scores
        run(str(ROOT / 'ahc'), '--contest', str(destination), 'export', '--solver', name)
        exported = destination / 'src/bin/exported_check.rs'
        shutil.copyfile(destination / f'submit/{name}.rs', exported)
        run('cargo', 'check', '--offline', '--bin', 'exported_check')
    for name in solvers:
        assert results[name] == results[f'pasted_{name}'], name
    assert results['a'] == results['manual_copy']
    assert results['a'] == results['fallback_copy']
    assert source.read_text() == initial, 'build/run/export must not regenerate edited sources'
    (destination / 'verification-scores.json').write_text(json.dumps(results, indent=2))
    print(f'Template verification passed: {destination}')


if __name__ == '__main__':
    main()
