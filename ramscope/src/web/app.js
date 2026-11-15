/* RamScope — interface (vanilla JS, aucune dépendance externe) */
'use strict';

const TOKEN = document.querySelector('meta[name="ramscope-token"]')?.content || '';
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));

let S = null;
let tab = 'overview';
let q = '';
let levels = new Set();
let onlyAction = false;
let auto = false;
let sel = new Map();
let expanded = new Set();
let busy = false;
const sort = {
  processes: { k: 'private', d: -1 },
  services: { k: 'level', d: 1 },
  startup: { k: 'level', d: 1 },
  tasks: { k: 'level', d: 1 },
};

const LEVELS = [
  ['junk', 'Superflu', 'Inutile dans la quasi-totalité des cas.'],
  ['trim', 'À alléger', 'Démarrage automatique ou télémétrie superflue : le gain intervient au prochain démarrage.'],
  ['opt', 'Selon usage', 'Utile seulement dans certains cas. À toi de trancher.'],
  ['unk', 'Non répertorié', 'RamScope ne connaît pas cet élément : il ne conseille rien.'],
  ['keep', 'À garder', 'Utile (pilote, sécurité, ton usage).'],
  ['sys', 'Système', 'Composant Windows normal : laisse-le.'],
  ['crit', 'Vital', 'Cœur de Windows : RamScope refuse toute modification.'],
];
const LVL_CLS = { critical: 'crit', system: 'sys', keep: 'keep', optional: 'opt', trim: 'trim', junk: 'junk', unknown: 'unk' };

/* ------------------------------------------------------------------ utils */
function esc(s) {
  return String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
}
function mb(b) { return b / 1048576; }
function fmt(b) {
  if (!b) return '0 Mo';
  const m = mb(b);
  if (m >= 1024) return (m / 1024).toFixed(2).replace('.', ',') + ' Go';
  if (m >= 10) return Math.round(m) + ' Mo';
  return m.toFixed(1).replace('.', ',') + ' Mo';
}
function dur(s) {
  const d = Math.floor(s / 86400), h = Math.floor((s % 86400) / 3600), m = Math.floor((s % 3600) / 60);
  return (d ? d + ' j ' : '') + h + ' h ' + m + ' min';
}
function badge(a) {
  const lock = a.locked ? ' <span title="Verrouillé par RamScope">🔒</span>' : '';
  return `<span class="b ${LVL_CLS[a.level] || 'unk'}">${esc(a.label)}${lock}</span>`;
}
function toast(msg, kind = '') {
  const el = document.createElement('div');
  el.className = 'toast ' + kind;
  el.innerHTML = msg;
  $('#toasts').appendChild(el);
  setTimeout(() => { el.style.opacity = '0'; el.style.transition = 'opacity .4s'; setTimeout(() => el.remove(), 400); }, kind === 'ko' ? 9000 : 5000);
}
async function api(path, body) {
  const opt = { method: body ? 'POST' : 'GET', headers: { 'X-RamScope-Token': TOKEN } };
  if (body) { opt.headers['Content-Type'] = 'application/json'; opt.body = JSON.stringify(body); }
  const r = await fetch(path, opt);
  const txt = await r.text();
  let j = null;
  try { j = JSON.parse(txt); } catch (e) { /* ignore */ }
  if (!r.ok) throw new Error((j && j.error) || txt || ('HTTP ' + r.status));
  return j;
}

/* ------------------------------------------------------- sélection / gains */
function keyOf(kind, item) {
  if (kind === 'services') return 's:' + item.name;
  if (kind === 'startup') return 'u:' + item.id;
  if (kind === 'tasks') return 't:' + item.path;
  return 'k:' + item.pid;
}
function selectable(item) {
  return item.advice && item.advice.action && !item.advice.locked;
}
function batchable(item) {
  return selectable(item) && item.advice.action.kind !== 'kill';
}
function entryFor(kind, item) {
  const a = item.advice.action;
  let label, sub;
  if (kind === 'services') { label = item.display; sub = `${item.start === 'auto' ? 'Automatique' : startLabel(item.start)} → ${startLabel(a.start)}`; }
  else if (kind === 'startup') { label = item.name; sub = 'Ne se lancera plus à l’ouverture de session'; }
  else if (kind === 'tasks') { label = item.name; sub = 'Tâche planifiée désactivée'; }
  else { label = item.name + ' (PID ' + item.pid + ')'; sub = 'Processus terminé — perte des données non enregistrées'; }
  return {
    key: keyOf(kind, item), kind, action: a, label, sub,
    gain: item.advice.gain || emptyGain(), boot: item.advice.gain_next_boot || emptyGain(),
    level: item.advice.level, running: !!item.running, name: item.name,
    host: kind === 'services' && item.host_shared > 1 && item.pid ? item.pid : 0,
    hostShared: item.host_shared || 1, hostRam: item.host_ram || 0,
  };
}
function startLabel(s) {
  return { auto: 'Automatique', delayed: 'Auto (différé)', manual: 'Manuel', disabled: 'Désactivé' }[s] || s;
}
function emptyGain() { return { bytes: 0, nature: 'unavailable', scope: 'private', additive: false }; }
function gainBytes(g) { return g && g.bytes ? g.bytes : 0; }
function gainNature(g) { return g && g.nature === 'measured' ? 'mesuré' : g && g.nature === 'estimated' ? 'estimé' : 'non chiffrable'; }
function gainKnown(g) { return !!g && g.nature !== 'unavailable'; }
function gainNatureBadge(g) {
  const n = gainNature(g);
  const cls = n === 'mesuré' ? 'measured' : n === 'estimé' ? 'estimated' : 'unavailable';
  return `<span class="nature ${cls}">${n}</span>`;
}
function gainText(g) { return gainKnown(g) ? `${fmt(gainBytes(g))} ${gainNatureBadge(g)}` : 'non chiffrable'; }
function gainLine(g) {
  if (!gainKnown(g)) return 'non chiffrable';
  const part = g.scope === 'host_part' ? ' <span class="sub">part d’hôte</span>' : '';
  return `${fmt(gainBytes(g))} ${gainNatureBadge(g)}${part}`;
}
function addGain(total, g) {
  if (!gainKnown(g) || (g.nature === 'estimated' && !gainBytes(g))) return;
  if (total.nature === 'unavailable') {
    total.bytes = gainBytes(g); total.nature = g.nature; total.scope = g.scope;
    return;
  }
  total.bytes += gainBytes(g);
  total.nature = total.nature === 'measured' && g.nature === 'measured' ? 'measured' : 'estimated';
}
function provenanceText(a) {
  const source = a && a.source ? String(a.source).trim() : '';
  const reviewed = a && a.reviewed ? String(a.reviewed).trim() : '';
  if (!source && !reviewed) return '';
  return source + (reviewed ? (source ? ' · revue : ' : 'revue : ') + reviewed : '');
}
function aggregateGains(entries) {
  const now = emptyGain(), boot = emptyGain();
  const hosts = new Map();
  entries.forEach(e => {
    if (e.kind === 'services' && e.host && e.hostShared > 1) {
      let h = hosts.get(e.host);
      if (!h) { h = { names: new Set(), expected: e.hostShared, ram: e.hostRam }; hosts.set(e.host, h); }
      h.names.add(e.name);
      h.expected = Math.max(h.expected, e.hostShared);
      h.ram = Math.max(h.ram, e.hostRam);
    } else {
      addGain(now, e.gain);
      addGain(boot, e.boot);
    }
  });
  hosts.forEach(h => {
    if (h.expected > 1 && h.names.size >= h.expected && h.ram > 0) {
      addGain(boot, { bytes: h.ram, nature: 'estimated', scope: 'host', additive: false });
    }
  });
  return { now, boot };
}
function toggle(kind, item) {
  if (!batchable(item)) return;
  const e = entryFor(kind, item);
  if (sel.has(e.key)) sel.delete(e.key); else sel.set(e.key, e);
  paint();
}
function selTotals() {
  const gains = aggregateGains(Array.from(sel.values()));
  let kills = 0;
  sel.forEach(e => { if (e.kind === 'processes') kills++; });
  return { now: gains.now, boot: gains.boot, kills };
}
function allItems(kind) {
  if (!S) return [];
  return kind === 'processes' ? S.processes : kind === 'services' ? S.services : kind === 'startup' ? S.startup : S.tasks;
}
function selectReversible() {
  ['services', 'startup', 'tasks'].forEach(kind => {
    allItems(kind).forEach(it => {
      if (batchable(it) && (it.advice.level === 'junk' || it.advice.level === 'trim')) {
        const e = entryFor(kind, it); sel.set(e.key, e);
      }
    });
  });
  paint();
  toast('Sélection : éléments réversibles à gain différé, classés « superflu » ou « à alléger ». Relis la liste avant d’appliquer.', 'ok');
}

