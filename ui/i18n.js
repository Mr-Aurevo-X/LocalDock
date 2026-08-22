/**
 * Copyright (c) 2026 Mr-Aurevo-X. All rights reserved.
 * SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
 */
const DICT = {
  fr: {
    langSwitchAria: "Langue",
    eyebrow: "Loopback only",
    lede: "Enregistre tes projets locaux, lance-les en loopback, inspecte les ports d’écoute — sans portail HTTP.",
    registryLabel: "Registre",
    registryHint:
      "Fichier local à ce PC et à ce compte Windows — pas partagé si tu installes ailleurs.",
    tabsAria: "Sections",
    tabHome: "Accueil",
    tabPorts: "Ports ouverts",
    tabHistory: "Historique",
    kpiPorts: "Ports ouverts",
    kpiApps: "Apps enregistrées",
    kpiRoots: "Racines de confiance",
    browse: "Parcourir",
    rootsTitle: "Racines + Scan",
    rootsHelp: "Ajoute des dossiers de confiance, puis scanne les apps de dev.",
    rootPlaceholder: "C:\\Projects ou /home/me/projects",
    addRoot: "Ajouter",
    importWin: "Importer Windows",
    importedWin: "Windows monté : {roots} racine(s), {apps} app(s).",
    hist_import_win: "Import Windows",
    appsTitle: "Apps",
    appsHelp: "Démarre et arrête les apps enregistrées via IPC local.",
    portsTitle: "Ports",
    portsHelp: "Écoutes loopback, plus alertes d’exposition LAN pour les apps LocalDock.",
    refresh: "Actualiser",
    privacy:
      "100 % local-first. Seule connexion hors machine : vérif. version GitHub (si activée dans À propos).",
    supportAria: "Soutien optionnel",
    supportNote: "Si le boulot te plaît, un café — sinon profite.",
    aboutBtn: "À propos",
    aboutTitle: "À propos — LocalDock",
    aboutIntro:
      "Lanceur local-only pour serveurs de dev en loopback (Mr-Aurevo-X). Gratuit, sans compte. Pas de portail HTTP.",
    aboutLegalLocal: "100 % local-first — pas de télémétrie",
    aboutLegalGh: "Seule connexion hors machine : vérif. version GitHub (option ci-dessous)",
    aboutLegalOff: "Si vérif. désactivée : zéro réseau hors actions utilisateur",
    aboutToggle: "Vérifier les nouvelles versions sur GitHub",
    aboutHintOn: "Quand activé : un appel API GitHub au démarrage (lecture seule, pas de téléchargement).",
    aboutHintOff: "Désactivé : aucune requête GitHub. Local-first strict hors actions utilisateur.",
    aboutRepoLabel: "Repo GitHub (releases)",
    aboutCopy: "Copier",
    aboutCopiedLink: "Lien copié.",
    aboutCopiedPath: "Chemin copié.",
    aboutPathsTitle: "Chemins locaux (désinstall / ménage)",
    aboutPathsIntro:
      "Identifie clairement quoi supprimer. Les préférences Mr-Aurevo-X sont partagées entre apps.",
    aboutPathsAria: "Chemins locaux",
    aboutLegalAria: "Documents légaux",
    aboutLegalTerms: "CGU",
    aboutLegalPrivacy: "Confidentialité",
    aboutLegalMentions: "Mentions",
    aboutLegalNotices: "Notices",
    aboutCopyright: "Copyright © 2026 Mr-Aurevo-X — tous droits réservés",
    aboutRedistrib:
      "Redistribution, reverse engineering ou suppression du copyright interdits sans accord écrit.",
    aboutClose: "Fermer",
    aboutVersion: "Version {ver}",
    aboutPathApp: "Install (dossier de l’exe)",
    aboutPathAppHint: "Dossier réel de l’exe lancé — à supprimer pour désinstaller.",
    aboutPathRegistry: "Registre LocalDock",
    aboutPathRegistryHint: "apps.json — liste des projets et racines autorisées (propre à ce PC).",
    aboutPathHistory: "Historique LocalDock",
    aboutPathHistoryHint: "history.json — journal local des scans / start / stop sur ce PC.",
    aboutPathSettings: "Préférences (langue, vérif. maj)",
    aboutPathSettingsHint: "Fichier partagé Mr-Aurevo-X — à garder si d’autres apps l’utilisent.",
    aboutLegalLoadFail: "Impossible de charger {file}",
    updateTitle: "Nouvelle version disponible",
    btnOpenRelease: "Ouvrir sur GitHub",
    btnLater: "Plus tard",
    releaseMsg: "Nouvelle version {ver} disponible",
    confirmTitle: "Confirmer",
    confirmOk: "Confirmer",
    confirmCancel: "Annuler",
    confirmKill: "Arrêter {label} (PID {pid}) sur le port {port} ?",
    confirmRemove: "Retirer {name} du registre LocalDock ?",
    noRoots: "Pas encore de racine. Ajoute un dossier local de confiance.",
    noApps: "Pas encore d’app. Scanne une racine et enregistre une proposition.",
    noPorts: "Aucune écoute loopback ni exposition LAN LocalDock.",
    noHistory: "Pas encore d’historique sur ce PC.",
    historyTitle: "Historique",
    historyHelp: "Actions locales sur ce PC (scan, enregistrement, start/stop, kill).",
    openSince: "Ouvert depuis {dur}",
    durDays: "{n} j",
    durHours: "{n} h {m} min",
    durMins: "{n} min",
    hist_add_root: "Racine ajoutée",
    hist_scan: "Scan",
    hist_register: "App enregistrée",
    hist_start: "Démarrage",
    hist_stop: "Arrêt",
    hist_kill: "Port tué",
    hist_unregister: "App retirée",
    noProposals: "Aucune proposition depuis le dernier scan.",
    scan: "Scanner",
    start: "Démarrer",
    stop: "Arrêter",
    remove: "Supprimer",
    openPort: "Ouvrir :{port}",
    register: "Enregistrer",
    running: "En cours",
    listening: "En écoute",
    stopped: "Arrêté",
    loopback: "Loopback",
    lanExposed: "LAN EXPOSÉ",
    lanListener: "Écoute hors loopback : {addrs}",
    lanPortHint: "Cette app LocalDock écoute au-delà du loopback.",
    kill: "Tuer",
    pidLine: "PID {pid} {name}",
    preferredPort: "Port préféré {port}",
    enterRoot: "Indique d’abord un chemin racine.",
    rootAdded: "Racine ajoutée.",
    scanDone: "Scan terminé : {n} proposition(s).",
    registered: "{name} enregistrée.",
    appStarted: "App démarrée.",
    appStopped: "App arrêtée.",
    appRemoved: "{name} retirée du registre.",
    killedPid: "PID {pid} arrêté.",
    supportOpened: "Ouverture {kind} (don / contact volontaire — pas un prix de licence).",
    bootError: "LocalDock n’a pas pu démarrer. Voir le message ci-dessus.",
    ipcMissing: "IPC Tauri indisponible. Lance cette UI via l’app LocalDock.",
  },
  en: {
    langSwitchAria: "Language",
    eyebrow: "Loopback only",
    lede: "Register local projects, start them on loopback, inspect listening ports — no HTTP portal.",
    registryLabel: "Registry",
    registryHint: "Local to this PC and this Windows account — not shared if you install elsewhere.",
    tabsAria: "Sections",
    tabHome: "Home",
    tabPorts: "Open ports",
    tabHistory: "History",
    kpiPorts: "Open ports",
    kpiApps: "Registered apps",
    kpiRoots: "Trusted roots",
    browse: "Browse",
    rootsTitle: "Roots + Scan",
    rootsHelp: "Add trusted folders, then scan them for dev apps.",
    rootPlaceholder: "C:\\Projects or /home/me/projects",
    addRoot: "Add root",
    importWin: "Import Windows",
    importedWin: "Mounted Windows: {roots} root(s), {apps} app(s).",
    hist_import_win: "Windows import",
    appsTitle: "Apps",
    appsHelp: "Start and stop registered apps through local IPC.",
    portsTitle: "Ports",
    portsHelp: "Loopback listeners, plus LAN exposure warnings for running LocalDock apps.",
    refresh: "Refresh",
    privacy:
      "100% local-first. Only off-machine call: GitHub version check (if enabled in About).",
    supportAria: "Optional support",
    supportNote: "If you like the work, a coffee — otherwise just use it.",
    aboutBtn: "About",
    aboutTitle: "About — LocalDock",
    aboutIntro:
      "Local-only launcher for loopback dev servers (Mr-Aurevo-X). Free, no account. No HTTP portal.",
    aboutLegalLocal: "100% local-first — no telemetry",
    aboutLegalGh: "Only off-machine call: GitHub version check (option below)",
    aboutLegalOff: "If the check is off: zero network except user actions",
    aboutToggle: "Check for new versions on GitHub",
    aboutHintOn: "When on: one GitHub API call at startup (read-only, no download).",
    aboutHintOff: "Off: no GitHub request. Strict local-first except user actions.",
    aboutRepoLabel: "GitHub repo (releases)",
    aboutCopy: "Copy",
    aboutCopiedLink: "Link copied.",
    aboutCopiedPath: "Path copied.",
    aboutPathsTitle: "Local paths (uninstall / cleanup)",
    aboutPathsIntro:
      "Shows clearly what to delete. Mr-Aurevo-X preferences are shared across apps.",
    aboutPathsAria: "Local paths",
    aboutLegalAria: "Legal documents",
    aboutLegalTerms: "Terms",
    aboutLegalPrivacy: "Privacy",
    aboutLegalMentions: "Notices",
    aboutLegalNotices: "Licenses",
    aboutCopyright: "Copyright © 2026 Mr-Aurevo-X — all rights reserved",
    aboutRedistrib:
      "Redistribution, reverse engineering, or removing copyright notices requires written permission.",
    aboutClose: "Close",
    aboutVersion: "Version {ver}",
    aboutPathApp: "Install (exe folder)",
    aboutPathAppHint: "Real folder of the launched exe — delete it to uninstall.",
    aboutPathRegistry: "LocalDock registry",
    aboutPathRegistryHint: "apps.json — registered projects and allowed roots (this PC only).",
    aboutPathHistory: "LocalDock history",
    aboutPathHistoryHint: "history.json — local log of scans / start / stop on this PC.",
    aboutPathSettings: "Preferences (language, update check)",
    aboutPathSettingsHint: "Shared Mr-Aurevo-X file — keep it if other apps still use it.",
    aboutLegalLoadFail: "Could not load {file}",
    updateTitle: "New version available",
    btnOpenRelease: "Open on GitHub",
    btnLater: "Later",
    releaseMsg: "New version {ver} is available",
    confirmTitle: "Confirm",
    confirmOk: "Confirm",
    confirmCancel: "Cancel",
    confirmKill: "Stop {label} (PID {pid}) on port {port}?",
    confirmRemove: "Remove {name} from the LocalDock registry?",
    noRoots: "No roots yet. Add a trusted local folder first.",
    noApps: "No registered apps yet. Scan a root and register a proposal.",
    noPorts: "No loopback listeners or LocalDock LAN exposures found.",
    noHistory: "No history on this PC yet.",
    historyTitle: "History",
    historyHelp: "Local actions on this PC (scan, register, start/stop, kill).",
    openSince: "Open for {dur}",
    durDays: "{n} d",
    durHours: "{n} h {m} min",
    durMins: "{n} min",
    hist_add_root: "Root added",
    hist_scan: "Scan",
    hist_register: "App registered",
    hist_start: "Started",
    hist_stop: "Stopped",
    hist_kill: "Port killed",
    hist_unregister: "App removed",
    noProposals: "No proposals from the last scan.",
    scan: "Scan",
    start: "Start",
    stop: "Stop",
    remove: "Remove",
    openPort: "Open :{port}",
    register: "Register",
    running: "Running",
    listening: "Listening",
    stopped: "Stopped",
    loopback: "Loopback",
    lanExposed: "LAN EXPOSED",
    lanListener: "Non-loopback listener: {addrs}",
    lanPortHint: "This running LocalDock app is listening beyond loopback.",
    kill: "Kill",
    pidLine: "PID {pid} {name}",
    preferredPort: "Preferred port {port}",
    enterRoot: "Enter a root path first.",
    rootAdded: "Root added.",
    scanDone: "Scan complete: {n} proposal(s).",
    registered: "{name} registered.",
    appStarted: "App started.",
    appStopped: "App stopped.",
    appRemoved: "{name} removed from the registry.",
    killedPid: "Killed PID {pid}.",
    supportOpened: "Opening {kind} (voluntary donation / contact — not a license fee).",
    bootError: "LocalDock failed to initialize. See the error above.",
    ipcMissing: "Tauri IPC is unavailable. Launch this UI through the LocalDock Tauri app.",
  },
};

