{
  const fileInput = document.getElementById('inspect-file');
  const previous = document.getElementById('inspect-prev'), next = document.getElementById('inspect-next');
  const status = document.getElementById('inspect-status'), content = document.getElementById('inspect-content');
  const display = document.getElementById('inspect-mode'), queryMode = document.getElementById('inspect-query-mode');
  const query = document.getElementById('inspect-query'), find = document.getElementById('inspect-find');
  const cancel = document.getElementById('inspect-cancel'), findStatus = document.getElementById('inspect-find-status');
  const tree = document.getElementById('tree'), treeStatus = document.getElementById('tree-status');
  const treePath = document.getElementById('tree-path'), treeRows = document.getElementById('tree-rows');
  const up = document.getElementById('tree-up'), treePrev = document.getElementById('tree-prev'), treeNext = document.getElementById('tree-next');
  const treeIndex = document.getElementById('tree-index'), treeGo = document.getElementById('tree-go');
  const treeTable = document.getElementById('tree-table'), treeTableControl = document.getElementById('tree-table-control');
  const treeHead = document.getElementById('tree-head'), treeNote = document.getElementById('tree-table-note');
  const treeNavigationStatus = document.getElementById('tree-navigation-status');
  const treeIndexLabel = document.getElementById('tree-index-label'), treeFilterControl = document.getElementById('tree-filter-control');
  const treeFilter = document.getElementById('tree-filter'), treeFilterApply = document.getElementById('tree-filter-apply');
  const treeFilterCancel = document.getElementById('tree-filter-cancel'), treeFilterClear = document.getElementById('tree-filter-clear');
  const treeFilterStatus = document.getElementById('tree-filter-status'), treeFilters = new Map();
  const deepQuery = document.getElementById('tree-search-query'), deepScope = document.getElementById('tree-search-scope');
  const deepFind = document.getElementById('tree-search-find'), deepCancel = document.getElementById('tree-search-cancel');
  const deepStatus = document.getElementById('tree-search-status'), deepResults = document.getElementById('tree-search-results');
  const deepPrev = document.getElementById('tree-search-prev'), deepNext = document.getElementById('tree-search-next');
  const deepPosition = document.getElementById('tree-search-position'), deepRows = document.getElementById('tree-search-rows');
  let treeFilterGeneration = 0;
  let deepGeneration = 0, deepMatches = [], deepPage = 0;
  const chunkSize = 16384;
  const treeLimit = 8 * 1024 * 1024, pageSize = 100;
  let file = null, fileLabel = '', fromDirectory = false, offset = 0, request = 0, fileGeneration = 0, root, path = [], page = 0;
  let searchGeneration = 0, lastMatch = -1, lastQuery = '';
  function cancelSearch() { searchGeneration++; cancel.disabled = true; }
  async function findNext() {
    if (!file || !query.value) { findStatus.textContent = 'Select a file and enter text to search.'; return; }
    const selected = file, text = query.value, searchKey = queryMode.value + '\0' + text;
    let pattern;
    if (queryMode.value === 'hex') {
      const digits = text.replace(/\s/g, '');
      if (!digits || digits.length % 2 || digits.length > 2048 || !/^[0-9a-f]+$/i.test(digits)) {
        findStatus.textContent = 'Enter up to 1024 complete hexadecimal bytes.'; return;
      }
      pattern = Uint8Array.from(digits.match(/../g), pair => parseInt(pair, 16));
    } else {
      if (text.length > 1024) { findStatus.textContent = 'Enter up to 1024 text characters.'; return; }
      pattern = new TextEncoder().encode(text);
    }
    if (searchKey !== lastQuery) lastMatch = -1;
    lastQuery = searchKey;
    const generation = ++searchGeneration, prefix = new Uint32Array(pattern.length);
    for (let i = 1, matched = 0; i < pattern.length; i++) {
      while (matched && pattern[i] !== pattern[matched]) matched = prefix[matched - 1];
      if (pattern[i] === pattern[matched]) matched++;
      prefix[i] = matched;
    }
    let position = lastMatch + 1, matched = 0;
    cancel.disabled = false; findStatus.textContent = 'Searching…';
    try {
      while (position < selected.size) {
        const bytes = new Uint8Array(await selected.slice(position, Math.min(selected.size, position + 262144)).arrayBuffer());
        if (generation !== searchGeneration) return;
        if (!bytes.length) throw Error('Unexpected empty file block.');
        for (let i = 0; i < bytes.length; i++) {
          while (matched && bytes[i] !== pattern[matched]) matched = prefix[matched - 1];
          if (bytes[i] === pattern[matched]) matched++;
          if (matched === pattern.length) {
            const found = position + i + 1 - pattern.length;
            offset = Math.max(0, found - 1024);
            if (display.value === 'hex') offset = Math.floor(offset / 16) * 16;
            await showChunk({start: found, end: found + pattern.length}, () => generation === searchGeneration);
            if (generation !== searchGeneration) return;
            lastMatch = found;
            cancel.disabled = true;
            findStatus.textContent = 'Match at byte ' + lastMatch + '. Find next continues after this match.';
            return;
          }
        }
        position += bytes.length;
        findStatus.textContent = 'Searching: ' + position + ' of ' + selected.size + ' bytes.';
      }
      lastMatch = -1; cancel.disabled = true;
      findStatus.textContent = 'No further matches. Find next will restart at the beginning.';
    } catch (error) {
      if (generation === searchGeneration) { cancel.disabled = true; findStatus.textContent = 'Cannot search file: ' + error.message; }
    }
  }
  function currentTreeValue() {
    let value = root;
    for (const key of path) value = value[key];
    return value;
  }
  function deepPath(keys) {
    let result = '$';
    for (const key of keys) {
      const shortened = typeof key === 'string' && key.length > 120 ? key.slice(0, 120) + '…' : key;
      result += '[' + JSON.stringify(shortened) + ']';
      if (result.length > 2048) return result.slice(0, 2048) + '…';
    }
    return result;
  }
  function deepValue(value) {
    if (value !== null && typeof value === 'object') return Array.isArray(value) ? 'Array (' + value.length + ')' : 'Object';
    if (typeof value === 'string' && value.length > 160) return JSON.stringify(value.slice(0, 160)) + '…';
    if (typeof value === 'number' && !Number.isFinite(value)) return String(value) + ' (outside browser range)';
    return JSON.stringify(value) + (typeof value === 'number' && Number.isInteger(value) && !Number.isSafeInteger(value)
      ? ' (precision may be lost)' : '');
  }
  function cancelDeepSearch() { deepGeneration++; deepCancel.disabled = true; }
  function drawDeepResults() {
    const start = deepPage * 25, end = Math.min(deepMatches.length, start + 25);
    deepResults.hidden = !deepMatches.length; deepPrev.disabled = deepPage === 0; deepNext.disabled = end >= deepMatches.length;
    deepPosition.textContent = (start + 1) + '–' + end + ' of ' + deepMatches.length + ' matches';
    deepRows.replaceChildren();
    for (let index = start; index < end; index++) {
      const match = deepMatches[index], row = deepRows.insertRow();
      row.insertCell().textContent = deepPath(match.path); row.insertCell().textContent = match.kind;
      row.insertCell().textContent = match.value;
      const button = document.createElement('button'); button.type = 'button'; button.textContent = 'Open';
      button.setAttribute('aria-label', 'Open ' + deepPath(match.path));
      button.addEventListener('click', () => {
        cancelTreeFilter(); path = match.path.slice(); page = 0; drawTree();
        treeNavigationStatus.textContent = 'Opened search result ' + (index + 1) + '.';
      });
      row.insertCell().append(button);
    }
  }
  async function searchTree() {
    if (root === undefined) return;
    const needle = deepQuery.value.trim().toLowerCase();
    if (!needle) { deepStatus.textContent = 'Enter a field name or value.'; return; }
    cancelDeepSearch();
    const generation = deepGeneration, owner = root, startPath = deepScope.value === 'current' ? path.slice() : [];
    let start = owner;
    for (const key of startPath) start = start[key];
    const alive = () => generation === deepGeneration && owner === root;
    const matches = [], traversal = [], current = startPath.slice(), limit = 1000, depthLimit = 256;
    let visited = 0, skipped = 0, limited = false;
    function add(keys, value, keyMatch, valueMatch) {
      matches.push({path: keys.slice(), kind: keyMatch && valueMatch ? 'Field and value' : keyMatch ? 'Field' : 'Value', value: deepValue(value)});
      if (matches.length >= limit) limited = true;
    }
    function frame(value) { return {value, index: 0, keys: Array.isArray(value) ? null : Object.keys(value)}; }
    function scalar(value) { return value === null || typeof value !== 'object'; }
    async function pause() {
      deepStatus.textContent = 'Searching: ' + visited + ' entries inspected.';
      await new Promise(resolve => setTimeout(resolve, 0)); return alive();
    }
    deepCancel.disabled = false;
    if (!(await pause())) return;
    if (scalar(start)) {
      if (String(start).toLowerCase().includes(needle)) add(current, start, false, true);
    } else traversal.push(frame(start));
    while (traversal.length && !limited) {
      const top = traversal[traversal.length - 1], length = top.keys ? top.keys.length : top.value.length;
      if (top.index >= length) { traversal.pop(); if (traversal.length) current.pop(); continue; }
      const key = top.keys ? top.keys[top.index++] : top.index++, child = top.value[key];
      current.push(key); visited++;
      const keyMatch = !!top.keys && key.toLowerCase().includes(needle);
      const valueMatch = scalar(child) && String(child).toLowerCase().includes(needle);
      if (keyMatch || valueMatch) add(current, child, keyMatch, valueMatch);
      if (!scalar(child) && !limited) {
        if (traversal.length >= depthLimit) { skipped++; current.pop(); }
        else traversal.push(frame(child));
      } else current.pop();
      if (visited % 1024 === 0 && !(await pause())) return;
    }
    if (!alive()) return;
    deepCancel.disabled = true; deepMatches = matches; deepPage = 0; drawDeepResults();
    deepStatus.textContent = matches.length + ' matches in ' + visited + ' inspected entries.'
      + (limited ? ' Reached the 1,000-match limit; refine the search.' : '')
      + (skipped ? ' Skipped ' + skipped + ' nested structures beyond 256 levels; search inside them separately.' : '');
  }
  deepFind.addEventListener('click', searchTree);
  deepCancel.addEventListener('click', () => { cancelDeepSearch(); deepStatus.textContent = 'Search cancelled. Previous results remain.'; });
  function deepSearchChanged() {
    if (!deepCancel.disabled) { cancelDeepSearch(); deepStatus.textContent = 'Search cancelled. Previous results remain.'; }
    else if (deepMatches.length) deepStatus.textContent = 'Previous results remain. Find matches to search the new query or scope.';
  }
  deepQuery.addEventListener('input', deepSearchChanged);
  deepScope.addEventListener('change', deepSearchChanged);
  deepQuery.addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); searchTree(); } });
  deepPrev.addEventListener('click', () => { if (deepPage > 0) { deepPage--; drawDeepResults(); } });
  deepNext.addEventListener('click', () => { if ((deepPage + 1) * 25 < deepMatches.length) { deepPage++; drawDeepResults(); } });
  function cancelTreeFilter() {
    treeFilterGeneration++; treeFilterCancel.disabled = true;
  }
  function clearTreeFilter() {
    cancelTreeFilter();
    if (root === undefined || !Array.isArray(currentTreeValue())) return;
    treeFilters.delete(JSON.stringify(path)); page = 0; drawTree();
  }
  async function filterTreeArray() {
    if (root === undefined) return;
    const value = currentTreeValue(); if (!Array.isArray(value)) return;
    const query = treeFilter.value.trim();
    if (!query) { clearTreeFilter(); return; }
    cancelTreeFilter();
    const generation = treeFilterGeneration, owner = root, key = JSON.stringify(path), needle = query.toLowerCase();
    const alive = () => generation === treeFilterGeneration && root === owner && JSON.stringify(path) === key;
    const matches = []; let truncated = false, operations = 0, scanned = 0;
    treeFilterCancel.disabled = false; treeFilterStatus.textContent = 'Filtering…';
    async function pause() {
      treeFilterStatus.textContent = 'Filtering: ' + scanned + ' of ' + value.length + ' entries.';
      await new Promise(resolve => setTimeout(resolve, 0)); return alive();
    }
    if (!(await pause())) return;
    for (let index = 0; index < value.length; index++) {
      const record = value[index]; let match = false;
      if (record !== null && typeof record === 'object') {
        for (const field in record) {
          if (!Object.prototype.hasOwnProperty.call(record, field)) continue;
          const child = record[field];
          if (field.toLowerCase().includes(needle) ||
              ((child === null || typeof child !== 'object') && String(child).toLowerCase().includes(needle))) { match = true; break; }
          if (++operations % 1024 === 0 && !(await pause())) return;
        }
      } else match = String(record).toLowerCase().includes(needle);
      if (match) {
        matches.push(index);
        if (matches.length === 10000 && index + 1 < value.length) { truncated = true; scanned = index + 1; break; }
      }
      scanned = index + 1;
      if (++operations % 1024 === 0 && !(await pause())) return;
    }
    if (!alive()) return;
    treeFilterCancel.disabled = true;
    treeFilters.delete(key); treeFilters.set(key, {query, indices: matches, truncated, scanned});
    if (treeFilters.size > 20) treeFilters.delete(treeFilters.keys().next().value);
    page = 0; drawTree();
  }
  treeFilterApply.addEventListener('click', filterTreeArray);
  treeFilterCancel.addEventListener('click', () => {
    cancelTreeFilter(); treeFilterStatus.textContent = 'Filtering cancelled. Previous results remain.';
  });
  treeFilterClear.addEventListener('click', clearTreeFilter);
  treeFilter.addEventListener('input', () => {
    if (!treeFilterCancel.disabled) { cancelTreeFilter(); treeFilterStatus.textContent = 'Filtering cancelled. Previous results remain.'; }
  });
  treeFilter.addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); filterTreeArray(); } });
  function drawTree() {
    const value = currentTreeValue(), array = Array.isArray(value);
    const container = value !== null && typeof value === 'object';
    const keys = container && !array ? Object.keys(value) : null;
    const activeFilter = array ? treeFilters.get(JSON.stringify(path)) : null;
    const count = !container ? 1 : array ? activeFilter ? activeFilter.indices.length : value.length : keys.length;
    const sourceIndex = index => activeFilter ? activeFilter.indices[index] : index;
    const table = array && treeTable.checked, size = table ? 25 : pageSize;
    const start = page * size, end = Math.min(count, start + size);
    treePath.textContent = '$' + path.map(key => '[' + JSON.stringify(key) + ']').join('') + ' · ' + count
      + (activeFilter ? ' matches of ' + value.length + ' entries' : ' entries');
    up.disabled = path.length === 0; treePrev.disabled = page === 0; treeNext.disabled = end >= count;
    treeTableControl.hidden = treeFilterControl.hidden = !array; treeIndex.max = String(Math.max(0, count - 1));
    treeIndexLabel.textContent = activeFilter ? 'Result position (zero based)' : 'Entry position (zero based)';
    if (array) {
      treeFilter.value = activeFilter ? activeFilter.query : '';
      treeFilterStatus.textContent = activeFilter ? activeFilter.indices.length + ' matches'
        + (activeFilter.truncated ? ' · stopped after ' + activeFilter.scanned + ' entries; refine the filter.' : '.') : 'No filter applied.';
    }
    treeIndex.disabled = treeGo.disabled = count === 0; treeIndex.value = String(start);
    treeNavigationStatus.textContent = ''; treeNote.textContent = '';
    treeRows.replaceChildren(); treeHead.replaceChildren();
    const heading = treeHead.insertRow();
    function addHeading(label) { const cell = document.createElement('th'); cell.textContent = label; heading.append(cell); }
    function openChild(steps) { cancelTreeFilter(); path.push(...steps); page = 0; drawTree(); }
    function addValue(cell, child, steps) {
      if (child !== null && typeof child === 'object') {
        const button = document.createElement('button'); button.type = 'button';
        button.textContent = Array.isArray(child) ? 'Array (' + child.length + ')' : 'Object';
        button.addEventListener('click', () => openChild(steps)); cell.append(button);
      } else {
        const truncated = typeof child === 'string' && child.length > 1024;
        const outsideRange = typeof child === 'number' && !Number.isFinite(child);
        let display = outsideRange ? String(child) : JSON.stringify(truncated ? child.slice(0, 1024) : child);
        if (display.length > 1024) display = display.slice(0, 1024) + '… (see raw file)';
        else if (truncated) display += '… (see raw file)';
        if (outsideRange) display += ' (outside browser range; see raw file)';
        if (typeof child === 'number' && Number.isInteger(child) && !Number.isSafeInteger(child))
          display += ' (precision may be lost; see raw file)';
        cell.textContent = display;
      }
    }
    if (!table) {
      addHeading('Key or index'); addHeading('Value');
      for (let i = start; i < end; i++) {
        const key = !container ? null : keys ? keys[i] : sourceIndex(i), child = container ? value[key] : value;
        const row = treeRows.insertRow(); row.insertCell().textContent = key === null ? '(value)' : String(key);
        addValue(row.insertCell(), child, [key]);
      }
      return;
    }
    const fields = [], fieldSet = new Set(); let omitted = false, otherValues = false;
    for (let i = start; i < end; i++) {
      const record = value[sourceIndex(i)];
      if (!record || typeof record !== 'object' || Array.isArray(record)) { otherValues = true; continue; }
      if (omitted) continue;
      for (const field of Object.keys(record)) {
        if (fieldSet.has(field)) continue;
        if (fields.length === 32) { omitted = true; break; }
        fields.push(field); fieldSet.add(field);
      }
    }
    addHeading('Index'); addHeading('Record');
    if (otherValues) addHeading('Value');
    fields.forEach(addHeading);
    treeNote.textContent = 'Up to 25 entries and 32 fields per page. Open a record to inspect every field.'
      + (omitted ? ' Some fields are omitted on this page.' : '');
    for (let i = start; i < end; i++) {
      const index = sourceIndex(i), record = value[index], plain = record !== null && typeof record === 'object' && !Array.isArray(record);
      const row = treeRows.insertRow(); row.insertCell().textContent = String(index);
      const open = row.insertCell();
      if (plain) {
        const button = document.createElement('button'); button.type = 'button'; button.textContent = 'Open';
        button.setAttribute('aria-label', 'Open record ' + index); button.addEventListener('click', () => openChild([index])); open.append(button);
      } else open.textContent = '—';
      if (otherValues) {
        const cell = row.insertCell();
        if (plain) cell.textContent = '—'; else addValue(cell, record, [index]);
      }
      fields.forEach(field => {
        const cell = row.insertCell();
        if (plain && Object.prototype.hasOwnProperty.call(record, field)) addValue(cell, record[field], [index, field]);
        else cell.textContent = '—';
      });
    }
  }
  function goTreeEntry() {
    if (root === undefined) return;
    cancelTreeFilter();
    const value = currentTreeValue(), activeFilter = Array.isArray(value) ? treeFilters.get(JSON.stringify(path)) : null;
    const count = value !== null && typeof value === 'object'
      ? Array.isArray(value) ? activeFilter ? activeFilter.indices.length : value.length : Object.keys(value).length : 1;
    const index = treeIndex.value === '' ? NaN : Number(treeIndex.value);
    if (!Number.isSafeInteger(index) || index < 0 || index >= count) {
      treeNavigationStatus.textContent = count ? 'Choose ' + (activeFilter ? 'a result' : 'an entry')
        + ' position from 0 to ' + (count - 1) + '.' : 'This value has no entries.';
      return;
    }
    page = Math.floor(index / (Array.isArray(value) && treeTable.checked ? 25 : pageSize));
    drawTree(); treeIndex.value = String(index); treeNavigationStatus.textContent = activeFilter
      ? 'Showing result ' + index + ' (source index ' + activeFilter.indices[index] + ').' : 'Showing entry ' + index + '.';
  }
  treeGo.addEventListener('click', goTreeEntry);
  treeIndex.addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); goTreeEntry(); } });
  treeTable.addEventListener('change', () => { cancelTreeFilter(); if (root !== undefined) { page = 0; drawTree(); } });
  async function loadTree(selected, generation) {
    tree.hidden = true;
    if (!selected.name.toLowerCase().endsWith('.json')) { treeStatus.textContent = 'Structured browsing is available for JSON; use raw paging for this file.'; return; }
    if (selected.size > treeLimit) { treeStatus.textContent = 'File exceeds the 8 MiB structured browsing limit; use raw paging.'; return; }
    treeStatus.textContent = 'Reading JSON structure…';
    try {
      const parsed = JSON.parse(await selected.text());
      if (generation !== fileGeneration) return;
      root = parsed; path = []; page = 0; drawTree(); tree.hidden = false;
      treeStatus.textContent = 'JSON structure · select an object or array to explore.';
    } catch (error) {
      if (generation === fileGeneration) treeStatus.textContent = 'Cannot parse JSON: ' + error.message + '. Raw paging is still available.';
    }
  }
  async function showChunk(highlight = null, valid = () => true) {
    const generation = ++request;
    if (!file) return;
    const selected = file, label = fileLabel, start = offset, end = Math.min(selected.size, start + chunkSize), mode = display.value;
    let readStart = mode === 'hex' ? start : Math.max(0, start - 3);
    let bytes = new Uint8Array(await selected.slice(readStart, end).arrayBuffer());
    if (mode === 'text') while (bytes.length && (bytes[0] & 0xc0) === 0x80) { bytes = bytes.subarray(1); readStart++; }
    if (generation !== request || !valid()) return;
    const begin = highlight ? highlight.start - readStart : -1, finish = highlight ? highlight.end - readStart : -1;
    if (mode === 'hex') {
      content.replaceChildren();
      let plain = '', firstMark = null;
      for (let i = 0; i < bytes.length; i += 16) {
        const row = bytes.subarray(i, i + 16), address = (readStart + i).toString(16).padStart(8, '0').toUpperCase();
        const hex = Array.from(row, value => value.toString(16).padStart(2, '0').toUpperCase()).join(' ').padEnd(47, ' ');
        const ascii = Array.from(row, value => value >= 32 && value <= 126 ? String.fromCharCode(value) : '.').join('');
        const line = address + '  ' + hex + '  |' + ascii + '|\n';
        if (highlight && highlight.start < readStart + i + row.length && highlight.end > readStart + i) {
          if (plain) { content.append(document.createTextNode(plain)); plain = ''; }
          const mark = document.createElement('mark'); mark.textContent = line; content.append(mark);
          if (!firstMark) firstMark = mark;
        } else plain += line;
      }
      if (plain) content.append(document.createTextNode(plain));
      if (firstMark) firstMark.scrollIntoView({block: 'nearest'});
    } else if (begin >= 0 && finish <= bytes.length) {
      const mark = document.createElement('mark');
      mark.textContent = new TextDecoder('utf-8').decode(bytes.subarray(begin, finish));
      content.replaceChildren(document.createTextNode(new TextDecoder('utf-8').decode(bytes.subarray(0, begin))), mark,
        document.createTextNode(new TextDecoder('utf-8').decode(bytes.subarray(finish), {stream: end < selected.size})));
      mark.scrollIntoView({block: 'nearest'});
    } else {
      content.textContent = new TextDecoder('utf-8').decode(bytes, {stream: end < selected.size});
    }
    status.textContent = label + ' · bytes ' + start + '–' + end + ' of ' + selected.size;
    previous.disabled = start === 0;
    next.disabled = end >= selected.size;
  }
  function selectFile(selected, directory = false, label = '') {
    cancelTreeFilter(); treeFilters.clear(); treeFilter.value = '';
    cancelDeepSearch(); deepMatches = []; deepPage = 0; deepQuery.value = ''; deepResults.hidden = true; deepRows.replaceChildren();
    deepStatus.textContent = 'Enter a field name or value.';
    file = selected || null; fileLabel = label || (file ? file.name : ''); fromDirectory = directory && !!file;
    offset = 0; request++; fileGeneration++; root = undefined;
    cancelSearch(); lastMatch = -1; lastQuery = ''; findStatus.textContent = 'Search starts at the beginning of the selected file.';
    display.value = file && !/\.(json|toml|txt)$/i.test(file.name) ? 'hex' : 'text';
    if (file) { showChunk().catch(error => { status.textContent = 'Cannot read file: ' + error.message; }); loadTree(file, fileGeneration); }
    else { content.textContent = ''; status.textContent = 'No file selected.'; previous.disabled = next.disabled = true; tree.hidden = true; treeStatus.textContent = 'Select a JSON file to explore its structure.'; }
  }
  fileInput.addEventListener('change', () => selectFile(fileInput.files[0] || null));
  openIndexedFile = (selected, label) => {
    fileInput.value = ''; selectFile(selected, true, label);
    document.querySelector('#file-inspector > details').open = true;
  };
  releaseIndexedFile = () => { if (fromDirectory) selectFile(null); };
  directoryInput.disabled = false;
  if (directoryInput.files.length) directoryInput.dispatchEvent(new Event('change'));
  else directoryStatus.textContent = 'No dataset folder selected.';
  function cancelPendingSearch() { if (!cancel.disabled) { cancelSearch(); findStatus.textContent = 'Search cancelled.'; } }
  previous.addEventListener('click', () => { cancelPendingSearch(); offset = Math.max(0, offset - chunkSize); showChunk().catch(error => { status.textContent = 'Cannot read file: ' + error.message; }); });
  next.addEventListener('click', () => { cancelPendingSearch(); offset += chunkSize; showChunk().catch(error => { status.textContent = 'Cannot read file: ' + error.message; }); });
  up.addEventListener('click', () => { cancelTreeFilter(); path.pop(); page = 0; drawTree(); });
  treePrev.addEventListener('click', () => { cancelTreeFilter(); page--; drawTree(); });
  treeNext.addEventListener('click', () => { cancelTreeFilter(); page++; drawTree(); });
  find.addEventListener('click', findNext);
  query.addEventListener('input', () => { cancelSearch(); lastMatch = -1; lastQuery = ''; findStatus.textContent = 'Search starts at the beginning of the selected file.'; });
  display.addEventListener('change', () => {
    cancelPendingSearch();
    if (display.value === 'hex') offset = Math.floor(offset / 16) * 16;
    if (file) showChunk().catch(error => { status.textContent = 'Cannot read file: ' + error.message; });
  });
  queryMode.addEventListener('change', () => {
    cancelSearch(); lastMatch = -1; lastQuery = '';
    query.placeholder = queryMode.value === 'hex' ? 'Example: 00 A1 FF' : 'Case-sensitive text';
    findStatus.textContent = 'Search starts at the beginning of the selected file.';
    if (queryMode.value === 'hex' && display.value !== 'hex') {
      display.value = 'hex'; offset = Math.floor(offset / 16) * 16;
      if (file) showChunk().catch(error => { status.textContent = 'Cannot read file: ' + error.message; });
    }
  });
  cancel.addEventListener('click', () => { cancelSearch(); findStatus.textContent = 'Search cancelled.'; });
}