/* --------------------------------------------------------------- chargement */
async function load(force) {
  if (busy) return;
  busy = true;
  $('#btnRefresh').disabled = true;
  try {
    S = await api('/api/snapshot' + (force ? '?force=1' : ''));
    // purge des sélections disparues
    const live = new Set();
    ['processes', 'services', 'startup', 'tasks'].forEach(k => allItems(k).forEach(it => live.add(keyOf(k, it))));
    Array.from(sel.keys()).forEach(k => { if (!live.has(k)) sel.delete(k); });
    paint();
  } catch (e) {
    $('#view').innerHTML = `<div class="note">Impossible de lire l’état de la machine : ${esc(e.message)}</div>`;
  } finally {
    busy = false;
    $('#btnRefresh').disabled = false;
  }
}

/* ------------------------------------------------------------------ rendu */
function paint() {
  if (!S) return;
  renderHead();
  renderHero();
  renderTabs();
  renderTools();
  renderView();
  renderActionBar();
}

function renderHead() {
  const m = S.meta;
  $('#badges').innerHTML =
    (m.demo ? '<span class="pill demo">DÉMO</span> ' : '') +
    (m.is_admin ? '<span class="pill admin">Administrateur</span>' : '<span class="pill readonly">Lecture seule</span>');
  $('#machine').innerHTML = `<b>${esc(m.computer)}</b> · ${esc(m.os)}<br>${esc(m.user)} · allumé depuis ${dur(m.uptime_secs)} · analyse ${m.duration_ms} ms`;
  let n = (m.notes || []).map(x => `<div class="note">${esc(x)}</div>`).join('');
  if (!m.is_admin) {
    n += m.can_elevate
      ? `<div class="note">Pour appliquer les changements, RamScope doit tourner en administrateur.
      <button class="btn warn sm ml8" id="btnElevate">Passer en administrateur</button></div>`
      : `<div class="note">Lecture seule stricte : l’élévation est désactivée pour cette session.
      Relance RamScope sans <code>--no-admin</code> pour l’autoriser.</div>`;
  }
  $('#notes').innerHTML = n;
  const be = $('#btnElevate');
  if (be) be.onclick = async () => {
    be.disabled = true;
    try {
      await api('/api/elevate', {});
      toast('RamScope redémarre en administrateur…', 'ok');
    } catch (e) {
      be.disabled = false;
      toast('Élévation impossible : ' + esc(e.message), 'ko');
    }
  };
}

function renderHero() {
  const M = S.memory;
  const pctUsed = Math.min(100, (M.used / M.total) * 100);
  const pctCache = Math.min(100 - pctUsed, (M.cache / M.total) * 100);
  $('#hero').innerHTML = `
    <div class="card">
      <div class="k">Mémoire physique</div>
      <div class="v">${fmt(M.used)} <span style="font-size:15px;color:var(--mut);font-weight:500">/ ${fmt(M.total)}</span></div>
      <div class="gauge">
        <i class="g-used" style="width:${pctUsed}%"></i>
        <i class="g-cache" style="width:${Math.max(0, pctCache)}%"></i>
        <i class="g-free" style="width:${Math.max(0, 100 - pctUsed - pctCache)}%"></i>
      </div>
      <div class="legend">
        <span><i class="dot" style="background:#22d3ee"></i>Utilisée ${M.load}%</span>
        <span><i class="dot" style="background:#1e3a5f"></i>Cache système ${fmt(M.cache)}</span>
        <span><i class="dot" style="background:#0f1725;border:1px solid #2b3648"></i>Libre ${fmt(M.available)}</span>
      </div>
    </div>
    <div class="card gain" title="Estimation, agrégée par hôte : un hôte partagé ne compte que si tous ses services sont inclus">
      <div class="k">Récupérable au prochain démarrage</div>
      <div class="v">${gainText(S.plan_gain_next_boot)}</div>
      <div class="s">${S.counts.actionable} réglages superflus détectés</div>
    </div>
    <div class="card gain2">
      <div class="k">Récupérable tout de suite</div>
      <div class="v">${fmt(immediatePotential())}</div>
      <div class="s">en fermant les applications non essentielles</div>
    </div>
    <div class="card" title="Commit Windows : mémoire validée (CommitTotal de GetPerformanceInfo)">
      <div class="k">Mémoire validée (commit)</div>
      <div class="v">${fmt(M.commit_used)}</div>
      <div class="s">sur ${fmt(M.commit_limit)} autorisés</div>
    </div>
    <div class="card" title="KernelTotal : mémoire noyau et pilotes. Compression : working set du processus Memory Compression, une approximation">
      <div class="k">Noyau + compression</div>
      <div class="v">${fmt(M.kernel)}</div>
      <div class="s">dont ${fmt(M.compressed)} de working set « Memory Compression » (approximation)</div>
    </div>`;
}

