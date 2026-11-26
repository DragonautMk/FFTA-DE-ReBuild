// Every offline page carries small menu metadata, not other pages' data.
if (data.wiki) {
  const groups = [
    ['Audio', ['audio']],
    ['Characters and animation', ['actors', 'battle', 'objects', 'script-actors']],
    ['Graphics', ['indexed-graphics', 'transfer-graphics']],
    ['Interface', ['interface']],
    ['Text and scripts', ['scripts', 'text', 'text-pointers', 'text-relative', 'text-scripts']],
    ['World and other data', ['other', 'world']]
  ];
  const navigation = document.getElementById('wiki-pages');
  const container = document.getElementById('wiki-page-links'); navigation.hidden = false;
  const pages = new Map(data.wiki.pages.map(page => [page.id, page]));
  const compare = (a, b) => a.localeCompare(b, 'en', {sensitivity:'base', numeric:true});
  function addLink(parent, page, section) {
    const link = document.createElement('a'), target = new URL(page.href, location.href);
    if (section) target.hash = section.id;
    link.href = target.href; link.textContent = section ? section.label : page.label;
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
    summary.textContent = page.label; details.append(summary);
    const overview = document.createElement('li'); addLink(overview, page); children.append(overview);
    sections.forEach(section => { const child = document.createElement('li'); addLink(child, page, section); children.append(child); });
    details.append(children); item.append(details); list.append(item);
  }
  const index = pages.get('index'); if (index) addLink(container, index);
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
    summary.textContent = label; menu.className = 'wiki-menu'; details.append(summary);
    entries.forEach(page => addPage(list, page, entries.length === 1));
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
  // Existing world bookmarks for moved views continue to open the same view.
  function redirectMovedView() {
    if (data.wiki.current !== 'world') return;
    const view = new URLSearchParams(location.search).get('view') || location.hash.slice(1);
    const targetPage = pages.get('interface');
    if (!targetPage || !(targetPage.sections || []).some(section => section.id === view)) return;
    const target = new URL(targetPage.href, location.href); target.search = location.search; target.hash = location.hash;
    location.replace(target.href);
  }
  window.addEventListener('hashchange', redirectMovedView); redirectMovedView();
}
