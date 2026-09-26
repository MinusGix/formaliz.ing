// Footnotes, as the Press renders them: references are <sup class="footnote-ref" id="fnref-N">,
// and notes are <li class="footnote" id="fn-N"> inside section.footnotes.
//
// Wide screens: each note's body is moved into the right margin beside its first reference,
//   pushed down as needed so notes never overlap. The foot section is hidden meanwhile.
// Narrow screens: hovering, focusing, or tapping a reference shows the note in a popover.
// Without JS, the notes stay at the foot, with links both ways.
(function () {
  "use strict";
  var article = document.querySelector("article.entry");
  var foot = article && article.querySelector("section.footnotes");
  if (!foot) return;

  var WIDE = window.matchMedia("(min-width: 1120px)");
  var CAN_HOVER = window.matchMedia("(hover: hover)");
  var GAP = 12; // px between stacked sidenotes

  var notes = Array.prototype.map.call(foot.querySelectorAll("li.footnote"), function (li) {
    var n = li.id.slice(3);
    return {
      n: n,
      li: li,
      body: li.querySelector(".footnote-body"),
      refs: Array.prototype.slice.call(article.querySelectorAll('a[href="#fn-' + n + '"]')),
      aside: null,
    };
  });
  var byRef = new Map();
  notes.forEach(function (note) {
    note.refs.forEach(function (a) { byRef.set(a, note); });
  });

  // ── Sidenotes ─────────────────────────────────────────────

  var column = null;

  function enterMargin() {
    column = document.createElement("div");
    column.className = "sidenote-column";
    notes.forEach(function (note) {
      var aside = document.createElement("aside");
      aside.className = "sidenote";
      aside.id = "sn-" + note.n;
      var label = document.createElement("span");
      label.className = "sidenote-label";
      label.textContent = note.n;
      aside.append(label, note.body);
      aside.addEventListener("mouseenter", function () { light(note, true); });
      aside.addEventListener("mouseleave", function () { light(note, false); });
      note.aside = aside;
      column.append(aside);
    });
    article.append(column);
    article.classList.add("sidenotes-on");
    layout();
  }

  function leaveMargin() {
    notes.forEach(function (note) {
      note.li.append(note.body);
      note.aside = null;
    });
    column.remove();
    column = null;
    article.classList.remove("sidenotes-on");
  }

  function layout() {
    if (!column) return;
    var origin = article.getBoundingClientRect().top;
    var floor = 0;
    notes.forEach(function (note) {
      var ref = note.refs[0];
      var want = ref ? lineTop(ref) - origin : floor;
      var top = Math.max(want, floor);
      note.aside.style.top = top + "px";
      floor = top + note.aside.offsetHeight + GAP;
    });
  }

  // Top of the text line holding the reference, rather than of the raised numeral.
  function lineTop(ref) {
    var sup = ref.closest("sup") || ref;
    var line = sup.parentElement;
    var lh = parseFloat(getComputedStyle(line).lineHeight) || 0;
    var r = sup.getBoundingClientRect();
    return lh ? r.bottom - lh * 0.8 : r.top;
  }

  function light(note, on) {
    if (note.aside) note.aside.classList.toggle("lit", on);
    note.refs.forEach(function (a) { a.classList.toggle("lit", on); });
  }

  function sync() {
    hidePopover();
    if (WIDE.matches && !column) enterMargin();
    else if (!WIDE.matches && column) leaveMargin();
  }

  // ── Popovers ──────────────────────────────────────────────

  var popover = null;
  var popoverRef = null;
  var hideTimer = 0;
  var showTimer = 0;

  function showPopover(note, ref) {
    clearTimeout(hideTimer);
    if (popoverRef === ref) return;
    hidePopover();
    popover = document.createElement("div");
    popover.className = "footnote-popover";
    popover.setAttribute("role", "note");
    var label = document.createElement("a");
    label.className = "sidenote-label";
    label.href = "#fn-" + note.n;
    label.textContent = note.n;
    label.addEventListener("click", hidePopover);
    popover.append(label, note.body.cloneNode(true));
    popover.addEventListener("mouseenter", function () { clearTimeout(hideTimer); });
    popover.addEventListener("mouseleave", scheduleHide);
    document.body.append(popover);
    popoverRef = ref;
    placePopover(ref);
  }

  function placePopover(ref) {
    var margin = 16;
    var r = ref.getBoundingClientRect();
    var vw = document.documentElement.clientWidth;
    var width = popover.offsetWidth;
    var left = Math.min(Math.max(r.left + r.width / 2 - width / 2, margin), vw - width - margin);
    var below = r.bottom + 8;
    var above = r.top - 8 - popover.offsetHeight;
    var fitsBelow = below + popover.offsetHeight <= window.innerHeight - margin;
    var top = fitsBelow || above < margin ? below : above;
    popover.style.left = left + window.scrollX + "px";
    popover.style.top = top + window.scrollY + "px";
  }

  function hidePopover() {
    clearTimeout(hideTimer);
    clearTimeout(showTimer);
    if (popover) popover.remove();
    popover = null;
    popoverRef = null;
  }

  function scheduleHide() {
    clearTimeout(showTimer);
    clearTimeout(hideTimer);
    hideTimer = setTimeout(hidePopover, 250);
  }

  // ── Wiring ────────────────────────────────────────────────

  byRef.forEach(function (note, ref) {
    ref.addEventListener("mouseenter", function () {
      if (column) { light(note, true); return; }
      if (!CAN_HOVER.matches) return;
      clearTimeout(showTimer);
      showTimer = setTimeout(function () { showPopover(note, ref); }, 120);
    });
    ref.addEventListener("mouseleave", function () {
      if (column) { light(note, false); return; }
      scheduleHide();
    });
    // Keyboard focus only; a tap also focuses, and its click handler decides.
    ref.addEventListener("focus", function () {
      if (!column && ref.matches(":focus-visible")) showPopover(note, ref);
    });
    ref.addEventListener("blur", function () { if (!column) scheduleHide(); });
    ref.addEventListener("click", function (event) {
      if (column) {
        // The note is already beside the text: draw the eye to it instead of jumping.
        event.preventDefault();
        note.aside.scrollIntoView({ block: "nearest", behavior: "smooth" });
        note.aside.classList.remove("flash");
        void note.aside.offsetWidth;
        note.aside.classList.add("flash");
      } else if (!CAN_HOVER.matches) {
        // Touch: the first tap opens the popover; its number link still reaches the foot.
        event.preventDefault();
        if (popoverRef === ref) hidePopover();
        else showPopover(note, ref);
      }
    });
  });

  document.addEventListener("click", function (event) {
    if (popover && !popover.contains(event.target) && !byRef.has(event.target)) hidePopover();
  });
  document.addEventListener("keydown", function (event) {
    if (event.key === "Escape") hidePopover();
  });

  WIDE.addEventListener("change", sync);
  // Text reflows as fonts load, MathJax typesets, and images arrive; follow it.
  new ResizeObserver(function () { layout(); }).observe(article);
  if (document.fonts) document.fonts.ready.then(layout);
  if (window.MathJax && MathJax.startup && MathJax.startup.promise) MathJax.startup.promise.then(layout);
  window.addEventListener("load", layout);
  sync();
})();
