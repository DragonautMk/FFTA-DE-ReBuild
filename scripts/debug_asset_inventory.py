"""Discover private debug assets and build portable, bounded save catalogs.

Source bytes are never rewritten. Content aliases retain every original path;
invalid/unknown snapshots remain downloadable without becoming validated cases.
"""
import hashlib
import html
import json
import os
from pathlib import Path
import re
from urllib.parse import quote
import zipfile

from extract_rom_data import DESCRIPTIONS

PRIMARY = {'.ffstate', '.sav', '.input', '.sps'}
SUPPORT = {'.json', '.jsonl', '.toml', '.cfg', '.ini', '.html', '.png', '.bmp',
           '.jpg', '.jpeg', '.svg', '.wav', '.csv', '.tsv', '.txt', '.md', '.log',
           '.cht', '.recipe', '.rec', '.recording', '.css', '.js', '.webp', '.gif',
           '.ppm', '.pcm16le', '.f32le', '.events', '.rgba', '.pcm'}
SKIP_DIRS = {'.git', '.agents', '.codex', '.aws', 'node_modules', '.venv', 'venv',
             '__pycache__', '.fingerprint', 'incremental', 'deps', 'build',
             '.cargo', 'registry', 'cache', 'caches', 'drive_c', 'dosdevices',
             'wine-prefix', 'wineprefix', 'sysroot', 'dist', 'FFTARecomp-debug-EUR'}
ARTIFACT_DIRS = {'target', 'debug_saves', 'saves', 'cheats', 'Data', 'DebugAssets'}
SOURCE_DIRS = {'source', 'source-copy', 'source-preimage', 'crates', 'src', 'vendor', 'TinyCC'}
CATALOG_PAGE = 256


def stamp(path):
    st = path.stat(follow_symlinks=False)
    return (st.st_dev, st.st_ino, st.st_size, st.st_mtime_ns, st.st_ctime_ns)


def state_identity(path):
    """Same v1/preview envelope checks as save_state::preview; no guest decoding."""
    with path.open('rb') as f:
        header = f.read(48)
        prefix = 0
        if header[:8] == b'FFTA-SL1':
            preview = f.read(240 * 160 * 4)
            if len(preview) != 240 * 160 * 4 or hashlib.sha256(preview).digest() != header[16:48]:
                raise ValueError('Snapshot preview checksum mismatch')
            prefix = 48 + len(preview)
            core = f.read(108)
            if int.from_bytes(header[8:16], 'little') != path.stat().st_size - prefix:
                raise ValueError('Snapshot envelope length mismatch')
        else:
            f.seek(0); core = f.read(108)
        size = path.stat().st_size - prefix
        if (len(core) != 108 or core[:8] != b'FFTA-ST\0'
                or int.from_bytes(core[8:12], 'little') != 1 or size > 64 * 1024 * 1024):
            raise ValueError('Invalid or incompatible snapshot header')
        payload = hashlib.sha256()
        content = hashlib.sha256(core)
        for block in iter(lambda: f.read(1024 * 1024), b''):
            payload.update(block)
            content.update(block)
        if payload.digest() != core[76:108]:
            raise ValueError('Snapshot payload checksum mismatch')
        return {'rom_sha256': core[12:44].hex(), 'bios_sha256': core[44:76].hex(),
                'state_content_sha256': content.hexdigest()}


