#!/usr/bin/env python3
"""Publish aggregate coverage figures from a locally generated coverage report."""
import argparse
import html
import json
import os
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[1]
METRICS = (
    ('rom', 'ROM size'),
    ('code_candidates', 'Candidate code'),
    ('identified_data', 'Identified data'),
    ('runtime_available', 'Available to runtime'),
    ('ordinary_resource_extents', 'Families with ordinary-use evidence'),
    ('unclassified', 'Unclassified'),
)


def project(report):
    if report.get('schema') not in (2, 3) or report.get('profile') != 'europe':
        raise ValueError('Expected an EUR coverage report with schema 2 or 3')
    totals = report['totals']
    result = {}
    for key, _ in METRICS:
        value = totals[key]
        if type(value) is not int or value < 0:
            raise ValueError(f'Invalid aggregate: {key}')
        result[key] = value
    if not result['rom'] or any(value > result['rom'] for value in result.values()):
        raise ValueError('Aggregate exceeds ROM size')
    return {'schema': 1, 'totals': result}


def publish(source, output):
    source = Path(source).resolve()
    output = Path(output).resolve()
    if source in (output / 'index.html', output / 'coverage.json'):
        raise ValueError('Source report must be outside the public output files')
    report = json.loads(source.read_text(encoding='utf-8'))
    public = project(report)
    output.mkdir(parents=True, exist_ok=True)
    totals = public['totals']
    rows = ''.join(
        '<tr><th scope="row">' + html.escape(label) + '</th><td>' +
        format(totals[key], ',') + '</td><td>' +
        format(totals[key] / totals['rom'] * 100, '.2f') + '%</td></tr>'
        for key, label in METRICS
    )
    page = ('<!doctype html><html lang="en"><head><meta charset="utf-8">'
            '<meta name="viewport" content="width=device-width,initial-scale=1">'
            '<title>ROM coverage overview</title><style>'
            'body{font:16px system-ui,sans-serif;max-width:52rem;margin:3rem auto;'
            'padding:0 1rem;line-height:1.5;color:#1b2a33}table{border-collapse:collapse;'
            'width:100%}th,td{text-align:left;padding:.6rem;border-bottom:1px solid #ddd}'
            '</style></head><body><h1>ROM coverage overview</h1>'
            '<p>Aggregate figures for the European profile.</p><table><thead><tr>'
            '<th scope="col">Category</th><th scope="col">Bytes</th><th scope="col">Share</th>'
            '</tr></thead><tbody>' + rows + '</tbody></table>'
            '<p>Categories may overlap. Code candidates come from static analysis; '
            'ordinary-use evidence describes resource families rather than distinct bytes read.</p>'
            '<p><a href="../ROM_COVERAGE.md">Method and local report</a></p>'
            '</body></html>\n')
    payload = json.dumps(public, separators=(',', ':')) + '\n'
    with tempfile.TemporaryDirectory(dir=output) as staging:
        staging = Path(staging)
        (staging / 'index.html').write_text(page, encoding='utf-8')
        (staging / 'coverage.json').write_text(payload, encoding='utf-8')
        os.replace(staging / 'coverage.json', output / 'coverage.json')
        os.replace(staging / 'index.html', output / 'index.html')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path, help='local coverage.json')
    parser.add_argument('--output', type=Path, default=ROOT / 'docs/development/rom-coverage')
    args = parser.parse_args()
    try:
        publish(args.report, args.output)
    except (OSError, ValueError, TypeError, KeyError) as error:
        parser.exit(1, f'Cannot publish public overview: {error}\n')
    print(f'Public overview: {args.output / "index.html"}')


if __name__ == '__main__':
    main()