let currentLang = "fr";

function t(key, vars) {
  const pack = DICT[currentLang] || DICT.fr;
  let text = pack[key] || DICT.fr[key] || key;
  if (vars) {
    Object.keys(vars).forEach((name) => {
      text = text.replaceAll(`{${name}}`, String(vars[name]));
    });
  }
  return text;
}

function applyDom(lang) {
  currentLang = lang === "en" ? "en" : "fr";
  document.documentElement.lang = currentLang;
  document.querySelectorAll("[data-i18n]").forEach((node) => {
    const key = node.getAttribute("data-i18n");
    if (key) {
      node.textContent = t(key);
    }
  });
  document.querySelectorAll("[data-i18n-placeholder]").forEach((node) => {
    const key = node.getAttribute("data-i18n-placeholder");
    if (key) {
      node.setAttribute("placeholder", t(key));
    }
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((node) => {
    const key = node.getAttribute("data-i18n-aria");
    if (key) {
      node.setAttribute("aria-label", t(key));
    }
  });
  document.querySelectorAll("#langSwitch .hub-lang-seg").forEach((seg) => {
    const active = seg.getAttribute("data-lang") === currentLang;
    seg.classList.toggle("is-active", active);
    seg.setAttribute("aria-pressed", active ? "true" : "false");
  });
}
