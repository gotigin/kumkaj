// kirano.shop start-up screen. Inlined into index.html by
// scripts/patch-web-shell.sh; runs before the Makepad module script.
//
// The ring shows real progress: the wasm download is counted as it streams
// (the decoded size is written into data-wasm-bytes at build time). After
// the download the page still has to compile and fetch its fonts and
// pictures, so the status changes to "Starting". The screen fades out when
// Makepad removes its own `.canvas_loader` element, which it does once the
// canvas has painted.
(function () {
  var boot = document.getElementById('boot');
  if (!boot) return;
  var arc = document.getElementById('boot-arc');
  var mark = document.getElementById('boot-mark');
  var status = document.getElementById('boot-status');
  var help = document.getElementById('boot-help');
  var tag = document.getElementById('boot-tag');
  var total = Number(boot.getAttribute('data-wasm-bytes')) || 0;
  var RING = 377;
  var DOWNLOAD_SHARE = 0.9; // the rest of the ring fills when the canvas paints
  var got = 0;
  var finished = false;
  var failed = false;

  function setProgress(p) {
    if (p < 0) p = 0;
    if (p > 1) p = 1;
    arc.style.strokeDashoffset = String(RING * (1 - p));
    mark.setAttribute('aria-valuenow', String(Math.round(p * 100)));
  }
  var saying = 'Loading…';
  var slow = false;
  function say(text) {
    saying = text;
    var shown = slow ? text + ' A slow connection can take a little longer.' : text;
    if (status.textContent !== shown) status.textContent = shown;
  }

  // 1. Count the wasm bytes as they arrive.
  var nativeFetch = window.fetch;
  if (nativeFetch && window.ReadableStream && window.Response && total > 0) {
    window.fetch = function (input) {
      var url = typeof input === 'string' ? input : (input && input.url) || '';
      var pending = nativeFetch.apply(this, arguments);
      if (!/\.wasm(\?|$)/.test(url)) return pending;
      pending.catch(function () {
        fail('This page could not be loaded. Check the connection.');
      });
      return pending.then(function (res) {
        try {
          if (!res.ok) {
            fail('This page could not be loaded (' + res.status + ').');
            return res;
          }
          if (!res.body) return res;
          var reader = res.body.getReader();
          var counted = new ReadableStream({
            pull: function (controller) {
              return reader.read().then(function (step) {
                if (step.done) {
                  controller.close();
                  window.fetch = nativeFetch;
                  if (!finished && !failed) {
                    setProgress(DOWNLOAD_SHARE);
                    say('Starting…');
                  }
                  return;
                }
                got += step.value.byteLength;
                if (!finished && !failed) {
                  var part = Math.min(1, got / total);
                  setProgress(part * DOWNLOAD_SHARE);
                  say('Loading ' + Math.round(part * 100) + '%');
                }
                controller.enqueue(step.value);
              });
            },
            cancel: function (reason) {
              return reader.cancel(reason);
            }
          });
          return new Response(counted, { status: res.status, statusText: res.statusText, headers: res.headers });
        } catch (e) {
          return res;
        }
      });
    };
  }

  // 2. A second line that changes, so a long wait is not a still picture.
  var lines = (tag.getAttribute('data-lines') || '').split('|').filter(Boolean);
  var still = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var lineAt = 0;
  var lineTimer = 0;
  if (lines.length > 1 && !still) {
    lineTimer = window.setInterval(function () {
      tag.classList.add('boot-tag--out');
      window.setTimeout(function () {
        lineAt = (lineAt + 1) % lines.length;
        tag.textContent = lines[lineAt];
        tag.classList.remove('boot-tag--out');
      }, 320);
    }, 2800);
  }

  // 3. Say so when it is slow, and say so when it failed.
  var slowTimer = window.setTimeout(function () {
    if (finished || failed) return;
    slow = true;
    say(saying);
  }, 15000);
  function fail(text) {
    if (finished || failed) return;
    failed = true;
    window.clearInterval(lineTimer);
    window.clearTimeout(slowTimer);
    boot.classList.add('boot--failed');
    boot.setAttribute('aria-busy', 'false');
    slow = false;
    say(text);
    help.hidden = false;
  }
  var reload = document.getElementById('boot-reload');
  if (reload) reload.addEventListener('click', function () { location.reload(); });
  // Only a failure of the page's own start-up counts: the Makepad module
  // script is inline, so its errors carry this page's address. Errors from
  // other scripts (analytics, a blocked request) do not stop the canvas.
  window.addEventListener('error', function (event) {
    var from = (event && event.filename) || '';
    if (from && from.split('#')[0].split('?')[0] === location.href.split('#')[0].split('?')[0]) {
      fail('This page could not start.');
    }
  });
  try {
    var probe = document.createElement('canvas');
    if (!(probe.getContext('webgl2') || probe.getContext('webgl'))) {
      fail('This page needs WebGL, which this browser has switched off.');
    }
  } catch (e) {}

  // 4. Leave when the canvas has painted.
  function finish() {
    if (finished) return;
    finished = true;
    window.clearInterval(lineTimer);
    window.clearTimeout(slowTimer);
    window.fetch = nativeFetch;
    if (failed) {
      boot.classList.remove('boot--failed');
      help.hidden = true;
    }
    setProgress(1);
    boot.setAttribute('aria-busy', 'false');
    boot.classList.add('boot--done');
    window.setTimeout(function () {
      if (boot.parentNode) boot.parentNode.removeChild(boot);
    }, 400);
  }
  function loaderGone() {
    return document.getElementsByClassName('canvas_loader').length === 0;
  }
  if (window.MutationObserver) {
    var watch = new MutationObserver(function () {
      if (loaderGone()) {
        watch.disconnect();
        finish();
      }
    });
    watch.observe(document.body, { childList: true });
  } else {
    var poll = window.setInterval(function () {
      if (loaderGone()) {
        window.clearInterval(poll);
        finish();
      }
    }, 100);
  }
})();