function immediatePotential() {
  return S.processes.filter(p => selectable(p)).reduce((a, p) => a + gainBytes(p.advice.gain), 0);
}

const TABS = [
  ['overview', 'Vue d’ensemble', null],
  ['plan', 'Plan d’optimisation', () => S.counts.actionable],
  ['processes', 'Processus', () => S.counts.processes],
  ['services', 'Services', () => S.counts.services],
  ['startup', 'Démarrage', () => S.startup.length],
  ['tasks', 'Tâches planifiées', () => S.tasks.length],
  ['restore', 'Restauration', () => (window.RB_COUNT || 0)],
];
function renderTabs() {
  $('#tabs').innerHTML = TABS.map(([id, label, n]) =>
    `<button class="${tab === id ? 'on' : ''}" data-tab="${id}">${label}${n ? `<span class="n">${n()}</span>` : ''}</button>`
  ).join('');
  $$('#tabs button').forEach(b => b.onclick = () => { tab = b.dataset.tab; if (tab === 'restore') loadRollbacks(); paint(); });
}

function renderTools() {
  const t = $('#tools');
  if (tab === 'overview' || tab === 'restore') { t.innerHTML = ''; return; }
  t.innerHTML = `
    <div class="search">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>
      <input id="q" placeholder="Rechercher un nom, un chemin, une raison…  (touche /)" value="${esc(q)}">
    </div>
    <div class="chips">
      ${LEVELS.map(([k, l]) => `<button class="chip ${levels.has(k) ? 'on' : ''}" data-lvl="${k}">${l}</button>`).join('')}
    </div>
    <label class="switch"><input type="checkbox" id="onlyAction" ${onlyAction ? 'checked' : ''}> Actionnable seulement</label>
    <label class="switch"><input type="checkbox" id="autoRefresh" ${auto ? 'checked' : ''}> Auto 15 s</label>
    ${tab === 'plan' ? `<button class="btn sm" id="btnSafe">Sélectionner les éléments réversibles à gain différé</button>` : ''}
    ${tab !== 'plan' ? `<button class="btn sm" id="btnSelView">Sélectionner les actions réversibles de cette vue</button>` : ''}`;
  const qi = $('#q');
  if (qi) {
    qi.oninput = () => { q = qi.value; renderView(); };
    qi.setSelectionRange(q.length, q.length);
  }
  $$('#tools .chip').forEach(c => c.onclick = () => {
    const k = c.dataset.lvl;
    if (levels.has(k)) levels.delete(k); else levels.add(k);
    renderTools(); renderView();
  });
  const oa = $('#onlyAction'); if (oa) oa.onchange = () => { onlyAction = oa.checked; renderView(); };
  const ar = $('#autoRefresh'); if (ar) ar.onchange = () => { auto = ar.checked; };
  const bs = $('#btnSafe'); if (bs) bs.onclick = selectReversible;
  const bv = $('#btnSelView'); if (bv) bv.onclick = () => {
    const kind = tab;
    if (kind === 'processes') {
      toast('La terminaison d’un processus ne peut pas être sélectionnée en masse : choisis-la ligne par ligne.', 'ko');
      return;
    }
    filtered(kind).forEach(it => { if (batchable(it)) { const e = entryFor(kind, it); sel.set(e.key, e); } });
    paint();
  };
}

function match(item, kind) {
  if (levels.size && !levels.has(LVL_CLS[item.advice.level])) return false;
  if (onlyAction && !selectable(item)) return false;
  if (!q) return true;
  const s = q.toLowerCase();
  const fields = [item.name, item.display, item.path, item.command, item.description, item.company,
    item.location, item.advice.reason, item.advice.label, item.dll, (item.triggers || []).join(' '), String(item.pid || '')];
  return fields.some(f => f && String(f).toLowerCase().includes(s));
}
function sortVal(it, k) {
  switch (k) {
    case 'level': return it.advice.level === 'critical' ? 6 : { junk: 0, trim: 1, optional: 2, unknown: 3, keep: 4, system: 5 }[it.advice.level];
    case 'name': return (it.display || it.name || '').toLowerCase();
    case 'private': return it.private || 0;
    case 'ws': return it.working_set || 0;
    case 'ram': return it.ram || 0;
    case 'pid': return it.pid || 0;
    case 'start': return { auto: 0, delayed: 1, manual: 2, disabled: 3 }[it.start];
    case 'state': return it.running ? 0 : 1;
    case 'enabled': return it.enabled ? 0 : 1;
    case 'gain': return gainBytes(it.advice.gain);
    default: return 0;
  }
}
function filtered(kind) {
  const s = sort[kind];
  return allItems(kind).filter(it => match(it, kind)).sort((a, b) => {
    const va = sortVal(a, s.k), vb = sortVal(b, s.k);
    if (va < vb) return -1 * s.d;
    if (va > vb) return 1 * s.d;
    return (b.private || b.ram || 0) - (a.private || a.ram || 0);
  });
}

function renderView() {
  const v = $('#view');
  if (tab === 'overview') return void (v.innerHTML = viewOverview());
  if (tab === 'plan') return void (v.innerHTML = viewPlan());
  if (tab === 'restore') return void (v.innerHTML = viewRestore());
  v.innerHTML = viewTable(tab);
  bindTable(tab);
}

