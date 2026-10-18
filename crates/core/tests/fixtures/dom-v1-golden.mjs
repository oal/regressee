// SPDX-License-Identifier: MPL-2.0
// Standalone cross-language contract check; no browser, dependencies or network.
// Run: node --test crates/core/tests/fixtures/dom-v1-golden.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { test } from 'node:test';

const golden = readFileSync(new URL('./dom-v1.ndjson', import.meta.url));
const records = golden.toString('utf8').trimEnd().split('\n').map(JSON.parse);
const utf8Compare = (a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b));
const attrCompare = (a, b) => utf8Compare(a[0], b[0]) || utf8Compare(a[1], b[1]);
test('canonical v1 Rust fixture has exactly reproducible JSON.stringify bytes', () => {
  const normalized = records.map(record => {
    if (record[0] !== 'element') return record;
    return [...record.slice(0, 4), [...record[4]].reverse().sort(attrCompare), ...record.slice(5)];
  });
  assert.deepEqual(Buffer.from(normalized.map(JSON.stringify).join('\n') + '\n'), golden);
  assert.equal('\ud800'.toWellFormed(), '�');
  assert.equal(JSON.stringify('\u2028\u2029'), '"\u2028\u2029"');
  assert.equal(createHash('sha256').update(golden).digest('hex'), 'd9876d0964cc64dc5279bd14f3fa6493134ac0dc7b7580ff9ba12484b58d807e');
});
test('canonical source plaintext fixture preserves each record and quoted whitespace', () => {
  const quote = JSON.stringify;
  const source = records.slice(1).map(record => {
    const [kind, depth, ...fields] = record;
    let text;
    switch (kind) {
      case 'document': text = '#document'; break;
      case 'doctype': text = '!doctype ' + fields.map(quote).join(' '); break;
      case 'element': {
        const [namespace, name, attributes, prefix] = fields;
        text = `element ${quote(namespace)} ${quote(name)}`;
        if (prefix !== undefined) text += ` prefix=${quote(prefix)}`;
        for (const [ns, attrName, value] of attributes) text += ` ${quote(ns)}:${quote(attrName)}=${quote(value)}`;
        break;
      }
      case 'text': text = `text ${quote(fields[0])}`; break;
      case 'shadow': text = '#shadow-root (open)'; break;
      case 'template': text = '#template-content'; break;
      default: assert.fail('unknown fixture grammar');
    }
    return `[${depth}] ${text}\n`;
  }).join('');
  assert.equal(source, readFileSync(new URL('./dom-v1-source.txt', import.meta.url), 'utf8'));
});
test('HTML names and actual element prefixes have collision-free canonical bytes', () => {
  const bytes = readFileSync(new URL('./dom-v1-html-names.ndjson', import.meta.url));
  const nodes = bytes.toString('utf8').trimEnd().split('\n').map(JSON.parse);
  assert.deepEqual(Buffer.from(nodes.map(JSON.stringify).join('\n') + '\n'), bytes);
  const elements = nodes.filter(node => node[0] === 'element');
  const attributes = elements.find(node => node[3] === 'div')[4];
  assert.deepEqual([...attributes].reverse().sort(attrCompare), attributes);
  const actualLocalName = ([namespace, name]) => namespace === '' ? name : name.slice(name.indexOf(':') + 1);
  assert.equal(actualLocalName(['', 'a:foo']), 'a:foo');
  assert.equal(actualLocalName(['urn:fixture', 'a:foo']), 'foo');
  const selected = elements.filter(node => node[3] === 'a:b' || node[3] === 'b');
  assert.equal(new Set(selected.map(JSON.stringify)).size, selected.length);
});
