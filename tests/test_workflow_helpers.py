import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('references', ROOT / 'scripts/setup-references.py')
references = importlib.util.module_from_spec(spec)
spec.loader.exec_module(references)


class WorkflowHelpers(unittest.TestCase):
    def test_links_reflect_changes_without_refresh(self):
        with tempfile.TemporaryDirectory(prefix='ahc-refs-test-') as directory:
            root = Path(directory)
            source, destination = root / 'original', root / 'references'
            source.mkdir()
            original = source / 'a.rs'
            original.write_text('first')
            references.setup(destination, {'abc': source}, root / 'backups')
            self.assertTrue((destination / 'abc').is_symlink())
            original.write_text('second')
            self.assertEqual((destination / 'abc/a.rs').read_text(), 'second')
            references.setup(destination, {'abc': source}, root / 'backups')

    def test_migration_preserves_copies_and_rejects_edits(self):
        import hashlib
        with tempfile.TemporaryDirectory(prefix='ahc-refs-test-') as directory:
            root = Path(directory)
            source, destination = root / 'original', root / 'references'
            source.mkdir()
            (source / 'a.rs').write_text('new source')
            (destination / 'abc').mkdir(parents=True)
            copy = destination / 'abc/a.rs'
            copy.write_text('local edit')
            manifest = {'abc/a.rs': {'sha256': hashlib.sha256(b'old copy').hexdigest()}}
            (destination / 'manifest.json').write_text(json.dumps(manifest))
            with self.assertRaises(SystemExit):
                references.setup(destination, {'abc': source}, root / 'backups')
            self.assertEqual(copy.read_text(), 'local edit')
            copy.write_text('old copy')
            references.setup(destination, {'abc': source}, root / 'backups')
            self.assertEqual(copy.read_text(), 'new source')
            backups = list((root / 'backups').glob('*/references/abc/a.rs'))
            self.assertEqual(len(backups), 1)
            self.assertEqual(backups[0].read_text(), 'old copy')

    def test_legacy_commands_preserve_argument_order_and_exit_code(self):
        with tempfile.TemporaryDirectory(prefix='ahc-command-test-') as directory:
            root = Path(directory)
            shutil.copytree(ROOT / 'templates/contest/scripts', root / 'scripts')
            fake = root / 'ahc'
            fake.write_text('#!/usr/bin/env python3\nimport json,sys\nprint(json.dumps(sys.argv[1:]))\nsys.exit(7 if "fail" in sys.argv else 0)\n')
            fake.chmod(0o755)
            cases = [
                ('build.sh', [], ['build', '--solver', 'a']),
                ('run-one.sh', ['beam', '12'], ['run', '12', '--solver', 'beam']),
                ('run-all.sh', ['beam', '20'], ['bench', '--solver', 'beam', '--cases', '20', '--builtin']),
                ('debug.sh', ['a', '3'], ['run', '3', '--solver', 'a', '--debug']),
            ]
            for name, args, expected in cases:
                result = subprocess.run([str(root / 'scripts' / name), *args], cwd='/', capture_output=True, text=True, check=True)
                self.assertEqual(json.loads(result.stdout), expected)
            result = subprocess.run([str(root / 'scripts/build.sh'), 'fail'], capture_output=True)
            self.assertEqual(result.returncode, 7)


if __name__ == '__main__':
    unittest.main()