/* ------------------------------------------------------------- vue overview */
function viewOverview() {
  const top = S.processes.slice().sort((a, b) => b.private - a.private).slice(0, 10);
  const max = top.length ? top[0].private : 1;
  const byCat = {};
  S.processes.forEach(p => { byCat[p.category] = (byCat[p.category] || 0) + p.private; });
  const cats = Object.entries(byCat).sort((a, b) => b[1] - a[1]);
  const maxCat = cats.length ? cats[0][1] : 1;
  const junk = S.services.filter(s => s.advice.level === 'junk' || s.advice.level === 'trim').length
    + S.startup.filter(s => s.advice.level === 'junk' || s.advice.level === 'trim').length
    + S.tasks.filter(s => s.advice.level === 'junk' || s.advice.level === 'trim').length;

  return `
  <div class="grid2">
    <div class="card">
      <div class="k">Top 10 — mémoire privée</div>
      <div class="toplist" style="margin-top:10px">
        ${top.map(p => `<div class="toprow">
          <div class="t"><b>${esc(p.name)}</b> <em>${esc(p.description || p.category)}</em>
            <div class="bar" style="margin-top:4px"><i style="width:${(p.private / max * 100).toFixed(1)}%"></i></div>
          </div>
          <div class="num" style="font-size:12.5px">${fmt(p.private)}</div>
        </div>`).join('')}
      </div>
      <p class="sub" style="margin-top:12px">La <b>mémoire privée</b> est celle qui disparaît vraiment quand tu fermes le programme. La « RAM totale » inclut des pages partagées avec d’autres processus.</p>
    </div>

    <div class="card">
      <div class="k">Ce qui se lance tout seul</div>
      <div style="display:flex;flex-direction:column;gap:10px;margin-top:10px">
        <div class="toprow"><div class="t"><b>${S.counts.services_auto}</b> services en démarrage automatique <em>sur ${S.counts.services} (${S.counts.services_running} actifs)</em></div><div class="num"><button class="btn sm" data-go="services">Voir</button></div></div>
        <div class="toprow"><div class="t"><b>${S.counts.startup_enabled}</b> programmes au démarrage de session</div><div class="num"><button class="btn sm" data-go="startup">Voir</button></div></div>
        <div class="toprow"><div class="t"><b>${S.counts.tasks_enabled}</b> tâches planifiées actives</div><div class="num"><button class="btn sm" data-go="tasks">Voir</button></div></div>
      </div>
      <div class="note good" style="margin-top:14px">
        <b>${junk} éléments</b> sont classés « superflu » ou « à alléger » : télémétrie, launchers de jeux, utilitaires constructeurs, updaters.
        <div style="margin-top:8px"><button class="btn primary sm" data-go="plan">Ouvrir le plan d’optimisation</button></div>
      </div>
    </div>

    <div class="card">
      <div class="k">Répartition de la RAM privée</div>
      <div class="toplist" style="margin-top:10px">
        ${cats.map(([c, b]) => `<div class="toprow">
          <div class="t"><b>${esc(c)}</b>
            <div class="bar" style="margin-top:4px"><i style="width:${(b / maxCat * 100).toFixed(1)}%"></i></div>
          </div><div class="num" style="font-size:12.5px">${fmt(b)}</div></div>`).join('')}
      </div>
      <p class="sub" style="margin-top:12px">Total privé de tous les processus : <b>${fmt(S.memory.sum_private)}</b>. L’écart avec la RAM utilisée vient du noyau, des pilotes, du cache et des pages partagées.</p>
    </div>

    <div class="card">
      <div class="k">Comment lire les verdicts</div>
      <div style="display:flex;flex-direction:column;gap:8px;margin-top:10px">
        ${LEVELS.map(([k, l, d]) => `<div style="display:flex;gap:10px;align-items:flex-start">
          <span class="b ${k}" style="min-width:104px;text-align:center">${l}</span>
          <span style="font-size:12.5px;color:var(--mut)">${d}</span></div>`).join('')}
      </div>
      <p class="sub" style="margin-top:12px">Règle d’or : <b>désactiver</b> plutôt que supprimer, et <b>manuel</b> plutôt que désactivé pour un service. Un service en manuel démarre quand un programme le demande — tu ne perds donc pas la fonction.</p>
    </div>
  </div>`;
}

/* ----------------------------------------------------------------- vue plan */
function viewPlan() {
  const groups = [
    ['junk', 'Superflu — éléments inutiles dans la quasi-totalité des cas', 'Télémétrie, démos magasin, composants abandonnés. Rien de ton usage quotidien ne dépend de ces éléments.'],
    ['trim', 'À alléger — gain au prochain démarrage', 'Démarrages automatiques et agents d’arrière-plan dont tu n’as pas besoin en permanence. Les logiciels concernés continueront de fonctionner quand tu les ouvriras.'],
    ['opt', 'Selon ton usage — à toi de décider', 'Ces éléments servent dans certains cas précis (imprimante, Bluetooth, tactile, jeux Xbox…). Lis la raison et coche seulement ce qui ne te concerne pas.'],
  ];
  let html = `<div class="note info">
    Passer un service en <b>manuel</b> ou désactiver un démarrage ne libère pas la RAM instantanément : le gain arrive au prochain redémarrage, et seulement si tous les services d’un hôte partagé sont concernés.
    Coche <b>« Arrêter aussi les services maintenant »</b> dans la barre du bas : la mémoire réellement rendue est mesurée après l’application, jamais ajoutée au gain différé.
  </div>`;
  let rowsCount = 0;
  groups.forEach(([lvl, title, desc]) => {
    const rows = [];
    ['services', 'startup', 'tasks'].forEach(kind => {
      allItems(kind).filter(it => it.advice.level === lvl && batchable(it) && match(it, kind))
        .forEach(it => rows.push({ kind, it }));
    });
    if (!rows.length) return;
    const weight = r => Math.max(gainBytes(r.it.advice.gain), gainBytes(r.it.advice.gain_next_boot));
    rows.sort((a, b) => weight(b) - weight(a));
    const agg = aggregateGains(rows.map(r => entryFor(r.kind, r.it)));
    rowsCount += rows.length;
    const gainTags = [];
    if (gainKnown(agg.now)) gainTags.push(`<span class="tag">${fmt(agg.now.bytes)} tout de suite ${gainNatureBadge(agg.now)}</span>`);
    if (gainKnown(agg.boot)) gainTags.push(`<span class="tag">${fmt(agg.boot.bytes)} au prochain démarrage ${gainNatureBadge(agg.boot)}</span>`);
    const tag = gainTags.length ? gainTags.join(' ') : '<span class="tag">non chiffrable</span>';
    html += `<div class="section">
      <h3><span class="b ${LVL_CLS[lvl]}">${esc(rows[0].it.advice.label)}</span> ${title}
        <span class="tag">${rows.length} éléments</span>
        <span class="gain-tags">${tag}</span>
        <button class="btn sm" data-grp="${lvl}">Cocher ce groupe</button></h3>
      <p class="desc">${desc}</p>
      <div class="tablewrap"><table><thead><tr>
        <th style="width:34px"></th><th>Élément</th><th>Type</th><th>Changement</th><th class="num">Gain</th><th></th>
      </tr></thead><tbody>
      ${rows.map(({ kind, it }) => {
        const e = entryFor(kind, it);
        const on = sel.has(e.key);
        const dkey = kind + ':' + rowId(kind, it);
        const open = expanded.has(dkey);
        const type = kind === 'services' ? 'Service' : kind === 'startup' ? 'Démarrage' : 'Tâche';
        const row = `<tr class="${on ? 'sel' : ''}">
          <td><input type="checkbox" class="cbx" data-k="${kind}" data-id="${esc(rowId(kind, it))}" aria-label="Sélectionner ${esc(it.display || it.name)}" ${on ? 'checked' : ''}></td>
          <td><div class="name">${esc(it.display || it.name)}</div><div class="sub mono">${esc(it.name !== (it.display || it.name) ? it.name : (it.command || it.path || ''))}</div></td>
          <td><span class="tag">${type}</span>${kind === 'services' ? `<div class="sub">${esc(it.state)} · ${startLabel(it.start)}</div>` : ''}</td>
          <td style="font-size:12.5px">${esc(e.sub)}</td>
          <td class="num">${gainLine(e.gain)}${gainKnown(e.boot) ? `<div class="sub">au prochain démarrage : ${gainText(e.boot)}</div>` : ''}</td>
          <td><button class="btn sm ghost" data-exp="${esc(dkey)}" aria-expanded="${open}">${open ? 'Masquer' : 'Détails'}</button></td>
        </tr>`;
        const detail = open ? `<tr class="detail"><td colspan="6">${detailBox(kind, it)}</td></tr>` : '';
        return row + detail;
      }).join('')}
      </tbody></table></div></div>`;
  });
  if (rowsCount === 0) html += `<div class="empty">Rien à optimiser avec les filtres actuels. Décoche les filtres, ou bonne nouvelle : ta configuration est déjà saine.</div>`;
  setTimeout(() => {
    $$('#view [data-grp]').forEach(b => b.onclick = () => {
      const lvl = b.dataset.grp;
      ['services', 'startup', 'tasks'].forEach(kind => allItems(kind)
        .filter(it => it.advice.level === lvl && batchable(it) && match(it, kind))
        .forEach(it => { const e = entryFor(kind, it); sel.set(e.key, e); }));
      paint();
    });
    bindChecks();
    bindDetails();
  }, 0);
  return html;
}

