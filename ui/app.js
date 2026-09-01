const invoke = window.__TAURI__?.core?.invoke;

const state = {
  registry: null,
  proposals: [],
  hasScanned: false,
  ports: [],
  history: [],
  version: "0.1.0",
  settings: { language: "fr", checkGithubUpdates: true },
  aboutPaths: [],
  releaseUrl: "https://github.com/Mr-Aurevo-X/LocalDock/releases/latest",
};

const els = {
  aboutDialog: document.querySelector("#aboutDialog"),
  aboutLegalBody: document.querySelector("#aboutLegalBody"),
  aboutPathsList: document.querySelector("#aboutPathsList"),
  aboutCopyHint: document.querySelector("#aboutCopyHint"),
  aboutPathCopyHint: document.querySelector("#aboutPathCopyHint"),
  aboutUpdateHint: document.querySelector("#aboutUpdateHint"),
  aboutVersion: document.querySelector("#aboutVersion"),
  apps: document.querySelector("#apps"),
  btnAbout: document.querySelector("#btnAbout"),
  btnBrowseRoot: document.querySelector("#btnBrowseRoot"),
  btnImportWin: document.querySelector("#btnImportWin"),
  btnCopyRepo: document.querySelector("#btnCopyRepo"),
  btnOpenRelease: document.querySelector("#btnOpenRelease"),
  btnUpdateLater: document.querySelector("#btnUpdateLater"),
  chkGithubUpdates: document.querySelector("#chkGithubUpdates"),
  confirmDialog: document.querySelector("#confirmDialog"),
  confirmBody: document.querySelector("#confirmBody"),
  history: document.querySelector("#history"),
  kpiApps: document.querySelector("#kpiApps"),
  kpiPorts: document.querySelector("#kpiPorts"),
  kpiRoots: document.querySelector("#kpiRoots"),
  message: document.querySelector("#message"),
  ports: document.querySelector("#ports"),
  refreshApps: document.querySelector("#refreshApps"),
  refreshHistory: document.querySelector("#refreshHistory"),
  refreshPorts: document.querySelector("#refreshPorts"),
  registryPath: document.querySelector("#registryPath"),
  rootForm: document.querySelector("#rootForm"),
  rootPath: document.querySelector("#rootPath"),
  roots: document.querySelector("#roots"),
  scanResults: document.querySelector("#scanResults"),
  tabSwitch: document.querySelector("#tabSwitch"),
  updateBanner: document.querySelector("#updateBanner"),
  updateDetail: document.querySelector("#updateDetail"),
};

function requireInvoke() {
  if (!invoke) {
    throw new Error(t("ipcMissing"));
  }
  return invoke;
}

function setMessage(text, kind = "info", sticky = false) {
  els.message.textContent = text;
  els.message.classList.toggle("error", kind === "error");
  els.message.classList.toggle("ok", kind === "ok");
  if (!sticky) {
    window.setTimeout(() => {
      if (els.message.textContent === text) {
        els.message.textContent = "";
        els.message.classList.remove("error", "ok");
      }
    }, 5000);
  }
}

