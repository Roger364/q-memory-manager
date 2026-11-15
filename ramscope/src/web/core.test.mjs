import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const core = require('./core.js');

function gain(bytes, nature, scope = 'private') {
  return { bytes, nature, scope, additive: false };
}

test('un kill et un élément verrouillé ne sont pas batchables', () => {
  assert.equal(core.batchable({ advice: { action: { kind: 'service' } } }), true);
  assert.equal(core.batchable({ advice: { action: { kind: 'kill' } } }), false);
  assert.equal(core.batchable({ advice: { action: { kind: 'service' }, locked: true } }), false);
  assert.equal(core.batchable({ advice: { action: null } }), false);
  assert.equal(core.batchable(null), false);
});

test('un zéro mesuré reste une mesure, un zéro estimé est ignoré', () => {
  const zeroMesure = gain(0, 'measured');
  const zeroEstime = gain(0, 'estimated');
  assert.equal(core.gainKnown(zeroMesure), true);
  assert.equal(core.gainKnown(zeroEstime), false);

  const total = core.emptyGain();
  core.addGain(total, zeroMesure);
  assert.equal(total.nature, 'measured');
  assert.equal(total.bytes, 0);
  core.addGain(total, zeroEstime);
  assert.equal(total.nature, 'measured');
  assert.equal(total.bytes, 0);
});

test('le gain différé ne devient jamais immédiat et les hôtes complets comptent une fois', () => {
  const measured = core.aggregateGains([
    { kind: 'startup', gain: gain(10, 'estimated'), boot: gain(20, 'estimated') },
    { kind: 'tasks', gain: gain(0, 'unavailable'), boot: gain(0, 'unavailable') },
  ]);
  assert.equal(measured.now.bytes, 10);
  assert.equal(measured.boot.bytes, 20);

  const host = core.aggregateGains([
    { kind: 'services', name: 'a', host: 7, hostShared: 2, hostRam: 100, gain: gain(0, 'unavailable'), boot: gain(0, 'unavailable') },
    { kind: 'services', name: 'b', host: 7, hostShared: 2, hostRam: 100, gain: gain(0, 'unavailable'), boot: gain(0, 'unavailable') },
  ]);
  assert.equal(host.now.bytes, 0);
  assert.equal(host.boot.bytes, 100);
  assert.equal(host.boot.nature, 'estimated');

  const incomplete = core.aggregateGains([
    { kind: 'services', name: 'a', host: 7, hostShared: 2, hostRam: 100, gain: gain(0, 'unavailable'), boot: gain(0, 'unavailable') },
  ]);
  assert.equal(incomplete.boot.bytes, 0);
  assert.equal(incomplete.boot.nature, 'unavailable');
});

test('un clic hérité ne ferme pas la modale', () => {
  assert.equal(core.modalShouldClose(false, true), false);
  assert.equal(core.modalShouldClose(true, false), false);
  assert.equal(core.modalShouldClose(true, true), true);
});