function rowId(kind, it) {
  return kind === 'services' ? it.name : kind === 'startup' ? it.id : kind === 'tasks' ? it.path : String(it.pid);
}
function findItem(kind, id) {
  return allItems(kind).find(it => rowId(kind, it) === id);
}
function bindChecks() {
  $$('#view input.cbx').forEach(c => c.onchange = () => {
    const it = findItem(c.dataset.k, c.dataset.id);
    if (it) toggle(c.dataset.k, it);
  });
}
function bindDetails() {
  $$('#view [data-exp]').forEach(b => b.onclick = () => {
    const k = b.dataset.exp;
    if (expanded.has(k)) expanded.delete(k); else expanded.add(k);
    renderView();
  });
}

/* --------------------------------------------------------------- vue table */
const COLS = {
  processes: [['name', 'Processus'], ['pid', 'PID', 'num'], ['private', 'RAM privée', 'num'], ['ws', 'RAM totale', 'num'], ['cat', 'Type'], ['level', 'Verdict'], ['act', '', 'num']],
  services: [['name', 'Service'], ['state', 'État'], ['start', 'Démarrage'], ['ram', 'RAM', 'num'], ['level', 'Verdict'], ['act', '', 'num']],
  startup: [['name', 'Entrée'], ['loc', 'Emplacement'], ['enabled', 'État'], ['ram', 'RAM actuelle', 'num'], ['level', 'Verdict'], ['act', '', 'num']],
  tasks: [['name', 'Tâche'], ['trig', 'Déclencheurs'], ['enabled', 'État'], ['level', 'Verdict'], ['act', '', 'num']],
};
function viewTable(kind) {
  const rows = filtered(kind);
  const s = sort[kind];
  const maxRam = Math.max(1, ...rows.map(r => r.private || r.ram || 0));
  const head = COLS[kind].map(([k, l, c]) => {
    const sortable = !['cat', 'loc', 'trig', 'act'].includes(k);
    return `<th class="${c || ''}" data-sort="${k}"${sortable ? ' tabindex="0" role="button"' : ''}>${l}${s.k === k ? ` <span class="ar">${s.d > 0 ? '▲' : '▼'}</span>` : ''}</th>`;
  }).join('');
  const body = rows.map(it => {
    const id = rowId(kind, it);
    const key = keyOf(kind, it);
    const on = sel.has(key);
    const can = selectable(it);
    const locked = it.advice.locked;
    const dkey = kind + ':' + id;
    let cells = '';
    const nameCell = `<div class="name">${esc(it.display || it.name)}</div>` +
      (kind === 'services' ? `<div class="sub mono">${esc(it.name)}</div>`
        : kind === 'processes' ? `<div class="sub">${esc(it.description || '')}${it.company ? ' · ' + esc(it.company) : ''}</div>`
          : `<div class="sub mono">${esc(kind === 'tasks' ? it.path : it.command)}</div>`);
    const ramCell = (b) => `<div>${fmt(b)}</div><div class="bar" style="margin-top:4px"><i style="width:${(b / maxRam * 100).toFixed(1)}%"></i></div>`;
    const act = `<div class="rowbtn">
        ${can ? `<button class="btn sm ${kind === 'processes' ? 'danger' : 'primary'}" data-do="${kind}" data-id="${esc(id)}">${kind === 'processes' ? 'Terminer' : kind === 'services' ? startLabel(it.advice.action.start) : 'Désactiver'}</button>` : ''}
        ${kind === 'processes' && can ? '<span class="tag bad">irréversible</span>' : ''}
        <button class="btn sm ghost" data-exp="${esc(dkey)}" aria-expanded="${expanded.has(dkey)}">${expanded.has(dkey) ? 'Masquer' : 'Détails'}</button>
      </div>`;
    if (kind === 'processes') {
      cells = `<td>${nameCell}</td><td class="num">${it.pid}</td><td class="num">${ramCell(it.private)}</td>
        <td class="num">${fmt(it.working_set)}</td><td><span class="tag">${esc(it.category)}</span>${it.has_window ? '<div class="sub">fenêtre ouverte</div>' : ''}</td>
        <td>${badge(it.advice)}</td><td class="num">${act}</td>`;
    } else if (kind === 'services') {
      cells = `<td>${nameCell}</td>
        <td><span class="tag ${it.running ? 'run' : 'off'}">${esc(it.state)}</span>${it.host_shared > 1 ? `<div class="sub">hôte partagé ×${it.host_shared}</div>` : ''}</td>
        <td><span class="tag ${it.start === 'auto' || it.start === 'delayed' ? 'on' : 'off'}">${startLabel(it.start)}</span></td>
        <td class="num">${ramCell(it.ram)}</td><td>${badge(it.advice)}</td><td class="num">${act}</td>`;
    } else if (kind === 'startup') {
      cells = `<td>${nameCell}</td><td><span class="tag">${esc(it.location)}</span></td>
        <td><span class="tag ${it.enabled ? 'on' : 'off'}">${it.enabled ? 'Activé' : 'Désactivé'}</span></td>
        <td class="num">${it.ram ? fmt(it.ram) : '—'}</td><td>${badge(it.advice)}</td><td class="num">${act}</td>`;
    } else {
      cells = `<td>${nameCell}</td><td>${it.triggers.map(t => `<span class="tag">${esc(t)}</span>`).join(' ')}</td>
        <td><span class="tag ${it.enabled ? 'on' : 'off'}">${it.enabled ? 'Activée' : 'Désactivée'}</span></td>
        <td>${badge(it.advice)}</td><td class="num">${act}</td>`;
    }
    const cb = locked
      ? `<span title="Élément verrouillé par RamScope" style="opacity:.5">🔒</span>`
      : batchable(it) ? `<input type="checkbox" class="cbx" data-k="${kind}" data-id="${esc(id)}" aria-label="Sélectionner ${esc(it.display || it.name)}" ${on ? 'checked' : ''}>` : '';
    let detail = '';
    if (expanded.has(dkey)) detail = `<tr class="detail"><td colspan="${COLS[kind].length + 1}">${detailBox(kind, it)}</td></tr>`;
    return `<tr class="${on ? 'sel' : ''} ${locked ? 'locked' : ''}"><td>${cb}</td>${cells}</tr>${detail}`;
  }).join('');
  const note = kind === 'processes'
    ? `<div class="note info">La <b>RAM privée</b> est ce que tu récupères en fermant le processus. Les processus vitaux sont verrouillés. « Terminer » ne sauvegarde pas le travail en cours.</div>`
    : kind === 'services'
      ? `<div class="note info">Un service en <b>manuel</b> reste disponible : il démarre quand un programme le demande. La RAM indiquée pour un <span class="mono">svchost.exe</span> est une <b>part estimée</b> (l’hôte est partagé entre plusieurs services).</div>`
      : kind === 'startup'
        ? `<div class="note info">RamScope utilise le même mécanisme que le Gestionnaire des tâches : l’entrée n’est <b>pas supprimée</b>, seulement désactivée. Tu peux la réactiver ici ou dans Windows.</div>`
        : `<div class="note info">Désactiver une tâche est réversible. Les tâches de maintenance (SSD, restauration, Defender) sont classées « à garder ».</div>`;
  return note + `<div class="tablewrap"><table><thead><tr><th style="width:34px"></th>${head}</tr></thead>
    <tbody>${body || `<tr><td colspan="${COLS[kind].length + 1}" class="empty">Aucun élément ne correspond aux filtres.</td></tr>`}</tbody></table></div>
    <p class="sub" style="margin-top:8px">${rows.length} élément(s) affiché(s) sur ${allItems(kind).length}</p>`;
}