function clear(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

function tag(name, className, text) {
  const node = document.createElement(name);
  if (className) {
    node.className = className;
  }
  if (text !== undefined) {
    node.textContent = text;
  }
  return node;
}

async function call(command, args = {}) {
  try {
    return await requireInvoke()(command, args);
  } catch (error) {
    const message = error?.message || String(error);
    setMessage(message, "error", true);
    throw error;
  }
}

function askConfirm(body) {
  return new Promise((resolve) => {
    if (!els.confirmDialog) {
      resolve(false);
      return;
    }
    els.confirmBody.textContent = body;
    const onClose = () => {
      els.confirmDialog.removeEventListener("close", onClose);
      resolve(els.confirmDialog.returnValue === "ok");
    };
    els.confirmDialog.addEventListener("close", onClose);
    if (typeof els.confirmDialog.showModal === "function") {
      els.confirmDialog.showModal();
    } else {
      resolve(false);
    }
  });
}

async function copyText(value, hintEl) {
  try {
    await navigator.clipboard.writeText(value);
  } catch (_) {
    const input = document.createElement("textarea");
    input.value = value;
    document.body.appendChild(input);
    input.select();
    document.execCommand("copy");
    input.remove();
  }
  if (hintEl) {
    hintEl.hidden = false;
    window.setTimeout(() => {
      hintEl.hidden = true;
    }, 2000);
  }
}

async function loadApps() {
  state.registry = await call("list_apps");
  renderRegistry();
  renderHomeKpis();
}

async function loadPorts() {
  state.ports = await call("list_ports");
  renderPorts();
  renderHomeKpis();
  if (state.registry) {
    renderApps();
  }
}

async function loadHistory() {
  state.history = await call("list_history");
  renderHistory();
}

function renderHomeKpis() {
  if (els.kpiPorts) {
    els.kpiPorts.textContent = String(state.ports.length);
  }
  if (els.kpiApps) {
    els.kpiApps.textContent = String(state.registry?.apps?.length || 0);
  }
  if (els.kpiRoots) {
    els.kpiRoots.textContent = String(state.registry?.allowed_roots?.length || 0);
  }
}

function showTab(tab) {
  document.querySelectorAll("[data-tab-panel]").forEach((panel) => {
    panel.hidden = panel.getAttribute("data-tab-panel") !== tab;
  });
  document.querySelectorAll("#tabSwitch [data-tab]").forEach((btn) => {
    const active = btn.getAttribute("data-tab") === tab;
    btn.classList.toggle("is-active", active);
    btn.setAttribute("aria-selected", active ? "true" : "false");
  });
  if (tab === "history") {
    loadHistory().catch(() => {});
  }
  if (tab === "ports") {
    loadPorts().catch(() => {});
  }
}

function renderRegistry() {
  const registry = state.registry;
  els.registryPath.textContent = registry?.registry_path || "—";
  renderRoots();
  renderApps();
}

function renderRoots() {
  clear(els.roots);
  const roots = state.registry?.allowed_roots || [];
  if (roots.length === 0) {
    els.roots.append(tag("p", "meta", t("noRoots")));
    return;
  }

  for (const root of roots) {
    const chip = tag("div", "chip");
    chip.append(tag("div", "meta", root));
    const scanButton = tag("button", "accent", t("scan"));
    scanButton.type = "button";
    scanButton.addEventListener("click", () => scanRoot(root));
    chip.append(scanButton);
    els.roots.append(chip);
  }
}

function renderApps() {
  clear(els.apps);
  const apps = state.registry?.apps || [];
  if (apps.length === 0) {
    els.apps.append(tag("p", "meta", t("noApps")));
    return;
  }

  for (const app of apps) {
    const lanExposures = lanExposuresForApp(app);
    const card = tag("article", "card");
    card.append(tag("h3", "", app.name));
    const listening = appIsListening(app);
    card.append(
      tag(
        "span",
        app.running || listening ? "badge ok" : "badge",
        app.running ? t("running") : listening ? t("listening") : t("stopped"),
      ),
    );
    if (lanExposures.length > 0) {
      card.append(tag("span", "badge danger", t("lanExposed")));
      card.append(
        tag(
          "div",
          "meta warn-text",
          t("lanListener", {
            addrs: lanExposures.map((port) => `${port.addr}:${port.port}`).join(", "),
          }),
        ),
      );
    }
    card.append(tag("div", "meta", app.cwd));
    card.append(tag("code", "", `${app.command} ${app.args.join(" ")}`.trim()));

    const actions = tag("div", "card-actions");
    const start = tag("button", "accent", t("start"));
    start.type = "button";
    start.disabled = app.running;
    start.addEventListener("click", () => startApp(app.id));
    actions.append(start);

    const stop = tag("button", "", t("stop"));
    stop.type = "button";
    stop.disabled = !app.running && !listening;
    stop.addEventListener("click", () => stopApp(app.id));
    actions.append(stop);

    if (app.preferred_port) {
      const open = tag("button", "", t("openPort", { port: app.preferred_port }));
      open.type = "button";
      open.addEventListener("click", () =>
        call("open_loopback", { url: `http://127.0.0.1:${app.preferred_port}` }),
      );
      actions.append(open);
    }

    const remove = tag("button", "danger", t("remove"));
    remove.type = "button";
    remove.addEventListener("click", () => removeApp(app));
    actions.append(remove);

    card.append(actions);
    els.apps.append(card);
  }
}

function renderPorts() {
  clear(els.ports);
  if (state.ports.length === 0) {
    els.ports.append(tag("p", "meta", t("noPorts")));
    return;
  }

  for (const port of state.ports) {
    const card = tag("article", "card");
    card.append(tag("h3", "", portLabel(port)));
    card.append(
      tag(
        "span",
        port.is_loopback ? "badge ok" : "badge danger",
        port.is_loopback ? t("loopback") : t("lanExposed"),
      ),
    );
    card.append(tag("div", "meta", `${port.addr}:${port.port}`));
    if (port.cwd) {
      card.append(tag("div", "meta", port.cwd));
    }
    if (port.image_path && port.image_path !== port.cwd) {
      card.append(tag("div", "meta", port.image_path));
    }
    const command = portCommand(port);
    if (command) {
      card.append(tag("code", "", command));
    }
    const since = formatOpenSince(port.started_unix);
    if (since) {
      card.append(tag("div", "meta", since));
    }
    card.append(
      tag("div", "meta", t("pidLine", { pid: port.pid || "—", name: port.process_name || "" }).trim()),
    );

    if (port.is_loopback) {
      const actions = tag("div", "card-actions");
      const kill = tag("button", "danger", t("kill"));
      kill.type = "button";
      kill.disabled = !port.pid || port.killable === false;
      kill.addEventListener("click", () => killPort(port));
      actions.append(kill);
      card.append(actions);
    } else {
      card.append(tag("div", "meta warn-text", t("lanPortHint")));
    }
    els.ports.append(card);
  }
}

function appIsListening(app) {
  if (!app?.preferred_port) {
    return false;
  }
  return state.ports.some((port) => port.port === app.preferred_port && port.is_loopback);
}

function portLabel(port) {
  if (port.app_name) {
    return port.app_name;
  }
  if (port.process_name) {
    return port.process_name;
  }
  if (port.cwd) {
    const parts = String(port.cwd)
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .filter(Boolean);
    if (parts.length > 0) {
      return parts[parts.length - 1];
    }
  }
  return port.process_name || `${port.addr}:${port.port}`;
}

function portCommand(port) {
  return String(port.command_line || "").trim();
}

function formatOpenSince(unix) {
  const started = Number(unix);
  if (!Number.isFinite(started) || started <= 0) {
    return "";
  }
  const sec = Math.max(0, Math.floor(Date.now() / 1000 - started));
  const hours = Math.floor(sec / 3600);
  const minutes = Math.floor((sec % 3600) / 60);
  if (hours >= 24) {
    const days = Math.floor(hours / 24);
    return t("openSince", { dur: t("durDays", { n: days }) });
  }
  if (hours > 0) {
    return t("openSince", { dur: t("durHours", { n: hours, m: minutes }) });
  }
  return t("openSince", { dur: t("durMins", { n: Math.max(1, minutes) }) });
}

function renderHistory() {
  if (!els.history) {
    return;
  }
  clear(els.history);
  const events = state.history || [];
  if (events.length === 0) {
    els.history.append(tag("p", "meta", t("noHistory")));
    return;
  }
  for (const event of events) {
    const card = tag("article", "card");
    card.append(tag("h3", "", t(`hist_${event.kind}`)));
    card.append(tag("div", "meta", event.at || ""));
    if (event.detail) {
      card.append(tag("div", "meta", event.detail));
    }
    els.history.append(card);
  }
}

function lanExposuresForApp(app) {
  if (!app.running) {
    return [];
  }

  const tree =
    Array.isArray(app.tree_pids) && app.tree_pids.length > 0
      ? app.tree_pids
      : app.child_pid
        ? [app.child_pid]
        : [];
  if (tree.length === 0) {
    return [];
  }

  const pids = new Set(tree);
  return state.ports.filter((port) => pids.has(port.pid) && !port.is_loopback);
}

function renderProposals() {
  clear(els.scanResults);
  if (state.proposals.length === 0) {
    if (state.hasScanned) {
      els.scanResults.append(tag("p", "meta", t("noProposals")));
    }
    return;
  }

  for (const proposal of state.proposals) {
    const card = tag("article", "card");
    card.append(tag("h3", "", proposal.name));
    card.append(tag("div", "meta", proposal.cwd));
    card.append(tag("code", "", `${proposal.command} ${proposal.args.join(" ")}`.trim()));
    if (proposal.preferred_port) {
      card.append(tag("span", "badge warn", t("preferredPort", { port: proposal.preferred_port })));
    }

    const register = tag("button", "accent", t("register"));
    register.type = "button";
    register.addEventListener("click", () => registerProposal(proposal));
    card.append(register);
    els.scanResults.append(card);
  }
}

function pathMeta(id) {
  if (id === "app") {
    return { label: t("aboutPathApp"), hint: t("aboutPathAppHint") };
  }
  if (id === "registry") {
    return { label: t("aboutPathRegistry"), hint: t("aboutPathRegistryHint") };
  }
  if (id === "history") {
    return { label: t("aboutPathHistory"), hint: t("aboutPathHistoryHint") };
  }
  return { label: t("aboutPathSettings"), hint: t("aboutPathSettingsHint") };
}

function renderAboutPaths() {
  if (!els.aboutPathsList) {
    return;
  }
  clear(els.aboutPathsList);
  for (const entry of state.aboutPaths) {
    const meta = pathMeta(entry.id);
    const item = tag("div", "about-path-item");
    item.append(tag("div", "about-path-label", meta.label));
    const row = tag("div", "about-repo-row");
    const input = document.createElement("input");
    input.className = "about-repo-input";
    input.readOnly = true;
    input.value = entry.path;
    row.append(input);
    const copy = tag("button", "btn accent", t("aboutCopy"));
    copy.type = "button";
    copy.addEventListener("click", () => copyText(entry.path, els.aboutPathCopyHint));
    row.append(copy);
    item.append(row);
    item.append(tag("p", "about-note", meta.hint));
    els.aboutPathsList.append(item);
  }
}

function syncAboutHints() {
  if (els.aboutVersion) {
    els.aboutVersion.textContent = t("aboutVersion", { ver: state.version });
  }
  if (els.aboutUpdateHint) {
    els.aboutUpdateHint.textContent = els.chkGithubUpdates?.checked ? t("aboutHintOn") : t("aboutHintOff");
  }
  renderAboutPaths();
}

async function loadLegal(doc) {
  const file = `legal/${doc}.${currentLang}.md`;
  try {
    const res = await fetch(file, { cache: "no-store" });
    let text = res.ok ? await res.text() : t("aboutLegalLoadFail", { file });
    text = text.replace(/\{\{PRODUCT\}\}/g, "LocalDock");
    els.aboutLegalBody.hidden = false;
    els.aboutLegalBody.textContent = text;
  } catch (error) {
    els.aboutLegalBody.hidden = false;
    els.aboutLegalBody.textContent = error?.message || String(error);
  }
}

async function openAbout() {
  if (!els.aboutDialog) {
    return;
  }
  syncAboutHints();
  if (typeof els.aboutDialog.showModal === "function") {
    els.aboutDialog.showModal();
  } else {
    els.aboutDialog.setAttribute("open", "");
  }
}

async function addRoot(event) {
  event.preventDefault();
  const path = els.rootPath.value.trim();
  if (!path) {
    setMessage(t("enterRoot"), "error");
    return;
  }
  state.registry = await call("add_root", { path });
  els.rootPath.value = "";
  renderRegistry();
  renderHomeKpis();
  setMessage(t("rootAdded"), "ok");
}

async function browseRoot() {
  try {
    const path = await call("pick_folder");
    if (!path) {
      return;
    }
    els.rootPath.value = path;
  } catch (_) {
    /* call() already surfaced the error */
  }
}

async function importWindowsLocals() {
  const result = await call("import_windows_locals");
  state.registry = result.snapshot;
  renderRegistry();
  renderHomeKpis();
  setMessage(
    t("importedWin", {
      roots: result.roots_added,
      apps: result.apps_added,
    }),
    "ok",
  );
}

async function scanRoot(root) {
  state.proposals = await call("scan", { root });
  state.hasScanned = true;
  renderProposals();
  setMessage(t("scanDone", { n: state.proposals.length }), "ok");
}

async function registerProposal(proposal) {
  state.registry = await call("register_app", {
    input: {
      ...proposal,
      force_loopback: true,
    },
  });
  renderRegistry();
  setMessage(t("registered", { name: proposal.name }), "ok");
}

async function startApp(id) {
  await call("start_app", { id });
  await Promise.all([loadApps(), loadPorts()]);
  setMessage(t("appStarted"), "ok");
}

async function stopApp(id) {
  await call("stop_app", { id });
  await Promise.all([loadApps(), loadPorts()]);
  setMessage(t("appStopped"), "ok");
}

async function removeApp(app) {
  const confirmed = await askConfirm(t("confirmRemove", { name: app.name }));
  if (!confirmed) {
    return;
  }
  state.registry = await call("remove_app", { id: app.id });
  renderRegistry();
  renderHomeKpis();
  setMessage(t("appRemoved", { name: app.name }), "ok");
}

async function killPort(port) {
  const confirmed = await askConfirm(
    t("confirmKill", {
      pid: port.pid,
      port: port.port,
      label: portLabel(port),
    }),
  );
  if (!confirmed) {
    return;
  }
  await call("kill_port", { port: port.port, pid: port.pid });
  await loadPorts();
  setMessage(t("killedPid", { pid: port.pid }), "ok");
}

async function openSupport(kind) {
  await call("open_support", { kind });
  setMessage(t("supportOpened", { kind }));
}

async function applyLanguage(lang, persist) {
  applyDom(lang);
  syncAboutHints();
  renderRegistry();
  renderPorts();
  renderProposals();
  renderHomeKpis();
  renderHistory();
  if (persist) {
    state.settings = await call("set_suite_language", { lang });
  }
}

async function checkUpdates() {
  if (!state.settings.checkGithubUpdates) {
    els.updateBanner.hidden = true;
    return;
  }
  try {
    const info = await requireInvoke()("check_github_latest");
    if (info?.htmlUrl) {
      state.releaseUrl = info.htmlUrl;
    }
    if (info?.newer && info.remote) {
      els.updateDetail.textContent = t("releaseMsg", { ver: info.remote });
      els.updateBanner.hidden = false;
    } else {
      els.updateBanner.hidden = true;
    }
  } catch (_) {
    els.updateBanner.hidden = true;
  }
}

function wireUi() {
  document.querySelectorAll("[data-support]").forEach((btn) => {
    btn.addEventListener("click", (event) => {
      event.preventDefault();
      const kind = btn.getAttribute("data-support");
      if (kind) {
        openSupport(kind).catch(() => {});
      }
    });
  });

  document.querySelectorAll(".legal-tab, [data-legal]").forEach((tab) => {
    tab.addEventListener("click", () => {
      const doc = tab.getAttribute("data-legal");
      if (doc) {
        loadLegal(doc).catch(() => {});
      }
    });
  });

  document.getElementById("langSwitch")?.addEventListener("click", (event) => {
    const seg = event.target.closest("[data-lang]");
    if (!seg) {
      return;
    }
    applyLanguage(seg.getAttribute("data-lang"), true).catch(() => {});
  });

  els.btnAbout?.addEventListener("click", openAbout);
  els.btnCopyRepo?.addEventListener("click", () => {
    const url = document.getElementById("aboutRepoUrl")?.value || "";
    copyText(url, els.aboutCopyHint);
  });
  els.chkGithubUpdates?.addEventListener("change", async () => {
    const enabled = Boolean(els.chkGithubUpdates.checked);
    state.settings = await call("set_check_github_updates", { enabled });
    syncAboutHints();
    await checkUpdates();
  });
  els.btnOpenRelease?.addEventListener("click", () => {
    call("open_release", { url: state.releaseUrl }).catch(() => {});
  });
  els.btnUpdateLater?.addEventListener("click", () => {
    els.updateBanner.hidden = true;
  });
  els.tabSwitch?.addEventListener("click", (event) => {
    const btn = event.target.closest("[data-tab]");
    if (btn) {
      showTab(btn.getAttribute("data-tab"));
    }
  });
  els.btnBrowseRoot?.addEventListener("click", browseRoot);
  els.btnImportWin?.addEventListener("click", () => {
    importWindowsLocals().catch(() => {});
  });
  els.refreshHistory?.addEventListener("click", () => {
    loadHistory().catch(() => {});
  });
}

async function init() {
  els.rootForm.addEventListener("submit", addRoot);
  els.refreshApps.addEventListener("click", loadApps);
  els.refreshPorts.addEventListener("click", loadPorts);
  wireUi();

  try {
    const [settings, version, paths] = await Promise.all([
      call("get_suite_settings"),
      call("get_app_version"),
      call("get_about_local_paths"),
    ]);
    state.settings = settings || state.settings;
    state.version = version || state.version;
    state.aboutPaths = paths || [];
    if (els.chkGithubUpdates) {
      els.chkGithubUpdates.checked = state.settings.checkGithubUpdates !== false;
    }
    applyDom(state.settings.language || "fr");
    syncAboutHints();
    await Promise.all([loadApps(), loadPorts(), loadHistory()]);
    await checkUpdates();
  } catch {
    applyDom("fr");
    setMessage(t("bootError"), "error", true);
  }
}

init();
