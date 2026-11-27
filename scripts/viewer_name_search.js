// The compact, source-linked name index is loaded only when this panel opens.
if (data.wiki.name_search) {
  const section = document.getElementById('wiki-name-search'), details = section.querySelector('details');
  const query = document.getElementById('wiki-name-query'), language = document.getElementById('wiki-name-language');
  const kind = document.getElementById('wiki-name-kind'), rows = document.getElementById('wiki-name-rows');
  const status = document.getElementById('wiki-name-status'), previous = document.getElementById('wiki-name-prev');
  const next = document.getElementById('wiki-name-next'), asset = data.wiki.name_search;
  const labels = {item:'Items', action:'Action-linked records', job:'Job metadata', region:'Region references'};
  const pages = new Map(data.wiki.pages.map(page => [page.id, page]));
  let entries = null, loading = false, page = 0;
  section.hidden = false; previous.disabled = next.disabled = true;
  function normalize(value) {
    return value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLocaleLowerCase();
  }
  function destination(entry, selected) {
    const region = entry.kind === 'region', source = pages.get(region ? 'world-map' : 'world');
    if (!source) return null;
    const target = new URL(source.href, location.href);
    const view = {item:'items', action:'action-records', job:'metadata-names', region:'world-regions'}[entry.kind];
    if (!view || !(source.sections || []).some(section => section.id === view)) return null;
    target.searchParams.set('view', view); target.hash = view;
    if (entry.kind === 'item') {
      target.searchParams.append('select', 'item-name-language:' + selected.option);
      target.searchParams.append('select', 'item-id:' + entry.id);
    } else if (entry.kind === 'action') {
      target.searchParams.append('select', 'action-name-language:' + selected.option);
      target.searchParams.append('select', 'action-record-id:' + entry.id);
    } else if (entry.kind === 'job') {
      target.searchParams.set('metadata_id', String(entry.id)); target.searchParams.set('metadata_language', language.value);
    } else {
      target.searchParams.set('region_id', String(entry.id));
      target.searchParams.append('select', 'world-region-language:' + selected.option);
    }
    return target.href;
  }
  function render() {
    if (!entries) return;
    const needle = normalize(query.value.trim()), selectedLanguage = language.value;
    const matches = entries.flatMap(entry => {
      const name = entry.names[selectedLanguage];
      if (!name || (kind.value !== 'all' && kind.value !== entry.kind)) return [];
      if (needle && !normalize(name.text).includes(needle) && !String(entry.id).includes(needle)) return [];
      return [{entry, name}];
    });
    matches.sort((a, b) => labels[a.entry.kind].localeCompare(labels[b.entry.kind], 'en') ||
      a.name.text.localeCompare(b.name.text, undefined, {sensitivity:'base', numeric:true}) || a.entry.id - b.entry.id);
    const size = 25, pagesCount = Math.max(1, Math.ceil(matches.length / size)); page = Math.min(page, pagesCount - 1);
    rows.replaceChildren();
    matches.slice(page * size, (page + 1) * size).forEach(({entry, name}) => {
      const row = rows.insertRow(); row.dataset.kind = entry.kind; row.dataset.recordId = String(entry.id);
      row.insertCell().textContent = labels[entry.kind]; row.insertCell().textContent = String(entry.id);
      const cell = row.insertCell(), href = destination(entry, name);
      if (href) { const link = document.createElement('a'); link.href = href; link.textContent = name.text; cell.append(link); }
      else cell.textContent = name.text;
    });
    previous.disabled = page === 0; next.disabled = page + 1 >= pagesCount;
    status.textContent = matches.length + (matches.length === 1 ? ' result' : ' results') + ' · page ' + (page + 1) + ' of ' + pagesCount;
  }
  function load() {
    if (entries || loading) return;
    loading = true; status.textContent = 'Loading the local name index…';
    const script = document.createElement('script'); script.src = new URL(asset.href, location.href).href;
    script.onload = () => {
      const loaded = globalThis.__FFTA_WIKI_NAMES__; loading = false; script.remove();
      if (!loaded || loaded.key !== asset.key || !Array.isArray(loaded.entries)) {
        status.textContent = 'The name index is invalid. Close and reopen to retry.'; return;
      }
      entries = loaded.entries;
      const ids = [...new Set(entries.flatMap(entry => Object.keys(entry.names)))].sort((a, b) => Number(a) - Number(b));
      ids.forEach(id => language.add(new Option(id, id)));
      render();
    };
    script.onerror = () => { loading = false; script.remove(); status.textContent = 'The name index could not be loaded. Close and reopen to retry.'; };
    document.head.append(script);
  }
  details.addEventListener('toggle', () => { if (details.open) load(); });
  [query, language, kind].forEach(control => control.addEventListener(control === query ? 'input' : 'change', () => { page = 0; render(); }));
  previous.addEventListener('click', () => { if (page) { page--; render(); } });
  next.addEventListener('click', () => { if (!next.disabled) { page++; render(); } });
  function openFromHash() { if (location.hash === '#wiki-name-search') { details.open = true; load(); } }
  window.addEventListener('hashchange', openFromHash); openFromHash();
}