function currentState(kind, it) {
  const lock = it.advice.locked ? ' · verrouillé' : '';
  if (kind === 'processes') return `PID ${it.pid} · ${it.has_window ? 'fenêtre ouverte' : 'sans fenêtre'} · ${it.category}${lock}`;
  if (kind === 'services') return `${it.state} · démarrage ${startLabel(it.start)}${lock}`;
  if (kind === 'startup') return `${it.enabled ? 'activé' : 'désactivé'} · ${it.location}`;
  return `${it.enabled ? 'active' : 'désactivée'} · ${(it.triggers || []).join(', ') || 'sans déclencheur'}`;
}
function reversibility(it) {
  const a = it.advice.action;
  if (!a) return 'Aucune action proposée';
  if (a.kind === 'kill') return 'Irréversible — le travail non enregistré sera perdu';
  if (a.kind === 'stop_service') return 'Partiel — le service peut redémarrer, mais son état et son cache sont perdus';
  return 'Réversible via le point de restauration';
}
function detailBox(kind, it) {
  const f = [];
  const add = (k, v) => { if (v !== undefined && v !== null && String(v).trim()) f.push(`<div><span>${esc(k)}</span><p class="mono">${esc(v)}</p></div>`); };
  const addText = (k, v) => { if (v) f.push(`<div><span>${esc(k)}</span><p>${esc(v)}</p></div>`); };
  const addHtml = (k, v) => { if (v) f.push(`<div><span>${esc(k)}</span><p>${v}</p></div>`); };

  const source = kind === 'processes' ? it.path : kind === 'services' ? it.path : kind === 'startup' ? it.command : it.path;
  add('Source détectée', source || (it.name || ''));
  if (kind === 'services' && it.dll) add('DLL de service', it.dll);
  addText('État actuel', currentState(kind, it));
  addHtml('Effet tout de suite', it.advice.action ? gainText(it.advice.gain) : 'Aucun changement');
  addHtml('Effet au prochain démarrage', it.advice.action ? gainText(it.advice.gain_next_boot) : 'Aucun changement');
  addText('Réversibilité', reversibility(it));
  addText('Justification', it.advice.reason);
  addText('Provenance de la règle', provenanceText(it.advice));

  if (kind === 'processes') {
    addText('Éditeur', it.company); addText('Description', it.description);
    add('PID / parent', it.pid + ' / ' + it.ppid); add('Threads', it.threads);
    add('Services hébergés', (it.services || []).join(', '));
    add('RAM partagée', fmt(Math.max(0, it.working_set - it.private)));
  } else if (kind === 'services') {
    add('Nom court', it.name); addText('Compte', it.account); add('PID hôte', it.pid || '—');
    add('RAM de l’hôte', fmt(it.host_ram) + (it.host_shared > 1 ? ` (partagé entre ${it.host_shared} services)` : ''));
    addText('Description Windows', it.description);
  } else if (kind === 'startup') {
    add('Identifiant', it.id); addText('Emplacement', it.location);
  } else {
    add('Chemin complet', it.path); addText('Programme', it.command); addText('Auteur', it.author);
    addText('Description', it.description); addText('Déclencheurs', (it.triggers || []).join(', '));
  }
  return `<div class="detailbox">${f.join('')}</div>`;
}

function bindTable(kind) {
  $$('#view thead th[data-sort]').forEach(th => {
    const activate = () => {
      const k = th.dataset.sort;
      if (['cat', 'loc', 'trig', 'act'].includes(k)) return;
      const s = sort[kind];
      if (s.k === k) s.d *= -1; else { s.k = k; s.d = (k === 'name' || k === 'level') ? 1 : -1; }
      renderView();
    };
    th.onclick = activate;
    th.onkeydown = e => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activate(); } };
  });
  bindChecks();
  bindDetails();
  $$('#view [data-do]').forEach(b => b.onclick = () => {
    const kind = b.dataset.do, it = findItem(kind, b.dataset.id);
    if (!it) return;
    const e = entryFor(kind, it);
    confirmApply([e], kind === 'processes');
  });
  $$('#view [data-go]').forEach(b => b.onclick = () => { tab = b.dataset.go; paint(); });
}

/* --------------------------------------------------------- barre d'action */
function renderActionBar() {
  const bar = $('#actionbar');
  const n = sel.size;
  bar.classList.toggle('show', n > 0);
  if (!n) return;
  const t = selTotals();
  $('#selCount').textContent = n + ' sélectionné' + (n > 1 ? 's' : '');
  const parts = [];
  if (gainKnown(t.now)) parts.push(`<b>${fmt(t.now.bytes)}</b> ${gainNatureBadge(t.now)} Tout de suite`);
  if (gainKnown(t.boot)) parts.push(`<b>${fmt(t.boot.bytes)}</b> ${gainNatureBadge(t.boot)} Au prochain démarrage`);
  $('#selGain').innerHTML = parts.length ? 'Gain : ' + parts.join(' · ') : 'Gain : non chiffrable';
  $('#stopNowWrap').style.display = Array.from(sel.values()).some(e => e.kind === 'services' && e.running) ? '' : 'none';
}

