"""Join authenticated physical M4A records to the existing viewer catalog."""
import struct


BANK_STARTS = (0x14d9fc, 0x14df78, 0x14e578, 0x14eb6c, 0x14f160)
CGB_START, CGB_END = 0x14f47c, 0x14f4fc
ROM_BASE = 0x08000000


def merge_physical_audio(segments, audio, source_catalog):
    """Add missing physical programs and direct waves without renumbering old IDs."""
    if not audio or not audio.get('instruments') or not isinstance(source_catalog, dict):
        raise ValueError('Original tone catalog is required')
    existing = source_catalog['tones']
    if not isinstance(existing, list) or len(existing) != 429:
        raise ValueError('Unexpected existing tone catalog')
    offsets, ids = {}, set()
    for tone in existing:
        offset, identity = tone['source_offset'], tone['id']
        if (type(offset) is not int or type(identity) is not int or identity < 0 or identity in ids
                or not BANK_STARTS[0] <= offset < BANK_STARTS[-1] or (offset - BANK_STARTS[0]) % 12
                or offset in offsets):
            raise ValueError('Invalid existing physical tone identity')
        ids.add(identity); offsets[offset] = identity
    projected = audio['instruments']
    projected_ids = {item['id'] for item in projected['tones']}
    if projected_ids != ids:
        raise ValueError('Existing tone projection differs from source catalog')
    new_tones, new_pcm, new_cgb = [], [], []
    pcm_offsets = {wave['source_offset'] for wave in audio['waves']}
    cgb_offsets = {wave['source_offset'] for wave in audio['cgb_waves']}
    tone_next = max(ids, default=-1) + 1
    pcm_next = max((wave['id'] for wave in audio['waves']), default=-1) + 1
    cgb_next = max((wave['id'] for wave in audio['cgb_waves']), default=-1) + 1
    for offset, role, chunk in segments:
        if role == 'physical-tone-records':
            if (not BANK_STARTS[0] <= offset < BANK_STARTS[-1] or (offset - BANK_STARTS[0]) % 12
                    or len(chunk) % 12 or offset + len(chunk) > BANK_STARTS[-1]):
                raise ValueError('Invalid physical tone segment')
            for position in range(0, len(chunk), 12):
                source = offset + position
                if source in offsets:
                    raise ValueError('Physical tone overlaps existing catalog')
                ctype, key, length, pan, word04, word08 = struct.unpack_from('<BBBBII', chunk, position)
                if ctype not in (0, 8, 1, 2, 3, 0x40, 0x80):
                    raise ValueError('Unsupported physical tone type')
                identity = tone_next; tone_next += 1; offsets[source] = identity
                new_tones.append({'id': identity, 'ctype': ctype, 'word04': word04, 'word08': word08,
                                  'source_offset': source, 'key': key, 'length': length, 'pan_or_sweep': pan,
                                  'physical_only': True})
        elif role == 'direct-pcm-waves':
            if offset in pcm_offsets or offset % 4 or len(chunk) < 17:
                raise ValueError('Invalid physical PCM source')
            header, flags, frequency, loop, length = struct.unpack_from('<HHIII', chunk)
            if (header != 0 or flags not in (0, 0x4000) or frequency == 0 or not 0 <= loop < length
                    or len(chunk) != 17 + length or length > 1024 * 1024):
                raise ValueError('Invalid physical PCM wave')
            signed = lambda value: value if value < 128 else value - 256
            new_pcm.append({'id': pcm_next, 'source_offset': offset, 'header_type': header, 'flags': flags,
                            'frequency_word': frequency, 'loop_start': loop, 'declared_length': length,
                            'samples': [signed(value) for value in chunk[16:-1]],
                            'interpolation_endpoint': signed(chunk[-1]), 'tone_ids': [], 'physical_only': True})
            pcm_offsets.add(offset); pcm_next += 1
        elif role == 'direct-cgb-waves':
            if not CGB_START <= offset < CGB_END or (offset - CGB_START) % 16 or len(chunk) % 16 or offset + len(chunk) > CGB_END:
                raise ValueError('Invalid physical CGB segment')
            for position in range(0, len(chunk), 16):
                source = offset + position
                if source in cgb_offsets:
                    raise ValueError('Physical CGB source overlaps existing catalog')
                amplitudes = [nibble for value in chunk[position:position + 16] for nibble in (value >> 4, value & 15)]
                new_cgb.append({'id': cgb_next, 'source_offset': source, 'amplitudes': amplitudes,
                                'tone_ids': [], 'physical_only': True})
                cgb_offsets.add(source); cgb_next += 1
        else:
            raise ValueError('Unexpected physical audio source role')
    if len(new_tones) != 70 or len(new_pcm) != 8 or len(new_cgb) != 6 or len(offsets) != 499:
        raise ValueError('Incomplete physical audio records')
    for start, end in zip(BANK_STARTS, BANK_STARTS[1:]):
        if any(offset not in offsets for offset in range(start, end, 12)):
            raise ValueError('Missing physical tone record')
    tones = [dict(item) for item in projected['tones']] + new_tones
    tones_by_id = {item['id']: item for item in tones}
    if len(tones_by_id) != len(tones):
        raise ValueError('Duplicate physical tone ID')
    banks = []
    for bank in projected['banks']:
        start = bank['source_address'] - ROM_BASE
        if start not in BANK_STARTS[:-1] or any(item['source_address'] == bank['source_address'] for item in banks):
            raise ValueError('Invalid physical tone bank')
        end = BANK_STARTS[BANK_STARTS.index(start) + 1]
        programs = [dict(item) for item in bank['programs']]; present = {}
        for item in programs:
            selector, identity = item['selector'], item['tone']
            if selector in present or start + selector * 12 >= end or offsets.get(start + selector * 12) != identity:
                raise ValueError('Existing program differs from physical tone directory')
            present[selector] = identity
        for selector, source in enumerate(range(start, end, 12)):
            if selector not in present:
                programs.append({'selector': selector, 'tone': offsets[source], 'physical_only': True})
        banks.append({**bank, 'programs': programs})
    if len(banks) != 4:
        raise ValueError('Missing physical tone banks')
    waves = [{**wave, 'tone_ids': list(wave.get('tone_ids', []))} for wave in audio['waves']] + new_pcm
    cgb_waves = [{**wave, 'tone_ids': list(wave.get('tone_ids', []))} for wave in audio['cgb_waves']] + new_cgb
    pcm_by_offset = {wave['source_offset']: wave for wave in waves}
    cgb_by_offset = {wave['source_offset']: wave for wave in cgb_waves}
    if len(pcm_by_offset) != len(waves) or len(cgb_by_offset) != len(cgb_waves):
        raise ValueError('Duplicate physical wave source')
    for tone in tones:
        ctype, source = tone['ctype'], tone['word04'] - ROM_BASE
        if ctype not in (0, 8, 3):
            continue
        wave = (cgb_by_offset if ctype == 3 else pcm_by_offset).get(source)
        if wave is None:
            raise ValueError('Missing direct physical tone waveform')
        if tone['id'] not in wave['tone_ids']:
            wave['tone_ids'].append(tone['id'])
    return {**audio, 'waves': waves, 'cgb_waves': cgb_waves,
            'instruments': {**projected, 'banks': banks, 'tones': tones},
            'physical_counts': {'new_tones': len(new_tones), 'new_pcm': len(new_pcm), 'new_cgb': len(new_cgb)}}
