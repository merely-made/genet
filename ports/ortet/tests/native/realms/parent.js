// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// The G5 realms headed sequence, driven by one native click.
//
// A parent document and a same-origin child iframe. The parent's script builds
// associated state on the child's subtree (an open shadow root holding a nested
// closed one, plus a parser-created template whose contents live in the child's
// inert template-contents owner), adopts that subtree into the parent, sends
// the parent's own link the other way, navigates the child twice, brings the
// subtree back into the child, keeps one link in the parent, and finally
// navigates the top level.
//
// Every stage is a JS-observable assertion. A failed assertion sets its own
// distinct, non-matching heading rather than approximating success, so the host
// fails on its bounded deadline with the failure written into the page. The
// three things the host correlates with this sequence — the presented frame,
// the published accessibility projection, and the live-node census — are all
// read outside the script, which cannot see any of them.

var state = document.getElementById("state");
var frame = document.getElementById("frame");
var activate = document.getElementById("activate");

// Everything the sequence holds across its timer turns. Cleared wholesale at
// the release stage so the arena has nothing of ours left to keep.
var held = {};

function fail(where) {
  state.textContent = "Ortet G5 realms failed: " + where;
  return false;
}

// Every stage runs inside its own timer turn, so the host's frame-cadence GC
// tick runs between stages instead of coalescing them into one batch. An
// exception thrown inside a timer callback would otherwise end the sequence
// silently — there is no further timer to notice the absence — so each stage
// reports its own throw as a failure heading.
function step(fn) {
  setTimeout(function () {
    try {
      fn();
    } catch (error) {
      fail("stage threw: " + error);
    }
  }, 40);
}

/// Wait a fixed, generous span and then hand over. A child navigation is
/// asynchronous and the host owns the fetch; the stage that follows asserts
/// that the new document actually arrived, so a wait that was too short fails
/// loudly instead of being retried into silence.
function after(delay, fn) {
  setTimeout(function () {
    try {
      fn();
    } catch (error) {
      fail("stage threw: " + error);
    }
  }, delay);
}

function childDocument() {
  return frame.contentDocument;
}

function childMark() {
  var doc = childDocument();
  if (!doc) return null;
  var mark = doc.getElementById("mark");
  return mark ? mark.textContent : null;
}

activate.addEventListener("click", function () {
  if (document.body.dataset.inlineReady !== "yes") {
    fail("the inline parser-time mutation was not observed");
    return;
  }
  after(400, buildAssociatedState);
});

// 1. Build associated state on the child's subtree, from the parent's realm.
//    Same-origin: the parent may reach into the child's arena, and an
//    `attachShadow` from here must attach in the child's document.
function buildAssociatedState() {
  if (childMark() !== "Child A") {
    fail("the first child document did not arrive");
    return;
  }
  var childDoc = childDocument();
  var payload = childDoc.getElementById("payload");
  var host = childDoc.getElementById("host");
  var outer = childDoc.getElementById("outer");
  var kept = childDoc.getElementById("kept");
  if (!payload || !host || !outer || !kept) {
    fail("the child document did not carry the payload subtree");
    return;
  }

  var openRoot;
  try {
    openRoot = host.attachShadow({ mode: "open" });
  } catch (error) {
    fail("attachShadow across the frame boundary threw: " + error);
    return;
  }
  var slot = childDoc.createElement("span");
  slot.id = "shadow-text";
  slot.textContent = "open shadow content";
  openRoot.appendChild(slot);
  var innerHost = childDoc.createElement("div");
  innerHost.id = "inner-host";
  openRoot.appendChild(innerHost);
  var closedRoot = innerHost.attachShadow({ mode: "closed" });
  closedRoot.textContent = "closed shadow content";

  if (host.shadowRoot !== openRoot) return fail("the open root is not the host's shadowRoot");
  if (innerHost.shadowRoot !== null) return fail("a closed root was reported through shadowRoot");
  if (closedRoot.textContent !== "closed shadow content") return fail("the closed root lost its contents");
  if (slot.ownerDocument !== childDoc) return fail("shadow content was not homed in the child document");

  // A parser-created template's contents belong to the *child's* inert
  // template-contents owner document, which is neither document.
  var contents = outer.content;
  if (!contents) return fail("the template exposed no contents fragment");
  var sourceInert = contents.ownerDocument;
  if (sourceInert === childDoc) return fail("template contents were homed on the child document itself");
  if (sourceInert === document) return fail("template contents were homed on the parent before any adoption");
  var slip = contents.firstChild;
  var inner = contents.lastChild;
  if (!slip || slip.textContent !== "template contents") return fail("the template lost its contents");
  if (!inner || !inner.content || inner.content.firstChild.textContent !== "nested contents") {
    return fail("the nested template lost its contents");
  }

  // A detached adopted component survives the source registration and is later
  // released while its destination document is still live. Its collection must
  // come from GC, independently of the documents destroyed by navigation.
  var retiredTree = childDoc.createElement("div");
  var retiredLeaf = childDoc.createElement("span");
  retiredLeaf.textContent = "detached adoption collection";
  retiredTree.appendChild(retiredLeaf);
  document.adoptNode(retiredTree);

  held = {
    childDoc: childDoc,
    payload: payload,
    host: host,
    openRoot: openRoot,
    innerHost: innerHost,
    closedRoot: closedRoot,
    slot: slot,
    outer: outer,
    inner: inner,
    slip: slip,
    contents: contents,
    nestedContents: inner.content,
    hostPrototype: Object.getPrototypeOf(host),
    rootPrototype: Object.getPrototypeOf(openRoot),
    retiredTree: retiredTree,
    retiredLeaf: retiredLeaf,
    kept: kept,
    sourceInert: sourceInert,
    away: document.getElementById("away")
  };
  state.textContent = "Ortet G5 realms associated state built";
  step(adoptIntoParent);
}