/* ------------------------------------------------------------- application */
function buildActions(entries) {
  const acts = [];
  const stopNow = $('#stopNow') && $('#stopNow').checked;
  entries.forEach(e => {
    acts.push(e.action);
    if (stopNow && e.kind === 'services' && e.running &&
      (e.action.start === 'manual' || e.action.start === 'disabled')) {
      acts.push({ kind: 'stop_service', name: e.name });
    }
  });
  return acts;
}
function describeAction(a) {
  if (a.kind === 'service') return `Service « ${a.name} » → ${startLabel(a.start)}`;
  if (a.kind === 'stop_service') return `Arrêter « ${a.name} » maintenant`;
  if (a.kind === 'start_service') return `Redémarrer « ${a.name} »`;
  if (a.kind === 'startup') return `Démarrage « ${a.id.split('|').pop()} » → ${a.enabled ? 'activé' : 'désactivé'}`;
  if (a.kind === 'task') return `Tâche « ${a.path.split('\\').pop()} » → ${a.enabled ? 'activée' : 'désactivée'}`;
  if (a.kind === 'kill') return `Terminer ${a.name} (PID ${a.pid})`;
  return JSON.stringify(a);
}

function actionNote(a) {
  if (a.kind === 'kill') return 'Irréversible : travail non enregistré perdu, aucune annulation.';
  if (a.kind === 'stop_service') return 'Partiel : le service peut redémarrer, mais son état et son cache sont perdus.';
  return 'Réversible via le point de restauration.';
}
function confirmApply(entries, danger) {
  if (!S.meta.is_admin) {
    toast(S.meta.can_elevate
      ? 'Droits administrateur requis. Clique sur « Passer en administrateur ».'
      : 'Élévation désactivée pour cette session : relance RamScope sans --no-admin.', 'ko');
    return;
  }
  if (entries.some(e => e.kind === 'processes') && entries.length > 1) {
    toast('Une terminaison de processus ne peut pas être appliquée dans un lot : traite-la seule, ligne par ligne.', 'ko');
    return;
  }
  const acts = buildActions(entries);
  const t = aggregateGains(entries);
  const kills = entries.filter(e => e.kind === 'processes').length;
  const irreversible = acts.filter(a => a.kind === 'kill').length;
  const reversible = acts.length - irreversible;
  const killNames = entries.filter(e => e.kind === 'processes').map(e => e.name).join(', ');
  openModal(`
    <header><h2>Confirmer ${acts.length} changement${acts.length > 1 ? 's' : ''}</h2>
      <p><b>${reversible}</b> action${reversible > 1 ? 's' : ''} réversible${reversible > 1 ? 's' : ''} · <b>${irreversible}</b> irréversible${irreversible > 1 ? 's' : ''}</p></header>
    <div class="body">
      ${kills ? `<div class="note">⚠️ ${kills} processus ${kills > 1 ? 'vont' : 'va'} être <b>terminé${kills > 1 ? 's' : ''}</b> : <b>${esc(killNames)}</b>. Le travail non enregistré dans ${kills > 1 ? 'ces programmes' : 'ce programme'} sera perdu. Cette action n’est pas réversible (il faut relancer le programme).</div>` : '<div class="note good">Aucun processus n’est terminé. Un point de restauration est créé automatiquement avant l’application des changements réversibles.</div>'}
      <ul class="changelist">
        ${acts.map(a => `<li class="${a.kind === 'kill' ? 'kill' : ''}">
          <span class="ic">${a.kind === 'kill' ? '⚠️' : a.kind === 'stop_service' ? '⏹' : '→'}</span>
          <div><b>${esc(describeAction(a))}</b>${a.kind === 'kill' ? ' <span class="tag bad">irréversible</span>' : ''}<div class="m">${esc(actionNote(a))}</div></div></li>`).join('')}
      </ul>
      <div class="note good" style="margin-top:14px">
        <div>Tout de suite : <b>${gainText(t.now)}</b></div>
        <div>Au prochain démarrage : <b>${gainText(t.boot)}</b></div>
      </div>
    </div>
    <footer>
      <button class="btn ghost" data-close>Annuler</button>
      <button class="btn ${danger ? 'danger' : 'primary'}" id="doApply">${danger ? 'Terminer quand même' : 'Appliquer'}</button>
    </footer>`);
  $('#doApply').onclick = async () => {
    const btn = $('#doApply');
    btn.disabled = true; btn.innerHTML = '<span class="spin"></span> Application…';
    try {
      const r = await api('/api/apply', { actions: acts, label: entries.length === 1 ? entries[0].label : `Lot de ${entries.length} réglages` });
      entries.forEach(e => sel.delete(e.key));
      showResults(r, false);
      await load(true);
    } catch (e) {
      toast('Échec : ' + esc(e.message), 'ko');
      btn.disabled = false; btn.textContent = 'Réessayer';
    }
  };
}

function showResults(r, isRestore) {
  const ok = r.results.filter(x => x.ok).length;
  const ko = r.results.filter(x => !x.ok);
  openModal(`
    <header><h2>${isRestore ? 'Restauration' : 'Application'} : ${ok}/${r.results.length} réussi${ok > 1 ? 's' : ''}</h2>
      <p>${r.rollback_id ? `Point de restauration <span class="mono">${esc(r.rollback_id)}</span> créé — onglet Restauration.` : isRestore ? 'Le point de restauration a été consommé.' : 'Aucun point créé (rien de réversible appliqué).'}</p></header>
    <div class="body"><ul class="changelist">
      ${r.results.map(x => `<li><span class="ic ${x.ok ? 'res-ok' : 'res-ko'}">${x.ok ? '✔' : '✖'}</span>
        <div><b>${esc(x.label)}</b><div class="m">${esc(x.message)}</div></div></li>`).join('')}
    </ul>
    ${r.memory ? `<div class="note info" style="margin-top:14px">Mémoire disponible mesurée : <b>${fmt(r.memory.available_before)}</b> → <b>${fmt(r.memory.available_after)}</b> (${r.memory.available_after >= r.memory.available_before ? '+' : '−'}${fmt(Math.abs(r.memory.available_after - r.memory.available_before))}, ${gainNature(r.memory.gain)}). C’est une mesure après coup, pas une estimation.</div>` : ''}
    ${ko.length && !isRestore ? `<div class="note" style="margin-top:14px">Certains éléments ont refusé la modification (service protégé par Windows, droits insuffisants, tâche verrouillée). C’est normal et sans danger : rien n’a été forcé.</div>` : ''}
    ${!isRestore && ok ? `<div class="note info" style="margin-top:14px">Un service reconfiguré (manuel ou désactivé) ne libère sa part qu’au prochain redémarrage, et seulement si tous les services de son hôte sont concernés. Pense à redémarrer pour constater le gain.</div>` : ''}
    </div>
    <footer><button class="btn primary" data-close>Terminé</button></footer>`);
  if (ok) toast(`${ok} changement(s) appliqué(s).`, 'ok');
}

