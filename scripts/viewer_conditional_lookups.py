"""Validated projections of documented European conditional lookup data."""


def _uint(value, maximum):
    return type(value) is int and 0 <= value <= maximum


def _records(value, count, fields):
    if not isinstance(value, list) or len(value) != count:
        raise ValueError('Invalid conditional record count')
    result = []
    for item in value:
        if not isinstance(item, dict) or any(not _uint(item.get(name), maximum) for name, maximum in fields.items()):
            raise ValueError('Invalid conditional record field')
        result.append({name: item[name] for name in fields})
    return result


DESCRIPTOR_FIELDS = {'field0': 0xffff, 'field2': 0xffff, 'field4': 0xffff, 'kind': 0xff, 'byte7': 0xff}
PACKED_FIELDS = {'byte0': 0xff, 'low5': 31, 'high3': 7, 'low2': 3, 'high6': 63, 'byte3': 0xff}


def project_descriptor_banks(catalog):
    banks = catalog['banks']
    if (catalog['pointer_offset'] != 0x6df4d8 or not isinstance(banks, list) or len(banks) != 23):
        raise ValueError('Invalid conditional descriptor directory')
    projected, last_end, total = [], 0x6df5b4, 0
    for index, bank in enumerate(banks, 1):
        source, records = bank['source_offset'], bank['records']
        if (type(bank['index']) is not int or bank['index'] != index or not _uint(source, 0xffffff)
                or source < last_end or not isinstance(records, list) or not 0 < len(records) <= 255
                or source + len(records) * 8 > 0x6e0b44):
            raise ValueError('Invalid conditional descriptor bank')
        projected.append({'index': index, 'source_offset': source,
                          'records': _records(records, len(records), DESCRIPTOR_FIELDS)})
        last_end = source + len(records) * 8; total += len(records)
    if total != 684:
        raise ValueError('Incomplete conditional descriptor banks')
    return {'pointer_offset': catalog['pointer_offset'], 'banks': projected}


def project_descriptor_zero(catalog):
    if (catalog['pointer_offset'] != 0x6df4d4 or catalog['bank_root'] != 0x6df534
            or catalog['first_row'] != 1):
        raise ValueError('Invalid conditional bank-zero source')
    return {'pointer_offset': catalog['pointer_offset'], 'bank_root': catalog['bank_root'], 'first_row': 1,
            'records': _records(catalog['records'], 15, DESCRIPTOR_FIELDS)}


def project_packed_lookup(catalog):
    if catalog['source_offset'] != 0x6e6ee8 or catalog['first_index'] != 1:
        raise ValueError('Invalid conditional packed lookup source')
    return {'source_offset': catalog['source_offset'], 'first_index': 1,
            'records': _records(catalog['records'], 33, PACKED_FIELDS)}


def project_four_byte_lookup(catalog):
    if catalog['source_offset'] != 0x6e6dbc:
        raise ValueError('Invalid conditional four-byte lookup source')
    return {'source_offset': catalog['source_offset'],
            'records': _records(catalog['records'], 74, {'first': 0xffff, 'second': 0xffff})}


def project_property_masks(catalog):
    words = catalog['words']
    if (catalog['source_offset'] != 0x6e0b44 or not isinstance(words, list) or len(words) != 35
            or any(not _uint(word, 0xffffffff) for word in words)):
        raise ValueError('Invalid conditional property masks')
    return {'source_offset': catalog['source_offset'], 'words': list(words)}


CONDITIONAL_LOOKUPS = {
    'gameplay.conditional-descriptor-banks': ('conditional-descriptor-banks-v1-json', 'banks',
        '31f6b49dc4ca7c7efc8138a1ec46bef0b325bcf2011606aabdf799294ea370c7', project_descriptor_banks),
    'gameplay.conditional-descriptor-bank-zero': ('conditional-descriptor-bank-zero-v1-json', 'zero',
        'e12f26c1103ebac6f472b3eba602b5edc4161638e0b555c6d8fddac7168d1215', project_descriptor_zero),
    'gameplay.conditional-packed-lookup': ('conditional-packed-lookup-v1-json', 'packed',
        '1960b9bc69696642eec89ac34c70aaf09c774824019a1c09ae96b8239655760d', project_packed_lookup),
    'gameplay.conditional-four-byte-lookup': ('conditional-four-byte-lookup-v1-json', 'four',
        '3d327931931b0393db1688595de5444ca4ba5bd695ff1d136d8579f7c7dc3066', project_four_byte_lookup),
    'gameplay.conditional-property-masks': ('conditional-property-masks-v1-json', 'masks',
        '8e631e3c90b6db34c3994b6ba4bfba959f253b551e2f7ae307834c415663004f', project_property_masks),
}