// 2. Adopt the whole subtree from the child into the parent.
function adoptIntoParent() {
  var landing = document.getElementById("landing");
  try {
    landing.appendChild(held.payload);
  } catch (error) {
    return fail("adopting the subtree into the parent threw: " + error);
  }

  if (held.payload.parentNode !== landing) return fail("the subtree did not arrive in the parent");
  if (held.payload.ownerDocument !== document) return fail("the subtree's ownerDocument did not follow");
  if (held.host.ownerDocument !== document) return fail("the shadow host's ownerDocument did not follow");
  if (held.slot.ownerDocument !== document) return fail("shadow content's ownerDocument did not follow");
  if (held.kept.ownerDocument !== document) return fail("the kept link's ownerDocument did not follow");
  if (document.getElementById("kept") !== held.kept) return fail("the kept link is a copy, not the same node");
  if (held.host.shadowRoot !== held.openRoot) return fail("the open shadow root did not travel with its host");
  if (held.innerHost.shadowRoot !== null) return fail("the closed root opened on the far side");
  if (held.closedRoot.textContent !== "closed shadow content") return fail("the closed root lost its contents on adoption");
  if (held.slot.textContent !== "open shadow content") return fail("the open root lost its contents on adoption");

  var contents = held.outer.content;
  var destinationInert = contents.ownerDocument;
  if (destinationInert === held.sourceInert) return fail("template contents kept the source's inert owner");
  if (destinationInert === document) return fail("template contents were homed on the destination document itself");
  if (contents.firstChild !== held.slip) return fail("template contents were rebuilt rather than re-homed");
  if (held.slip.ownerDocument !== destinationInert) return fail("a template content node was not re-homed");
  if (held.inner.content.firstChild.textContent !== "nested contents") return fail("the nested template lost its contents on adoption");
  if (held.inner.content.ownerDocument !== destinationInert) return fail("the nested template was not re-homed recursively");
  held.parentInert = destinationInert;

  state.textContent = "Ortet G5 realms subtree adopted into the parent";
  step(adoptAwayIntoChild);
}

// 3. Send the parent's own link the other way, into the child document that is
//    about to be navigated away from.
function adoptAwayIntoChild() {
  var childDoc = held.childDoc;
  try {
    childDoc.body.appendChild(held.away);
  } catch (error) {
    return fail("adopting the parent's link into the child threw: " + error);
  }
  if (held.away.ownerDocument !== childDoc) return fail("the away link's ownerDocument did not follow into the child");
  if (document.getElementById("away")) return fail("the away link is still in the parent tree");

  state.textContent = "Ortet G5 realms link adopted into the child";
  step(navigateChildOnce);
}

// 4. First child navigation. The document holding the away link is discarded
//    with it.
function navigateChildOnce() {
  try {
    frame.src = "child-b.html";
  } catch (error) {
    return fail("navigating the child threw: " + error);
  }
  after(600, function () {
    if (childMark() !== "Child B") return fail("the second child document did not arrive");
    // The away link went down with the document it was adopted into. Reading
    // it is allowed to throw — its arena is gone — but it must never answer
    // that it is back in the parent, and the parent's tree must not hold it.
    var stillHere = false;
    try {
      stillHere = held.away.ownerDocument === document;
    } catch (error) {
      stillHere = false;
    }
    if (stillHere) return fail("the away link came back to the parent");
    if (document.getElementById("away")) return fail("the away link is back in the parent tree");
    if (document.getElementById("kept") !== held.kept) return fail("discard changed ordinary wrapper identity");
    if (held.host.shadowRoot !== held.openRoot) return fail("discard changed shadow wrapper identity");
    if (held.outer.content !== held.contents || held.inner.content !== held.nestedContents) return fail("discard changed template wrapper identity");
    if (held.retiredTree.firstChild !== held.retiredLeaf || held.retiredLeaf.ownerDocument !== document) return fail("discard changed detached wrapper identity");
    held.away = null;
    state.textContent = "Ortet G5 realms child navigated once";
    step(adoptBackIntoChild);
  });
}

