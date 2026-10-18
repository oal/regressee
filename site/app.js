/* Regressee landing page behaviour.
   Four small jobs, none of them required for the page to read:
   1. remember an appearance choice (System / Light / Dark) like the app does;
   2. open the download the visitor's platform actually needs;
   3. settle and drive the seam between the old way and Regressee;
   4. light the check's run line as it comes into view. */

(function () {
  "use strict";

  var root = document.documentElement;
  var reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  var splitMode = window.matchMedia("(min-width: 821px)");

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

  themeRadios.forEach(function (radio) {
    radio.addEventListener("change", function () { applyTheme(radio.value); });
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

  /* --------------------------------------------------- the seam */
  var hero = document.querySelector(".hero");
  var seam = hero && hero.querySelector(".seam");
  var MIN_SPLIT = 30;
  var MAX_SPLIT = 70;

  function setSplit(percent) {
    var value = Math.max(MIN_SPLIT, Math.min(MAX_SPLIT, percent));
    hero.style.setProperty("--split", value.toFixed(1) + "%");
    if (seam) {
      seam.setAttribute("aria-valuenow", String(Math.round(value)));
      seam.setAttribute("aria-valuetext", Math.round(value) + " percent for the old way, " + Math.round(100 - value) + " percent for Regressee");
    }
  }

  function currentSplit() {
    var inline = parseFloat(hero.style.getPropertyValue("--split"));
    if (!isNaN(inline)) return inline;
    var base = parseFloat(getComputedStyle(hero).getPropertyValue("--split"));
    return isNaN(base) ? 46 : base;
  }

  /* Settle the seam once on load so the drag is discoverable. */
  requestAnimationFrame(function () {
    document.body.classList.remove("settling");
    if (!reducedMotion.matches && splitMode.matches && hero) {
      hero.style.setProperty("--split", "46%");
    }
  });

  if (hero && seam) {
    var dragging = false;

    var onPointerMove = function (event) {
      if (!dragging) return;
      setSplit((event.clientX / window.innerWidth) * 100);
    };

    var onPointerUp = function () {
      if (!dragging) return;
      dragging = false;
      hero.classList.remove("is-dragging");
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerUp);
    };

    /* Both the seam column and its pill are drag handles; the pill sits to the
       left of the seam so it never covers the app evidence. The move and release
       listeners live on the window so the drag keeps working past the handle. */
    var bindDrag = function (handle) {
      handle.addEventListener("pointerdown", function (event) {
        if (!splitMode.matches) return;
        dragging = true;
        hero.classList.add("is-dragging");
        window.addEventListener("pointermove", onPointerMove);
        window.addEventListener("pointerup", onPointerUp);
        window.addEventListener("pointercancel", onPointerUp);
        event.preventDefault();
      });
    };

    bindDrag(seam);
    var pill = seam.querySelector(".seam-pill");
    if (pill) bindDrag(pill);

    seam.addEventListener("keydown", function (event) {
      if (!splitMode.matches) return;
      var step = event.shiftKey ? 10 : 1;
      var next = null;
      if (event.key === "ArrowLeft") next = currentSplit() - step;
      else if (event.key === "ArrowRight") next = currentSplit() + step;
      else if (event.key === "Home") next = MIN_SPLIT;
      else if (event.key === "End") next = MAX_SPLIT;
      if (next === null) return;
      event.preventDefault();
      setSplit(next);
    });

    /* Leaving split mode must not strand an inline width on the stacked layout. */
    var onSplitModeChange = function (event) {
      if (!event.matches) hero.style.removeProperty("--split");
    };
    if (splitMode.addEventListener) splitMode.addEventListener("change", onSplitModeChange);
    else splitMode.addListener(onSplitModeChange);
  }

  /* ------------------------------------------------ the run line */
  var runline = document.getElementById("runline");
  var fill = document.getElementById("track-fill");
  if (runline && fill && "IntersectionObserver" in window) {
    var stops = Array.prototype.slice.call(runline.querySelectorAll(".stop"));

    if (reducedMotion.matches) {
      stops.forEach(function (stop) { stop.classList.add("is-active"); });
      fill.style.width = "100%";
    } else {
      var timers = [];
      fill.style.width = "0%";
      var run = function () {
        stops.forEach(function (stop, index) {
          timers.push(window.setTimeout(function () {
            stop.classList.add("is-active");
            fill.style.width = (stops.length > 1 ? (index / (stops.length - 1)) * 100 : 100) + "%";
          }, index * 140));
        });
      };

      var observer = new IntersectionObserver(function (entries) {
        entries.forEach(function (entry) {
          if (!entry.isIntersecting) return;
          observer.disconnect();
          run();
        });
      }, { threshold: 0.35 });
      observer.observe(runline);

      reducedMotion.addEventListener && reducedMotion.addEventListener("change", function (event) {
        if (!event.matches) return;
        timers.forEach(window.clearTimeout);
        stops.forEach(function (stop) { stop.classList.add("is-active"); });
        fill.style.width = "100%";
      });
    }
  }
})();