/* ------------------------------------------------------------- restauration */
let RB = [];
async function loadRollbacks() {
  try { RB = await api('/api/rollbacks'); window.RB_COUNT = RB.length; if (tab === 'restore') { renderTabs(); renderView(); } }
  catch (e) { RB = []; }
}
function viewRestore() {
  if (!RB.length) {
    return `<div class="empty">Aucun point de restauration pour l’instant.<br>
      <span class="sub">Chaque lot appliqué crée automatiquement un point ici, avec l’état exact d’avant modification.</span></div>`;
  }
  return `<div class="note info">Restaurer remet chaque élément dans son état d’avant : type de démarrage des services, activation des entrées de démarrage et des tâches, et redémarrage des services arrêtés.</div>
  ${RB.map(p => `<div class="section"><h3>${esc(p.label)} <span class="tag">${esc(p.created_at)}</span>
      <span class="tag">${p.entries.length} changement(s)</span>
      <button class="btn sm primary" data-rb="${esc(p.id)}">Restaurer cet état</button></h3>
    <div class="tablewrap"><table><thead><tr><th>Changement appliqué</th><th>Retour prévu</th></tr></thead><tbody>
      ${p.entries.map(e => `<tr><td>${esc(e.label)}</td><td class="sub">${e.undo ? esc(describeAction(e.undo)) : '—'}</td></tr>`).join('')}
    </tbody></table></div></div>`).join('')}`;
}
document.addEventListener('click', ev => {
  const b = ev.target.closest('[data-rb]');
  if (!b) return;
  const p = RB.find(x => x.id === b.dataset.rb);
  if (!p) return;
  openModal(`<header><h2>Restaurer « ${esc(p.label)} » ?</h2><p>${p.entries.length} changement(s) vont être annulés.</p></header>
    <div class="body"><ul class="changelist">${p.entries.filter(e => e.undo).map(e => `<li><span class="ic">↩</span><div><b>${esc(describeAction(e.undo))}</b></div></li>`).join('')}</ul></div>
    <footer><button class="btn ghost" data-close>Annuler</button><button class="btn primary" id="doRestore">Restaurer</button></footer>`);
  $('#doRestore').onclick = async () => {
    const btn = $('#doRestore'); btn.disabled = true; btn.innerHTML = '<span class="spin"></span> Restauration…';
    try {
      const r = await api('/api/restore', { id: p.id });
      showResults(r, true);
      await loadRollbacks(); await load(true);
    } catch (e) { toast('Échec : ' + esc(e.message), 'ko'); btn.disabled = false; btn.textContent = 'Réessayer'; }
  };
});

/* ------------------------------------------------------------------ modale */
let lastFocus = null;
function openModal(html) {
  lastFocus = document.activeElement;
  $('#modal').innerHTML = html;
  $('#mask').classList.add('show');
  $$('#modal [data-close]').forEach(b => b.onclick = closeModal);
  const first = $('#modal button, #modal input, #modal [href]');
  if (first) first.focus();
}
function closeModal() {
  const wasOpen = $('#mask').classList.contains('show');
  $('#mask').classList.remove('show');
  if (wasOpen && lastFocus && lastFocus.focus) lastFocus.focus();
  lastFocus = null;
}
// Un clic peut changer de cible entre l’appui et le relâchement quand une modale
// s’ouvre après un appel réseau (export) : seul un appui commencé sur le fond
// ferme la modale, jamais un clic hérité du bouton qui l’a ouverte.
let maskPressed = false;
$('#mask').addEventListener('pointerdown', e => { maskPressed = e.target === $('#mask'); });
$('#mask').addEventListener('click', e => {
  if (maskPressed && e.target === $('#mask')) closeModal();
  maskPressed = false;
});
document.addEventListener('keydown', e => {
  if (e.key === 'Escape' && $('#mask').classList.contains('show')) closeModal();
  if (e.key === '/' && !$('#mask').classList.contains('show') && !/input|textarea/i.test(document.activeElement.tagName)) {
    const i = $('#q'); if (i) { e.preventDefault(); i.focus(); }
  }
});

/* ------------------------------------------------------------------ divers */
async function elevate() {
  try {
    await api('/api/elevate', {});
    toast('RamScope se relance en administrateur…', 'ok');
    setTimeout(() => location.reload(), 4000);
  } catch (e) { toast(esc(e.message), 'ko'); }
}
window.elevate = elevate;

$('#btnRefresh').onclick = () => load(true);
$('#btnClearSel').onclick = () => { sel.clear(); paint(); };
$('#btnApply').onclick = () => {
  const entries = Array.from(sel.values());
  if (!entries.length) return;
  confirmApply(entries, entries.some(e => e.kind === 'processes'));
};
$('#stopNow').onchange = renderActionBar;
$('#btnExport').onclick = async () => {
  try {
    const r = await api('/api/export', {});
    openModal(`<header><h2>Rapport exporté</h2><p>Deux fichiers ont été écrits sur le disque.</p></header>
      <div class="body"><ul class="changelist">
        <li><span class="ic">📄</span><div><b>Rapport HTML</b><div class="m mono">${esc(r.html)}</div></div></li>
        <li><span class="ic">🧾</span><div><b>Données JSON</b><div class="m mono">${esc(r.json)}</div></div></li>
      </ul><div class="note info" style="margin-top:14px">Le rapport HTML s’ouvre dans n’importe quel navigateur, s’imprime en PDF et se partage sans RamScope.</div></div>
      <footer><button class="btn ghost" id="openDir">Ouvrir le dossier</button><button class="btn primary" data-close>Fermer</button></footer>`);
    const od = $('#openDir'); if (od) od.onclick = () => api('/api/open-dir', {}).catch(() => { });
  } catch (e) { toast('Export impossible : ' + esc(e.message), 'ko'); }
};
$('#btnQuit').onclick = async () => {
  openModal(`<header><h2>Quitter RamScope ?</h2><p>Le moteur local s’arrête. Tes points de restauration sont conservés sur le disque.</p></header>
    <div class="body"><p class="sub">Tu peux fermer cette fenêtre ensuite.</p></div>
    <footer><button class="btn ghost" data-close>Rester</button><button class="btn danger" id="doQuit">Quitter</button></footer>`);
  $('#doQuit').onclick = async () => { try { await api('/api/quit', {}); } catch (e) { } closeModal(); toast('RamScope arrêté. Tu peux fermer cette fenêtre.', 'ok'); };
};
document.addEventListener('click', e => {
  const g = e.target.closest('[data-go]');
  if (g) { tab = g.dataset.go; paint(); }
});

setInterval(() => { if (auto && !busy && !$('#mask').classList.contains('show')) load(true); }, 15000);
load(true);
loadRollbacks();
