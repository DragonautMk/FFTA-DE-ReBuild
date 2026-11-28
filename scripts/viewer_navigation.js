// Every offline page carries small menu metadata, not other pages' data.
if (data.wiki) {
  const groups = [
    ['Audio', ['audio', 'audio-samples', 'audio-noise']],
    ['Characters and animation', ['actors', 'battle', 'battle-alternate', 'objects', 'objects-physical', 'physical-unit-sprites', 'script-actors']],
    ['Graphics', ['indexed-graphics', 'indexed-attributes', 'transfer-graphics']],
    ['Interface', ['interface', 'interface-curves', 'interface-motion-curves', 'interface-effect-motion', 'interface-effects', 'interface-fixed-maps', 'interface-full-screen', 'interface-height', 'interface-indexed-ui', 'interface-layouts', 'interface-maps', 'interface-planes', 'interface-queued-maps', 'interface-rectangles', 'interface-screen-palettes', 'interface-ui']],
    ['Text and scripts', ['scripts', 'text', 'text-short', 'text-inline-ui', 'text-pointers', 'text-relative', 'text-scripts']],
    ['World and other data', ['other', 'checksums', 'selection-groups', 'selection-tables', 'task-cycles', 'world', 'world-encounters', 'world-map', 'world-metadata16']]
  ];
  const navigation = document.getElementById('wiki-pages');
  const container = document.getElementById('wiki-page-links'); navigation.hidden = false;
  const pages = new Map(data.wiki.pages.map(page => [page.id, page]));
  const compare = (a, b) => a.localeCompare(b, 'en', {sensitivity:'base', numeric:true});
  const dateFormat = new Intl.DateTimeFormat('en-GB', {year:'numeric', month:'short', day:'2-digit',
    hour:'2-digit', minute:'2-digit', hourCycle:'h23', timeZoneName:'short'});
  function appendChange(parent, change, summary = false) {
    if (!change || !['added', 'updated'].includes(change.kind) || !Number.isFinite(Date.parse(change.at))) return;
    const badge = document.createElement('time'); badge.className = 'wiki-update-badge'; badge.dateTime = change.at;
    badge.title = change.at; badge.textContent = (summary || change.kind === 'updated' ? 'Updated ' : 'Added ') + dateFormat.format(new Date(change.at));
    parent.append(' ', badge);
  }
  function latestChange(entries) {
    return entries.flatMap(page => (page.sections || []).map(section => section.change)
      .filter(change => change && typeof change.at === 'string' && Number.isFinite(Date.parse(change.at))))
      .sort((a, b) => b.at.localeCompare(a.at))[0];
  }
  function addLink(parent, page, section) {
    const link = document.createElement('a'), target = new URL(page.href, location.href);
    if (section) target.hash = section.id;
    link.href = target.href; link.textContent = section ? section.label : page.label;
    if (section) appendChange(link, section.change);
    if (!section && page.id === data.wiki.current) link.setAttribute('aria-current', 'page');
    parent.append(link);
  }
  function addPage(list, page, directSections) {
    const sections = [...(page.sections || [])].sort((a, b) => compare(a.label, b.label));
    if (directSections || !sections.length) {
      const item = document.createElement('li'); addLink(item, page); list.append(item);
      sections.forEach(section => { const child = document.createElement('li'); addLink(child, page, section); list.append(child); });
      return;
    }
    const item = document.createElement('li'), details = document.createElement('details');
    const summary = document.createElement('summary'), children = document.createElement('ul');
    summary.textContent = page.label; appendChange(summary, latestChange([page]), true); details.append(summary);
    const overview = document.createElement('li'); addLink(overview, page); children.append(overview);
    sections.forEach(section => { const child = document.createElement('li'); addLink(child, page, section); children.append(child); });
    details.append(children); item.append(details); list.append(item);
  }
  const index = pages.get('index'); if (index) addLink(container, index);
  if (index && data.wiki.name_search) {
    const search = document.createElement('a'), target = new URL(index.href, location.href);
    target.hash = 'wiki-name-search'; search.href = target.href; search.textContent = 'Search names'; container.append(search);
  }
  const assigned = new Set(['index', ...groups.flatMap(([, ids]) => ids)]);
  const allGroups = groups.map(([label, ids]) => [label, ids.filter(id => pages.has(id)).map(id => pages.get(id))]);
  const extra = data.wiki.pages.filter(page => !assigned.has(page.id));
  if (extra.length) allGroups.push(['Other pages', extra]);
  allGroups.sort((a, b) => compare(a[0], b[0]));
  allGroups.forEach(([label, entries]) => {
    entries.sort((a, b) => compare(a.label, b.label));
    if (!entries.length) return;
    const details = document.createElement('details'), summary = document.createElement('summary');
    const menu = document.createElement('div'), list = document.createElement('ul');
    details.className = 'wiki-page-category'; details.dataset.category = label;
    summary.textContent = label; appendChange(summary, latestChange(entries), true); menu.className = 'wiki-menu'; details.append(summary);
    if (label === 'Audio') {
      const links = entries.flatMap(page => [{page}, ...(page.sections || []).map(section => ({page, section}))]);
      links.sort((a, b) => compare(a.section ? a.section.label : a.page.label, b.section ? b.section.label : b.page.label));
      links.forEach(({page, section}) => { const item = document.createElement('li'); addLink(item, page, section); list.append(item); });
    } else entries.forEach(page => addPage(list, page, entries.length === 1));
    menu.append(list); details.append(menu); container.append(details);
  });
  navigation.addEventListener('toggle', event => {
    if (!event.target.matches('.wiki-page-category') || !event.target.open) return;
    container.querySelectorAll(':scope > details').forEach(details => { if (details !== event.target) details.open = false; });
  }, true);
  navigation.addEventListener('click', event => {
    if (event.target.closest('a')) container.querySelectorAll(':scope > details').forEach(details => { details.open = false; });
  });
  document.addEventListener('click', event => {
    if (!navigation.contains(event.target)) container.querySelectorAll(':scope > details').forEach(details => { details.open = false; });
  });
  navigation.addEventListener('keydown', event => {
    if (event.key !== 'Escape') return;
    const open = event.target.closest('.wiki-page-category');
    if (open && open.open) { open.open = false; open.querySelector(':scope > summary').focus(); event.preventDefault(); }
  });
  const current = pages.get(data.wiki.current);
  if (current && current.id !== 'index') {
    document.title = current.label + ' · Extracted resource wiki';
    document.querySelector('h1').textContent = document.title;
  }
  // Existing bookmarks for moved views continue to open the same view.
  function redirectMovedView() {
    const params = new URLSearchParams(location.search);
    const view = params.get('view') || location.hash.slice(1);
    const moved = {
      'world-encounters': ['encounters'],
      'world-map': ['terrain-seeds', 'script-paths', 'world', 'world-regions'],
      'interface-planes': ['bg-planes'],
      'interface-ui': ['ui-choices', 'ui-context-templates', 'ui-tiles'],
      'interface-maps': ['transfer-directory', 'obj-tiles', 'firaga-transfers', 'firaga-palette-rows',
        'indexed-tilemap-controls', 'tilemaps', 'signed-map-deltas', 'byte-tilemaps', 'compact-cell-bank']
    };
    let destination = null;
    if (data.wiki.current === 'world') {
      destination = Object.keys(moved).find(key => moved[key].includes(view)) || 'interface';
    } else if (data.wiki.current === 'interface') {
      destination = ['interface-ui', 'interface-maps', 'interface-planes'].find(key => moved[key].includes(view));
    } else if (data.wiki.current === 'text') {
      destination = ({texts:'text-short', 'font-atlas':'text-short', 'name-keyboard':'text-short',
        'pointer-texts':'text-pointers', 'relative-texts':'text-relative',
        'text-reader-registry':'text-relative', 'script-texts':'text-scripts'})[view];
    } else if (data.wiki.current === 'audio' && view === 'cgb-noise') destination = 'audio-noise';
    else if (data.wiki.current === 'battle' && (params.get('sprite_catalog') === 'alternate' ||
        params.getAll('select').includes('equipment-catalog:1'))) destination = 'battle-alternate';
    const targetPage = pages.get(destination);
    if (!targetPage || !(targetPage.sections || []).some(section => section.id === (destination === 'battle-alternate' ? 'equipment-sprites' : view))) return;
    const target = new URL(targetPage.href, location.href); target.search = location.search; target.hash = location.hash;
    if (destination === 'battle-alternate') {
      const selections = target.searchParams.getAll('select').filter(value => !value.startsWith('equipment-catalog:'));
      target.searchParams.delete('select'); selections.forEach(value => target.searchParams.append('select', value));
      target.searchParams.set('sprite_catalog', 'alternate');
    }
    location.replace(target.href);
  }
  window.addEventListener('hashchange', redirectMovedView); redirectMovedView();
}
