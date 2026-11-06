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
})();
