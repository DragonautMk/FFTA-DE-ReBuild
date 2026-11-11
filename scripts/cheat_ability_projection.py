#!/usr/bin/env python3
"""Create an explicitly derived private USA ability catalog from local inputs."""
import argparse
import binascii
import hashlib
import json
import re
from pathlib import Path

USA_SHA256 = '43fc8204c6dceee58828aebc7af0c72eb807e99f35ad641c8bb0a4fa8b6edc19'
MASTER_CHECKSUM = 0x37AE
SOURCE = 'XP-reviewed-usa-ability-projection'
THIRD_APPEARANCE_SOURCE = 'derived-us-third-appearance'
THIRD_APPEARANCE_ID = 'CH-9020'
THIRD_APPEARANCE_SOURCE_VALUES_ID = 'CH-9022'
THIRD_APPEARANCE_REFERENCE = 'https://etherealgames.com/gba/f/final-fantasy-tactics-advance/code-breaker-codes-us/'
FIRST_JOB_SOURCE = 'derived-usa-first-record-job-byte'
FIRST_JOB_ID = 'CH-9021'
FIRST_HUMAN_TITLES_ID = 'CH-9023'
FIRST_HUMAN_JOB_ABILITY_ID = 'CH-9024'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def character_reference_parameters(table):
    """Parse source descriptions for browsing only; these are not safe domains."""
    race_text = table.split('## Tabella Razza', 1)[1].split('## Tabella Apparenza', 1)[0]
    appearance_text = table.split('## Tabella Apparenza', 1)[1]
    headers = {'01 Human': 1, '02 Bangaa': 2, '03 Numou': 3, '04 Viera': 4, '05 Moogle': 5}
    cases = {value: [] for value in headers.values()}
    job_labels = []
    current = None
    for raw in race_text.splitlines():
        line = raw.strip()
        if line in headers:
            current = headers[line]
            continue
        if line == '06 Monster':
            break
        match = re.fullmatch(r'([0-9A-F]{2})\s+(.+)', line)
        if match and current is not None:
            value = int(match[1], 16)
            label = match[2].strip()
            if value in [item['value'] for item in job_labels]:
                raise ValueError('Duplicate source job value')
            cases[current].append(value)
            if value == 1:
                label += ' (source; USA row 01 unresolved)'
            elif value == 2:
                label += ' (source; USA row 02 is Soldier)'
            job_labels.append({'value': value, 'label': label})
    expected = {1: list(range(1, 13)), 2: list(range(13, 20)), 3: list(range(20, 28)),
                4: list(range(28, 36)), 5: list(range(36, 44))}
    if cases != expected:
        raise ValueError('Source playable job groups are incomplete, reordered or ambiguous')
    appearance_values, appearance_labels = [], []
    for raw in appearance_text.splitlines():
        line = raw.strip()
        match = re.fullmatch(r'([0-9A-F]{2})(?:-([0-9A-F]{2}))?\s+-\s+(.+)', line)
        if not match:
            if re.match(r'^[0-9A-F]{2}(?:-|\s+-)', line):
                raise ValueError('Malformed source appearance row')
            continue
        first, last = int(match[1], 16), int(match[2] or match[1], 16)
        if first > last:
            raise ValueError('Invalid source appearance range')
        label = match[3].strip()
        for value in range(first, last + 1):
            if value in appearance_values:
                raise ValueError('Duplicate source appearance value')
            appearance_values.append(value)
            appearance_labels.append({'value': value, 'label': label})
    if appearance_values != list(range(0xe1)) + [0xf0]:
        raise ValueError('Source appearance values are incomplete, reordered or ambiguous')
    for label in job_labels + appearance_labels:
        value = label['label']
        if not value or len(value) > 80 or len(value.encode('utf-8')) > 256 or any(ord(c) < 32 or 0x7f <= ord(c) <= 0x9f for c in value):
            raise ValueError('Invalid source reference label')
    return [
        {'name': 'race', 'allowed': {'kind': 'list', 'values': list(cases)},
         'labels': [{'value': value, 'label': name.split(' ', 1)[1]} for name, value in headers.items()],
         'slots': [{'line': 1, 'field': 'value', 'offset': 2, 'digits': 2}]},
        {'name': 'job', 'allowed': {'kind': 'dependent-list', 'on': 'race',
                                    'cases': [{'when': race, 'values': values} for race, values in cases.items()]},
         'labels': job_labels,
         'slots': [{'line': 0, 'field': 'value', 'offset': 0, 'digits': 2}, {'line': 1, 'field': 'value', 'offset': 0, 'digits': 2}]},
        {'name': 'appearance', 'allowed': {'kind': 'list', 'values': appearance_values},
         'labels': appearance_labels, 'slots': [{'line': 0, 'field': 'value', 'offset': 2, 'digits': 2}]},
    ], {'race_job': race_text, 'appearance': appearance_text}


