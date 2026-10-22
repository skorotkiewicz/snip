// Exercise the embedded UI in Chromium without npm dependencies or CDN access.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';

const source = await readFile(process.argv[2] || new URL('../src/web.rs', import.meta.url), 'utf8');
const html = source.slice(source.indexOf('<!DOCTYPE html>'), source.lastIndexOf('"##;'))
    .replace(/<script src="[^"]*"><\/script>/g, '')
    .replace(/<link id="hljs-theme"[^>]*>/, '<link id="hljs-theme" rel="stylesheet" href="data:text/css,">');
const hostileAuthor = 'user" onmouseover="window.injected=1';
const content = Array.from({ length: 45 }, (_, i) => `line ${i + 1}: <script>window.injected=1</script>`).join('\n');
const snippets = [
    { id: 1, author: hostileAuthor, content, description: 'Long snippet', language: 'plaintext', created_at: '2026-10-22T10:00:00Z' },
    { id: 2, author: 'alice', content: 'short snippet', description: 'Short snippet', language: 'plaintext', created_at: '2026-10-22T10:00:00Z' },
    { id: 3, author: 'alice', content: 'long wrapped line '.repeat(150), description: 'Wrapped snippet', language: 'plaintext', created_at: '2026-10-22T10:00:00Z' },
];
const server = http.createServer((req, res) => {
    if (req.url.startsWith('/api/')) {
        res.setHeader('Content-Type', 'application/json');
        res.end(JSON.stringify(req.url.endsWith('/comments')
            ? { comments: [{ id: 1, author: hostileAuthor, content: '<img src=x onerror="window.injected=1">', created_at: '2026-10-22T10:00:00Z', children: [] }] }
            : /^\/api\/snippets\/\d+$/.test(req.url) ? snippets[0] : { snippets, total: snippets.length }));
    } else {
        res.setHeader('Content-Type', 'text/html');
        res.end(html);
    }
});
server.listen(0, '127.0.0.1');
await once(server, 'listening');
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = await mkdtemp(path.join(os.tmpdir(), 'snip-browser-'));
const browser = spawn(process.env.CHROMIUM || 'chromium', [
    '--headless=new', '--no-sandbox', '--disable-dev-shm-usage', '--disable-gpu',
    '--disable-background-networking', '--no-first-run', '--remote-debugging-pipe',
    `--user-data-dir=${profile}`,
], { stdio: ['ignore', 'ignore', 'pipe', 'pipe', 'pipe'] });
let errors = '';
browser.stderr.on('data', chunk => { errors = (errors + chunk).slice(-4000); });
const pending = new Map();
let buffer = '';
let sequence = 0;
browser.stdio[4].on('data', chunk => {
    buffer += chunk;
    let end;
    while ((end = buffer.indexOf('\0')) !== -1) {
        const message = JSON.parse(buffer.slice(0, end));
        buffer = buffer.slice(end + 1);
        const request = pending.get(message.id);
        if (request) {
            pending.delete(message.id);
            clearTimeout(request.timeout);
            if (message.error) request.reject(new Error(JSON.stringify(message.error)));
            else request.resolve(message.result);
        }
    }
});
browser.once('error', error => {
    for (const request of pending.values()) request.reject(error);
});
function command(method, params = {}, sessionId) {
    return new Promise((resolve, reject) => {
        const id = ++sequence;
        const timeout = setTimeout(() => reject(new Error(`Timed out: ${method}\n${errors}`)), 15000);
        pending.set(id, { resolve, reject, timeout });
        browser.stdio[3].write(JSON.stringify({ id, method, params, sessionId }) + '\0');
    });
}
let session;
async function evaluate(expression) {
    const result = await command('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true }, session);
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
}
async function waitFor(expression) {
    await evaluate(`new Promise((resolve, reject) => {
        const deadline = Date.now() + 5000;
        const timer = setInterval(() => {
            if (${expression}) { clearInterval(timer); resolve(); }
            else if (Date.now() > deadline) { clearInterval(timer); reject(new Error('DOM condition timed out')); }
        }, 20);
    })`);
}
try {
    const { targetId } = await command('Target.createTarget', { url: 'about:blank' });
    ({ sessionId: session } = await command('Target.attachToTarget', { targetId, flatten: true }));
    await command('Page.enable', {}, session);
    await command('Network.enable', {}, session);
    await command('Network.setBlockedURLs', { urls: ['https://cdnjs.cloudflare.com/*'] }, session);
    await command('Emulation.setDeviceMetricsOverride', { width: 1100, height: 900, deviceScaleFactor: 1, mobile: false }, session);
    await command('Page.navigate', { url: origin }, session);
    await waitFor('document.querySelectorAll(".snippet").length === 3');
    assert.equal(await evaluate('document.querySelector("[onmouseover]") !== null'), false, 'usernames must not inject attributes');
    assert.equal(await evaluate('window.injected'), undefined, 'snippet content must stay text');
    assert.equal(await evaluate('document.querySelector("#snippet-1 .snippet-meta a").getAttribute("href")'), `/u/${encodeURIComponent(hostileAuthor)}`, 'profile URLs must encode usernames');
    assert.equal(await evaluate('document.querySelector("#snippet-content-1 code").textContent'), content);
    await waitFor('document.querySelector("#snippet-1 .snippet-expand")?.hidden === false');
    assert.equal(await evaluate('document.querySelector("#snippet-content-1").getBoundingClientRect().height <= 288'), true);
    assert.equal(await evaluate('document.querySelector("#snippet-2 .snippet-expand").hidden'), true, 'short snippets need no toggle');
    assert.equal(await evaluate('document.querySelector("#snippet-3 .snippet-expand").hidden'), false, 'wrapped lines must collapse too');
    await evaluate('document.querySelector("#snippet-1 .snippet-expand").click()');
    assert.equal(await evaluate('document.querySelector("#snippet-1 .snippet-expand").getAttribute("aria-expanded")'), 'true');
    assert.equal(await evaluate('document.querySelector("#snippet-1 .snippet-expand").textContent'), 'Show less');
    assert.equal(await evaluate('document.querySelector("#snippet-content-1").getBoundingClientRect().height > 288'), true);
    await evaluate('document.querySelector("#snippet-1 .snippet-expand").click()');
    assert.equal(await evaluate('document.querySelector("#snippet-1 .snippet-expand").getAttribute("aria-expanded")'), 'false');
    await command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true }, session);
    await waitFor('document.querySelector("#snippet-content-1").getBoundingClientRect().height <= 288');
    // The single-snippet page uses the same controls and safely renders reply authors.
    await evaluate(`localStorage.setItem('snip_api_key', 'test-key')`);
    await command('Page.navigate', { url: `${origin}/s/1` }, session);
    await waitFor('document.querySelectorAll(".comment").length === 1');
    assert.equal(await evaluate('document.querySelector("[onmouseover], [onerror]") !== null'), false, 'comments must escape content and author attributes');
    await evaluate('document.querySelector("[data-comment-action=reply]").click()');
    assert.equal(await evaluate('document.getElementById("comment-reply-author").textContent'), hostileAuthor);
    assert.equal(await evaluate('document.querySelector(".snippet-expand").hidden'), false);
    if (process.env.SNIP_SCREENSHOT) {
        const { data } = await command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true }, session);
        await writeFile(process.env.SNIP_SCREENSHOT, Buffer.from(data, 'base64'));
    }
    console.log('Web UI passed: expansion, collapse, short and wrapped snippets, mobile, detail page, and HTML escaping.');
} finally {
    for (const request of pending.values()) clearTimeout(request.timeout);
    browser.kill();
    await once(browser, 'exit').catch(() => {});
    server.close();
    await rm(profile, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
}
