"""Half-open physical ROM interval accounting shared by atlas and audit import."""
import collections


def union(ranges):
    result = []
    for start, end in sorted(ranges):
        if start < 0 or end <= start:
            raise ValueError(f"invalid interval {start}..{end}")
        if result and start <= result[-1][1]:
            result[-1][1] = max(end, result[-1][1])
        else:
            result.append([start, end])
    return result


def length(ranges):
    return sum(b - a for a, b in union(ranges))


def intersection(left, right):
    left, right = union(left), union(right)
    i = j = 0
    result = []
    while i < len(left) and j < len(right):
        a, b = max(left[i][0], right[j][0]), min(left[i][1], right[j][1])
        if a < b:
            result.append([a, b])
        if left[i][1] < right[j][1]:
            i += 1
        else:
            j += 1
    return union(result)


def complement(ranges, size):
    cursor, result = 0, []
    for a, b in union(ranges):
        if b > size:
            raise ValueError("range outside ROM")
        if cursor < a:
            result.append([cursor, a])
        cursor = b
    if cursor < size:
        result.append([cursor, size])
    return result


def multiple_owners(owners):
    """Intervals covered by >=2 distinct owners; duplicate roles count once."""
    events = collections.defaultdict(list)
    for owner, spans in owners.items():
        for a, b in union(spans):
            events[a].append((owner, 1))
            events[b].append((owner, -1))
    active, result, previous = set(), [], 0
    for offset in sorted(events):
        if len(active) >= 2 and previous < offset:
            result.append([previous, offset])
        for owner, direction in events[offset]:
            if direction == 1:
                active.add(owner)
            else:
                active.remove(owner)
        previous = offset
    return union(result)


def checked_span(start, end, size):
    if not (type(start) is int and type(end) is int and 0 <= start < end <= size):
        raise ValueError(f"out-of-ROM interval {start}..{end}")
    return [start, end]