def scan(roots, workspace, reject_rom, digest, *, excluded_paths=()):
    """Inventory eligible files, reporting omissions instead of hiding them."""
    rows, exclusions, guards, seen = [], [], {}, set()
    workspace = workspace.absolute()
    excluded_paths = {Path(p).absolute() for p in excluded_paths}
    scopes = []
    for root in roots:
        root = Path(root).absolute()
        if root in scopes:
            continue
        if root.is_symlink() or not root.is_dir():
            raise ValueError(f'Scan root must be an existing non-symlink directory: {root}')
        scopes.append(root)
    for scope_index, root in enumerate(scopes):
        external = not root.is_relative_to(workspace)
        def source_name(p):
            return p.relative_to(workspace).as_posix() if not external else f'external-{scope_index + 1}/{p.relative_to(root).as_posix()}'
        for directory, dirs, files in os.walk(root, followlinks=False, onerror=lambda e: (_ for _ in ()).throw(e)):
            here = Path(directory)
            if '.debug-artifacts-lease' in dirs or '.debug-artifacts-lease' in files:
                exclusions.append({'source':source_name(here), 'reason':'active or interrupted artifact lease; source scope left untouched'})
                dirs[:] = []
                continue
            retained = []
            for name in sorted(dirs):
                p = here / name
                if p in excluded_paths or name in SKIP_DIRS or name.startswith(('.wine', 'wineprefix')):
                    exclusions.append({'source': source_name(p), 'reason': 'excluded directory (build/cache/Wine/export or explicit exclusion)'})
                elif p.is_symlink() or (hasattr(p, 'is_junction') and p.is_junction()):
                    exclusions.append({'source': source_name(p), 'reason': 'directory link; never followed'})
                else:
                    retained.append(name)
            dirs[:] = retained
            parts = here.relative_to(root).parts
            artifact = external or root != workspace or bool(set(parts) & ARTIFACT_DIRS)
            support = artifact and not bool(set(parts) & SOURCE_DIRS)
            for name in sorted(files):
                p = here / name
                if p in excluded_paths or p in seen:
                    continue
                ext = p.suffix.lower()
                raw_capture = name in {'vram.bin', 'oam.bin', 'palette.bin', 'io.bin'} or (
                    'audio.json' in files and name.startswith(('left-', 'right-')) and ext == '.bin')
                candidate = ext in PRIMARY or support and (ext in SUPPORT or raw_capture) or artifact and ext == '.zip'
                if not candidate:
                    continue
                seen.add(p)
                source = source_name(p)
                if p.is_symlink() or not p.is_file():
                    exclusions.append({'source': source, 'reason': 'not a regular non-symlink file'})
                    continue
                if name in DESCRIPTIONS or name in {'rom-data.json', 'dataset.json'}:
                    exclusions.append({'source': source, 'reason': 'extracted ROM dataset'})
                    continue
                before = stamp(p)
                try:
                    reject_rom(p)
                except (ValueError, zipfile.BadZipFile) as e:
                    exclusions.append({'source': source, 'reason': str(e).replace(str(p), source)})
                    continue
                if ext == '.zip':
                    try:
                        with zipfile.ZipFile(p) as z:
                            members = [m.filename for m in z.infolist() if not m.is_dir()]
                            if not any(Path(n).suffix.lower() in PRIMARY for n in members):
                                exclusions.append({'source': source, 'reason': 'archive has no save/snapshot/input assets'})
                                continue
                            if any(Path(n).suffix.lower() in {'.exe', '.dll', '.so', '.c', '.gba', '.agb', '.bios'} for n in members):
                                exclusions.append({'source': source, 'reason': 'archive contains build/ROM/BIOS payloads'})
                                continue
                    except zipfile.BadZipFile:
                        exclusions.append({'source': source, 'reason': 'unreadable ZIP archive'})
                        continue
                row = {'source': source, 'bytes': before[2], 'sha256': digest(p),
                       'kind': {'.ffstate': 'snapshot', '.sav': 'cartridge-save', '.input': 'input', '.sps': 'import'}.get(ext, 'support'),
                       'path': p}
                if ext == '.ffstate':
                    try:
                        row.update(state_identity(p)); row['snapshot_check'] = 'envelope-verified; gameplay and runtime compatibility unverified'
                    except ValueError as e:
                        row['issue'] = str(e)
                if stamp(p) != before:
                    raise ValueError(f'Debug asset changed during discovery: {p}')
                guards[p] = before
                rows.append(row)
    return rows, exclusions, guards


