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


def digest(data):
    return hashlib.sha256(data).hexdigest()


def project(catalog_bytes, table_bytes, rom, *, expected_sha=USA_SHA256, expected_crc=MASTER_CHECKSUM):
    if digest(rom) != expected_sha or len(rom) < 65536:
        raise ValueError('The ROM does not match the explicitly recognized USA image')
    if binascii.crc_hqx(rom[:65536], 0xffff) != expected_crc:
        raise ValueError('The ROM does not match the declared master checksum')
    table = table_bytes.decode('utf-8')
    section = table.split('## Tabella Abilità', 1)[1].split('## Tabella Razza', 1)[0]
    values = [int(m.group(1), 16) for m in re.finditer(r'^([0-9A-F]{2})\s+', section, re.MULTILINE)]
    if values != list(range(0x23)):
        raise ValueError('Ability table is incomplete, reordered or has duplicate values')
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
        })
        derived.append(item)
    return {'schema': 1, 'derivation': {
        'kind': 'private-usa-primary-ability-projection',
        'source_catalog_sha256': digest(catalog_bytes), 'source_table_sha256': digest(table_bytes),
        'rom_sha256': expected_sha, 'master_crc16': f'{expected_crc:04X}',
        'scope': 'Source-listed ability-set labels only. This is a derived USA declaration based on the master checksum; original provenance and region are retained. Game effects and physical-device behavior are unverified.',
        'reference': 'https://gamefaqs.gamespot.com/gba/560436-final-fantasy-tactics-advance/faqs/26118',
    }, 'entries': derived}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('catalog', 'table', 'rom', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    projection = project(args.catalog.read_bytes(), args.table.read_bytes(), args.rom.read_bytes())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x', encoding='utf-8') as output:
        json.dump(projection, output, ensure_ascii=False, indent=2)
        output.write('\n')
    print('Created a derived USA ability catalog; source inputs were not modified.')


if __name__ == '__main__':
    main()
