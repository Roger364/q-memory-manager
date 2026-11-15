/* RamScope — décisions pures de l'interface, sans DOM ni dépendance. */
(function (root) {
  'use strict';

  function emptyGain() {
    return { bytes: 0, nature: 'unavailable', scope: 'private', additive: false };
  }
  function gainBytes(g) { return g && g.bytes ? g.bytes : 0; }
  function gainNature(g) {
    return g && g.nature === 'measured' ? 'mesuré' : g && g.nature === 'estimated' ? 'estimé' : 'non chiffrable';
  }
  function gainKnown(g) {
    return !!g && g.nature !== 'unavailable' && !(g.nature === 'estimated' && !gainBytes(g));
  }
  function addGain(total, g) {
    if (!gainKnown(g)) return;
    if (total.nature === 'unavailable') {
      total.bytes = gainBytes(g); total.nature = g.nature; total.scope = g.scope;
      return;
    }
    total.bytes += gainBytes(g);
    total.nature = total.nature === 'measured' && g.nature === 'measured' ? 'measured' : 'estimated';
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
  function selectable(item) {
    return !!(item && item.advice && item.advice.action && !item.advice.locked);
  }
  function batchable(item) {
    return selectable(item) && item.advice.action.kind !== 'kill';
  }
  // Seul un appui commencé sur le fond ferme la modale : un clic hérité du
  // bouton qui l'a ouverte ne doit pas la refermer.
  function modalShouldClose(maskPressed, targetIsMask) {
    return !!maskPressed && !!targetIsMask;
  }

  const api = {
    emptyGain, gainBytes, gainNature, gainKnown, addGain, aggregateGains,
    selectable, batchable, modalShouldClose,
  };
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.RamScopeCore = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
