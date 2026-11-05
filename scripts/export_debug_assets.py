#!/usr/bin/env python3
"""Create an explicitly private, standalone Windows debug-asset export.

The ordinary release packager remains private-data-free. This command only reads
its source collections and discovered workspace assets and publishes a new ZIP;
it never edits a source collection.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tempfile
import zipfile
from datetime import datetime
import debug_asset_inventory as inventory

ROOT = Path(__file__).resolve().parents[1]
PREFIX = 'FFTARecomp-debug-EUR'
BASE_PREFIX = 'FFTARecomp-portable/'
EXCLUDED = {'.gba', '.agb', '.bin', '.ffstate', '.sav', '.input'}


def regular(path):
    path = Path(path)
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'Expected a regular non-symlink file: {path}')
    return path


def tree(path):
    path = Path(path)
    if path.is_symlink() or not path.is_dir():
        raise ValueError(f'Expected a non-symlink directory: {path}')
    for directory, subdirs, files in os.walk(path, followlinks=False):
        here = Path(directory)
        subdirs.sort(); files.sort()
        for name in subdirs:
            if (here / name).is_symlink():
                raise ValueError(f'Symlink in source collection: {here / name}')
        for name in files:
            file = regular(here / name)
            yield file, file.relative_to(path).as_posix()


def safe_relative(value):
    if not value or '\\' in value or re.match(r'^[A-Za-z]:', value):
        raise ValueError(f'Unsafe path: {value!r}')
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in ('', '.', '..') for part in value.split('/')):
        raise ValueError(f'Unsafe path: {value!r}')
    return path.as_posix()


def digest(path):
    h = hashlib.sha256()
    with regular(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def check_catalog(ready):
    catalog = json.loads(regular(ready / 'catalog.json').read_text(encoding='utf-8'))
    if catalog.get('schema') != 1 or not isinstance(catalog.get('cases'), list):
        raise ValueError('Expected the schema-1 local debug-save catalog')
    entries = [cp for case in catalog['cases'] for cp in case.get('checkpoints', [])]
    for name in ('variant_slots', 'variants', 'generated_standard_saves', 'factory_standard_saves', 'loaded_checkpoints'):
        entries.extend(catalog.get(name, []))
    if isinstance(catalog.get('eur_base'), dict): entries.append(catalog['eur_base'])
    entries.extend(x['ready'] for x in catalog.get('inputs', []) if isinstance(x.get('ready'), dict))
    checked = {}
    for item in entries:
        name = item.get('file')
        if not name:
            continue
        name = safe_relative(name)
        source = ready.joinpath(*PurePosixPath(name).parts)
        current = ready
        for part in PurePosixPath(name).parts:
            current = current / part
            if current.is_symlink():
                raise ValueError(f'Catalog symlink: {name}')
        if name not in checked: checked[name] = digest(source)
        actual = checked[name]
        if not isinstance(item.get('sha256'), str) or actual != item['sha256']:
            raise ValueError(f'Catalog checksum mismatch: {name}')
    return checked


def reject_rom(path):
    suffix=path.suffix.lower()
    if suffix in ('.gba','.agb','.bios') or path.name.lower() in ('gba_bios.bin','bios.bin'):
        raise ValueError(f'ROM/BIOS is not a debug asset: {path}')
    with regular(path).open('rb') as f:
        header=f.read(192)
    if len(header)>=192 and header[0xB2]==0x96 and header[0xAC:0xB0] in (b'AFXE',b'AFXP'):
        raise ValueError(f'Game ROM found in debug assets: {path}')
    if suffix=='.zip':
        with zipfile.ZipFile(path) as nested:
            for info in nested.infolist():
                safe_relative(info.filename.rstrip('/'))
                if stat.S_ISLNK(info.external_attr >> 16):
                    raise ValueError(f'Linked entry inside source ZIP: {path.name}/{info.filename}')
                name=info.filename.lower()
                if name.endswith(('.gba','.agb','.bios','gba_bios.bin','bios.bin')):
                    raise ValueError(f'ROM/BIOS inside source ZIP: {path.name}/{info.filename}')
                if not info.is_dir():
                    with nested.open(info) as f: header=f.read(192)
                    if len(header)>=192 and header[0xB2]==0x96 and header[0xAC:0xB0] in (b'AFXE',b'AFXP'):
                        raise ValueError(f'Game ROM inside source ZIP: {path.name}/{info.filename}')


def pe64(path):
    with regular(path).open('rb') as f:
        header = f.read(64)
        if len(header) < 64 or header[:2] != b'MZ':
            raise ValueError(f'Expected x86-64 Windows executable: {path}')
        f.seek(int.from_bytes(header[60:64], 'little'))
        if f.read(6) != b'PE\0\0\x64\x86':
            raise ValueError(f'Expected x86-64 Windows executable: {path}')


def export(args):
    base = regular(args.base)
    launcher = regular(args.launcher); runtime = regular(args.runtime)
    pe64(launcher); pe64(runtime)
    assets = args.assets
    ready = assets / 'ready'
    legacy = args.legacy
    original_files = list(tree(assets)); legacy_files = list(tree(legacy))
    for path, _ in original_files + legacy_files: reject_rom(path)
    catalog = check_catalog(ready)
    source_catalog = json.loads((ready / 'catalog.json').read_text(encoding='utf-8'))
    projected_catalog = inventory.complete_catalog(source_catalog)
    if not (ready / 'EUR' / 'ffta.sav').is_file():
        raise ValueError('EUR collection needs ffta.sav')
    eur_hashes = list((ready / 'EUR' / 'SaveStates').iterdir())
    if len(eur_hashes) != 1 or not re.fullmatch(r'[0-9a-f]{64}', eur_hashes[0].name):
        raise ValueError('EUR snapshot directory must contain one ROM SHA-256')
    recordings = []
    if args.recordings.exists():
        if args.recordings.is_symlink() or not args.recordings.is_dir():
            raise ValueError('Recordings source must be a non-symlink directory')
        for directory in sorted(args.recordings.iterdir()):
            if directory.is_symlink() or not directory.is_dir():
                raise ValueError(f'Invalid recording directory: {directory}')
            data=json.loads(regular(directory/'recording.json').read_text(encoding='utf-8'))
            if data.get('kind')!='ffta-input-recording' or data.get('status')!='complete':
                continue
            for filename,key in [('start.ffstate','start_sha256'),('end.ffstate','end_sha256'),('input.input','input_sha256')]:
                if digest(directory/filename)!=data.get(key):raise ValueError(f'Recording checksum mismatch: {directory.name}/{filename}')
            recordings.append(directory)
    pair=[args.replay/'start.ffstate',args.replay/'input.input']
    if any(p.exists() for p in pair) and not all(p.is_file() and not p.is_symlink() for p in pair):
        raise ValueError('F-09 snapshot and input must both be regular files')
    original_sources=[]
    if args.source_manifest.exists():
        sources=json.loads(regular(args.source_manifest).read_text(encoding='utf-8'))
        for label,item in sorted(sources.items()):
            if not re.fullmatch(r'[a-zA-Z0-9_-]+',label) or not isinstance(item,dict):
                raise ValueError('Invalid original source manifest entry')
            name=safe_relative(item.get('path',''))
            path=ROOT.joinpath(*PurePosixPath(name).parts)
            current=ROOT
            for part in PurePosixPath(name).parts:
                current=current/part
                if current.is_symlink():raise ValueError(f'Original source symlink: {name}')
            reject_rom(path)
            if digest(path)!=item.get('sha256'):raise ValueError(f'Original source checksum mismatch: {label}')
            original_sources.append((label,path,name,item['sha256']))
    out = args.output.absolute()
    if out.exists() or out.is_symlink():
        raise ValueError(f'Output already exists: {out}')
    out.parent.mkdir(parents=True, exist_ok=True)
    entries = {}; names_casefold = set(); source_archives = {}; source_guards = {}
    def reserve(name):
        name = safe_relative(name)
        folded = name.casefold()
        if folded in names_casefold:
            raise ValueError(f'Duplicate archive path: {name}')
        names_casefold.add(folded)
        return name
    def add_stream(z, name, reader, provenance):
        name = reserve(name); h = hashlib.sha256(); count = 0
        with z.open(PREFIX + '/' + name, 'w', force_zip64=True) as target:
            while True:
                block = reader.read(1024 * 1024)
                if not block: break
                target.write(block); h.update(block); count += len(block)
        entries[name] = {'sha256': h.hexdigest(), 'bytes': count, 'source': provenance}
    def add_file(z, name, path, provenance):
        path = Path(path).absolute()
        before = inventory.stamp(regular(path))
        with path.open('rb') as f: add_stream(z, name, f, provenance)
        if inventory.stamp(path) != before:
            raise ValueError(f'Source changed during export: {path}')
        source_guards.setdefault(path, before)
        source_archives.setdefault(path, name)
    def add_bytes(z, name, data, provenance):
        import io
        add_stream(z, name, io.BytesIO(data), provenance)
    fd, tmpname = tempfile.mkstemp(prefix='.debug-assets-', suffix='.tmp', dir=out.parent)
    os.close(fd); temp = Path(tmpname)
    try:
        with zipfile.ZipFile(base) as source, zipfile.ZipFile(temp, 'w', zipfile.ZIP_DEFLATED, compresslevel=6, allowZip64=True) as z:
            manifest_name = BASE_PREFIX + 'PACKAGE-MANIFEST.json'
            manifest = json.loads(source.read(manifest_name))
            if manifest.get('kind') != 'ffta-portable-package' or manifest.get('platform') != 'windows':
                raise ValueError('Base ZIP must be a Windows portable package')
            expected = manifest.get('files')
            if not isinstance(expected, dict): raise ValueError('Invalid base package manifest')
            found = set()
            for info in source.infolist():
                if not info.filename.startswith(BASE_PREFIX):
                    raise ValueError(f'Unexpected base archive path: {info.filename}')
                if info.is_dir(): continue
                name = safe_relative(info.filename[len(BASE_PREFIX):])
                if name == 'PACKAGE-MANIFEST.json': continue
                if (name.startswith('Data/') or name.lower().endswith(tuple(EXCLUDED)) or
                    stat.S_ISLNK(info.external_attr >> 16)):
                    raise ValueError(f'Private or linked payload in base package: {name}')
                if name not in expected: raise ValueError(f'Undeclared base package payload: {name}')
                if name in found: raise ValueError(f'Duplicate base payload: {name}')
                if name in ('FFTARecomp.exe', 'recomp.exe', 'LEGGIMI.txt', '.gitignore'):
                    with source.open(info) as f:
                        h=hashlib.sha256()
                        for block in iter(lambda: f.read(1024*1024),b''):h.update(block)
                    actual=h.hexdigest()
                else:
                    with source.open(info) as f: add_stream(z, name, f, 'base portable ZIP')
                    actual=entries[name]['sha256']
                if actual != expected[name].get('sha256'):
                    raise ValueError(f'Base package checksum mismatch: {name}')
                found.add(name)
            if found != set(expected): raise ValueError('Base package manifest does not match ZIP')
            base_readme=source.read(BASE_PREFIX+'LEGGIMI.txt')
            # Accept the legacy Italian marker in previously generated base ZIPs.
            markers=(b'No personal cases or results are included.',b'Nessun caso personale o risultato e incluso.')
            old=next((marker for marker in markers if marker in base_readme),None)
            if old is None:raise ValueError('Unexpected portable base instructions')
            add_bytes(z,'LEGGIMI.txt',base_readme.replace(old,b'This private copy includes debug cases: read LEGGIMI-DEBUG.txt.'),'updated base instructions')
            add_bytes(z,'.gitignore',b'*\n','private extracted-folder ignore')
            add_file(z,'FFTARecomp.exe',launcher,'selected Windows launcher')
            add_file(z,'recomp.exe',runtime,'selected Windows runtime')
            # Keep the normal manifest accurate for the portable base only.
            for name in ('FFTARecomp.exe','recomp.exe','LEGGIMI.txt','.gitignore'):
                expected[name]={'sha256':entries[name]['sha256'],'source_sha256':entries[name]['sha256'],'bytes':entries[name]['bytes']}
            manifest['scope']='Portable base payload identities only; this private debug export also contains game saves, snapshots and recordings. See DEBUG-ASSETS-MANIFEST.json.'
            add_bytes(z,'PACKAGE-MANIFEST.json',(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n').encode(),'updated portable manifest')
            # The shared catalog resolves its file paths relative to this directory.
            for path, rel in original_files:
                if rel.startswith('ready/'):
                    dest='Data/config/debug-catalog/'+rel[len('ready/'):]
                else:
                    dest='DebugAssets/originali/'+rel
                if rel == 'ready/catalog.json':
                    add_file(z,'DebugAssets/original-catalog.json',path,'unchanged source catalog')
                    add_bytes(z,dest,(json.dumps(projected_catalog,ensure_ascii=False,indent=2)+'\n').encode(),'derived accessible catalog; original retained')
                else:
                    add_file(z,dest,path,'debug_saves/'+rel)
            for name, expected_hash in catalog.items():
                if entries['Data/config/debug-catalog/'+name]['sha256'] != expected_hash:
                    raise ValueError(f'Catalog source changed during export: {name}')
            for path, rel in legacy_files:
                add_file(z,'DebugAssets/casi-precedenti/'+rel,path,'saves/debug-cases/'+rel)
            # A separate working copy of the existing EUR slots. Catalog masters remain intact.
            for path, rel in tree(ready/'EUR'):
                if '/' not in rel and rel.endswith('.sav') or rel.startswith('SaveStates/'):
                    add_file(z,'Data/saves/'+rel,path,'debug_saves/ready/EUR/'+rel)
            for label,path,name,expected_hash in original_sources:
                dest='DebugAssets/fonti-verificate/'+label+path.suffix.lower()
                add_file(z,dest,path,name)
                if entries[dest]['sha256']!=expected_hash:raise ValueError(f'Original source changed during export: {label}')
            if original_sources:
                add_file(z,'DebugAssets/fonti-verificate/source-manifest.json',args.source_manifest,'verified source manifest')
            for directory in recordings:
                for path,rel in tree(directory):
                    reject_rom(path)
                    add_file(z,'Data/config/recordings/'+directory.name+'/'+rel,path,'accepted recording/'+directory.name+'/'+rel)
            if all(p.is_file() for p in pair):
                for path in pair:add_file(z,'DebugAssets/replay-F-09/'+path.name,path,'F-09 replay/'+path.name)
            roots = [ROOT] + list(args.scan_root)
            print('Discovering workspace debug assets (including experimental and incomplete material)...',flush=True)
            rows, exclusions, guards = inventory.scan(roots, ROOT, reject_rom, digest,
                excluded_paths=[out, temp, *args.exclude_scan])
            discovered_snapshot_paths = sum(r['kind']=='snapshot' for r in rows)
            report_roots = sorted({r['path'].parent for r in rows if r['path'].suffix.lower()=='.html'},key=lambda p:(len(p.parts),str(p)))
            selected_roots = []
            for root in report_roots:
                if any(root.is_relative_to(parent) for parent in selected_roots): continue
                selected_roots.append(root)
            dependency_dirs = {r['path'].parent for r in rows if r['path'].name in {'recording.json','replay.json','capture.json'}}
            protected = set(source_archives)
            protected.update(r['path'] for r in rows if r['kind']=='snapshot' and (
                r['path'].parent in dependency_dirs or any(r['path'].is_relative_to(root) for root in selected_roots)))
            rows, snapshot_omissions = inventory.select_snapshots(rows, protected=protected,
                keep=getattr(args,'keep_snapshot',[]),all_snapshots=getattr(args,'all_snapshots',False))
            retained_paths = {r['path'] for r in rows} | set(source_archives)
            guards = {p:s for p,s in guards.items() if p in retained_paths}
            print(f'Selected {len(rows):,} source files; omitted {len(snapshot_omissions):,} intermediate snapshots; copying payloads...',flush=True)
            # Reuse existing archive bytes and equal payloads; retain all source aliases.
            payloads = {meta['sha256']: name for name, meta in entries.items()}
            # Keep report-relative images, JSON, raw dumps and nested attachments usable.
            bundle_files = {}; report_bundles = []
            for root in selected_roots:
                contents = [r for r in rows if r['path'].is_relative_to(root)]
                identity = hashlib.sha256(''.join(r['sha256']+r['path'].relative_to(root).as_posix() for r in sorted(contents,key=lambda r:r['path'])).encode()).hexdigest()
                folder = 'DebugAssets/reports/'+identity
                report_bundles.append({'source':str(root.relative_to(ROOT)) if root.is_relative_to(ROOT) else root.name,'directory':folder,'files':len(contents)})
                for r in contents:
                    dest = folder+'/'+r['path'].relative_to(root).as_posix()
                    bundle_files[r['path']] = dest
                    if dest not in entries: add_file(z,dest,r['path'],r['source'])
                    if entries[dest]['sha256'] != r['sha256']: raise ValueError(f'Report changed: {r["source"]}')
            recording_files = {}
            by_path = {r['path']: r for r in rows}
            recording_groups = []
            for row in rows:
                if row['path'].name != 'recording.json': continue
                try: data = json.loads(row['path'].read_text(encoding='utf-8'))
                except (ValueError, UnicodeError): continue
                if not isinstance(data, dict) or data.get('kind') != 'ffta-input-recording': continue
                parent = row['path'].parent
                siblings = [r for p,r in by_path.items() if p.parent == parent]
                identity = hashlib.sha256(''.join(r['sha256']+r['path'].name for r in sorted(siblings,key=lambda r:r['path'].name)).encode()).hexdigest()
                folder = 'DebugAssets/recordings/' + identity
                problems = []
                for filename,key in [('start.ffstate','start_sha256'),('end.ffstate','end_sha256'),('input.input','input_sha256')]:
                    r = by_path.get(parent / filename)
                    if r is None or r['sha256'] != data.get(key): problems.append(filename)
                recording_groups.append({'source': row['source'], 'directory': folder,
                    'status': data.get('status','unknown'), 'identity_check': 'mismatch/missing: '+', '.join(problems) if problems else 'verified'})
                for r in siblings:
                    dest = folder+'/'+r['path'].name
                    recording_files[r['path']] = dest
                    if dest not in entries: add_file(z,dest,r['path'],r['source'])
                    if entries[dest]['sha256'] != r['sha256']: raise ValueError(f'Recording changed: {r["source"]}')
            for row in rows:
                path = row['path']; sha = row['sha256']
                dest = recording_files.get(path) or bundle_files.get(path) or source_archives.get(path) or payloads.get(sha)
                if dest is None:
                    dest = 'DebugAssets/library/'+sha[:2]+'/'+sha+path.suffix.lower()
                    add_file(z,dest,path,row['source'])
                if entries[dest]['sha256'] != sha: raise ValueError(f'Asset changed: {row["source"]}')
                row['archive'] = dest; payloads.setdefault(sha,dest)
            known_saves = {}
            for path, sha in catalog.items():
                if path.endswith('.sav') and path.split('/')[0] in ('EUR','USA'):
                    known_saves.setdefault(sha,set()).add(path.split('/')[0])
            pages = inventory.catalog_pages(rows, source_catalog.get('rom_sha256',{}), known_saves)
            for name,data in pages.items():
                encoded=(json.dumps(data,ensure_ascii=False,indent=2)+'\n').encode()
                if len(encoded)>4*1024*1024: raise ValueError(f'Catalog exceeds runtime limit: {name}')
                add_bytes(z,name,encoded,'generated discovery catalog')
            summary={'source_files':len(rows),'unique_payloads':len({r['archive'] for r in rows}),
                'snapshots':sum(r['kind']=='snapshot' for r in rows),'cartridge_saves':sum(r['kind']=='cartridge-save' for r in rows),
                'inputs':sum(r['kind']=='input' for r in rows),'recording_directories':len(recording_groups),
                'report_bundles':len(report_bundles),
                'discovered_snapshot_paths':discovered_snapshot_paths,
                'omitted_intermediate_snapshots':len(snapshot_omissions),
                'catalogs':len(pages),'catalogued_source_files':sum('catalog' in r for r in rows),
                'raw_only_or_invalid_saves':sum(r['kind'] in ('snapshot','cartridge-save') and 'catalog' not in r for r in rows)}
            export_inventory={'schema':1,'kind':'ffta-private-debug-asset-inventory',
                'summary':summary,'scan_roots':[str(p) for p in roots],
                'policy':'All available workspace saves/snapshots/inputs/imports and diagnostic text, reports, images/audio; content aliases retained. No ROM/BIOS, extracted ROM datasets, native/generated code, builds, caches, Wine profiles, prior exports or external linked folders. Extra roots require --scan-root.',
                'files':[{k:v for k,v in r.items() if k!='path'} for r in rows],
                'snapshot_selection':{'mode':'all' if getattr(args,'all_snapshots',False) else 'representative',
                    'policy':'Preserve collection masters, original sources, reproduction boundaries, independent slots, recording/report dependencies and explicit keeps. Numbered capture sequences retain first/last; intermediate steps are omitted without deleting originals. This filename/provenance heuristic does not prove scene equivalence.',
                    'omitted':snapshot_omissions},
                'recordings':recording_groups,'reports':report_bundles,'exclusions':exclusions}
            add_bytes(z,'DEBUG-ASSETS-INVENTORY.json',(json.dumps(export_inventory,ensure_ascii=False,indent=2)+'\n').encode(),'complete discovery inventory and omissions')
            add_bytes(z,'DEBUG-ASSETS.html',inventory.index_html(rows,pages,exclusions,export_inventory['snapshot_selection']),'searchable local asset index')
            for path,before in guards.items():
                if path in source_guards and source_guards[path] != before:
                    raise ValueError(f'Source changed between collection and discovery: {path}')
                source_guards[path]=before
            add_bytes(z,'Data/config/av.cfg',b'debug = true\ndebug.extended_saves = true\n','debug portable defaults')
            instructions = ("""FFTARecomp - private debug assets (Windows x86-64)

