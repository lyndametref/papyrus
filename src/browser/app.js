/* Papyrus local browser progressive enhancement.
   Everything works without this script. */
(function () {
  "use strict";

  var toggle = document.querySelector(".sidebar-toggle");
  var typeList = document.getElementById("type-list");
  var narrow = window.matchMedia ? window.matchMedia("(max-width: 480px)") : null;

  function setCollapsed(collapsed) {
    if (!typeList || !toggle) {
      return;
    }
    if (collapsed) {
      typeList.setAttribute("data-collapsed", "true");
      toggle.setAttribute("aria-expanded", "false");
    } else {
      typeList.removeAttribute("data-collapsed");
      toggle.setAttribute("aria-expanded", "true");
    }
  }

  function applyViewport() {
    if (narrow && narrow.matches) {
      if (typeList && !typeList.hasAttribute("data-collapsed")) {
        setCollapsed(true);
      }
    } else {
      setCollapsed(false);
    }
  }

  if (toggle && typeList) {
    toggle.addEventListener("click", function () {
      setCollapsed(typeList.getAttribute("data-collapsed") !== "true");
    });
    applyViewport();
    if (narrow) {
      if (narrow.addEventListener) {
        narrow.addEventListener("change", applyViewport);
      } else if (narrow.addListener) {
        narrow.addListener(applyViewport);
      }
    }
  }

  var form = document.querySelector("form[data-dirty-guard]");
  if (form) {
    var dirty = false;
    form.addEventListener("input", function () {
      dirty = true;
    });
    window.addEventListener("beforeunload", function (event) {
      if (!dirty) {
        return;
      }
      event.preventDefault();
      event.returnValue = "";
    });
  }
})();
