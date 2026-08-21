/**
 * Copyright (c) 2026 Mr-Aurevo-X. All rights reserved.
 * SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
 * Void Glow titlebar — min / max / close text, Tauri window API.
 */
(function () {
  "use strict";

  function currentWindow() {
    const api = window.__TAURI__;
    if (!api) {
      return null;
    }
    if (api.webviewWindow && typeof api.webviewWindow.getCurrentWebviewWindow === "function") {
      return api.webviewWindow.getCurrentWebviewWindow();
    }
    if (api.window && typeof api.window.getCurrentWindow === "function") {
      return api.window.getCurrentWindow();
    }
    return null;
  }

  async function callWin(method) {
    try {
      const win = currentWindow();
      if (win && typeof win[method] === "function") {
        await win[method]();
      }
    } catch (_) {
      /* chrome is best-effort */
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    document.body.classList.add("frameless");
    document.getElementById("toolWinMin")?.addEventListener("click", () => callWin("minimize"));
    document.getElementById("toolWinMax")?.addEventListener("click", () => callWin("toggleMaximize"));
    document.getElementById("toolWinClose")?.addEventListener("click", () => callWin("close"));
  });
})();
