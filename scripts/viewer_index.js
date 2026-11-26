// Keep the overview small: group index entries without loading topic payloads.
const indexRows = document.getElementById('rows'), indexSearch = document.getElementById('search');
const openScopes = new Set(), openFamilies = new Set();
const pageById = new Map(data.wiki.pages.map(page => [page.id, page]));
const familyNames = {audio:'Audio', engine:'Engine', font:'Fonts', gameplay:'Gameplay',
                     graphics:'Graphics', text:'Text', ui:'Interface'};
function compareLabels(a, b) { return a.localeCompare(b, 'en', {sensitivity:'base', numeric:true}); }
function familyLabel(id) {
  const prefix = id.split('.')[0];
  return familyNames[prefix] || (prefix ? prefix[0].toUpperCase() + prefix.slice(1).replace(/[-_]/g, ' ') : 'Other');
}
function renderRows() {
  const query = indexSearch.value.trim().toLocaleLowerCase(), groups = new Map();
  const matches = data.resources.filter(item => [item.id, item.path, item.description, item.format, item.status]
    .some(value => String(value).toLocaleLowerCase().includes(query)));
  for (const item of matches) {
    const scope = item.viewer_scope || 'other', page = pageById.get(scope), label = page ? page.label : 'Other resources';
    if (!groups.has(scope)) groups.set(scope, {label, families:new Map()});
    const families = groups.get(scope).families, family = familyLabel(item.id);
    if (!families.has(family)) families.set(family, []);
    families.get(family).push(item);
  }
  indexRows.replaceChildren();
  const categories = [...groups.entries()].sort((a, b) => compareLabels(a[1].label, b[1].label) || compareLabels(a[0], b[0]));
  for (const [scope, group] of categories) {
    const details = document.createElement('details'), summary = document.createElement('summary');
    details.className = 'resource-category'; details.dataset.scope = scope;
    const total = [...group.families.values()].reduce((sum, items) => sum + items.length, 0);
    summary.textContent = group.label + ' · ' + total; details.append(summary);
    details.open = !!query || openScopes.has(scope);
    details.addEventListener('toggle', () => {
      if (query) return;
      if (details.open) openScopes.add(scope); else openScopes.delete(scope);
    });
    for (const [family, entries] of [...group.families.entries()].sort((a, b) => compareLabels(a[0], b[0]))) {
      const nested = document.createElement('details'), heading = document.createElement('summary'), key = scope + '\0' + family;
      nested.className = 'resource-family'; nested.dataset.family = family;
      heading.textContent = family + ' · ' + entries.length; nested.append(heading);
      nested.open = !!query || openFamilies.has(key);
      nested.addEventListener('toggle', () => {
        if (query) return;
        if (nested.open) openFamilies.add(key); else openFamilies.delete(key);
      });
      const list = document.createElement('ul'); list.className = 'resource-list';
      entries.sort((a, b) => compareLabels(a.description, b.description) || compareLabels(a.id, b.id) || compareLabels(a.path, b.path));
      entries.forEach(item => {
        const entry = document.createElement('li'), title = document.createElement('strong'), file = document.createElement('span');
        entry.dataset.resourceId = item.id; title.textContent = item.description + ' (' + item.id + ')';
        file.className = 'resource-file'; file.textContent = item.path + ' · ' + item.format + ' · ' + item.status;
        const links = document.createElement('div'); links.className = 'resource-links';
        const page = pageById.get(item.viewer_scope);
        if (page) {
          const link = document.createElement('a'); link.href = page.href; link.textContent = 'Browse ' + page.label; links.append(link);
        }
        if (item.embedded_view && data.wiki.resource_directory) {
          const link = document.createElement('a'); link.href = data.wiki.resource_directory + item.embedded_view;
          link.textContent = 'Browse extracted file'; links.append(link);
        }
        if (directoryFiles.has(item.path)) {
          const file = directoryFiles.get(item.path), link = document.createElement('a');
          link.href = '#file-inspector'; link.textContent = 'Inspect selected file';
          link.addEventListener('click', () => { if (openIndexedFile) openIndexedFile(file, item.path); }); links.append(link);
        }
        entry.append(title, file, links); list.append(entry);
      });
      nested.append(list); details.append(nested);
    }
    indexRows.append(details);
  }
  document.getElementById('index-count').textContent = matches.length + ' of ' + data.resources.length + ' resources';
}
indexSearch.addEventListener('input', renderRows);
directoryInput.addEventListener('change', () => {
  if (releaseIndexedFile) releaseIndexedFile();
  directoryFiles = new Map();
  const ambiguous = new Set(), selected = [...directoryInput.files];
  for (const file of selected) {
    const parts = file.webkitRelativePath.split('/');
    if (parts.length < 2 || parts.some(part => !part || part === '.' || part === '..')) continue;
    const path = parts.slice(1).join('/');
    if (ambiguous.has(path)) continue;
    if (directoryFiles.has(path)) { directoryFiles.delete(path); ambiguous.add(path); }
    else directoryFiles.set(path, file);
  }
  const matches = data.resources.filter(item => directoryFiles.has(item.path)).length;
  disconnectDirectory.disabled = selected.length === 0;
  directoryStatus.textContent = selected.length
    ? selected.length + ' files selected · ' + matches + ' of ' + data.resources.length
      + ' indexed resources match by path. Contents have not been rechecked.'
      + (ambiguous.size ? ' ' + ambiguous.size + ' ambiguous paths were omitted.' : '')
    : 'No dataset folder selected.';
  renderRows();
});
disconnectDirectory.addEventListener('click', () => {
  if (releaseIndexedFile) releaseIndexedFile();
  directoryInput.value = ''; directoryFiles = new Map(); disconnectDirectory.disabled = true;
  directoryStatus.textContent = 'No dataset folder selected.'; renderRows();
});
renderRows();
