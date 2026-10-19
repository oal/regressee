/* Regressee landing page behaviour.
   Three small jobs, none of them required for the page to read:
   1. remember an appearance choice (System / Light / Dark) like the app does;
   2. open the download the visitor's platform actually needs;
   3. the proofing loupe: drag it, or move it with the arrow keys, to read the
      baseline pull underneath the check. Home returns it to the heading
      change; the page also keeps a small, honest record of this session's
      readings next to the ink bar. */

(function () {
  "use strict";

  var root = document.documentElement;

  /* ------------------------------------------------ appearance choice */
  var themeRadios = Array.prototype.slice.call(document.querySelectorAll('input[name="theme"]'));

  function storedTheme() {
    try {
      var value = localStorage.getItem("regressee.theme");
      return value === "light" || value === "dark" ? value : "system";
    } catch (error) {
      return "system";
    }
  }

  function applyTheme(choice) {
    root.setAttribute("data-theme", choice);
    themeRadios.forEach(function (radio) { radio.checked = radio.value === choice; });
    try {
      if (choice === "system") localStorage.removeItem("regressee.theme");
      else localStorage.setItem("regressee.theme", choice);
    } catch (error) { /* session-only when storage is unavailable */ }
  }

  /* ------------------------------------- this session's record (decorative) */
  var slots = Array.prototype.slice.call(document.querySelectorAll("[data-slot]"));
  var noted = [];

  function note(text) {
    if (!slots.length || noted.indexOf(text) !== -1) return;
    noted.push(text);
    slots.forEach(function (slot, index) {
      if (index >= noted.length) return;
      slot.textContent = noted[index];
      slot.classList.add("on");
    });
  }

  themeRadios.forEach(function (radio) {
    radio.addEventListener("change", function () {
      applyTheme(radio.value);
      note("appearance: " + radio.value);
    });
  });
  applyTheme(storedTheme());

  /* --------------------------------------------- platform download */
  var heroDownload = document.getElementById("hero-download");
  var RELEASES = "https://github.com/oal/regressee/releases/download/v0.1.0-preview.10/";
  if (heroDownload) {
    var platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    if (/win/i.test(platform)) {
      heroDownload.textContent = "Download for Windows";
      heroDownload.href = RELEASES + "regressee-desktop-windows-x86_64.zip";
    } else if (/linux/i.test(platform) && !/android/i.test(platform)) {
      heroDownload.textContent = "Download for Linux";
      heroDownload.href = RELEASES + "regressee-desktop-linux-x86_64.tar.gz";
    }
    /* Anything else keeps "Download beta" pointing at the download section. */
  }

  /* --------------------------------------------------- the proofing loupe */
  var proofPage = document.getElementById("proof-page");
  var loupe = document.getElementById("loupe");
  var heading = document.getElementById("demo-heading");
  var loupeMode = window.matchMedia("(min-width: 821px)");
  if (proofPage && loupe) {
    var RADIUS = 78; /* half of the loupe's 156px box in styles.css */
    var EDGE = 4;
    var x = 0;
    var y = 0;
    var home = null;
    var proofW = 0;
    var proofH = 0;
    var dragging = false;
    var loupeUsed = false;

    function clamp() {
      x = Math.min(Math.max(x, RADIUS + EDGE), Math.max(RADIUS + EDGE, proofW - RADIUS - EDGE));
      y = Math.min(Math.max(y, RADIUS + EDGE), Math.max(RADIUS + EDGE, proofH - RADIUS - EDGE));
    }

    function place() {
      proofPage.style.setProperty("--loupe-x", x.toFixed(1) + "px");
      proofPage.style.setProperty("--loupe-y", y.toFixed(1) + "px");
      var percent = Math.round((x / Math.max(1, proofW)) * 100);
      loupe.setAttribute("aria-valuenow", String(percent));
      loupe.setAttribute("aria-valuetext", percent + " percent across the proof — hold it over a circled correction to read the baseline pull");
    }

    function measure() {
      proofW = proofPage.clientWidth;
      proofH = proofPage.clientHeight;
      proofPage.style.setProperty("--proof-w", proofW + "px");
      proofPage.style.setProperty("--proof-h", proofH + "px");
      var anchor = document.querySelector(".circled") || heading;
      if (!home && anchor) {
        var pageRect = proofPage.getBoundingClientRect();
        var anchorRect = anchor.getBoundingClientRect();
        home = {
          x: anchorRect.left - pageRect.left + anchorRect.width * 0.5,
          y: anchorRect.top - pageRect.top + anchorRect.height * 0.5
        };
        x = home.x;
        y = home.y;
      }
      clamp();
      place();
    }

    function moveTo(clientX, clientY) {
      var rect = proofPage.getBoundingClientRect();
      x = clientX - rect.left;
      y = clientY - rect.top;
      clamp();
      place();
    }

    loupe.addEventListener("pointerdown", function (event) {
      if (!loupeMode.matches) return;
      dragging = true;
      loupeUsed = true;
      loupe.classList.add("is-dragging");
      if (loupe.setPointerCapture) loupe.setPointerCapture(event.pointerId);
      moveTo(event.clientX, event.clientY);
      event.preventDefault();
    });

    loupe.addEventListener("pointermove", function (event) {
      if (!dragging) return;
      moveTo(event.clientX, event.clientY);
    });

    function endDrag() {
      if (!dragging) return;
      dragging = false;
      loupe.classList.remove("is-dragging");
      note("loupe: baseline read");
    }

    loupe.addEventListener("pointerup", endDrag);
    loupe.addEventListener("pointercancel", endDrag);

    loupe.addEventListener("keydown", function (event) {
      var stepX = event.shiftKey ? proofW * 0.10 : proofW * 0.01;
      var stepY = event.shiftKey ? proofH * 0.10 : proofH * 0.02;
      var handled = true;
      if (event.key === "ArrowLeft") { x -= stepX; }
      else if (event.key === "ArrowRight") { x += stepX; }
      else if (event.key === "ArrowUp") { y -= stepY; }
      else if (event.key === "ArrowDown") { y += stepY; }
      else if (event.key === "Home") { x = home ? home.x : x; y = home ? home.y : y; }
      else { handled = false; }
      if (!handled) return;
      event.preventDefault();
      loupeUsed = true;
      clamp();
      place();
      note("loupe: baseline read");
    });

    /* Measure now, and again once the font has settled, so the loupe starts on
       the heading while the visitor has not moved it themselves. */
    measure();
    if (document.fonts && document.fonts.ready && document.fonts.ready.then) {
      document.fonts.ready.then(function () {
        if (!loupeUsed) home = null;
        measure();
      });
    }
    window.addEventListener("resize", measure);
    if ("ResizeObserver" in window) new ResizeObserver(measure).observe(proofPage);
  }
})();
