const invoke = window.__TAURI__?.core?.invoke;

const state = {
  registry: null,
  proposals: [],
  ports: [],
};

const els = {
  apps: document.querySelector("#apps"),
  message: document.querySelector("#message"),
  ports: document.querySelector("#ports"),
  refreshApps: document.querySelector("#refreshApps"),
  refreshPorts: document.querySelector("#refreshPorts"),
  registryPath: document.querySelector("#registryPath"),
  rootForm: document.querySelector("#rootForm"),
  rootPath: document.querySelector("#rootPath"),
  roots: document.querySelector("#roots"),
  scanResults: document.querySelector("#scanResults"),
};

function requireInvoke() {
  if (!invoke) {
    throw new Error("Tauri IPC is unavailable. Launch this UI through the LocalDock Tauri app.");
  }
  return invoke;
}

function setMessage(text, sticky = false) {
  els.message.textContent = text;
  if (!sticky) {
    window.setTimeout(() => {
      if (els.message.textContent === text) {
        els.message.textContent = "";
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
    setMessage(message, true);
    throw error;
  }
}

async function loadApps() {
  state.registry = await call("list_apps");
  renderRegistry();
}

async function loadPorts() {
  state.ports = await call("list_ports");
  renderPorts();
  if (state.registry) {
    renderApps();
  }
}

function renderRegistry() {
  const registry = state.registry;
  els.registryPath.textContent = registry?.registry_path || "Unavailable";
  renderRoots();
  renderApps();
}

function renderRoots() {
  clear(els.roots);
  const roots = state.registry?.allowed_roots || [];
  if (roots.length === 0) {
    els.roots.append(tag("p", "meta", "No roots yet. Add a trusted local folder first."));
    return;
  }

  for (const root of roots) {
    const chip = tag("div", "chip");
    chip.append(tag("div", "meta", root));
    const scanButton = tag("button", "", "Scan");
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
    els.apps.append(tag("p", "meta", "No registered apps yet. Scan a root and register a proposal."));
    return;
  }

  for (const app of apps) {
    const lanExposures = lanExposuresForApp(app);
    const card = tag("article", "card");
    card.append(tag("h3", "", app.name));
    card.append(tag("span", app.running ? "badge ok" : "badge", app.running ? "Running" : "Stopped"));
    if (lanExposures.length > 0) {
      card.append(tag("span", "badge danger", "LAN EXPOSED"));
      card.append(
        tag(
          "div",
          "meta warn-text",
          `Non-loopback listener: ${lanExposures
            .map((port) => `${port.addr}:${port.port}`)
            .join(", ")}`,
        ),
      );
    }
    card.append(tag("div", "meta", app.cwd));
    card.append(tag("code", "", `${app.command} ${app.args.join(" ")}`.trim()));

    const actions = tag("div", "card-actions");
    const start = tag("button", "", "Start");
    start.type = "button";
    start.disabled = app.running;
    start.addEventListener("click", () => startApp(app.id));
    actions.append(start);

    const stop = tag("button", "", "Stop");
    stop.type = "button";
    stop.disabled = !app.running;
    stop.addEventListener("click", () => stopApp(app.id));
    actions.append(stop);

    if (app.preferred_port) {
      const open = tag("button", "", `Open :${app.preferred_port}`);
      open.type = "button";
      open.addEventListener("click", () =>
        call("open_loopback", { url: `http://127.0.0.1:${app.preferred_port}` }),
      );
      actions.append(open);
    }

    card.append(actions);
    els.apps.append(card);
  }
}

function renderPorts() {
  clear(els.ports);
  if (state.ports.length === 0) {
    els.ports.append(tag("p", "meta", "No loopback listeners or LocalDock LAN exposures found."));
    return;
  }

  for (const port of state.ports) {
    const card = tag("article", "card");
    card.append(tag("h3", "", `${port.addr}:${port.port}`));
    card.append(tag("span", port.is_loopback ? "badge ok" : "badge danger", port.is_loopback ? "Loopback" : "LAN EXPOSED"));
    card.append(tag("div", "meta", `PID ${port.pid || "unknown"} ${port.process_name || ""}`.trim()));

    if (port.is_loopback) {
      const actions = tag("div", "card-actions");
      const kill = tag("button", "danger", "Kill");
      kill.type = "button";
      kill.disabled = !port.pid;
      kill.addEventListener("click", () => killPort(port.port, port.pid));
      actions.append(kill);
      card.append(actions);
    } else {
      card.append(tag("div", "meta warn-text", "This running LocalDock app is listening beyond loopback."));
    }
    els.ports.append(card);
  }
}

function lanExposuresForApp(app) {
  if (!app.running || !app.child_pid) {
    return [];
  }

  return state.ports.filter((port) => port.pid === app.child_pid && !port.is_loopback);
}

function renderProposals() {
  clear(els.scanResults);
  if (state.proposals.length === 0) {
    els.scanResults.append(tag("p", "meta", "No proposals from the last scan."));
    return;
  }

  for (const proposal of state.proposals) {
    const card = tag("article", "card");
    card.append(tag("h3", "", proposal.name));
    card.append(tag("div", "meta", proposal.cwd));
    card.append(tag("code", "", `${proposal.command} ${proposal.args.join(" ")}`.trim()));
    if (proposal.preferred_port) {
      card.append(tag("span", "badge warn", `Preferred port ${proposal.preferred_port}`));
    }

    const register = tag("button", "", "Register");
    register.type = "button";
    register.addEventListener("click", () => registerProposal(proposal));
    card.append(register);
    els.scanResults.append(card);
  }
}

async function addRoot(event) {
  event.preventDefault();
  const path = els.rootPath.value.trim();
  if (!path) {
    setMessage("Enter a root path first.");
    return;
  }
  state.registry = await call("add_root", { path });
  els.rootPath.value = "";
  renderRegistry();
  setMessage("Root added.");
}

async function scanRoot(root) {
  state.proposals = await call("scan", { root });
  renderProposals();
  setMessage(`Scan complete: ${state.proposals.length} proposal(s).`);
}

async function registerProposal(proposal) {
  state.registry = await call("register_app", {
    input: {
      ...proposal,
      force_loopback: true,
    },
  });
  renderRegistry();
  setMessage(`${proposal.name} registered.`);
}

async function startApp(id) {
  await call("start_app", { id });
  await Promise.all([loadApps(), loadPorts()]);
  setMessage("App started.");
}

async function stopApp(id) {
  await call("stop_app", { id });
  await Promise.all([loadApps(), loadPorts()]);
  setMessage("App stopped.");
}

async function killPort(port, pid) {
  const confirmed = window.confirm(`Kill PID ${pid} listening on loopback port ${port}?`);
  if (!confirmed) {
    return;
  }
  await call("kill_port", { port, pid });
  await loadPorts();
  setMessage(`Killed PID ${pid}.`);
}

async function init() {
  els.rootForm.addEventListener("submit", addRoot);
  els.refreshApps.addEventListener("click", loadApps);
  els.refreshPorts.addEventListener("click", loadPorts);

  try {
    await Promise.all([loadApps(), loadPorts()]);
  } catch {
    setMessage("LocalDock failed to initialize. See the error above.", true);
  }
}

init();