def project(catalog_bytes, table_bytes, rom, *, expected_sha=USA_SHA256, expected_crc=MASTER_CHECKSUM, character_examples=False, third_appearance=False, character_reference_values=False, first_job_thief=False, third_appearance_source_values=False, first_human_job_titles=False, first_human_job_ability=False):
    if character_examples and character_reference_values:
        raise ValueError('Choose either the two examples or the source reference lists')
    if digest(rom) != expected_sha or len(rom) < 65536:
        raise ValueError('The ROM does not match the explicitly recognized USA image')
    if binascii.crc_hqx(rom[:65536], 0xffff) != expected_crc:
        raise ValueError('The ROM does not match the declared master checksum')
    table = table_bytes.decode('utf-8')
    section = table.split('## Tabella Abilità', 1)[1].split('## Tabella Razza', 1)[0]
    rows = [(int(m.group(1), 16), m.group(2).strip())
            for m in re.finditer(r'^([0-9A-F]{2})[ \t]+([^\r\n]+)', section, re.MULTILINE)]
    values = [value for value, _ in rows]
    ability_rows = dict(rows)
    if values != list(range(0x23)):
        raise ValueError('Ability table is incomplete, reordered or has duplicate values')
    if any(not label or len(label) > 80 or len(label.encode('utf-8')) > 256
           or any(ord(char) < 32 or 0x7f <= ord(char) <= 0x9f for char in label)
           for _, label in rows):
        raise ValueError('Ability table contains an invalid label')
    original = json.loads(catalog_bytes)
    if original.get('schema') != 1:
        raise ValueError('Unsupported original catalog schema')
    entries = original.get('entries', [])
    by_id = {entry['id']: entry for entry in entries}
    if len(by_id) != len(entries):
        raise ValueError('Original catalog has duplicate entry IDs')

    def provenance(entry):
        matches = [p for p in entry['provenance'] if p.get('source') == 'XP']
        if len(matches) != 1:
            raise ValueError('The selected entry lacks a unique XP source')
        return matches[0]

    master = by_id['CH-0225']
    master_source = provenance(master)
    if master['format'].lower() != 'codebreaker' or master['region'] != 'unknown':
        raise ValueError('Unexpected original master format or region')
    raw_master = master_source['raw_code_lines']
    if len(raw_master) != 2:
        raise ValueError('The master is not the reviewed two-line form')
    game, flags = (int(v, 16) for v in raw_master[0].split())
    hook, slot = (int(v, 16) for v in raw_master[1].split())
    if game != expected_crc or flags > 15 or flags & 8 == 0 or hook >> 28 != 1 or hook & 0x0e000001 or slot > 7:
        raise ValueError('The master does not match the reviewed checksum and hook shape')

    def base(entry, source):
        result = dict(entry)
        result['region'] = 'USA'
        result['provenance'] = [{
            'source': SOURCE,
            'original_source': 'XP',
            'original_region': entry['region'],
            'original_provenance': source,
        }]
        return result

    derived_master = base(master, master_source)
    derived_master['provenance'][0]['raw_code_lines'] = raw_master
    derived = [derived_master]
    for number in range(226, 234):
        entry = by_id[f'CH-{number:04}']
        source = provenance(entry)
        codes = entry['codes']
        raw = source['raw_code_lines']
        if entry['format'].lower() != 'codebreaker' or entry['region'] != 'unknown' or source.get('master_id') != 'CH-0225':
            raise ValueError(f'Unexpected original identity for {entry["id"]}')
        if len(codes) != 1 or len(raw) != 1 or raw[0].partition(':')[0].strip() != codes[0] or not re.fullmatch(r'[0-9A-Fa-f]{8} 00[xX]{2}', codes[0]):
            raise ValueError(f'Unexpected annotated source line for {entry["id"]}')
        item = base(entry, source)
        if number == 226:
            item['notes'] = list(entry['notes']) + [
                'In one USA battle checkpoint, value 09 changed the command label to White Magic, but Battle Tech remained and First Aid healed 65 to 106 HP in both copies.',
            ]
        projected = item['provenance'][0]
        projected.update(master_id='CH-0225', raw_code_lines=list(codes), parameter={
            'name': 'ability_set_label', 'allowed': {'kind': 'list', 'values': values},
            'slots': [{'line': 0, 'field': 'value', 'offset': 2, 'digits': 2}],
            'labels': [{'value': value, 'label': label} for value, label in rows],
        })
        derived.append(item)
    if character_examples or character_reference_values:
        race_table = table.split('## Tabella Razza', 1)[1].split('## Tabella Apparenza', 1)[0]
        moogle = race_table.split('05 Moogle', 1)[1].split('06 Monster', 1)[0]
        appearance = table.split('## Tabella Apparenza', 1)[1]
        if not re.search(r'^26\s+Gunner\s*$', moogle, re.MULTILINE) or not re.search(r'^2B\s+Time Mage\s*$', moogle, re.MULTILINE):
            raise ValueError('The source job/race examples no longer match the local table')
        if not re.search(r'^02\s+-\s+Marche\b', appearance, re.MULTILINE) or not re.search(r'^0D\s+-\s+Llend', appearance, re.MULTILINE):
            raise ValueError('The source appearance examples no longer match the local table')
        # The guide supplies these two tuples as examples, not a complete
        # compatibility table.  CH-0236 has a separately unresolved address.
        reference = character_reference_parameters(table) if character_reference_values else None
        baseline = int(by_id['CH-0234']['codes'][0].split()[0], 16)
        for number in (234, 235, 237, 238, 239, 240, 241):
            entry = by_id[f'CH-{number:04}']
            source = provenance(entry)
            codes = entry['codes']
            if entry['format'].lower() != 'codebreaker' or entry['region'] != 'unknown' or source.get('master_id') != 'CH-0225':
                raise ValueError(f'Unexpected original identity for {entry["id"]}')
            if len(codes) != 2 or source['raw_code_lines'] != codes or not re.fullmatch(r'[0-9A-Fa-f]{8} [xX]{2}[yY]{2}', codes[0]) or not re.fullmatch(r'[0-9A-Fa-f]{8} [xX]{2}[zZ]{2}', codes[1]):
                raise ValueError(f'Unexpected character source lines for {entry["id"]}')
            first = int(codes[0].split()[0], 16)
            second = int(codes[1].split()[0], 16)
            if first != baseline + (number - 234) * 0x108 or second != first + 2 or first >> 28 != 8:
                raise ValueError(f'Unexpected character source addresses for {entry["id"]}')
            item = base(entry, source)
            # The observed USA checkpoint does not yet establish the meaning
            # of every field. An unknown region keeps these derived records
            # non-executable even in older runtimes; the original is untouched.
            item['region'] = entry['region']
            projected = item['provenance'][0]
            if reference is not None:
                projected.update(master_id='CH-0225', raw_code_lines=list(codes),
                                 reference_parameters=reference[0], reference_source_tables=reference[1],
                                 interpretation='Reference only: source lists include uncertain, non-working and crashing values. No combination is authorized for execution; original source labels and warnings are retained.')
                derived.append(item)
                continue
            projected.update(master_id='CH-0225', raw_code_lines=list(codes), parameters=[
                {'name': 'race', 'allowed': {'kind': 'list', 'values': [5]},
                 'slots': [{'line': 1, 'field': 'value', 'offset': 2, 'digits': 2}]},
                {'name': 'job', 'allowed': {'kind': 'dependent-list', 'on': 'race',
                 'cases': [{'when': 5, 'values': [0x26, 0x2b]}]},
                 'slots': [{'line': 0, 'field': 'value', 'offset': 0, 'digits': 2}, {'line': 1, 'field': 'value', 'offset': 0, 'digits': 2}]},
                {'name': 'appearance', 'allowed': {'kind': 'dependent-list', 'on': 'job',
                 'cases': [{'when': 0x26, 'values': [2]}, {'when': 0x2b, 'values': [0x0d]}]},
                 'slots': [{'line': 0, 'field': 'value', 'offset': 2, 'digits': 2}]},
            ], interpretation='Reference only: the cited guide identifies the fields as job, appearance and race, but the original XP notes differ and a USA checkpoint leaves one field unresolved. These examples cannot be activated until reviewed.')
            derived.append(item)
    if third_appearance or third_appearance_source_values:
        # This is a separate US source, not a correction of XP's CH-0236.
        # Only value 02 has been observed for this address in the isolated
        # USA checkpoint; retain the existing quarantine for the XP tuples.
        if third_appearance and THIRD_APPEARANCE_ID in by_id:
            raise ValueError('The reserved derived appearance ID exists in the original catalog')
        if third_appearance_source_values and THIRD_APPEARANCE_SOURCE_VALUES_ID in by_id:
            raise ValueError('The reserved source-listed appearance ID exists in the original catalog')
        if raw_master != ['000037AE 000A', '10000488 0007']:
            raise ValueError('The original master differs from the separately cited USA listing')
        appearance = table.split('## Tabella Apparenza', 1)[1]
        if not re.search(r'^02\s+-\s+Marche\b', appearance, re.MULTILINE):
            raise ValueError('The source appearance value no longer matches the local table')
        derived_master['provenance'].append({
            'source': THIRD_APPEARANCE_SOURCE,
            'raw_code_lines': list(raw_master),
            'reference': THIRD_APPEARANCE_REFERENCE,
            'original_master_source': 'XP',
            'scope': 'Identical master lines in the local XP record and the separately cited US listing.',
        })
        code = '32000294 00XX'
        if third_appearance:
            derived.append({
                'id': THIRD_APPEARANCE_ID, 'format': 'codebreaker', 'region': 'USA',
                'section': 'derived third-character appearance',
                'title': 'Third character appearance (separate US source)',
                'codes': [code], 'aliases': [], 'status': 'observed-one-usa-checkpoint',
                'notes': [
                    'Derived private entry, not the imported CH-0236 or a correction of it.',
                    'Only value 02 was observed on one roster screen; other scenes and device behavior remain unverified.',
                ],
                'provenance': [{
                    'source': THIRD_APPEARANCE_SOURCE, 'master_id': 'CH-0225',
                    'raw_code_lines': [code], 'reference': THIRD_APPEARANCE_REFERENCE,
                    'source_placeholder': '??',
                    'interpretation': 'The listing names the address as character 3 appearance. Value 02 comes from the local appearance table and one isolated USA observation.',
                    'parameter': {'name': 'appearance', 'allowed': {'kind': 'list', 'values': [2]},
                                  'slots': [{'line': 0, 'field': 'value', 'offset': 2, 'digits': 2}],
                                  'labels': [{'value': 2, 'label': 'Appearance 02 (observed on one roster screen)'}]},
                }],
            })
        if third_appearance_source_values:
            reference, _ = character_reference_parameters(table)
            source_labels = {item['value']: item['label'] for item in reference[2]['labels']}
            for value, phrase in [(0x00, 'glitches'), (0x60, 'non-working'), (0xaf, 'non-working'),
                                  (0xb0, 'crashes'), (0xb1, 'non-working'), (0xbf, 'non-working')]:
                if phrase not in source_labels[value].lower():
                    raise ValueError('Source appearance warnings changed; review the exclusions')
            values = list(range(0x01, 0x60)) + list(range(0xc0, 0xe1)) + [0xf0]
            labels = [{'value': value, 'label': source_labels[value]} for value in values]
            for item in labels:
                if item['value'] == 0xca:
                    item['label'] += ' [green portrait in one USA scene]'
                elif item['value'] == 0xf0:
                    item['label'] += ' [blue portrait in one USA scene]'
            derived.append({
                'id': THIRD_APPEARANCE_SOURCE_VALUES_ID, 'format': 'codebreaker', 'region': 'USA',
                'section': 'derived third-character source-listed appearances',
                'title': 'Third appearance: source values (unverified)',
                'codes': [code], 'aliases': [], 'status': 'not-runtime-tested',
                'notes': [
                    'Source-listed values, not verified compatibility or safe transitions; 02, 03, 0D, CA and F0 were observed on one USA roster screen.',
                    'The source marks 00 as glitching, 60-AF and B1-BF as non-working, and B0 as crashing; those values are excluded.',
                    'Other values may also fail. This record is not the original CH-0236, and the slot does not establish actor identity.',
                ],
                'provenance': [{
                    'source': THIRD_APPEARANCE_SOURCE, 'master_id': 'CH-0225',
                    'raw_code_lines': [code], 'reference': THIRD_APPEARANCE_REFERENCE,
                    'source_placeholder': '??',
                    'interpretation': 'The separate US listing names the third appearance address. Values and descriptions come from the local table; gameplay and physical devices remain unverified.',
                    'parameter': {'name': 'appearance', 'allowed': {'kind': 'list', 'values': values},
                                  'slots': [{'line': 0, 'field': 'value', 'offset': 2, 'digits': 2}],
                                  'labels': labels},
                }],
            })
    if first_job_thief or first_human_job_titles or first_human_job_ability:
        # A new conditional one-byte plan derived from a bounded USA observation,
        # not a transcription or correction of the original two-line source.
        if first_job_thief and FIRST_JOB_ID in by_id:
            raise ValueError('The reserved derived job ID exists in the original catalog')
        if first_human_job_titles and FIRST_HUMAN_TITLES_ID in by_id:
            raise ValueError('The reserved derived human title ID exists in the original catalog')
        if first_human_job_ability and FIRST_HUMAN_JOB_ABILITY_ID in by_id:
            raise ValueError('The reserved derived job/ability ID exists in the original catalog')
        if raw_master != ['000037AE 000A', '10000488 0007']:
            raise ValueError('The original master differs from the observed USA master')
        player = by_id['CH-0234']
        player_source = provenance(player)
        if (player['format'].lower() != 'codebreaker' or player['region'] != 'unknown'
                or player_source.get('master_id') != 'CH-0225'
                or player['codes'] != ['82000084 xxyy', '82000086 xxzz']
                or player_source['raw_code_lines'] != player['codes']):
            raise ValueError('The original first-character source differs from the observed addresses')
        human = table.split('## Tabella Razza', 1)[1].split('01 Human', 1)[1].split('02 Bangaa', 1)[0]
        if not re.search(r'^05\s+Thief\s*$', human, re.MULTILINE):
            raise ValueError('The source human job value no longer matches the observation')
        derived_master['provenance'].append({
            'source': FIRST_JOB_SOURCE, 'raw_code_lines': list(raw_master),
            'original_master_source': 'XP',
            'scope': 'Known USA checksum and the original master; the conditional job plan is independently derived.',
        })
        lines = ['72000086 0201', '32000087 00XX']
        if first_job_thief:
            derived.append({
                'id': FIRST_JOB_ID, 'format': 'codebreaker', 'region': 'USA',
                'section': 'derived first-record job byte',
                'title': 'First record: observed human Soldier to Thief',
                'codes': lines, 'aliases': [], 'status': 'observed-one-usa-checkpoint',
                'notes': [
                    'Independently derived from an isolated USA observation; not the original CH-0234.',
                    'Condition checks bytes 86-87 are 01 02 before writing 05 to byte 87; it does not identify the character or validate the entire state.',
                    'One checkpoint and ability screen observed. Other scenes, maintained writes and physical-device behavior remain unverified.',
                    'This byte write does not perform the ordinary job-change equipment handling; an ordinary Hunter change displayed an unequip notice.',
                ],
                'provenance': [{
                    'source': FIRST_JOB_SOURCE, 'master_id': 'CH-0225', 'raw_code_lines': lines,
                    'original_source': 'XP', 'original_entry': 'CH-0234',
                    'interpretation': 'Derived conditional single-byte plan; 05 changed the displayed title to Thief in one human USA checkpoint. Other field and gameplay effects are unverified.',
                    'parameter': {'name': 'job', 'allowed': {'kind': 'list', 'values': [5]},
                                  'slots': [{'line': 1, 'field': 'value', 'offset': 2, 'digits': 2}],
                                  'labels': [{'value': 5, 'label': 'Thief (one USA ability screen)'}]},
                }],
            })
        if first_human_job_titles:
            observed = [(0x03, 'Paladin'), (0x04, 'Fighter'), (0x05, 'Thief'),
                        (0x06, 'Ninja'), (0x07, 'White Mage'), (0x08, 'Black Mage'),
                        (0x09, 'Illusionist'), (0x0a, 'Blue Mage'),
                        (0x0b, 'Archer'), (0x0c, 'Hunter')]
            rows = [(int(m.group(1), 16), m.group(2).strip())
                    for m in re.finditer(r'^([0-9A-F]{2})[ \t]+([^\r\n]+)', human, re.MULTILINE)]
            if rows != [(1, 'Soldier'), (2, '-')] + observed:
                raise ValueError('The source human job rows differ from the observed comparison')
            derived.append({
                'id': FIRST_HUMAN_TITLES_ID, 'format': 'codebreaker', 'region': 'USA',
                'section': 'derived first-record human displayed job title',
                'title': 'First human record: observed job titles',
                'codes': lines, 'aliases': [], 'status': 'observed-one-usa-checkpoint',
                'notes': [
                    'A derived one-byte plan, not the original CH-0234 or a verified full job change.',
                    'Condition checks bytes 86-87 are 01 02; it does not establish actor identity or the rest of the state.',
                    'Each listed title was observed in one USA ability screen; the visible ability list remained unchanged. Other effects are unverified.',
                    'This byte write does not perform the ordinary job-change equipment handling; an ordinary Hunter change displayed an unequip notice.',
                ],
                'provenance': [{
                    'source': FIRST_JOB_SOURCE, 'master_id': 'CH-0225', 'raw_code_lines': lines,
                    'original_source': 'XP', 'original_entry': 'CH-0234',
                    'interpretation': 'Conditional single-byte plan observed to change displayed titles in one USA checkpoint; actual job mechanics and compatibility are unverified.',
                    'parameter': {'name': 'job_title', 'allowed': {'kind': 'list', 'values': [v for v, _ in observed]},
                                  'slots': [{'line': 1, 'field': 'value', 'offset': 2, 'digits': 2}],
                                  'labels': [{'value': v, 'label': label} for v, label in observed]},
                }],
            })
        if first_human_job_ability:
            ability_entry = by_id['CH-0226']
            ability_source = provenance(ability_entry)
            if (ability_entry['format'].lower() != 'codebreaker' or ability_entry['region'] != 'unknown'
                    or ability_source.get('master_id') != 'CH-0225'
                    or ability_entry['codes'] != ['320000B5 00xx']
                    or len(ability_source['raw_code_lines']) != 1
                    or ability_source['raw_code_lines'][0].partition(':')[0].strip() != ability_entry['codes'][0]):
                raise ValueError('The original primary ability source differs from the observed address')
            pairs = [
                (0x03, 'Paladin', 0x03, 'Chivalry Paladin'),
                (0x04, 'Fighter', 0x04, 'Fighter Tech Fighter'),
                (0x05, 'Thief', 0x18, 'Steal Thief'),
                (0x06, 'Ninja', 0x10, 'Ninja Skill Ninja'),
                (0x07, 'White Mage', 0x09, 'White Magic White Mage'),
                (0x08, 'Black Mage', 0x0a, 'Black Magic Black Mage'),
                (0x09, 'Illusionist', 0x11, 'Phantasm Skill Illusionist'),
                (0x0a, 'Blue Mage', 0x0b, 'Blue Magic Blue Mage'),
                (0x0b, 'Archer', 0x1b, 'Aim Archer'),
                (0x0c, 'Hunter', 0x14, 'Hunt Hunter'),
            ]
            human_rows = [(int(m.group(1), 16), m.group(2).strip())
                          for m in re.finditer(r'^([0-9A-F]{2})[ \t]+([^\r\n]+)', human, re.MULTILINE)]
            if human_rows != [(1, 'Soldier'), (2, '-')] + [(job, name) for job, name, _, _ in pairs]:
                raise ValueError('The source human job rows differ from the observed comparison')
            if any(ability_rows.get(value) != label for _, _, value, label in pairs):
                raise ValueError('The source ability/job associations differ from the reviewed table')
            paired_lines = ['72000086 0201', '320000B5 00YY', '72000086 0201', '32000087 00XX']
            derived.append({
                'id': FIRST_HUMAN_JOB_ABILITY_ID, 'format': 'codebreaker', 'region': 'USA',
                'section': 'derived first-record job and primary ability labels',
                'title': 'First human: job and ability labels (unverified)',
                'codes': paired_lines, 'aliases': [], 'status': 'not-runtime-tested',
                'notes': [
                    'Source-listed job/primary ability labels; not a verified complete job or learned-ability change.',
                    'In one USA checkpoint, byte B6 was already 0A; the paired write set B5 to 0A and 142 candidate progress bytes stayed unchanged along the observed route.',
                    'For Hunter/Hunt in one USA checkpoint, an ordinary job change displayed an unequip notice and changed nine additional record bytes; this pair did not.',
                    'Both writes require bytes 86-87 to be 01 02; this does not establish actor identity or complete state safety.',
                ],
                'provenance': [{
                    'source': FIRST_JOB_SOURCE, 'master_id': 'CH-0225', 'raw_code_lines': paired_lines,
                    'original_source': 'XP', 'original_entries': ['CH-0226', 'CH-0234'],
                    'interpretation': 'Independently derived conditional pair; title and primary ability labels are source-listed. Gameplay, learning and device behavior are unverified.',
                    'parameters': [
                        {'name': 'job_title', 'allowed': {'kind': 'list', 'values': [job for job, _, _, _ in pairs]},
                         'slots': [{'line': 3, 'field': 'value', 'offset': 2, 'digits': 2}],
                         'labels': [{'value': job, 'label': name} for job, name, _, _ in pairs]},
                        {'name': 'primary_ability', 'allowed': {'kind': 'dependent-list', 'on': 'job_title',
                         'cases': [{'when': job, 'values': [ability]} for job, _, ability, _ in pairs]},
                         'slots': [{'line': 1, 'field': 'value', 'offset': 2, 'digits': 2}],
                         'labels': [{'value': ability, 'label': label} for _, _, ability, label in pairs]},
                    ],
                }],
            })
    result = {'schema': 1, 'derivation': {
        'kind': 'private-usa-ability-and-character-examples' if character_examples else 'private-usa-primary-ability-projection',
        'source_catalog_sha256': digest(catalog_bytes), 'source_table_sha256': digest(table_bytes),
        'rom_sha256': expected_sha, 'master_crc16': f'{expected_crc:04X}',
        'scope': 'Source-listed USA ability-set labels and two reference-only character examples. Character regions remain unknown and cannot be activated pending field verification. Original provenance and region are retained; game/device effects are unverified.' if character_examples else 'Source-listed ability-set labels only. This is a derived USA declaration based on the master checksum; original provenance and region are retained. Game effects and physical-device behavior are unverified.',
        'reference': 'https://gamefaqs.gamespot.com/gba/560436-final-fantasy-tactics-advance/faqs/26118',
    }, 'entries': derived}
    if third_appearance:
        result['derivation']['kind'] += '-with-third-appearance'
        result['derivation']['scope'] += ' Separately derived third-character appearance value 02 is included with its own source and limited observation.'
        result['derivation']['third_appearance_reference'] = THIRD_APPEARANCE_REFERENCE
    if third_appearance_source_values:
        result['derivation']['kind'] += '-with-third-appearance-source-values'
        result['derivation']['scope'] += ' Source-listed third-appearance values exclude source-reported faults but remain unverified; no safety or compatibility is implied.'
        result['derivation']['third_appearance_reference'] = THIRD_APPEARANCE_REFERENCE
    if character_reference_values:
        result['derivation']['kind'] += '-with-character-reference-values'
        result['derivation']['scope'] += ' Source-listed character values are reference-only and cannot be executed; the original table and warnings are preserved.'
    if first_job_thief:
        result['derivation']['kind'] += '-with-first-job-thief'
        result['derivation']['scope'] += ' Independently derived conditional first-record job byte 05 is included with one-checkpoint evidence; identity and wider behavior remain unverified.'
    if first_human_job_titles:
        result['derivation']['kind'] += '-with-first-human-job-titles'
        result['derivation']['scope'] += ' Conditional first-record human job titles 03..0C were observed on one USA ability screen; class mechanics and identity remain unverified.'
    if first_human_job_ability:
        result['derivation']['kind'] += '-with-first-human-job-ability'
        result['derivation']['scope'] += ' Conditional source-listed job and primary ability labels are included; only one pair was observed on one USA screen and full behavior remains unverified.'
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('catalog', 'table', 'rom', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--character-examples', action='store_true', help='include two reference-only job/race/appearance examples')
    parser.add_argument('--character-reference-values', action='store_true', help='include non-executable source-listed character values for browsing')
    parser.add_argument('--third-appearance', action='store_true', help='include separately sourced third-character appearance value 02 for the recognized USA ROM')
    parser.add_argument('--third-appearance-source-values', action='store_true', help='include unverified source-listed third appearances, excluding source-reported faults')
    parser.add_argument('--first-job-thief', action='store_true', help='include the conditional first-record job byte observed on one USA checkpoint')
    parser.add_argument('--first-human-job-titles', action='store_true', help='include conditional human job titles observed on one USA screen')
    parser.add_argument('--first-human-job-ability', action='store_true', help='include conditional source-listed job and primary ability label pairs')
    args = parser.parse_args()
    projection = project(args.catalog.read_bytes(), args.table.read_bytes(), args.rom.read_bytes(), character_examples=args.character_examples, third_appearance=args.third_appearance, character_reference_values=args.character_reference_values, first_job_thief=args.first_job_thief, third_appearance_source_values=args.third_appearance_source_values, first_human_job_titles=args.first_human_job_titles, first_human_job_ability=args.first_human_job_ability)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x', encoding='utf-8') as output:
        json.dump(projection, output, ensure_ascii=False, indent=2)
        output.write('\n')
    print('Created a derived USA catalog; source inputs were not modified.')


if __name__ == '__main__':
    main()
