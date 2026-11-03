import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest import mock
from urllib.parse import unquote
import zipfile

REAL_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REAL_ROOT/'scripts'))
import package_ffta_portable as package
import report_home


class Portable(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta-public-package-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)/"portable sources ' à"
        self.root.mkdir()
        self.runtime = self.root/'runtime';(self.runtime/'tcc').mkdir(parents=True)
        self.output = self.root/'out'
        for name in ['recomp.exe','ffta-play.exe','recomp','ffta-play','tcc/libtcc.dll','tcc/libtcc.so']:
            (self.runtime/name).write_bytes(b'PUBLIC SYNTHETIC runtime '+name.encode())
        for name in package.TOOL_FILES+package.DOC_FILES:
            path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('[bundled](debug_tools.md) [checkout](../unshipped-proof.json)\n' if name.startswith('docs/') else 'PUBLIC SYNTHETIC tool\n')
        for name in package.LICENSE_FILES:
            path=self.root/'Licenses'/name;path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('PUBLIC SYNTHETIC notice [verbatim](missing.txt)\n')
        for name in ['COPYING','README.md','build.py','desktop_abi.c','libtcc.h','source.zip']:
            path=self.root/'TinyCC'/name;path.parent.mkdir(exist_ok=True);path.write_bytes(b'PUBLIC SYNTHETIC TinyCC')
        (self.root/'gamedb.sqlite').write_bytes(b'PUBLIC SYNTHETIC database')
        # Unlisted files must not be discovered even inside otherwise public dirs.
        self.secret=b'UNSELECTED PERSONAL SENTINEL'
        for name in ['ffta.gba','gba_bios.bin','Data/saves/progress.sav','Data/config/input.cfg',
                     'runtime/ffta.gba','runtime/Data/saves/ffta.sav','runtime/generated.c',
                     'docs/PRIVATE-NOTES.md','Licenses/PERSONAL.txt','scripts/private.input','target/proof.ffstate',
                     'cheats/catalogo.json','runtime/cheats/catalogo.json']:
            path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(self.secret)

    def build(self, platform='windows', dest=None, extra=()):
        dest=dest or self.output/('portable.zip' if platform=='windows' else 'portable.tar.gz')
        with mock.patch.object(package,'ROOT',self.root),contextlib.redirect_stdout(io.StringIO()):
            package.main(argv=['--platform',platform,'--runtime-dir',str(self.runtime),'--output',str(dest),*extra])
        return dest

    def check_payloads(self, payloads):
        manifest=json.loads(payloads['PACKAGE-MANIFEST.json'])
        self.assertEqual(set(manifest['files']),set(payloads)-{'PACKAGE-MANIFEST.json'})
        for name,meta in manifest['files'].items():
            self.assertEqual(hashlib.sha256(payloads[name]).hexdigest(),meta['sha256'],name)
            self.assertEqual(len(payloads[name]),meta['bytes'])
        for name,data in payloads.items():
            self.assertNotIn(self.secret,data,name)
            self.assertFalse(name.startswith('Data/'),name)
            self.assertNotIn('PRIVATE',name)
        self.assertIn('scripts/windows/debug_runtime.ps1',payloads)
        self.assertIn('scripts/linux/debug_runtime.sh',payloads)
        self.assertIn('docs/debug_tools.md',payloads)
        self.assertIn('docs/debug_operations.md',payloads)
        self.assertIn('docs/debug_content.md',payloads)
        self.assertIn('docs/debug_selector.md',payloads)
        self.assertIn('docs/debug_events.md',payloads)
        self.assertIn('docs/cheats.md',payloads)
        self.assertNotIn('cheats/catalogo.json',payloads)
        self.assertIn('docs/menu_clan_atlas.md',payloads)
        self.assertIn('docs/extended_saves.md',payloads)
        self.assertNotIn('docs/development/TASK_HANDOFF.md',payloads)
        self.assertEqual(payloads['Licenses/README.md'],(self.root/'Licenses/README.md').read_bytes())
        self.assertIn(b'checkout reference',payloads['docs/debug_tools.md'])
        self.assertIn(b'[bundled](debug_tools.md)',payloads['docs/debug_tools.md'])
        self.assertIn(b'/Data/\n',payloads['.gitignore'])
        self.assertIn(b'/cheats/\n',payloads['.gitignore'])
        return manifest


    def test_private_build_prefix_rejection_handles_utf8_utf16_and_preserves_archive(self):
        archive=self.build();before=archive.read_bytes()
        prefix='/home/PRIVATE-BUILDER'
        for encoding in ['utf-8','utf-16-le']:
            (self.runtime/'recomp.exe').write_bytes(b'PUBLIC FIXTURE '+prefix.encode(encoding))
            with self.assertRaisesRegex(SystemExit,'Private build prefix'):
                self.build(extra=['--private-prefix',prefix])
            self.assertEqual(archive.read_bytes(),before)
        (self.runtime/'recomp.exe').write_bytes(b'PUBLIC ANONYMOUS RUNTIME')
        archive=self.build(extra=['--private-prefix',prefix])
        with zipfile.ZipFile(archive) as z:
            data=json.loads(z.read('FFTARecomp-portable/PACKAGE-MANIFEST.json'))
            self.assertEqual(data['private_prefix_checks'],1)
            self.assertNotIn(prefix,z.read('FFTARecomp-portable/PACKAGE-MANIFEST.json').decode())

    def test_windows_archive_is_exact_whitelist_private_free_and_manifest_complete(self):
        archive=self.build()
        with zipfile.ZipFile(archive) as z:
            self.assertIsNone(z.testzip())
            data={n.removeprefix('FFTARecomp-portable/'):z.read(n) for n in z.namelist() if not n.endswith('/')}
            self.check_payloads(data)
            self.assertTrue(all(not n.startswith('FFTARecomp-portable/Data/') or n.endswith('/') for n in z.namelist()))

    def test_linux_archive_preserves_binary_and_shell_modes_and_all_selected_bytes(self):
        archive=self.build('linux')
        with tarfile.open(archive) as z:
            data={p.name.removeprefix('FFTARecomp-portable/'):z.extractfile(p).read() for p in z.getmembers() if p.isfile()}
            self.check_payloads(data)
            for name in ['recomp','FFTARecomp',*[n for n in package.TOOL_FILES if n.endswith('.sh')]]:
                self.assertEqual(z.getmember('FFTARecomp-portable/'+name).mode,0o755)
            self.assertTrue(all(p.uid==0 and p.gid==0 for p in z.getmembers()))

    def test_bad_input_or_game_header_keeps_existing_archive_and_cleans_staging(self):
        archive=self.build();before=archive.read_bytes()
        missing=self.root/'Licenses/LICENSE-MIT';old=missing.read_bytes();missing.unlink()
        with self.assertRaises(SystemExit):self.build()
        self.assertEqual(archive.read_bytes(),before)
        missing.write_bytes(old)
        rom=bytearray(192);rom[0xB2]=0x96;rom[0xAC:0xB0]=b'AFXP'
        (self.runtime/'recomp.exe').write_bytes(rom)
        with self.assertRaisesRegex(SystemExit,'Game ROM header'):self.build()
        self.assertEqual(archive.read_bytes(),before)
        self.assertEqual([p.name for p in self.output.iterdir()],['portable.zip'])

    @unittest.skipIf(os.name=='nt','POSIX symlink fixture')
    def test_symlink_file_or_ancestor_refuses_and_output_cannot_replace_input(self):
        target=self.root/'TinyCC/COPYING';data=target.read_bytes();target.unlink()
        external=self.root/'notice';external.write_bytes(data);target.symlink_to(external)
        with self.assertRaises(SystemExit):self.build()
        target.unlink();target.write_bytes(data)
        real=self.root/'real scripts';shutil.move(str(self.root/'scripts'),real)
        (self.root/'scripts').symlink_to(real,target_is_directory=True)
        with self.assertRaises(SystemExit):self.build()
        (self.root/'scripts').unlink();shutil.move(str(real),self.root/'scripts')
        with self.assertRaisesRegex(SystemExit,'must not replace'):self.build(dest=self.runtime/'recomp.exe')

    def test_docker_source_and_ignore_cover_the_complete_public_bundle(self):
        inputs=set(package.DOC_FILES+package.TOOL_FILES)
        for file in ['Dockerfile.linux','Dockerfile.windows']:
            sources=set()
            for line in (REAL_ROOT/file).read_text().splitlines():
                if line.startswith('COPY ') and '--from' not in line:
                    sources.update(shlex.split(line)[1:-1])
            self.assertTrue(inputs<=sources,(file,inputs-sources))
            if file == 'Dockerfile.linux':
                self.assertIn('scripts/build_desktop.py', sources)
        ignore=(REAL_ROOT/'.dockerignore').read_text().splitlines()
        self.assertTrue({'!'+p for p in inputs}<=set(ignore))
        self.assertIn('!scripts/build_desktop.py', ignore)
        for pattern in ['**/*.gba','**/*.sav','**/*.ffstate','**/*.input','**/Data/**']:
            self.assertIn(pattern,ignore)

    def test_ci_upload_matches_the_actual_default_linux_archive(self):
        with mock.patch.object(package, 'ROOT', self.root), contextlib.redirect_stdout(io.StringIO()):
            package.main(argv=['--platform', 'linux', '--runtime-dir', str(self.runtime)])
        archive = self.root/'dist/FFTARecomp-linux-portable.tar.gz'
        self.assertTrue(archive.is_file())
        workflow = (REAL_ROOT/'.github/workflows/installer-linux.yml').read_text()
        upload = re.search(r'^\s+path: (\S+)\s*$', workflow, re.M)
        self.assertIsNotNone(upload)
        self.assertEqual(upload.group(1), archive.relative_to(self.root).as_posix())
        self.assertIn('if-no-files-found: error', workflow)

    def test_bundled_markdown_links_keep_fragments_reserved_paths_and_plain_checkout_refs(self):
        data=b'[here](child%20name.md#part) [proof](../changes/result.json) [web](https://example.test) [same](#part)'
        out=package.bundled_document('docs/index.md',data,{'docs/child name.md'}).decode()
        self.assertIn('[here](child%20name.md#part)',out)
        self.assertIn('[same](#part)',out)
        self.assertIn('[web](https://example.test)',out)
        self.assertNotIn('[proof](',out)

    def test_portable_homepage_only_links_existing_guides(self):
        template=(REAL_ROOT/'scripts/report_home.html').read_text()
        (self.root/'docs/debug_tools.md').write_text('# Common guide')
        page=report_home.render([],template,self.root)
        match=re.search(r'<div class="guides">(.*?)</div>',page,re.S)
        self.assertTrue(match)
        for href in re.findall(r'href="([^"]+)"',match[1]):
            self.assertTrue((self.root/unquote(href)).is_file(),href)
        self.assertNotIn('docs/development/PROJECT_STATE.md',match[1])

    @unittest.skipIf(os.name=='nt','Bash public runtime fixture')
    def test_moved_runtime_wrapper_uses_selected_root_logs_and_preserves_exit(self):
        root=self.root/"moved bundle ' à";(root/'scripts/linux').mkdir(parents=True)
        wrapper=root/'scripts/linux/debug_runtime.sh';shutil.copy2(REAL_ROOT/'scripts/linux/debug_runtime.sh',wrapper)
        runtime=root/'recomp';runtime.write_text('#!/usr/bin/env python3\nimport os,sys,json\nprint(json.dumps({"cwd":os.getcwd(),"argv":sys.argv[1:]}))\nprint("PUBLIC SYNTHETIC ERROR",file=sys.stderr)\nraise SystemExit(7)\n');runtime.chmod(0o755)
        result=subprocess.run(['bash',str(wrapper),'help','debug-control'],cwd='/tmp',capture_output=True,text=True)
        self.assertEqual(result.returncode,7,result.stderr)
        logs=list((root/'Data/config/debug-logs').glob('*.log'));self.assertEqual(len(logs),1)
        text=logs[0].read_text();self.assertIn('PUBLIC SYNTHETIC ERROR',text)
        first=json.loads(text.splitlines()[0]) if text.splitlines()[0].startswith('{') else json.loads(text.splitlines()[1])
        self.assertEqual(first['cwd'],str(root));self.assertEqual(first['argv'],['help','debug-control'])
        self.assertIn(str(logs[0]),result.stdout)

    @unittest.skipIf(os.name=='nt','Bash mock Docker fixture')
    def test_home_wrapper_failure_keeps_old_page_and_discoverable_log(self):
        root=self.root/'home bundle';(root/'scripts/linux').mkdir(parents=True)
        wrapper=root/'scripts/linux/report_home.sh';shutil.copy2(REAL_ROOT/'scripts/linux/report_home.sh',wrapper)
        (root/'index.html').write_text('OLD ACCEPTED PAGE');before=(root/'index.html').stat().st_mtime_ns
        tools=self.root/'fake tools';tools.mkdir();docker=tools/'docker'
        docker.write_text('#!/bin/sh\nif [ "$1" = image ]; then exit 0; fi\nprintf "PUBLIC SYNTHETIC DOCKER FAILURE\\n" >&2\nexit 6\n');docker.chmod(0o755)
        env={**os.environ,'PATH':str(tools)+os.pathsep+os.environ['PATH']}
        result=subprocess.run(['bash',str(wrapper)],cwd='/tmp',env=env,capture_output=True,text=True)
        self.assertEqual(result.returncode,6)
        self.assertEqual((root/'index.html').read_text(),'OLD ACCEPTED PAGE');self.assertEqual((root/'index.html').stat().st_mtime_ns,before)
        logs=list((root/'target/build-logs').glob('*.log'));self.assertEqual(len(logs),1)
        self.assertIn('PUBLIC SYNTHETIC DOCKER FAILURE',logs[0].read_text())
        self.assertFalse(list(root.glob('.report-home-*.tmp')))


if __name__=='__main__':unittest.main()