def select_snapshots(rows, *, protected=(), keep=(), all_snapshots=False):
    """Reduce numbered capture sequences, never infer scene equality from images.

    Slot numbers and IN/DS/GEN identities are independent saved cases. Their
    numbering must not be mistaken for chronological capture steps. Catalog and
    replay/report inputs always take priority over this filename heuristic.
    """
    protected = {Path(p).absolute() for p in protected}
    keep = {Path(p).absolute() for p in keep}
    snapshots = [r for r in rows if r['kind'] == 'snapshot']
    missing = keep - {r['path'] for r in snapshots}
    if missing:
        raise ValueError('Explicit kept snapshot was not discovered: ' + str(sorted(missing)[0]))
    reasons, series, omissions = {}, {}, []
    for row in snapshots:
        p = row['path']; stem = p.stem
        if all_snapshots:
            reasons[p] = 'integral snapshot export requested'
        elif p in keep:
            reasons[p] = 'explicit --keep-snapshot'
        elif p in protected:
            reasons[p] = 'catalog, original source or report/replay dependency'
        elif stem in {'seed','start','end','expected','origin','capture','left','right','previous'} or stem.endswith(('-before','-after')):
            reasons[p] = 'named reproduction/comparison boundary'
        elif re.match(r'^(slot-\d+|(?:DS|IN|GEN|NEW)-)', stem, re.I):
            reasons[p] = 'independent numbered save/case'
        else:
            match = re.fullmatch(r'(.+?)[-_](\d+)', stem)
            directory = re.fullmatch(r'(\d+)[-_].+',p.parent.name)
            if match:
                key = (str(p.parent), match[1], row.get('rom_sha256',''), row.get('bios_sha256',''))
                series.setdefault(key, []).append((int(match[2]), row))
            elif directory and stem in {'navigation-end','end-state','step-end'}:
                key = (str(p.parent.parent), stem, row.get('rom_sha256',''), row.get('bios_sha256',''))
                series.setdefault(key, []).append((int(directory[1]), row))
            else:
                reasons[p] = 'descriptive or standalone checkpoint'
    for group in series.values():
        group.sort(key=lambda x:(x[0],x[1]['source']))
        # Distinct numbers only; never discard same-number scene branches.
        numbers = {number for number,_ in group}
        for number,row in group:
            if number in (min(numbers),max(numbers)):
                reasons[row['path']] = 'first/last point of numbered capture sequence'
            else:
                representative = group[0][1] if number-min(numbers) < max(numbers)-number else group[-1][1]
                omissions.append({'source':row['source'],'sha256':row['sha256'],
                    'bytes':row['bytes'],'reason':'intermediate numbered micro-snapshot',
                    'representative':representative['source']})
    omitted_sources = {r['source'] for r in omissions}
    selected = []
    for row in rows:
        if row['kind'] == 'snapshot':
            if row['source'] in omitted_sources: continue
            row['selection'] = reasons[row['path']]
        selected.append(row)
    return selected, omissions


def complete_catalog(data):
    """Project previously hidden fields using the existing runtime schema."""
    data = json.loads(json.dumps(data))
    paths = {i.get('file') for case in data['cases'] for i in case.get('checkpoints', [])}
    paths.update(i.get('file') for key in ('variants', 'variant_slots') for i in data.get(key, []))
    for item in data.get('loaded_checkpoints', []):
        if item['file'] not in paths:
            item = dict(item, name=Path(item['file']).stem, catalog_key='LOADED/' + item['file'])
            data.setdefault('variants', []).append(item)
            paths.add(item['file'])
    base = data.get('eur_base')
    if isinstance(base, dict) and base.get('file'):
        data.setdefault('generated_standard_saves', []).append(dict(base, id='BASE-EUR', region='EUR', scope='Existing EUR base cartridge; original source status unchanged.'))
    data['export_projection'] = 'Loaded checkpoints and EUR base exposed without changing source catalog or recorded scene acceptance.'
    return data


