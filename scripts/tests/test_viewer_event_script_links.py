"""Check authenticated, bidirectional event/script references."""
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from test_viewer_event_scripts_browser import catalog
from test_viewer_mission_events_browser import fixture as mission_fixture
from test_viewer_navbar_split_browser import wait_for
from viewer_browser_harness import Chrome


def fixture(work, *, missing_missions=False, split=True):
    mission_fixture(work); dataset=work/'dataset'; manifest=dataset/'manifest.toml'; text=manifest.read_text().replace('profile="test"','profile="europe"')
    digests={}
    for name,offset in (('mission-records70',0x713b3c),('event-records12',0x71c760)):
        path=dataset/(name+'.json');old=path.read_bytes();data=json.loads(old);data['source_offset']=offset
        if name=='event-records12':
            original=data['records'];data['records']=[{**original[1], 'index':i, 'event_key01':1 if 1<=i<=22 else 0 if i==0 else 4 if i==23 else 255} for i in range(25)]
        raw=json.dumps(data).encode();path.write_bytes(raw);digests[name]=hashlib.sha256(raw).hexdigest();text=text.replace(hashlib.sha256(old).hexdigest(),digests[name])
    archive=catalog();archive['archive_offset']=0xb946ec;archive['key_offset']=0xd68c2c;raw=json.dumps(archive).encode();(dataset/'script-archive.json').write_bytes(raw)
    digests['script-archive']=hashlib.sha256(raw).hexdigest()
    text+=('[[resources]]\nid="engine.script-archive"\npath="script-archive.json"\nformat="script-archive-v1-json"\n'
           f'sha256="{digests["script-archive"]}"\n');manifest.write_text(text)
    with patch.multiple(viewer, EUROPE_EVENT_LINK_SHA256=digests['event-records12'],
                        EUROPE_MISSION_LINK_SHA256='0'*64 if missing_missions else digests['mission-records70'],
                        EUROPE_SCRIPT_LINK_SHA256=digests['script-archive']):
        return viewer.generate(dataset,work/('split' if split else 'single'),split=split)


def payload(path):return json.JSONDecoder().raw_decode(path.read_text().split('const data = ',1)[1])[0]


class EventScriptProjectionTest(unittest.TestCase):
    def test_source_guards_and_optional_missions(self):
        with tempfile.TemporaryDirectory(prefix='viewer-event-script-projection-') as directory:
            index=fixture(Path(directory));folder=index.parent/'index.html.parts';home,world,scripts=map(payload,(index,folder/'world.html',folder/'scripts.html'))
            self.assertNotIn('event_script_links',home);self.assertEqual(world['event_script_links'],scripts['event_script_links'])
            value=world['event_script_links'];self.assertEqual(value['key_count'],5);self.assertEqual(value['rows'][1]['mission_count'],25)
            self.assertEqual(len(value['rows']),25);self.assertTrue(value['mission_counts_available'])
            events=world['mission_events']['events'];archive=scripts['script_archive'];catalogs={'events':events}
            args=('europe',viewer.EUROPE_EVENT_LINK_SHA256,None,viewer.EUROPE_SCRIPT_LINK_SHA256,catalogs,archive)
            result=viewer.project_event_script_links(*args);self.assertFalse(result['mission_counts_available']);self.assertIsNone(result['rows'][1]['mission_count'])
            for idx,value in ((0,'usa'),(1,'0'*64),(3,'0'*64)):
                bad=list(args);bad[idx]=value;self.assertIsNone(viewer.project_event_script_links(*bad))
            bad_archive=dict(archive,key_offset=0);self.assertIsNone(viewer.project_event_script_links(*args[:-1],bad_archive))

    def test_missing_mission_authentication_keeps_event_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-event-script-no-missions-') as directory:
            index=fixture(Path(directory),missing_missions=True);data=payload(index.parent/'index.html.parts/scripts.html')['event_script_links']
            self.assertFalse(data['mission_counts_available']);self.assertTrue(all(row['mission_count'] is None for row in data['rows']))


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class EventScriptBrowserTest(unittest.TestCase):
    def test_bidirectional_navigation_pagination_and_bookmarks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-event-script-browser-') as directory:
            work=Path(directory);index=fixture(work);parts=index.parent/'index.html.parts';browser=Chrome(work,parts/'world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const badge=q('#mission-records>details>summary time');if(!badge||badge.dateTime!=='2026-11-27T21:58:37Z')throw Error('world badge');
                  q('#event-record-id').value='24';change(q('#event-record-id'));if(!q('#event-script-navigation').hidden)throw Error('out of range');
                  q('#event-record-id').value='1';change(q('#event-record-id'));const a=q('#event-script-open');
                  return !q('#event-script-navigation').hidden&&a.href.includes('scripts.html?')&&a.href.includes('event-script-key%3A2');})()'''))
                browser.evaluate("document.querySelector('#event-script-open').click()")
                wait_for(browser,"data.wiki.current==='scripts' && document.querySelector('#event-script-key').value==='1'")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),badge=q('#event-scripts>details>summary time');
                  if(!badge||badge.dateTime!=='2026-11-27T20:48:14Z'||q('#event-script-event-references').hidden||
                     q('#event-script-event-rows').rows.length!==20||q('#event-script-event-rows').rows[0].cells[1].textContent!=='25')throw Error('reverse links');
                  q('#event-script-event-next').click();return q('#event-script-event-rows').rows.length===2&&q('#event-script-event-rows a').textContent==='21';})()'''))
                href=browser.evaluate("document.querySelector('#event-script-event-rows a').href");browser.evaluate("document.querySelector('#event-script-event-rows a').click()")
                wait_for(browser,"data.wiki.current==='world' && document.querySelector('#event-record-id').value==='21'")
                self.assertTrue(browser.evaluate("document.querySelector('#event-record-panel').open && !document.querySelector('#event-script-navigation').hidden"))
                before=browser.evaluate('performance.timeOrigin');browser.command('Page.reload',{})
                wait_for(browser, 'performance.timeOrigin !== '+json.dumps(before)+" && document.querySelector('#event-record-id').value==='21' && document.querySelector('#event-record-panel').open")
            finally:browser.close()

    def test_single_page_and_header_target_clears_references(self):
        with tempfile.TemporaryDirectory(prefix='viewer-event-script-single-') as directory:
            work=Path(directory);page=fixture(work,split=False);browser=Chrome(work,page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#event-script-key').value='2';change(q('#event-script-key'));if(q('#event-script-event-references').hidden)throw Error('reference panel');
                  const button=q('#event-script-target button');if(!button)throw Error('header target');button.click();
                  return q('#event-script-event-references').hidden;})()'''))
                browser.evaluate("document.querySelector('#event-record-id').value='1';document.querySelector('#event-record-id').dispatchEvent(new Event('change'));document.querySelector('#event-script-open').click()")
                wait_for(browser,"document.querySelector('#event-script-key').value==='1' && !document.querySelector('#event-script-event-references').hidden")
            finally:browser.close()


if __name__=='__main__':unittest.main()
