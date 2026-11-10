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
THIRD_APPEARANCE_REFERENCE = 'https://etherealgames.com/gba/f/final-fantasy-tactics-advance/code-breaker-codes-us/'


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


def project(catalog_bytes, table_bytes, rom, *, expected_sha=USA_SHA256, expected_crc=MASTER_CHECKSUM, character_examples=False, third_appearance=False, character_reference_values=False):
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
    if third_appearance:
        # This is a separate US source, not a correction of XP's CH-0236.
        # Only value 02 has been observed for this address in the isolated
        # USA checkpoint; retain the existing quarantine for the XP tuples.
        if THIRD_APPEARANCE_ID in by_id:
            raise ValueError('The reserved derived appearance ID exists in the original catalog')
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
    if character_reference_values:
        result['derivation']['kind'] += '-with-character-reference-values'
        result['derivation']['scope'] += ' Source-listed character values are reference-only and cannot be executed; the original table and warnings are preserved.'
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('catalog', 'table', 'rom', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--character-examples', action='store_true', help='include two reference-only job/race/appearance examples')
    parser.add_argument('--character-reference-values', action='store_true', help='include non-executable source-listed character values for browsing')
    parser.add_argument('--third-appearance', action='store_true', help='include separately sourced third-character appearance value 02 for the recognized USA ROM')
    args = parser.parse_args()
    projection = project(args.catalog.read_bytes(), args.table.read_bytes(), args.rom.read_bytes(), character_examples=args.character_examples, third_appearance=args.third_appearance, character_reference_values=args.character_reference_values)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x', encoding='utf-8') as output:
        json.dump(projection, output, ensure_ascii=False, indent=2)
        output.write('\n')
    print('Created a derived USA catalog; source inputs were not modified.')


if __name__ == '__main__':
    main()