def catalog_pages(rows, roms, known_saves):
    """One selectable item per unique payload/region, bounded for old runtimes."""
    grouped = {}
    state_directories = {}
    for row in rows:
        region = next((name for name, sha in roms.items() if sha == row.get('rom_sha256')), None)
        if region:
            parent = Path(row['source']).parent
            state_directories.setdefault(parent,set()).add(region)
            for ancestor in parent.parents:
                if ancestor.name == 'SaveStates':
                    state_directories.setdefault(ancestor.parent,set()).add(region)
                    break
    for row in rows:
        regions = set()
        if row['kind'] == 'snapshot' and 'rom_sha256' in row:
            regions = {name for name, sha in roms.items() if sha == row['rom_sha256']}
        elif row['kind'] == 'cartridge-save' and row['bytes'] == 65536:
            regions = known_saves.get(row['sha256'], set()) or state_directories.get(Path(row['source']).parent,set())
            if not regions:
                matches = set(re.findall(r'(?:^|[-_/ ])(EUR|USA|EUROPE)(?=$|[-_/ ])', row['source'].upper()))
                matches = {'EUR' if x == 'EUROPE' else x for x in matches}
                if len(matches) == 1:
                    regions = matches
        if not regions:
            if row['kind'] in ('snapshot', 'cartridge-save'):
                row.setdefault('issue', 'ROM region unidentified; use the raw file with its original compatible runtime/ROM')
            continue
        for region in sorted(regions):
            key = (region, row['kind'], row.get('state_content_sha256',row['sha256']))
            grouped.setdefault(key, []).append(row)
    pages, batch = {}, {}
    for (region, kind, identity), aliases in sorted(grouped.items()):
        # Prefer a preview-bearing collection copy over a raw alias.
        aliases.sort(key=lambda r:(not r['source'].startswith('debug_saves/'), -r['bytes'], r['source']))
        row = aliases[0]; sha=row['sha256']
        group = batch.setdefault(region, [])
        page = len(group) // CATALOG_PAGE + 1
        name = f'DEBUG-CATALOG-{region}-{page:03}.json'
        catalog = pages.setdefault(name, {'schema': 1, 'cases': [], 'variants': [], 'generated_standard_saves': []})
        scope = ('Discovered export asset; scene/gameplay acceptance unverified. '
                 'Snapshot envelope identity only; loading still checks ROM, BIOS and runtime. '
                 if kind == 'snapshot' else
                 'Discovered cartridge copy; region inferred from collection hash or source path, not a new gameplay check. ')
        item = {'file': row['archive'], 'sha256': sha, 'region': region,
                'name': Path(row['source']).name + ' — ' + row['source'],
                'source': row['source'], 'scope': scope + f'{len(aliases)} source path(s); see DEBUG-ASSETS-INVENTORY.json.',
                'catalog_key': 'ASSET/' + region + '/' + identity}
        if kind == 'snapshot':
            catalog['variants'].append(item)
        else:
            item['id'] = 'ASSET-' + sha
            catalog['generated_standard_saves'].append(item)
        for alias in aliases:
            alias.setdefault('catalog',name)
            alias.setdefault('catalogs',[]).append(name)
        group.append(identity)
    return pages