1. Extract FFTARecomp-debug-EUR to the project root or another location of your choice.
2. Launch FFTARecomp.exe and select your own European ROM. No ROM is included.
3. Debug and Extended saves are enabled in the portable copy. Game > Saves contains
   the EUR working copies (slots and SaveStates). The complete catalog is under
   Debug > Sessions and cases > Debug catalog. Use Open isolated case to open a case;
   use Import to free snapshot slot to import a snapshot in an isolated copy.
   Loaded checkpoints and the EUR base are now included in that catalog.
   Open DEBUG-ASSETS.html for exported assets, source paths and extra catalogs.
   Choose/reload a DEBUG-CATALOG-EUR/USA-*.json file from the menu to use those saves.
   Set Region, Recorded status, Kind and DS case filters to All when browsing.
4. Catalog masters are in Data/config/debug-catalog; other originals and previous
   cases are in DebugAssets. The entire existing USA collection is included in
   the catalog, but this export has not verified USA compatibility.
5. Snapshots require compatible ROM/hash and BIOS identities (normally HLE/no BIOS).
   Do NOT overwrite personal data; this is a separate folder with private copies.

Private ZIP: contains saves and generated data. Do not publish or commit it.
The base package and binaries come from the inputs selected for the script.
Includes verified original sources, selected complete recordings and the
F-09 replay when available.
Includes discovered target debug assets and incomplete/experimental material.
By default, numbered micro-snapshot sequences keep first/last points and protected
catalog/replay/report checkpoints. --all-snapshots retains every intermediate;
--keep-snapshot PATH preserves an explicitly selected checkpoint.
without promoting it to validated gameplay. DEBUG-ASSETS-INVENTORY.json records
all source aliases, recording completeness, invalid/unidentified saves and exclusions.
Contains no ROM, BIOS, cache or extracted ROM datasets. Recordings are bound to their
original identities: check recording.json before using them.
""").encode('utf-8')
            add_bytes(z,'LEGGIMI-DEBUG.txt',instructions,'generated instructions')
            output_manifest={'schema':1,'kind':'ffta-private-debug-assets','region_default':'EUR',
                'edition':'extended' if getattr(args,'all_snapshots',False) else 'standard',
                'usa':'existing collection; not validated by this export','rom_included':False,
                'catalog_verified_files':len(catalog),'verified_original_sources':len(original_sources),'complete_recordings':len(recordings),'f09_replay_included':all(p.is_file() for p in pair),'eur_rom_sha256':eur_hashes[0].name,
                'discovery':summary,
                'source_base_sha256':digest(base),'files':entries}
            add_bytes(z,'DEBUG-ASSETS-MANIFEST.json',(json.dumps(output_manifest,ensure_ascii=False,indent=2)+'\n').encode(),'generated export manifest')
        # Verify CRC and every copied byte before publishing; refuse an existing destination.
        with zipfile.ZipFile(temp) as archive:
            if archive.testzip() is not None: raise ValueError('ZIP CRC validation failed')
            for name, metadata in entries.items():
                h=hashlib.sha256();count=0
                with archive.open(PREFIX+'/'+name) as f:
                    for block in iter(lambda:f.read(1024*1024),b''):
                        h.update(block);count+=len(block)
                if h.hexdigest()!=metadata['sha256'] or count!=metadata['bytes']:
                    raise ValueError(f'ZIP validation failed: {name}')
        for path,before in source_guards.items():
            if inventory.stamp(path)!=before: raise ValueError(f'Source changed before publication; retry with stable inputs: {path}')
        os.link(temp,out)  # atomic no-overwrite publish, same filesystem
        print(f'Created {out} ({out.stat().st_size:,} bytes); {len(entries)} files; catalog: {len(catalog)} verified files.')
        print('Discovery: '+json.dumps(summary,sort_keys=True))
    finally:
        temp.unlink(missing_ok=True)


def main(argv=None, *, extended=False):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--base',type=Path,default=ROOT/'dist/debug-tools/FFTARecomp-windows-debug-tools.zip',help='existing Windows portable debug-tools ZIP')
    p.add_argument('--runtime',type=Path,default=ROOT/'target/x86_64-pc-windows-gnu/release/recomp.exe')
    p.add_argument('--launcher',type=Path,default=ROOT/'target/x86_64-pc-windows-gnu/release/ffta-play.exe')
    p.add_argument('--assets',type=Path,default=ROOT/'debug_saves')
    p.add_argument('--legacy',type=Path,default=ROOT/'saves/debug-cases')
    p.add_argument('--source-manifest',type=Path,default=ROOT/'target/debug-saves-audit/source-manifest.json')
    p.add_argument('--recordings',type=Path,default=ROOT/'target/debug-tools-recording/final recording case/Data/config/recordings')
    p.add_argument('--replay',type=Path,default=ROOT/'target/desktop-perf/ai-turn-replay')
    p.add_argument('--output',type=Path,help='new ZIP destination (default: dist/debug-assets with edition and timestamp)')
    p.add_argument('--scan-root',type=Path,action='append',default=[],help='additional local asset root; repeatable (workspace is always scanned)')
    p.add_argument('--exclude-scan',type=Path,action='append',default=[],help='explicit discovery exclusion, recorded in inventory; repeatable')
    p.add_argument('--all-snapshots',action='store_true',default=extended,help='include consecutive intermediate snapshots (always enabled by the extended script)')
    p.add_argument('--keep-snapshot',type=Path,action='append',default=[],help='preserve an additional discovered snapshot in representative mode; repeatable')
    args=p.parse_args(argv)
    if args.output is None:
        edition = 'extended-' if args.all_snapshots else ''
        args.output = ROOT/'dist/debug-assets'/('FFTARecomp-debug-assets-'+edition+'EUR-'+datetime.now().strftime('%Y%m%d-%H%M%S')+'.zip')
    try: export(args)
    except (OSError,ValueError,KeyError,zipfile.BadZipFile,json.JSONDecodeError) as exc:
        p.exit(1,f'Export failed: {exc}\n')

if __name__ == '__main__': main()
