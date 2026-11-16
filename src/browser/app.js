/* Papyrus local browser progressive enhancement.
   Everything works without this script. */
(function () {
  "use strict";

  var navToggle = document.querySelector(".nav-toggle");
  var siteNav = document.getElementById("site-nav");

  function applyNav(collapsed) {
    if (collapsed) {
      document.documentElement.setAttribute("data-nav", "collapsed");
    } else {
      document.documentElement.removeAttribute("data-nav");
    }
    if (navToggle) {
      navToggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
    }
    if (siteNav) {
      if (collapsed) {
        siteNav.setAttribute("hidden", "");
      } else {
        siteNav.removeAttribute("hidden");
      }
    }
  }

  if (navToggle) {
    applyNav(document.documentElement.getAttribute("data-nav") === "collapsed");
    navToggle.addEventListener("click", function () {
      var collapsed =
        document.documentElement.getAttribute("data-nav") === "collapsed";
      var next = collapsed ? "expanded" : "collapsed";
      applyNav(next === "collapsed");
      try {
        localStorage.setItem("papyrus.nav", next);
      } catch (e) {
        /* Storage unavailable: the session keeps toggling in memory. */
      }
    });
  }

  var form = document.querySelector("form[data-dirty-guard]");
  if (form) {
    var dirty = false;
    var status = document.querySelector("[data-dirty-status]");
    var note = form.querySelector("[data-form-note]");
    form.addEventListener("input", function () {
      if (dirty) {
        return;
      }
      dirty = true;
      if (status) {
        status.removeAttribute("hidden");
      }
      if (note) {
        note.textContent = "Leaving with unsaved changes warns only when JavaScript is available.";
      }
    });
    window.addEventListener("beforeunload", function (event) {
      if (!dirty) {
        return;
      }
      event.preventDefault();
      event.returnValue = "";
    });
  }

  function markDirty(target) {
    var formEl = target.closest ? target.closest("form") : null;
    if (!formEl) {
      return;
    }
    var evt = document.createEvent("Event");
    evt.initEvent("input", true, true);
    formEl.dispatchEvent(evt);
  }

  function tagNoun(field) {
    return field === "specs" ? "UI specification" : "Reference";
  }

  function addTag(tagbox, field) {
    var addInput = tagbox.querySelector(".tag-add");
    if (!addInput) {
      return;
    }
    var value = addInput.value.trim();
    if (!value) {
      return;
    }
    var count = tagbox.querySelectorAll(".tag").length + 1;
    var tag = document.createElement("span");
    tag.className = "tag";
    var input = document.createElement("input");
    input.className = "tag-input";
    input.type = "text";
    input.name = field;
    input.value = value;
    input.id = field + "-tag-" + count;
    input.setAttribute("aria-label", tagNoun(field) + " " + count);
    var remove = document.createElement("button");
    remove.className = "tag-remove";
    remove.type = "button";
    remove.setAttribute("aria-label", "Remove " + value);
    remove.setAttribute("data-remove", "");
    remove.textContent = "×";
    remove.addEventListener("click", function () {
      tag.remove();
      markDirty(remove);
    });
    tag.appendChild(input);
    tag.appendChild(remove);
    tagbox.insertBefore(tag, addInput);
    addInput.value = "";
    markDirty(addInput);
  }

  var tagboxes = document.querySelectorAll("[data-tagbox]");
  for (var i = 0; i < tagboxes.length; i++) {
    var box = tagboxes[i];
    var field = box.getAttribute("data-tagbox");
    var addInputEl = box.querySelector(".tag-add");
    var removes = box.querySelectorAll("[data-remove]");
    for (var r = 0; r < removes.length; r++) {
      (function (btn) {
        btn.addEventListener("click", function () {
          var tagEl = btn.closest ? btn.closest(".tag") : btn.parentNode;
          if (tagEl && tagEl.remove) {
            tagEl.remove();
          } else if (tagEl && tagEl.parentNode) {
            tagEl.parentNode.removeChild(tagEl);
          }
          markDirty(btn);
        });
      })(removes[r]);
    }
    if (addInputEl) {
      addInputEl.addEventListener("keydown", function (event) {
        if (event.key === "Enter") {
          event.preventDefault();
          addTag(box, field);
        }
      });
    }
  }
  var addButtons = document.querySelectorAll("[data-add]");
  for (var a = 0; a < addButtons.length; a++) {
    (function (btn) {
      btn.addEventListener("click", function () {
        var f = btn.getAttribute("data-add");
        var b = document.querySelector('[data-tagbox="' + f + '"]');
        if (b) {
          addTag(b, f);
        }
      });
    })(addButtons[a]);
  }

  /* Backlog board: drag-and-drop is enhancement only. The visible select and
     Change status button remain the complete no-script path. */
  var boardCards = document.querySelectorAll(".board-card");
  var boardColumns = document.querySelectorAll(".board-column");
  if (boardCards.length && boardColumns.length) {
    var draggedTask = null;

    function clearDropTargets() {
      for (var c = 0; c < boardColumns.length; c++) {
        boardColumns[c].removeAttribute("data-drop-target");
      }
    }

    function boardCardFor(taskId) {
      for (var i = 0; i < boardCards.length; i++) {
        if (boardCards[i].getAttribute("data-task") === taskId) {
          return boardCards[i];
        }
      }
      return null;
    }

    for (var i = 0; i < boardCards.length; i++) {
      (function (card) {
        card.addEventListener("dragstart", function (event) {
          if (!event.dataTransfer) {
            return;
          }
          draggedTask = card.getAttribute("data-task");
          event.dataTransfer.effectAllowed = "move";
          try {
            event.dataTransfer.setData("text/plain", draggedTask || "");
          } catch (err) {
            /* Some browsers restrict dataTransfer in synthetic events. */
          }
        });
        card.addEventListener("dragend", function () {
          draggedTask = null;
          clearDropTargets();
        });
      })(boardCards[i]);
    }

    for (var c = 0; c < boardColumns.length; c++) {
      (function (column) {
        column.addEventListener("dragover", function (event) {
          if (draggedTask === null) {
            return;
          }
          event.preventDefault();
          if (event.dataTransfer) {
            event.dataTransfer.dropEffect = "move";
          }
          column.setAttribute("data-drop-target", "true");
        });
        column.addEventListener("dragleave", function () {
          column.removeAttribute("data-drop-target");
        });
        column.addEventListener("drop", function (event) {
          event.preventDefault();
          column.removeAttribute("data-drop-target");
          if (draggedTask === null) {
            return;
          }
          var status = column.getAttribute("data-status");
          var card = boardCardFor(draggedTask);
          draggedTask = null;
          if (!status || !card) {
            return;
          }
          var form = card.querySelector("form.card-status");
          var select = form ? form.querySelector("select[name=\"status\"]") : null;
          if (!form || !select) {
            return;
          }
          var available = false;
          for (var o = 0; o < select.options.length; o++) {
            if (select.options[o].value === status) {
              available = true;
              break;
            }
          }
          if (!available) {
            return;
          }
          select.value = status;
          form.submit();
        });
      })(boardColumns[c]);
    }
  }

  /* Math placeholders (TASK-145): render each data-tex element with the
     vendored KaTeX. The element's own source is the no-script fallback.
     KaTeX clears the element before parsing, so a thrown error restores that
     source and marks the element; a missing global marks every element. */
  var mathElements = document.querySelectorAll("[data-tex]");
  if (mathElements.length) {
    if (!window.katex || typeof window.katex.render !== "function") {
      for (var m = 0; m < mathElements.length; m++) {
        mathElements[m].setAttribute("data-render-error", "math");
      }
    } else {
      for (var k = 0; k < mathElements.length; k++) {
        (function (element) {
          var source = element.getAttribute("data-tex");
          var displayMode = element.className.indexOf("math--display") !== -1;
          try {
            window.katex.render(source, element, {
              throwOnError: false,
              displayMode: displayMode,
              trust: false,
              strict: "warn",
              output: "htmlAndMathml",
            });
          } catch (e) {
            element.textContent = source;
            element.setAttribute("data-render-error", "math");
          }
        })(mathElements[k]);
      }
    }
  }

  /* Diagrams (TASK-146): render each mermaid figure with the vendored bundle.
     The server-rendered `<pre>` is the no-script fallback and the client
     input. Every figure renders in isolation; a failure keeps the source. */
  var diagramFigures = document.querySelectorAll('figure[data-diagram="mermaid"]');
  if (diagramFigures.length) {
    function sanitiseDiagramId(path) {
      var out = "";
      var limit = path.length < 64 ? path.length : 64;
      for (var s = 0; s < limit; s++) {
        var character = path.charAt(s);
        out += /[A-Za-z0-9_-]/.test(character) ? character : "_";
      }
      return out;
    }

    function appendDiagramNote(figure, text) {
      if (figure.querySelector("figcaption.render-note")) {
        return;
      }
      var note = document.createElement("figcaption");
      note.className = "render-note";
      note.textContent = text;
      figure.appendChild(note);
    }

    function failDiagram(figure) {
      figure.setAttribute("data-render-error", "diagram");
      appendDiagramNote(figure, "Diagram not rendered: Mermaid rejected the source.");
    }

    function neutraliseDiagramLinks(root) {
      var links = root.querySelectorAll("a");
      for (var l = 0; l < links.length; l++) {
        try {
          links[l].removeAttribute("href");
          links[l].removeAttribute("xlink:href");
          links[l].removeAttribute("target");
        } catch (err) {
          /* A stubborn attribute must not abort the other figures. */
        }
      }
    }

    function mountDiagram(figure, svg) {
      var scroll = figure.querySelector(".diagram-scroll");
      if (!scroll || typeof svg !== "string" || svg === "") {
        failDiagram(figure);
        return;
      }
      var fragment;
      try {
        var range = document.createRange();
        range.selectNodeContents(scroll);
        fragment = range.createContextualFragment(svg);
      } catch (e) {
        failDiagram(figure);
        return;
      }
      scroll.textContent = "";
      scroll.appendChild(fragment);
      try {
        neutraliseDiagramLinks(scroll);
      } catch (e) {
        /* Neutralisation is best-effort per figure, never fatal. */
      }
    }

    function renderDiagram(figure, source, id) {
      var result;
      try {
        result = window.mermaid.render(id, source);
      } catch (e) {
        failDiagram(figure);
        return;
      }
      if (result && typeof result.then === "function") {
        result.then(
          function (rendered) {
            mountDiagram(figure, rendered ? rendered.svg : "");
          },
          function () {
            failDiagram(figure);
          }
        );
      } else {
        mountDiagram(figure, result ? result.svg : "");
      }
    }

    function ensureConstructibleStyleSheet() {
      var Native = window.CSSStyleSheet;
      if (!Native) {
        return;
      }
      try {
        new Native();
        return;
      } catch (missing) {
        /* Not constructible here: install the substitute below. */
      }
      function ConstructibleStyleSheet() {
        var inert = document.implementation.createHTMLDocument("");
        var style = inert.createElement("style");
        inert.head.appendChild(style);
        return style.sheet;
      }
      ConstructibleStyleSheet.prototype = Native.prototype;
      try {
        window.CSSStyleSheet = ConstructibleStyleSheet;
      } catch (unwritable) {
        /* An unwritable global keeps the existing failure contract. */
      }
    }

    if (!window.mermaid || typeof window.mermaid.render !== "function") {
      for (var d = 0; d < diagramFigures.length; d++) {
        diagramFigures[d].setAttribute("data-render-error", "diagram");
        appendDiagramNote(
          diagramFigures[d],
          "Diagram not rendered: the diagram script did not load."
        );
      }
    } else {
      ensureConstructibleStyleSheet();
      window.mermaid.initialize({
        startOnLoad: false,
        securityLevel: "strict",
        htmlLabels: false,
        theme: "neutral",
        fontFamily: "system-ui, sans-serif",
        flowchart: { htmlLabels: false },
        maxTextSize: 100000,
        maxEdges: 500,
      });
      var diagramPrefix =
        "papyrus-mmd-" + sanitiseDiagramId(window.location.pathname) + "-";
      for (var f = 0; f < diagramFigures.length; f++) {
        (function (figure, index) {
          if (figure.getAttribute("data-render-error")) {
            return;
          }
          var pre = figure.querySelector("pre");
          if (!pre) {
            return;
          }
          renderDiagram(figure, pre.textContent, diagramPrefix + index);
        })(diagramFigures[f], f);
      }
    }
  }

})();
