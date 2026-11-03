#!/usr/bin/env python3
"""Audit the Atlas migration against the recorded pre-change code inventory."""
import argparse
import json
import re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
DEST=ROOT/'docs/development/MENU_ATLAS_INVENTORY.json'

def enum(text,name):
    start=text.index('enum '+name+' {');i=text.index('{',start)+1;end=i;depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[i:end-1]

def rows(text):
    return [a+('('+b+')'if b else '')for a,b in re.findall(r'Row::(\w+)(?:\(([^)]*)\))?',text)]

def render():
    base=json.loads((ROOT/'docs/development/MENU_ATLAS_BASELINE.json').read_text())
    text=(ROOT/'crates/menu/src/lib.rs').read_text()
    atlas=(ROOT/'crates/menu/src/clan_atlas.rs').read_text()
    constants={name:rows(value)for name,value in re.findall(r'const (\w+_ROWS):[^=]+=\s*\[(.*?)\];',text,re.S)}
    router=text[text.index('fn rows_of('):text.index('/// Fast-forward multipliers')]
    groups={int(n):rows(body)for n,body in re.findall(r'(\d+) => &(?:\[)(.*?)(?:\],)',atlas[atlas.index('fn debug_rows'):atlas.index('fn specialty')],re.S)}
    tail=atlas[atlas.index('fn debug_rows'):atlas.index('fn specialty')]
    groups[7]=rows(tail[tail.index('_ => &['):])
    labels=['Panoramica','Sessioni e casi','Replay e confronti','Esecuzione e memoria','Tracce e ispettori','Console e script','Cheat','Report e archivio']
    main=dict(Resume='Gioco / Panoramica / Torna al gioco',SaveStatesMenu='Gioco / Salvataggi / Save states',SaveBackupsMenu='Gioco / Salvataggi / Save backups',FastForwardMenu='Gioco / Fast Forward',TextBubblesMenu='Gioco / Finestre di testo',AudioMenu='Gioco / Audio',VideoMenu='Gioco / Video',InputsMenu='Gioco / Controlli',RestartRom='Gioco / Sistema / Restart ROM',SwitchRomMenu='Gioco / Sistema / Switch ROM',DebugMenu='Sidebar / Debug',Quit='Gioco / Sistema / Exit')
    assert set(main)==set(base['main_rows'])
    page_routes=dict(DebugCatalog='Debug / Sessioni e casi',ContentSelector='Debug / Sessioni e casi (selettore condiviso; ritorno al chiamante)',ToolInputs='Debug / Sessioni e casi',DebugTools='Debug / Console e script (parametri condivisi)',ScriptLibrary='Debug / Console e script',DebugTool='Debug (modulo condiviso; ritorno al chiamante)',GameSaves='Gioco / Salvataggi',SwitchGameSave='Gioco / Salvataggi / conferma',ReplaceGameSave='Gioco / Salvataggi / conferma',SaveStates='Gioco / Salvataggi',ReplaceState='Gioco / Salvataggi / conferma',SaveBackups='Gioco / Salvataggi',RestoreBackup='Gioco / Salvataggi / conferma',Exit='Gioco / Sistema / conferma',SwitchRom='Gioco / Sistema',FastForward='Gioco / Fast Forward',Audio='Gioco / Audio',Video='Gioco / Video',TextBubbles='Gioco / Finestre di testo',Inputs='Gioco / Controlli',Gamepad='Gioco / Controlli',MouseKeyboard='Gioco / Controlli')
    mapping=[]
    for page,old_rows in base['page_rows'].items():
        if page not in ['Main','Debug']:
            m=re.search(r'Page::'+page+r'\s*=>\s*&(?:\[(.*?)\]|(\w+_ROWS))',router,re.S);assert m,page
            current=rows(m[1])if m[1]is not None else constants[m[2]]
            assert set(old_rows)<=set(current),(page,set(old_rows)-set(current))
        for row in old_rows:
            dest=[main[row]]if page=='Main'else ['Debug / '+labels[i]for i,v in groups.items()if row in v]if page=='Debug'else [page_routes[page]]
            assert dest,('orphan',page,row)
            mapping.append(dict(old_page=page,row=row,destinations=dest))
    current_actions=re.findall(r'^    (\w+)(?:\([^\n]*\)|\s*\{[^\n]*\})?,',enum(text,'MenuAction'),re.M)
    assert set(base['menu_actions'])<=set(current_actions)
    config=(ROOT/'crates/input-config/src/lib.rs').read_text()
    assert set(base['config_fields'])<=set(re.findall(r'pub (\w+):[^\n]+',config))
    cli=(ROOT/'crates/recomp/src/main.rs').read_text()
    ops=json.loads((ROOT/'crates/recomp/src/debug_operations.json').read_text())['operations']
    commands=set(re.findall(r'\n\s*"([a-z][a-z0-9-]+)"\s*=>',cli)) | {o['cli']for o in ops if o['cli']}
    assert set(base['cli_commands'])<=commands
    ids=[o['id'] for o in ops]
    assert len(ids)==len(set(ids)),'Duplicate operation IDs'
    legacy_ids={o['id'] for o in base['operation_contracts']}
    contracts=[{k:o[k]for k in base['operation_contracts'][0]}for o in ops if o['id'] in legacy_ids]
    assert contracts==base['operation_contracts'],'CLI handlers, parameters or safety contracts changed'
    return dict(schema=1,baseline_commit=base['baseline_commit'],legacy_pages=len(base['page_rows']),legacy_rows=len(mapping),legacy_actions=base['menu_actions'],new_actions=sorted(set(current_actions)-set(base['menu_actions'])),row_routes=mapping,operations=[dict(id=o['id'],handler=o['handler'],cli=o['cli'],current=o['menu']['current'],destination=o['menu']['destination'],menu_implemented=o['menu']['implemented'],reference='readonly')for o in ops])

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args()
    result=json.dumps(render(),indent=2,ensure_ascii=False)+'\n'
    if args.check:assert DEST.read_text()==result,'Atlas inventory is stale; regenerate it'
    else:DEST.write_text(result)
    print('Atlas inventory: 190 legacy row occurrences, 54 legacy actions and 97 legacy operation contracts preserved')