// 5. Adopt the subtree back into the child — the return half of the round trip,
//    into a document that did not exist when the subtree left.
function adoptBackIntoChild() {
  var childDoc = childDocument();
  var dock = childDoc.getElementById("dock");
  if (!dock) return fail("the second child document has no dock");
  try {
    dock.appendChild(held.payload);
  } catch (error) {
    return fail("adopting the subtree back into the child threw: " + error);
  }
  if (held.payload.ownerDocument !== childDoc) return fail("the subtree's ownerDocument did not follow back");
  if (held.host.ownerDocument !== childDoc) return fail("the shadow host did not follow back");
  var returned = held.host.shadowRoot;
  if (returned !== held.openRoot) return fail("the return hop changed shadow wrapper identity");
  if (Object.getPrototypeOf(returned) !== held.rootPrototype || Object.getPrototypeOf(held.host) !== held.hostPrototype) return fail("the return hop changed creation prototypes");
  if (returned.childNodes.length !== 2) return fail("the returned shadow root lost children");
  if (returned.firstChild.id !== "shadow-text") return fail("the returned shadow root lost its first child");
  if (returned.firstChild.textContent !== "open shadow content") return fail("shadow content lost its text on the return hop");
  if (held.slot.parentNode !== returned) return fail("shadow content is not parented by the returned root");
  if (held.slot.ownerDocument !== childDoc) return fail("shadow content's ownerDocument did not follow back");
  if (returned.firstChild !== held.slot || returned.lastChild !== held.innerHost) return fail("the return hop changed shadow child identity");
  if (held.closedRoot.firstChild.getRootNode() !== held.closedRoot) return fail("the return hop changed closed root identity");
  if (held.innerHost.shadowRoot !== null) return fail("the closed root opened on the return hop");
  if (held.closedRoot.textContent !== "closed shadow content") return fail("the closed root lost its contents on the return hop");
  if (held.outer.content.ownerDocument === held.parentInert) return fail("template contents kept the parent's inert owner");
  if (held.outer.content !== held.contents || held.inner.content !== held.nestedContents || held.contents.firstChild !== held.slip) return fail("the return hop changed template identity");
  if (held.outer.content.firstChild.textContent !== "template contents") return fail("template contents were rebuilt on the return hop");

  state.textContent = "Ortet G5 realms subtree adopted back into the child";
  step(keepOneLinkInTheParent);
}

// 6. One link leaves the subtree and stays in the parent. It is the node the
//    accessibility receipt looks for; everything else goes down with the child.
function keepOneLinkInTheParent() {
  var landing = document.getElementById("landing");
  try {
    landing.appendChild(held.kept);
  } catch (error) {
    return fail("keeping the adopted link in the parent threw: " + error);
  }
  if (held.kept.ownerDocument !== document) return fail("the kept link's ownerDocument did not follow home");
  var home = document.getElementById("kept");
  if (home !== held.kept) return fail("the kept link changed identity on its return home");
  if (home.textContent !== "Adopted link kept in the parent") return fail("the kept link lost its text");
  if (held.kept.parentNode !== landing) return fail("the held kept link is not where the parent put it");

  state.textContent = "Ortet G5 realms link kept in the parent";
  step(navigateChildTwice);
}

// 7. Second child navigation, discarding the subtree with the document that
//    held it. Only the kept link survives, in the parent.
function navigateChildTwice() {
  try {
    frame.src = "child-c.html";
  } catch (error) {
    return fail("navigating the child a second time threw: " + error);
  }
  after(600, function () {
    if (childMark() !== "Child C") return fail("the third child document did not arrive");
    var survivor = document.getElementById("kept");
    if (!survivor || survivor.textContent !== "Adopted link kept in the parent") {
      return fail("the kept link went down with the child");
    }
    state.textContent = "Ortet G5 realms child navigated twice";
    step(release);
  });
}

// 8. Release everything the sequence held and let several frame-cadence GC
//    ticks run before anything is read back.
function release() {
  if (held.retiredTree.firstChild !== held.retiredLeaf || held.retiredTree.isConnected) return fail("the retained detached component changed before release");
  var kept = held.kept;
  held = {};
  kept = null;
  step(function () {
    step(function () {
      setTimeout(function () {
        // The heading under which the host publishes the projection this
        // receipt reads: the kept link present with its Click action, the away
        // link gone with the child document it was adopted into.
        state.textContent = "Ortet G5 realms adoption settled";
        setTimeout(navigateTopLevel, 260);
      }, 240);
    });
  });
}

// 9. The top-level navigation. Reaching parent2.js at all is the proof; the
//    completion heading this host waits for is set there, not here.
function navigateTopLevel() {
  location.href = location.href.replace("parent.html", "parent2.html");
}
