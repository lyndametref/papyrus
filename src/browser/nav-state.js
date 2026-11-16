/* Papyrus navigation state: apply the stored preference before first paint.
   Loaded synchronously in <head>; no dependency on app.js. */
(function () {
  "use strict";

  var stored = null;
  try {
    stored = localStorage.getItem("papyrus.nav");
  } catch (e) {
    stored = null;
  }
  if (stored === "collapsed") {
    document.documentElement.setAttribute("data-nav", "collapsed");
  }

  function reveal() {
    var toggle = document.querySelector(".nav-toggle");
    if (toggle) {
      toggle.removeAttribute("hidden");
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", reveal);
  } else {
    reveal();
  }
})();