def index_html(rows, pages, exclusions, selection=None):
    """A local searchable index, independent of menu filtering and slot limits."""
    data = [{k: v for k, v in row.items() if k != 'path'} for row in rows]
    encoded = json.dumps(data, ensure_ascii=False).replace('<', '\\u003c').replace('>', '\\u003e').replace('&', '\\u0026')
    links = ''.join(f'<li><a href="{quote(n)}">{html.escape(Path(n).name)}</a></li>' for n in pages)
    selection = selection or {}
    selected_note = f"<p>Snapshot selection: {html.escape(selection.get('mode','all'))}. Omitted intermediate snapshot paths: {len(selection.get('omitted',[]))}. The full inventory records each omission and its representative. Original files are unchanged.</p>"
    return ('''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>Private debug assets</title><style>body{font:16px system-ui;max-width:1200px;margin:32px auto;padding:0 16px;background:#111820;color:#e7edf4}a{color:#80c7ff}input{font:inherit;width:95%;padding:10px}td,th{text-align:left;padding:9px;border-bottom:1px solid #43515e;overflow-wrap:anywhere}table{width:100%;table-layout:fixed}th:first-child{width:52%}.issue{color:#ffd18b}summary{cursor:pointer}small{color:#b6c4d0}</style>
<h1>Private debug assets</h1><p>Search exported snapshots, cartridge saves, inputs, recordings and supporting reports. Saves initially show one row per distinct state content; search also finds the retained source aliases.</p>''' + selected_note + '''
<p>Open a catalog below with <b>Debug → Sessions and cases → Debug catalog → Choose / reload catalog</b>. Set Region, Recorded status, Kind and DS case to <b>All</b>. Choose <b>Open isolated case</b>. EUR and USA require their compatible ROMs; snapshots also require their original BIOS/runtime. Catalog filenames below identify the region.</p>
<p>Discovery does not validate gameplay. Invalid or unidentified states remain available as raw files with an explanation. Recording directories retain their sibling files; incomplete recordings remain diagnostic material.</p>
<details><summary>Additional save catalogs</summary><ul>''' + links + '''</ul></details>
<p><a href="DEBUG-ASSETS-INVENTORY.json">Full inventory, checksums and explicit exclusions</a> · <a href="Data/config/debug-catalog/catalog.json">Original collection with loaded checkpoints exposed</a></p>
<label>Asset type <select id="kind"><option value="saves">Saves and snapshots</option><option value="input">Inputs</option><option value="support">Reports and supporting files</option><option value="all">All exported source paths</option></select></label>
<label>Search assets <input id="search" type="search" placeholder="Filename, scene, source path, type, region, issue…"></label><p id="count"></p>
<table><thead><tr><th>Original source</th><th>Type / identity</th><th>Files / catalog</th></tr></thead><tbody id="rows"></tbody></table><p id="more"></p>
<script>const assets=''' + encoded + ''';
const search=document.querySelector('#search'),kind=document.querySelector('#kind'),body=document.querySelector('#rows');
function link(td,name,label){let a=document.createElement('a');a.href=name.split('/').map(encodeURIComponent).join('/');a.textContent=label;td.append(a,document.createElement('br'));}
function render(){let q=search.value.toLowerCase();let matches=assets.filter(x=>(kind.value==='all'||kind.value==='saves'&&['snapshot','cartridge-save'].includes(x.kind)||x.kind===kind.value)&&JSON.stringify(x).toLowerCase().includes(q));let groups=new Map();for(let x of matches){let key=kind.value==='saves'?(x.state_content_sha256||x.sha256):x.source;let g=groups.get(key);if(g)g.aliases.push(x.source);else groups.set(key,{...x,aliases:[x.source]});}let found=[...groups.values()];document.querySelector('#count').textContent=found.length+' results · '+matches.length+' source paths';body.replaceChildren();for(let x of found.slice(0,500)){let tr=document.createElement('tr'),s=document.createElement('td'),k=document.createElement('td'),a=document.createElement('td');s.textContent=x.source;if(x.aliases.length>1){let d=document.createElement('details'),t=document.createElement('summary'),p=document.createElement('p');t.textContent=x.aliases.length+' source aliases';p.textContent=x.aliases.join('\\n');d.append(t,p);s.append(d);}k.textContent=x.kind+(x.rom_sha256?' · ROM '+x.rom_sha256.slice(0,12):'');if(x.issue){let p=document.createElement('p');p.className='issue';p.textContent=x.issue;k.append(p);}if(x.selection){let p=document.createElement('small');p.textContent=x.selection;k.append(document.createElement('br'),p);}link(a,x.archive,'Open / save file');for(let c of x.catalogs||[x.catalog].filter(Boolean))link(a,c,'Catalog: '+c);tr.append(s,k,a);body.append(tr);}document.querySelector('#more').textContent=found.length>500?'Showing first 500 matches. Refine search to see the remaining matches.':'';}search.addEventListener('input',render);kind.addEventListener('change',render);render();</script></html>''').encode('utf-8')
