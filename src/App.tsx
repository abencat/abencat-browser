import { useEffect, useMemo, useRef, useState } from "react";
import type { MouseEvent, PointerEvent as ReactPointerEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { Check, Copy, Edit3, Layers3, Play, Plus, Search, Square, Trash2 } from "lucide-react";
import type {
  AppSettings, BrowserProfile, ControllerState, ProxyEntry, AutomationInfo,
  FingerprintAudit, ProxyApplyTarget, ProxyCheckResult, LaunchResult,
  ModalMode, ContextMenuState, TableColumnKey, SecurityStatus,
} from "./types";
import {
  tableColumnDefaults, tableColumnMins, isTauriRuntime, emptyState, emptyProxyEntry,
  detectionSites, screenPresets, localeOptions, timezoneOptions, cpuOptions, memoryOptions,
  gpuOptions, pick, randomSeed, splitLines, listText, makePreviewProfile, previewState,
  formatDate, normalizedCountry, regionCode, proxySummary, autoLocale, autoTimezone, proxyLocationLine,
} from "./constants";
import { Modal, TextField, NumberField, SelectField, TextAreaField, CopyRow } from "./components/ui";
import { useI18n, setLang, LANGS } from "./i18n";
import { LockScreen } from "./components/LockScreen";
import { SecurityModal } from "./components/SecurityModal";

export default function App() {
  const { t, lang } = useI18n();
  const [security, setSecurity] = useState<SecurityStatus | null>(null);
  const [state, setState] = useState<ControllerState>(emptyState);
  const [search, setSearch] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [modal, setModal] = useState<ModalMode>(null);
  const [draft, setDraft] = useState<BrowserProfile | null>(null);
  const [settingsDraft, setSettingsDraft] = useState<AppSettings>(emptyState.settings);
  const [permanentScripts, setPermanentScripts] = useState("");
  const [selectedId, setSelectedId] = useState("");
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [batchDraft, setBatchDraft] = useState({ count: 5, proxy: "", autoLocale: false, autoTimezone: true });
  const [moveProjectId, setMoveProjectId] = useState("");
  const [contextMenu, setContextMenu] = useState<ContextMenuState>(null);
  const [proxyChecking, setProxyChecking] = useState(false);
  const [automation, setAutomation] = useState<AutomationInfo | null>(null);
  const [audit, setAudit] = useState<FingerprintAudit | null>(null);
  const [auditing, setAuditing] = useState(false);
  const [proxyDraft, setProxyDraft] = useState({ protocol: "SOCKS5", host: "", port: "", username: "", password: "" });
  const [proxyBulk, setProxyBulk] = useState("");
  const [proxyBusyId, setProxyBusyId] = useState("");
  const [browserInstalled, setBrowserInstalled] = useState<boolean | null>(null);
  const [downloadingBrowser, setDownloadingBrowser] = useState(false);
  const tableCardRef = useRef<HTMLElement | null>(null);
  const [columnWidths, setColumnWidths] = useState<Record<TableColumnKey, number>>(tableColumnDefaults);
  const [tableViewportWidth, setTableViewportWidth] = useState(0);
  const resizeRef = useRef<{ key: TableColumnKey; x: number; width: number } | null>(null);
  const rowDragRef = useRef<{ sourceId: string; startY: number; dragging: boolean } | null>(null);
  const suppressRowClickRef = useRef(false);

  const activeProject = useMemo(
    () => state.projects.find((project) => project.id === state.activeProjectId) || state.projects[0],
    [state.activeProjectId, state.projects],
  );

  const activeProfiles = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return state.profiles
      .filter((profile) => profile.projectId === state.activeProjectId)
      .filter((profile) => {
        if (!keyword) return true;
        return `${profile.name} ${profile.note || ""} ${profile.proxy} ${profile.proxyIp} ${profile.proxyCountry} ${profile.locale} ${profile.timezone} ${profile.seed}`.toLowerCase().includes(keyword);
      });
  }, [search, state.activeProjectId, state.profiles]);

  const runningSet = useMemo(() => new Set(state.runningIds), [state.runningIds]);
  const selectedSet = useMemo(() => new Set(selectedIds), [selectedIds]);
  const selectedProfile = state.profiles.find((profile) => profile.id === selectedId);
  const scriptCount = activeProfiles.reduce((sum, profile) => sum + profile.injectionScripts.length, 0);
  const runningCount = activeProfiles.filter((profile) => runningSet.has(profile.id)).length;
  const runPercent = activeProfiles.length ? Math.round((runningCount / activeProfiles.length) * 100) : 0;

  const notify = (text: string) => {
    setError("");
    setMessage(text);
  };

  const fail = (err: unknown) => {
    setError(err instanceof Error ? err.message : String(err));
    setMessage("");
  };

  const previewOnly = () => {
    notify(t("msg.previewOnly"));
    return true;
  };

  const copyText = async (value: string) => {
    if (!value) return;
    try {
      await navigator.clipboard.writeText(value);
      notify(t("msg.copied"));
    } catch {
      notify(t("msg.copyFail"));
    }
  };

  const refresh = async () => {
    if (!isTauriRuntime()) {
      setState(previewState);
      setSettingsDraft(previewState.settings);
      setLoading(false);
      return;
    }
    try {
      const next = await invoke<ControllerState>("get_state");
      setState(next);
      setSettingsDraft(next.settings);
    } catch (err) {
      fail(err);
    } finally {
      setLoading(false);
    }
  };

  const loadSecurity = async () => {
    if (!isTauriRuntime()) {
      setSecurity({ hasMasterPassword: false, locked: false });
      return;
    }
    try {
      const next = await invoke<SecurityStatus>("get_security_status");
      setSecurity(next);
      return next;
    } catch (err) {
      fail(err);
    }
  };

  const unlockApp = async (password: string) => {
    await invoke("unlock", { password });
    await loadSecurity();
    await refresh();
    await loadAutomation();
  };

  const setMasterPassword = async (password: string) => {
    await reloadAfter(() => invoke("set_master_password", { password }).then(() => undefined), "主密码已更新");
    await loadSecurity();
  };

  const removeMasterPassword = async () => {
    if (!window.confirm("确认移除主密码？将改用机器绑定密钥。")) return;
    await reloadAfter(() => invoke("remove_master_password").then(() => undefined), "主密码已移除");
    await loadSecurity();
  };

  useEffect(() => {
    void loadSecurity();
    void refresh();
    void loadAutomation();
    const timer = window.setInterval(() => void refresh(), 6000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    const element = tableCardRef.current;
    if (!element || typeof ResizeObserver === "undefined") return;
    const updateWidth = () => setTableViewportWidth(element.clientWidth);
    updateWidth();
    const observer = new ResizeObserver(updateWidth);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const closeMenu = () => setContextMenu(null);
    window.addEventListener("click", closeMenu);
    window.addEventListener("blur", closeMenu);
    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("blur", closeMenu);
    };
  }, []);

  useEffect(() => {
    const clearDrag = () => {
      rowDragRef.current = null;
      setDraggingId(null);
    };
    window.addEventListener("pointerup", clearDrag);
    window.addEventListener("blur", clearDrag);
    return () => {
      window.removeEventListener("pointerup", clearDrag);
      window.removeEventListener("blur", clearDrag);
    };
  }, []);

  useEffect(() => {
    setSelectedIds((ids) => ids.filter((id) => state.profiles.some((profile) => profile.id === id)));
    if (selectedId && !state.profiles.some((profile) => profile.id === selectedId)) {
      setSelectedId("");
    }
  }, [selectedId, state.profiles]);

  const reloadAfter = async (work: () => Promise<void>, okText: string) => {
    if (!isTauriRuntime()) {
      previewOnly();
      return;
    }
    try {
      await work();
      await refresh();
      notify(okText);
    } catch (err) {
      fail(err);
    }
  };

  const selectProject = (projectId: string) =>
    reloadAfter(() => invoke("set_active_project", { projectId }), "项目已切换").then(() => {
      setSelectedId("");
      setSelectedIds([]);
    });

  const createProject = async () => {
    const name = window.prompt("项目名称", "新项目");
    if (!name) return;
    await reloadAfter(() => invoke("create_project", { name }), "项目已创建");
  };

  const renameProject = async () => {
    if (!activeProject) return;
    const name = window.prompt("项目名称", activeProject.name);
    if (!name) return;
    await reloadAfter(() => invoke("rename_project", { projectId: activeProject.id, name }), "项目已改名");
  };

  const deleteProject = async () => {
    if (!activeProject) return;
    if (!window.confirm(`确认删除项目「${activeProject.name}」？项目内浏览器会移动到其他项目。`)) return;
    await reloadAfter(() => invoke("delete_project", { projectId: activeProject.id }), "项目已删除");
  };

  const createProfile = async () => {
    const [screenWidth, screenHeight] = pick(screenPresets).split("x").map(Number);
    const next = makePreviewProfile({
      id: "__new__",
      projectId: state.activeProjectId,
      name: "新浏览器",
      note: "",
      seed: randomSeed(),
      screenWidth,
      screenHeight,
      hardwareConcurrency: pick(cpuOptions),
      deviceMemory: pick(memoryOptions),
      gpuRenderer: pick(gpuOptions),
      injectionScripts: [],
      oneShotInjectionScripts: [],
      extraArgs: [],
      screenMode: "random",
      fontMode: "random",
      mediaDevices: "random",
      webglImage: "random",
      createdAt: "",
      updatedAt: "",
      lastLaunchedAt: "",
    });
    setDraft(next);
    setPermanentScripts("");
    setAudit(null);
    setModal("profile");
  };

  const updateDraft = (patch: Partial<BrowserProfile>) => {
    if (!draft) return;
    setDraft({ ...draft, ...patch });
  };

  const openEditor = (profile: BrowserProfile) => {
    setDraft({ ...profile });
    setPermanentScripts(listText(profile.injectionScripts));
    setAudit(null);
    setModal("profile");
  };

  const randomizeSelected = (next: BrowserProfile) => {
    const screen = pick(screenPresets).split("x").map(Number);
    return {
      ...next,
      screenWidth: next.screenMode === "random" ? screen[0] : next.screenWidth,
      screenHeight: next.screenMode === "random" ? screen[1] : next.screenHeight,
      hardwareConcurrency: next.fontMode === "random" ? pick(cpuOptions) : next.hardwareConcurrency,
      deviceMemory: next.mediaDevices === "random" ? pick(memoryOptions) : next.deviceMemory,
      gpuRenderer: next.webglImage === "random" ? pick(gpuOptions) : next.gpuRenderer,
    };
  };

  const saveProfile = async () => {
    if (!draft) return;
    const next = randomizeSelected({
      ...draft,
      injectionScripts: splitLines(permanentScripts),
      oneShotInjectionScripts: draft.oneShotInjectionScripts || [],
      extraArgs: draft.extraArgs || [],
    });
    if (next.id === "__new__") {
      await reloadAfter(async () => {
        const created = await invoke<BrowserProfile>("create_profile", {
          projectId: next.projectId,
          name: next.name,
        });
        await invoke("save_profile", {
          profile: {
            ...created,
            ...next,
            id: created.id,
            projectId: created.projectId,
            createdAt: created.createdAt,
            updatedAt: created.updatedAt,
          },
        });
      }, "已添加浏览器");
    } else {
      await reloadAfter(() => invoke("save_profile", { profile: next }).then(() => undefined), "已保存浏览器");
    }
    setModal(null);
  };

  const chooseBrowser = async () => {
    if (!isTauriRuntime() && previewOnly()) return;
    const selected = await open({ multiple: false, filters: [{ name: "Chrome", extensions: ["exe"] }] });
    if (selected) setSettingsDraft((current) => ({ ...current, browserPath: String(selected) }));
  };

  const saveSettings = async () => {
    await reloadAfter(() => invoke("save_settings", { patch: settingsDraft }).then(() => undefined), "设置已保存");
    setModal(null);
  };

  const loadBrowserStatus = async () => {
    if (!isTauriRuntime()) { setBrowserInstalled(true); return; }
    try {
      const status = await invoke<{ installed: boolean }>("browser_status");
      setBrowserInstalled(status.installed);
    } catch { /* ignore */ }
  };

  const downloadBrowser = async () => {
    if (!isTauriRuntime() && previewOnly()) return;
    setDownloadingBrowser(true);
    notify(t("settings.downloading"));
    try {
      const path = await invoke<string>("download_browser", { force: false });
      setBrowserInstalled(true);
      setSettingsDraft((current) => ({ ...current, browserPath: path }));
      await refresh();
      notify(t("msg.kernelReady"));
    } catch (err) {
      fail(err);
    } finally {
      setDownloadingBrowser(false);
    }
  };

  const openSettings = () => {
    setSettingsDraft(state.settings);
    void loadBrowserStatus();
    setModal("settings");
  };

  const addScriptPath = async () => {
    if (!isTauriRuntime() && previewOnly()) return;
    const selected = await open({ multiple: true, filters: [{ name: "JavaScript", extensions: ["js", "user.js"] }] });
    if (!selected) return;
    const values = Array.isArray(selected) ? selected : [selected];
    const merged = Array.from(new Set([...splitLines(permanentScripts), ...values.map(String)]));
    setPermanentScripts(listText(merged));
  };

  const testDraftProxy = async (target: ProxyApplyTarget = "proxy") => {
    if (!draft) return;
    if (!draft.proxy.trim()) return notify("请先填写代理地址。");
    if (!isTauriRuntime() && previewOnly()) return;
    setProxyChecking(true);
    notify("正在获取代理出口信息...");
    try {
      const result = await invoke<ProxyCheckResult>("test_proxy", {
        proxy: draft.proxy,
        protocol: draft.proxyProtocol || "SOCKS5",
      });
      const patch: Partial<BrowserProfile> = {
        proxyIp: result.ip,
        proxyCountry: result.countryCode,
        webrtcIp: result.ip,
        location: result.countryCode,
      };
      if (target === "locale") {
        patch.locale = result.locale || draft.locale;
      }
      if (target === "timezone") {
        patch.timezone = result.timezone || draft.timezone;
      }
      updateDraft(patch);
      const suffix = target === "locale" ? "，只同步语言" : target === "timezone" ? "，只同步时区" : "，只更新出口信息";
      notify(`代理获取成功：${result.countryCode || "未知"} · ${result.ip}${suffix}`);
    } catch (err) {
      fail(err);
    } finally {
      setProxyChecking(false);
    }
  };

  const toggleLocaleAutoSync = (checked: boolean) => {
    const timezoneChecked = draft ? autoTimezone(draft) : false;
    updateDraft({ autoLocale: checked, autoLocaleTimezone: checked && timezoneChecked });
    if (checked && draft?.proxy.trim()) void testDraftProxy("locale");
  };

  const toggleTimezoneAutoSync = (checked: boolean) => {
    const localeChecked = draft ? autoLocale(draft) : false;
    updateDraft({ autoTimezone: checked, autoLocaleTimezone: checked && localeChecked });
    if (checked && draft?.proxy.trim()) void testDraftProxy("timezone");
  };

  const launch = async (profileId: string) => {
    if (!isTauriRuntime() && previewOnly()) return;
    try {
      const result = await invoke<LaunchResult>("launch_profile", { profileId });
      setState((current) => ({
        ...current,
        profiles: current.profiles.map((profile) => profile.id === result.profile.id ? result.profile : profile),
        runningIds: Array.from(new Set([...current.runningIds, profileId])),
      }));
      const summary = proxySummary(result.profile);
      if (result.proxyLookupError) {
        notify(`浏览器已启动，代理IP获取失败：${result.proxyLookupError}`);
      } else {
        notify(summary ? `浏览器已启动，正在后台获取 IP：${summary}` : "浏览器已启动，正在后台获取 IP");
      }
    } catch (err) {
      fail(err);
    }
  };

  const stop = (profileId: string) =>
    reloadAfter(() => invoke("stop_profile", { profileId }), "浏览器已停止");

  const launchSelected = () => {
    const target = selectedProfile || activeProfiles[0];
    if (!target) return notify(t("msg.selectFirst"));
    if (runningSet.has(target.id)) void stop(target.id);
    else void launch(target.id);
  };

  const batchLaunch = () => {
    const ids = selectedIds.length ? selectedIds : activeProfiles.map((profile) => profile.id);
    if (!ids.length) return notify(t("msg.selectFirst"));
    void reloadAfter(() => invoke("batch_launch", { profileIds: ids }).then(() => undefined), "已提交批量启动");
  };

  const batchStop = () => {
    const ids = selectedIds.length ? selectedIds : state.runningIds;
    if (!ids.length) return notify("没有可停止的浏览器。");
    void reloadAfter(() => invoke("batch_stop", { profileIds: ids }).then(() => undefined), "已提交批量关闭");
  };

  const clone = () => {
    if (!selectedProfile) return notify(t("msg.selectFirst"));
    void reloadAfter(() => invoke("clone_profile", { profileId: selectedProfile.id }).then(() => undefined), "已克隆浏览器，随机项已重新生成。");
  };

  const cloneById = (profileId: string) => {
    void reloadAfter(() => invoke("clone_profile", { profileId }).then(() => undefined), "已克隆浏览器，随机项已重新生成。");
  };

  const removeSelected = async () => {
    if (selectedIds.length) {
      if (!window.confirm(`删除选中的 ${selectedIds.length} 个浏览器？用户目录会保留。`)) return;
      await reloadAfter(() => invoke("batch_delete_profiles", { profileIds: selectedIds }).then(() => undefined), "已批量删除浏览器");
      setSelectedIds([]);
      setSelectedId("");
      return;
    }
    if (!selectedProfile) return notify(t("msg.selectFirst"));
    if (!window.confirm(`删除 ${selectedProfile.name}？用户目录会保留。`)) return;
    await reloadAfter(() => invoke("delete_profile", { profileId: selectedProfile.id }), "已删除浏览器");
    setSelectedId("");
  };

  const removeProfileById = async (profileId: string) => {
    const profile = state.profiles.find((item) => item.id === profileId);
    if (!profile) return;
    if (!window.confirm(`删除 ${profile.name}？用户目录会保留。`)) return;
    await reloadAfter(() => invoke("delete_profile", { profileId }), "已删除浏览器");
    setSelectedId("");
    setSelectedIds((ids) => ids.filter((id) => id !== profileId));
  };

  const openFolderById = (profileId: string) => {
    void reloadAfter(() => invoke("open_profile_folder", { profileId }), "已打开浏览器目录");
  };

  const moveSelected = () => {
    if (!selectedIds.length && !selectedProfile) return notify(t("msg.selectFirst"));
    const fallback = state.projects.find((project) => project.id !== state.activeProjectId);
    setMoveProjectId(fallback?.id || state.activeProjectId);
    setModal("batchMove");
  };

  const moveProfileById = (profileId: string) => {
    setSelectedId(profileId);
    setSelectedIds([]);
    const profile = state.profiles.find((item) => item.id === profileId);
    const fallback = state.projects.find((project) => project.id !== (profile?.projectId || state.activeProjectId));
    setMoveProjectId(fallback?.id || state.activeProjectId);
    setModal("batchMove");
  };

  const runBatchMove = async () => {
    const ids = selectedIds.length ? selectedIds : selectedProfile ? [selectedProfile.id] : [];
    if (!ids.length || !moveProjectId) return;
    await reloadAfter(() => invoke("batch_move_profiles", { profileIds: ids, projectId: moveProjectId }).then(() => undefined), "已移动到项目");
    setSelectedIds([]);
    setSelectedId("");
    setModal(null);
  };

  const runBatchCreate = async () => {
    await reloadAfter(
      () => invoke("batch_create_profiles", {
        request: {
          projectId: state.activeProjectId,
          count: batchDraft.count,
          proxy: batchDraft.proxy,
          autoLocale: batchDraft.autoLocale,
          autoTimezone: batchDraft.autoTimezone,
          autoLocaleTimezone: batchDraft.autoLocale && batchDraft.autoTimezone,
        },
      }).then(() => undefined),
      "批量浏览器已创建",
    );
    setModal(null);
  };

  const clearSelectedScripts = async () => {
    const ids = selectedIds.length ? selectedIds : selectedProfile ? [selectedProfile.id] : [];
    if (!ids.length) return notify(t("msg.selectFirst"));
    if (!window.confirm(`清除选中 ${ids.length} 个浏览器的脚本？`)) return;
    const updates = state.profiles.filter((profile) => ids.includes(profile.id)).map((profile) => ({
      ...profile,
      injectionScripts: [],
      oneShotInjectionScripts: [],
    }));
    await reloadAfter(async () => {
      for (const profile of updates) await invoke("save_profile", { profile });
    }, "已清除脚本");
  };

  const loadAutomation = async () => {
    if (!isTauriRuntime()) return;
    try {
      const info = await invoke<AutomationInfo>("get_automation_info");
      setAutomation(info);
    } catch (err) {
      fail(err);
    }
  };

  const openAutomation = () => {
    void loadAutomation();
    setModal("automation");
  };

  const runAudit = async (profile: BrowserProfile | null) => {
    if (!profile) return;
    if (!isTauriRuntime()) {
      setAudit({ score: 92, issues: [], warnings: ["前端预览：真实评分请在 Tauri 应用内运行"] });
      return;
    }
    setAuditing(true);
    try {
      const result = await invoke<FingerprintAudit>("audit_fingerprint", { profileId: profile.id });
      setAudit(result);
    } catch (err) {
      fail(err);
    } finally {
      setAuditing(false);
    }
  };

  const openDetectionSite = async (url: string) => {
    const target = draft && draft.id !== "__new__" ? draft.id : selectedProfile?.id || activeProfiles[0]?.id;
    if (!target) return notify("请先选择或保存一个浏览器。");
    if (!isTauriRuntime() && previewOnly()) return;
    try {
      const result = await invoke<LaunchResult>("launch_profile_at", { profileId: target, url });
      setState((current) => ({
        ...current,
        runningIds: Array.from(new Set([...current.runningIds, target])),
        endpoints: result.endpoint ? { ...current.endpoints, [target]: result.endpoint } : current.endpoints,
      }));
      notify(`已在指纹浏览器中打开检测站：${url}`);
    } catch (err) {
      fail(err);
    }
  };

  const openProxyPool = () => {
    setProxyBulk("");
    setProxyDraft({ protocol: "SOCKS5", host: "", port: "", username: "", password: "" });
    setModal("proxyPool");
  };

  const openSecurity = () => {
    void loadSecurity();
    setModal("security");
  };

  const addProxyEntry = async () => {
    if (!proxyDraft.host.trim() || !proxyDraft.port.trim()) return notify("请填写代理主机和端口。");
    await reloadAfter(
      () => invoke("save_proxy", { entry: { ...emptyProxyEntry, ...proxyDraft } }).then(() => undefined),
      "代理已加入代理池",
    );
    setProxyDraft({ protocol: proxyDraft.protocol, host: "", port: "", username: "", password: "" });
  };

  const importProxyBulk = async () => {
    if (!proxyBulk.trim()) return notify("请粘贴代理列表，每行一个。");
    await reloadAfter(async () => {
      const count = await invoke<number>("import_proxies", { text: proxyBulk, protocol: proxyDraft.protocol });
      notify(`已导入 ${count} 个代理`);
    }, "代理已批量导入");
    setProxyBulk("");
  };

  const deleteProxyEntry = async (proxyId: string) => {
    await reloadAfter(() => invoke("delete_proxy", { proxyId }), "代理已删除");
  };

  const checkProxyEntry = async (proxyId: string) => {
    if (!isTauriRuntime() && previewOnly()) return;
    setProxyBusyId(proxyId);
    try {
      const entry = await invoke<ProxyEntry>("check_proxy_entry", { proxyId });
      await refresh();
      notify(entry.status === "ok" ? `代理可用：${entry.lastCountry || "?"} · ${entry.lastIp}` : "代理检测失败");
    } catch (err) {
      fail(err);
      await refresh();
    } finally {
      setProxyBusyId("");
    }
  };

  const assignProxyToSelected = async (proxyId: string) => {
    const ids = selectedIds.length ? selectedIds : selectedProfile ? [selectedProfile.id] : [];
    if (!ids.length) return notify("请先在列表中勾选要绑定的浏览器。");
    await reloadAfter(async () => {
      const count = await invoke<number>("assign_proxy_to_profiles", { proxyId, profileIds: ids });
      notify(`已为 ${count} 个浏览器绑定该代理`);
    }, "代理已绑定");
  };

  const exportData = async () => {
    if (!isTauriRuntime() && previewOnly()) return;
    try {
      const payload = await invoke<string>("export_profiles", { projectId: activeProject?.id });
      const target = await save({ defaultPath: `cloak-profiles-${activeProject?.name || "all"}.json`, filters: [{ name: "JSON", extensions: ["json"] }] });
      if (!target) return;
      await writeTextFile(String(target), payload);
      notify("导出完成");
    } catch (err) {
      fail(err);
    }
  };

  const importData = async () => {
    if (!isTauriRuntime() && previewOnly()) return;
    try {
      const source = await open({ multiple: false, filters: [{ name: "JSON", extensions: ["json"] }] });
      if (!source) return;
      const payload = await readTextFile(String(source));
      const count = await invoke<number>("import_profiles", { payload });
      await refresh();
      notify(`已导入 ${count} 个浏览器`);
    } catch (err) {
      fail(err);
    }
  };

  const chooseScriptsForTargets = async (targets: string[]) => {
    if (!targets.length) return notify(t("msg.selectFirst"));
    if (!isTauriRuntime() && previewOnly()) return;
    const selected = await open({ multiple: true, filters: [{ name: "JavaScript", extensions: ["js", "user.js"] }] });
    if (!selected) return;
    const values = (Array.isArray(selected) ? selected : [selected]).map(String);
    const updates = state.profiles.filter((profile) => targets.includes(profile.id)).map((profile) => ({
      ...profile,
      injectionScripts: values,
    }));
    await reloadAfter(async () => {
      for (const profile of updates) await invoke("save_profile", { profile });
    }, `已载入脚本到 ${updates.length} 个浏览器`);
  };

  const chooseScriptsForSelected = async () => {
    const targets = selectedIds.length ? selectedIds : selectedProfile ? [selectedProfile.id] : [];
    await chooseScriptsForTargets(targets);
  };

  const renameProjectById = async (projectId: string) => {
    const project = state.projects.find((item) => item.id === projectId);
    if (!project) return;
    const name = window.prompt("项目名称", project.name);
    if (!name) return;
    await reloadAfter(() => invoke("rename_project", { projectId, name }), "项目已改名");
  };

  const deleteProjectById = async (projectId: string) => {
    const project = state.projects.find((item) => item.id === projectId);
    if (!project) return;
    if (!window.confirm(`确认删除项目「${project.name}」？项目内浏览器会移动到其他项目。`)) return;
    await reloadAfter(() => invoke("delete_project", { projectId }), "项目已删除");
  };

  const openProjectMenu = (event: MouseEvent, projectId: string) => {
    event.preventDefault();
    event.stopPropagation();
    setContextMenu({ kind: "project", id: projectId, x: event.clientX, y: event.clientY });
  };

  const openProfileMenu = (event: MouseEvent, profileId: string) => {
    event.preventDefault();
    event.stopPropagation();
    setSelectedId(profileId);
    setContextMenu({ kind: "profile", id: profileId, x: event.clientX, y: event.clientY });
  };

  const toggleSelected = (profileId: string, checked: boolean) => {
    setSelectedId(profileId);
    setSelectedIds((ids) => checked ? Array.from(new Set([...ids, profileId])) : ids.filter((id) => id !== profileId));
  };

  const toggleAllSelected = (checked: boolean) => {
    setSelectedIds(checked ? activeProfiles.map((profile) => profile.id) : []);
  };

  const reorderRow = async (sourceId: string, targetId: string) => {
    if (!sourceId || sourceId === targetId || search.trim()) {
      setDraggingId(null);
      return;
    }
    const profiles = state.profiles.filter((profile) => profile.projectId === state.activeProjectId);
    const from = profiles.findIndex((profile) => profile.id === sourceId);
    const to = profiles.findIndex((profile) => profile.id === targetId);
    if (from < 0 || to < 0) {
      setDraggingId(null);
      return;
    }
    const ordered = [...profiles];
    const [moved] = ordered.splice(from, 1);
    ordered.splice(to, 0, moved);
    const orderedIds = ordered.map((profile) => profile.id);
    const others = state.profiles.filter((profile) => profile.projectId !== state.activeProjectId);
    setState((current) => ({ ...current, profiles: [...others, ...ordered] }));
    setDraggingId(null);
    if (!isTauriRuntime()) return;
    try {
      await invoke("reorder_profiles", { projectId: state.activeProjectId, profileIds: orderedIds });
    } catch (err) {
      fail(err);
      await refresh();
    }
  };

  const endRowDrag = () => {
    rowDragRef.current = null;
    setDraggingId(null);
  };

  const beginRowPointerDrag = (profileId: string, event: ReactPointerEvent<HTMLTableRowElement>) => {
    if (event.button !== 0 || search.trim()) return;
    const target = event.target as HTMLElement;
    if (target.closest("button, input, textarea, select, .col-resizer")) return;
    rowDragRef.current = { sourceId: profileId, startY: event.clientY, dragging: false };
  };

  const moveRowPointerDrag = (event: ReactPointerEvent<HTMLTableRowElement>) => {
    const active = rowDragRef.current;
    if (!active) return;
    if (!active.dragging && Math.abs(event.clientY - active.startY) > 4) {
      active.dragging = true;
      suppressRowClickRef.current = true;
      setDraggingId(active.sourceId);
    }
  };

  const endRowPointerDrag = (targetId: string) => {
    const active = rowDragRef.current;
    if (!active) {
      setDraggingId(null);
      return;
    }
    rowDragRef.current = null;
    setDraggingId(null);
    if (active.dragging) {
      void reorderRow(active.sourceId, targetId);
      window.setTimeout(() => {
        suppressRowClickRef.current = false;
      }, 0);
    }
  };

  const beginColumnResize = (key: TableColumnKey, event: MouseEvent<HTMLSpanElement>) => {
    event.preventDefault();
    event.stopPropagation();
    resizeRef.current = { key, x: event.clientX, width: columnWidths[key] };
    document.body.classList.add("resizing-columns");

    const onMove = (moveEvent: globalThis.MouseEvent) => {
      const active = resizeRef.current;
      if (!active) return;
      const minWidth = tableColumnMins[active.key];
      const width = Math.max(minWidth, active.width + moveEvent.clientX - active.x);
      setColumnWidths((current) => ({ ...current, [active.key]: width }));
    };
    const onEnd = () => {
      resizeRef.current = null;
      document.body.classList.remove("resizing-columns");
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onEnd);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onEnd);
  };

  const columnStyle = useMemo(
    () => {
      const baseWidth = Object.values(columnWidths).reduce((sum, value) => sum + value, 0);
      const targetWidth = Math.max(baseWidth, tableViewportWidth);
      const extraWidth = Math.max(0, targetWidth - baseWidth);
      const growKeys: TableColumnKey[] = ["browser", "proxy", "region", "fingerprint", "last"];
      const widths = { ...columnWidths };
      const browserExtra = Math.floor(extraWidth * 0.4);
      widths.browser += browserExtra;
      const restExtra = extraWidth - browserExtra;
      const growBase = growKeys.reduce((sum, key) => sum + columnWidths[key], 0);
      let usedRest = 0;
      growKeys.forEach((key, index) => {
        const share = index === growKeys.length - 1
          ? restExtra - usedRest
          : Math.floor(restExtra * (columnWidths[key] / growBase));
        widths[key] += share;
        usedRest += share;
      });
      return {
        minWidth: targetWidth,
        widths,
      };
    },
    [columnWidths, tableViewportWidth],
  );

  const applyScreenPreset = (value: string) => {
    if (!value) return;
    const [width, height] = value.split("x").map(Number);
    updateDraft({ screenWidth: width, screenHeight: height });
  };

  const selectedAll = activeProfiles.length > 0 && activeProfiles.every((profile) => selectedSet.has(profile.id));
  const selectedSome = activeProfiles.some((profile) => selectedSet.has(profile.id));

  if (security?.locked) {
    return <LockScreen onUnlock={unlockApp} />;
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="logo">A</div>
          <div>
            <h1>Abencat Browser</h1>
            <p>{t("app.subtitle")}</p>
          </div>
        </div>
        <div className="side-scroll">
          <div className="side-card">
            <div className="side-title">工作台</div>
            <nav className="nav">
              <button className="nav-item active"><span className="material">◎</span><span>{t("nav.browsers")}</span></button>
              <button className="nav-item" onClick={openProxyPool}><span className="material">⌘</span><span>{t("nav.proxy")}</span></button>
              <button className="nav-item" onClick={openAutomation}><span className="material">↯</span><span>{t("nav.automation")}</span></button>
              <button className="nav-item" onClick={openSecurity}><span className="material">🔒</span><span>{t("nav.security")}</span></button>
              <button className="nav-item" onClick={openSettings}><span className="material">⚙</span><span>{t("nav.settings")}</span></button>
            </nav>
          </div>
          <div className="side-card projects">
            <div className="project-head">{t("projects.title")}</div>
            <div className="project-actions">
              <button onClick={createProject}>{t("projects.add")}</button>
              <button onClick={renameProject}>{t("projects.rename")}</button>
              <button className="danger" onClick={deleteProject}>{t("projects.delete")}</button>
            </div>
            <div className="project-list">
              {state.projects.map((project) => {
                const count = state.profiles.filter((profile) => profile.projectId === project.id).length;
                return (
                  <button
                    key={project.id}
                    className={`project ${project.id === state.activeProjectId ? "active" : ""}`}
                    onClick={() => void selectProject(project.id)}
                    onContextMenu={(event) => openProjectMenu(event, project.id)}
                  >
                    <span>{project.name}</span>
                    <span className="count">{count}</span>
                  </button>
                );
              })}
            </div>
          </div>
        </div>
        <div className="side-footer">
          <select className="lang-select" value={lang} onChange={(e) => setLang(e.target.value)} title="Language">
            {LANGS.map((l) => <option key={l.code} value={l.code}>{l.name}</option>)}
          </select>
          <button className="path-btn" onClick={() => setModal("about")}><span>ⓘ</span> {t("about")}</button>
        </div>
      </aside>

      <main className="main">
        <header className="header">
          <div className="title-row">
            <h2>{t("header.browsers")}</h2>
          </div>
          <div className="header-actions">
            <button className="btn" onClick={openSettings}>{t("header.browserPath")}</button>
          </div>
        </header>

        <div className="body-scroll">
          <section className="stats">
            <div className="stat-card blue">
              <div className="stat-top"><div><p className="stat-label">{t("stats.total")}</p><h3 className="stat-value">{activeProfiles.length}</h3></div><div className="stat-icon">B</div></div>
              <div className="stat-foot"><span className="mini-dot" /><span>{t("stats.currentProject")}</span></div>
            </div>
            <div className="stat-card green">
              <div className="stat-top"><div><p className="stat-label">{t("stats.running")}</p><h3 className="stat-value">{runningCount}</h3></div><div className="stat-icon">▶</div></div>
              <div className="progress"><div style={{ width: `${runPercent}%` }} /></div>
            </div>
            <div className="stat-card purple">
              <div className="stat-top"><div><p className="stat-label">{t("stats.scripts")}</p><h3 className="stat-value">{scriptCount}</h3></div><div className="stat-icon">JS</div></div>
              <div className="stat-foot"><span className="mini-dot" /><span>{t("stats.scriptsFoot")}</span></div>
            </div>
            <div className="stat-card amber">
              <div className="stat-top"><div><p className="stat-label">{t("stats.projects")}</p><h3 className="stat-value">{state.projects.length}</h3></div><div className="stat-icon">P</div></div>
              <div className="stat-foot"><span className="mini-dot" /><span>{t("stats.switchLeft")}</span></div>
            </div>
          </section>

          <section className="action-bar">
            <div className="action-group">
              <button className="btn primary" onClick={createProfile}><Plus size={16} />{t("action.new")}</button>
              <button className={`btn launch ${selectedProfile && runningSet.has(selectedProfile.id) ? "danger" : ""}`} onClick={launchSelected}>
                {selectedProfile && runningSet.has(selectedProfile.id) ? <Square size={15} /> : <Play size={15} />}
                {selectedProfile && runningSet.has(selectedProfile.id) ? t("action.stop") : t("action.start")}
              </button>
              <button className="btn" onClick={() => selectedProfile ? void stop(selectedProfile.id) : notify(t("msg.selectFirst"))}>{t("action.stop")}</button>
              <button className="btn" onClick={() => selectedProfile ? openEditor(selectedProfile) : notify(t("msg.selectFirst"))}><Edit3 size={15} />{t("action.edit")}</button>
              <button className="btn" onClick={clone}><Copy size={15} />{t("action.clone")}</button>
              <button className="btn" onClick={moveSelected}>{t("action.moveTo")}</button>
              <div className="toolbar-search">
                <Search size={16} />
                <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder={t("action.search")} />
              </div>
              <button className="btn" onClick={() => void importData()}>{t("action.import")}</button>
              <button className="btn" onClick={() => void chooseScriptsForSelected()}>{t("action.loadScripts")}</button>
              <button className="btn danger" onClick={() => void removeSelected()}><Trash2 size={15} />{t("action.delete")}</button>
            </div>
          </section>

          <section className={`batch-bar ${selectedIds.length ? "show" : ""}`}>
            <div>{t("batch.selected")} <span>{selectedIds.length}</span> {t("batch.units")}</div>
            <div className="batch-actions">
              <button className="batch-btn blue" onClick={batchLaunch}>{t("batch.launch")}</button>
              <button className="batch-btn gray" onClick={batchStop}>{t("batch.stop")}</button>
              <button className="batch-btn green" onClick={() => void chooseScriptsForSelected()}>{t("batch.loadScripts")}</button>
              <button className="batch-btn gray" onClick={() => void clearSelectedScripts()}>{t("batch.clearScripts")}</button>
              <button className="batch-btn amber" onClick={moveSelected}>{t("batch.move")}</button>
              <button className="batch-btn cyan" onClick={() => setModal("batchCreate")}>{t("batch.create")}</button>
              <button className="batch-btn red" onClick={() => void removeSelected()}>{t("batch.delete")}</button>
            </div>
          </section>

          <section
            className="table-card"
            ref={tableCardRef}
          >
            <table style={{ minWidth: columnStyle.minWidth, width: columnStyle.minWidth }}>
              <colgroup>
                <col style={{ width: columnStyle.widths.select }} />
                <col style={{ width: columnStyle.widths.status }} />
                <col style={{ width: columnStyle.widths.browser }} />
                <col style={{ width: columnStyle.widths.proxy }} />
                <col style={{ width: columnStyle.widths.region }} />
                <col style={{ width: columnStyle.widths.fingerprint }} />
                <col style={{ width: columnStyle.widths.scripts }} />
                <col style={{ width: columnStyle.widths.last }} />
              </colgroup>
              <thead>
                <tr>
                  <th className="check-cell"><input className="select-box" type="checkbox" checked={selectedAll} ref={(node) => { if (node) node.indeterminate = selectedSome && !selectedAll; }} onChange={(event) => toggleAllSelected(event.target.checked)} /></th>
                  <th className="center">{t("col.status")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("status", event)} /></th>
                  <th>{t("col.browser")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("browser", event)} /></th>
                  <th>{t("col.proxy")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("proxy", event)} /></th>
                  <th>{t("col.region")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("region", event)} /></th>
                  <th>{t("col.fingerprint")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("fingerprint", event)} /></th>
                  <th className="center">{t("col.scripts")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("scripts", event)} /></th>
                  <th className="last-col">{t("col.last")}<span className="col-resizer" onMouseDown={(event) => beginColumnResize("last", event)} /></th>
                </tr>
              </thead>
              <tbody>
                {loading ? (
                  <tr><td colSpan={8}><div className="empty-state"><p>{t("loading")}</p></div></td></tr>
                ) : activeProfiles.map((profile) => {
                  const running = runningSet.has(profile.id);
                  const selected = selectedId === profile.id || selectedSet.has(profile.id);
                  const scripts = profile.injectionScripts.length;
                  return (
                    <tr
                      key={profile.id}
                      className={`${selected ? "selected" : ""} ${draggingId === profile.id ? "dragging" : ""}`}
                      onPointerDown={(event) => beginRowPointerDrag(profile.id, event)}
                      onPointerMove={moveRowPointerDrag}
                      onPointerUp={() => endRowPointerDrag(profile.id)}
                      onPointerCancel={endRowDrag}
                      onClick={() => {
                        if (suppressRowClickRef.current) {
                          suppressRowClickRef.current = false;
                          return;
                        }
                        setSelectedId(profile.id);
                      }}
                      onContextMenu={(event) => openProfileMenu(event, profile.id)}
                    >
                      <td className="check-cell">
                        <input className="select-box" type="checkbox" checked={selectedSet.has(profile.id)} onClick={(event) => event.stopPropagation()} onChange={(event) => toggleSelected(profile.id, event.target.checked)} />
                      </td>
                      <td className="center">
                        <button className={`state-icon-btn ${running ? "stop" : "play"}`} title={running ? "停止" : "启动"} onClick={(event) => { event.stopPropagation(); running ? void stop(profile.id) : void launch(profile.id); }}>
                          <span />
                        </button>
                      </td>
                      <td title={profile.note || "无备注"}><div className="env-name">{profile.name || "未命名浏览器"}</div><div className="env-note">{profile.note || "备注"}</div></td>
                      <td title={profile.proxy || "无"}><div className="proxy">{profile.proxy || "无"}</div><div className="region-sub">{proxyLocationLine(profile)}</div></td>
                      <td><div className="region"><div className="flag">{regionCode(profile)}</div><div><div className="region-main">{profile.locale}</div><div className="region-sub">{profile.timezone}</div></div></div></td>
                      <td title={`屏幕 ${profile.screenWidth}x${profile.screenHeight}\nSeed ${profile.seed}\nCPU ${profile.hardwareConcurrency}\n内存 ${profile.deviceMemory}G`}>
                        <div className="fp-main">{profile.screenWidth}x{profile.screenHeight}</div>
                        <div className="fp-sub">Seed {profile.seed} · CPU {profile.hardwareConcurrency} / RAM {profile.deviceMemory}G</div>
                      </td>
                      <td className="center" title={scripts ? profile.injectionScripts.join("\n") : "未载入脚本"}><span className="script-pill">{scripts}</span></td>
                      <td title={profile.lastLaunchedAt || "-"}><span className="last">{formatDate(profile.lastLaunchedAt)}</span></td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            {!loading && !activeProfiles.length && (
              <div className="empty-state">
                <div><Layers3 size={48} /><p>{t("empty.noEnv")}</p></div>
              </div>
            )}
          </section>
        </div>

        <footer className="footer">
          <span className="ready"><span className="status-dot on" /><span>{error || message || t("ready")}</span></span>
          <span className="sep" /><span>API: {state.apiPort ? `127.0.0.1:${state.apiPort}` : "—"}</span>
        </footer>
      </main>

      {modal === "profile" && draft && (
        <Modal title={draft.id === "__new__" ? t("editor.new") : t("editor.edit")} onClose={() => setModal(null)} dialogClassName="profile-dialog">
          <div className="profile-editor">
            <section className="editor-panel">
              <div className="form-grid compact-form">
                <TextField label={t("editor.name")} value={draft.name} onChange={(name) => updateDraft({ name })} />
                <TextField label={t("editor.proxy")} value={draft.proxy} placeholder="http://host:8080 / socks5://host:1080" onChange={(proxy) => updateDraft({ proxy })} />
                <div className="proxy-check-line span-2 compact">
                  <button className="btn mini-btn" type="button" onClick={() => void testDraftProxy("proxy")} disabled={proxyChecking}>{proxyChecking ? t("editor.getting") : t("editor.getProxyInfo")}</button>
                  <span>{t("editor.exit")}：{[normalizedCountry(draft.proxyCountry) || draft.proxyCountry || "-", draft.proxyIp || draft.webrtcIp || "-"].join(" · ")}</span>
                </div>
                <div className="inline-field span-2">
                  <SelectField label={t("editor.locale")} value={draft.locale} options={localeOptions} onChange={(locale) => updateDraft({ locale })} />
                  <label className="check-label"><input type="checkbox" checked={autoLocale(draft)} onChange={(event) => toggleLocaleAutoSync(event.target.checked)} />{t("editor.autoByProxy")}</label>
                  <button className="btn mini-btn" type="button" onClick={() => void testDraftProxy("locale")} disabled={proxyChecking}>{proxyChecking ? t("editor.getting") : t("editor.get")}</button>
                  <button className="btn mini-btn" type="button" onClick={() => updateDraft({ locale: "zh-CN" })}>{t("editor.reset")}</button>
                </div>
                <div className="inline-field span-2">
                  <SelectField label={t("editor.timezone")} value={draft.timezone} options={timezoneOptions} onChange={(timezone) => updateDraft({ timezone })} />
                  <label className="check-label"><input type="checkbox" checked={autoTimezone(draft)} onChange={(event) => toggleTimezoneAutoSync(event.target.checked)} />{t("editor.autoByProxy")}</label>
                  <button className="btn mini-btn" type="button" onClick={() => void testDraftProxy("timezone")} disabled={proxyChecking}>{proxyChecking ? t("editor.getting") : t("editor.get")}</button>
                  <button className="btn mini-btn" type="button" onClick={() => updateDraft({ timezone: "Asia/Shanghai" })}>{t("editor.reset")}</button>
                </div>
                <div className="inline-field span-2">
                  <label className="mini-select">{t("editor.commonScreen")}
                    <select className="input" value={`${draft.screenWidth}x${draft.screenHeight}`} onChange={(event) => applyScreenPreset(event.target.value)}>
                      <option value="">{t("editor.custom")}</option>
                      {screenPresets.map((item) => <option key={item} value={item}>{item}</option>)}
                    </select>
                  </label>
                  <NumberField label={t("editor.width")} value={draft.screenWidth} onChange={(screenWidth) => updateDraft({ screenWidth })} />
                  <NumberField label={t("editor.height")} value={draft.screenHeight} onChange={(screenHeight) => updateDraft({ screenHeight })} />
                  <label className="check-label"><input type="checkbox" checked={draft.screenMode === "random"} onChange={(event) => updateDraft({ screenMode: event.target.checked ? "random" : "custom" })} />{t("editor.random")}</label>
                </div>
                <div className="inline-field">
                  <NumberField label={t("editor.cpu")} value={draft.hardwareConcurrency} onChange={(hardwareConcurrency) => updateDraft({ hardwareConcurrency })} />
                  <label className="check-label"><input type="checkbox" checked={draft.fontMode === "random"} onChange={(event) => updateDraft({ fontMode: event.target.checked ? "random" : "custom" })} />{t("editor.random")}</label>
                </div>
                <div className="inline-field">
                  <NumberField label={t("editor.memory")} value={draft.deviceMemory} onChange={(deviceMemory) => updateDraft({ deviceMemory })} />
                  <label className="check-label"><input type="checkbox" checked={draft.mediaDevices === "random"} onChange={(event) => updateDraft({ mediaDevices: event.target.checked ? "random" : "real" })} />{t("editor.random")}</label>
                </div>
                <div className="inline-field span-2">
                  <SelectField label={t("editor.gpu")} value={draft.gpuRenderer || gpuOptions[0]} options={gpuOptions} onChange={(gpuRenderer) => updateDraft({ gpuRenderer })} />
                  <label className="check-label"><input type="checkbox" checked={draft.webglImage === "random"} onChange={(event) => updateDraft({ webglImage: event.target.checked ? "random" : "noise" })} />{t("editor.random")}</label>
                </div>
              </div>
            </section>
            <section className="editor-panel editor-side">
              <TextAreaField label={t("editor.note")} value={draft.note || ""} placeholder={t("editor.notePlaceholder")} onChange={(note) => updateDraft({ note })} className="note-box" />
              <div className="script-head">
                <label>{t("editor.scriptsLabel")}</label>
                <button className="btn small" onClick={() => void addScriptPath()} type="button">{t("editor.chooseFile")}</button>
              </div>
              <textarea className="textarea script-box" value={permanentScripts} onChange={(event) => setPermanentScripts(event.target.value)} />
              <div className="audit-block">
                <div className="audit-head">
                  <label>{t("editor.audit")}</label>
                  <button className="btn small" type="button" onClick={() => void runAudit(draft)} disabled={auditing || draft.id === "__new__"}>{auditing ? t("editor.auditing") : t("editor.runAudit")}</button>
                </div>
                {draft.id === "__new__" && <p className="muted">{t("editor.saveFirst")}</p>}
                {audit && (
                  <div className="audit-result">
                    <div className={`audit-score ${audit.score >= 85 ? "good" : audit.score >= 60 ? "warn" : "bad"}`}>{audit.score}<span>/100</span></div>
                    <div className="audit-lists">
                      {audit.issues.map((item, i) => <div key={`i${i}`} className="audit-issue">✕ {item}</div>)}
                      {audit.warnings.map((item, i) => <div key={`w${i}`} className="audit-warn">! {item}</div>)}
                      {!audit.issues.length && !audit.warnings.length && <div className="audit-ok">{t("editor.auditOk")}</div>}
                    </div>
                  </div>
                )}
                <div className="detection-row">
                  <span className="copy-label">{t("editor.detectionSites")}</span>
                  <div className="detection-buttons">
                    {detectionSites.map((site) => (
                      <button key={site.url} className="btn mini-btn" type="button" onClick={() => void openDetectionSite(site.url)}>{site.label}</button>
                    ))}
                  </div>
                </div>
              </div>
            </section>
          </div>
          <div className="dialog-actions profile-actions">
            <button className="btn" onClick={() => setModal(null)}>{t("common.cancel")}</button>
            <button className="btn primary" onClick={() => void saveProfile()}><Check size={16} />{t("common.save")}</button>
          </div>
        </Modal>
      )}

      {modal === "settings" && (
        <Modal title={t("settings.title")} onClose={() => setModal(null)}>
          <label>{t("settings.browserProgram")}
            <div className="path-field">
              <input className="input" value={settingsDraft.browserPath} onChange={(event) => setSettingsDraft({ ...settingsDraft, browserPath: event.target.value })} />
              <button className="btn" onClick={chooseBrowser}>{t("settings.choose")}</button>
            </div>
          </label>
          <label>{t("settings.dataDir")}<input className="input" value={settingsDraft.dataRoot} onChange={(event) => setSettingsDraft({ ...settingsDraft, dataRoot: event.target.value })} /></label>
          <div className="kernel-row">
            <div>
              <div className="kernel-label">{t("settings.kernel")}</div>
              <div className={`kernel-status ${browserInstalled ? "ok" : "missing"}`}>
                {browserInstalled === null ? "…" : browserInstalled ? t("settings.kernelInstalled") : t("settings.kernelMissing")}
              </div>
            </div>
            <button className="btn" disabled={downloadingBrowser} onClick={() => void downloadBrowser()}>
              {downloadingBrowser ? t("settings.downloading") : t("settings.downloadKernel")}
            </button>
          </div>
          <div className="dialog-actions">
            <button className="btn" onClick={() => setModal(null)}>{t("common.cancel")}</button>
            <button className="btn primary" onClick={() => void saveSettings()}>{t("common.save")}</button>
          </div>
        </Modal>
      )}

      {modal === "batchCreate" && (
        <Modal title={t("batchCreate.title")} onClose={() => setModal(null)}>
          <div className="form-grid">
            <NumberField label={t("batchCreate.count")} value={batchDraft.count} onChange={(count) => setBatchDraft({ ...batchDraft, count })} />
            <TextField label={t("batchCreate.proxy")} value={batchDraft.proxy} placeholder="socks5://user:pass@host:port" onChange={(proxy) => setBatchDraft({ ...batchDraft, proxy })} />
          </div>
          <label className="check-label standalone"><input type="checkbox" checked={batchDraft.autoLocale} onChange={(event) => setBatchDraft({ ...batchDraft, autoLocale: event.target.checked })} />{t("batchCreate.autoLocale")}</label>
          <label className="check-label standalone"><input type="checkbox" checked={batchDraft.autoTimezone} onChange={(event) => setBatchDraft({ ...batchDraft, autoTimezone: event.target.checked })} />{t("batchCreate.autoTimezone")}</label>
          <div className="dialog-actions">
            <button className="btn" onClick={() => setModal(null)}>{t("common.cancel")}</button>
            <button className="btn primary" onClick={() => void runBatchCreate()}>{t("batchCreate.create")}</button>
          </div>
        </Modal>
      )}

      {modal === "batchMove" && (
        <Modal title={selectedIds.length > 1 ? t("batchMove.title") : t("batchMove.single")} onClose={() => setModal(null)}>
          <label>{t("batchMove.target")}
            <select className="input" value={moveProjectId} onChange={(event) => setMoveProjectId(event.target.value)}>
              {state.projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}
            </select>
          </label>
          <div className="dialog-actions">
            <button className="btn" onClick={() => setModal(null)}>{t("common.cancel")}</button>
            <button className="btn primary" onClick={() => void runBatchMove()}>{t("common.save")}</button>
          </div>
        </Modal>
      )}

      {modal === "automation" && (
        <Modal title={t("auto.title")} onClose={() => setModal(null)} dialogClassName="wide-dialog">
          <div className="about-body automation-body">
            <p>{t("auto.intro")}</p>
            <div className="api-meta">
              <CopyRow label={t("auto.apiAddr")} value={automation?.apiBase || (state.apiPort ? `http://127.0.0.1:${state.apiPort}` : t("auto.starting"))} onCopy={copyText} />
              <CopyRow label={t("auto.token")} value={automation?.token || "—"} onCopy={copyText} />
            </div>
            <div className="api-section">
              <h4>{t("auto.httpApi")}</h4>
              <ul className="api-list">
                <li><code>GET /api/v1/profiles</code></li>
                <li><code>GET /api/v1/browser/start?id=&lt;id&gt;&amp;token=&lt;token&gt;</code> → ws / debugPort</li>
                <li><code>GET /api/v1/browser/stop?id=&lt;id&gt;&amp;token=&lt;token&gt;</code></li>
                <li><code>GET /api/v1/browser/active</code></li>
              </ul>
            </div>
            <div className="api-section">
              <h4>{t("auto.runningEndpoints")} ({Object.keys(state.endpoints || {}).length})</h4>
              {Object.values(state.endpoints || {}).length ? (
                <table className="endpoint-table">
                  <thead><tr><th>{t("auto.envCol")}</th><th>{t("auto.portCol")}</th><th>WebSocket</th></tr></thead>
                  <tbody>
                    {Object.values(state.endpoints).map((ep) => {
                      const profile = state.profiles.find((p) => p.id === ep.profileId);
                      return (
                        <tr key={ep.profileId}>
                          <td>{profile?.name || ep.profileId}</td>
                          <td>{ep.debugPort}</td>
                          <td className="ws-cell">
                            <code>{ep.wsEndpoint || t("auto.discovering")}</code>
                            {ep.wsEndpoint && <button className="btn mini-btn" onClick={() => void copyText(ep.wsEndpoint)}>{t("common.copy")}</button>}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              ) : <p className="muted">{t("auto.noRunning")}</p>}
            </div>
            <div className="api-section">
              <h4>{t("auto.examples")}</h4>
              <pre>{`# Python — Selenium attaches to the running fingerprint browser
import requests
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

api = "${automation?.apiBase || "http://127.0.0.1:50327"}"
token = "${automation?.token || "<token>"}"
r = requests.get(f"{api}/api/v1/browser/start",
                 params={"id": "<profileId>", "token": token}).json()
opts = Options()
opts.add_experimental_option("debuggerAddress", r["data"]["debuggerAddress"])
driver = webdriver.Chrome(options=opts)
driver.get("https://browserleaks.com")`}</pre>
              <pre>{`// Node — Puppeteer / Playwright over the ws endpoint
const res = await fetch(\`\${api}/api/v1/browser/start?id=\${id}&token=\${token}\`);
const { data } = await res.json();
const browser = await puppeteer.connect({ browserWSEndpoint: data.ws });`}</pre>
            </div>
          </div>
          <div className="dialog-actions">
            <button className="btn" onClick={() => void loadAutomation()}>{t("common.refresh")}</button>
            <button className="btn primary" onClick={() => setModal(null)}>{t("common.ok")}</button>
          </div>
        </Modal>
      )}

      {modal === "proxyPool" && (
        <Modal title={t("pool.title")} onClose={() => setModal(null)} dialogClassName="wide-dialog">
          <div className="proxy-pool">
            <div className="proxy-add">
              <div className="form-grid compact-form">
                <label className="mini-select">{t("pool.protocol")}
                  <select className="input" value={proxyDraft.protocol} onChange={(e) => setProxyDraft({ ...proxyDraft, protocol: e.target.value })}>
                    {["SOCKS5", "HTTP", "HTTPS", "SOCKS4"].map((p) => <option key={p} value={p}>{p}</option>)}
                  </select>
                </label>
                <TextField label={t("pool.host")} value={proxyDraft.host} onChange={(host) => setProxyDraft({ ...proxyDraft, host })} />
                <TextField label={t("pool.port")} value={proxyDraft.port} onChange={(port) => setProxyDraft({ ...proxyDraft, port })} />
                <TextField label={t("pool.username")} value={proxyDraft.username} onChange={(username) => setProxyDraft({ ...proxyDraft, username })} />
                <TextField label={t("pool.password")} value={proxyDraft.password} onChange={(password) => setProxyDraft({ ...proxyDraft, password })} />
                <div className="proxy-add-actions">
                  <button className="btn primary" onClick={() => void addProxyEntry()}>{t("pool.add")}</button>
                </div>
              </div>
              <div className="proxy-bulk">
                <label>{t("pool.bulkLabel")}</label>
                <textarea className="textarea" rows={3} value={proxyBulk} onChange={(e) => setProxyBulk(e.target.value)} placeholder={"1.2.3.4:1080:user:pass\nsocks5://5.6.7.8:1080"} />
                <button className="btn" onClick={() => void importProxyBulk()}>{t("pool.bulkImport")}</button>
              </div>
            </div>
            <div className="proxy-table-wrap">
              <table className="endpoint-table">
                <thead><tr><th>{t("pool.name")}</th><th>{t("pool.protocol")}</th><th>{t("pool.exitIp")}</th><th>{t("pool.status")}</th><th>{t("pool.actions")}</th></tr></thead>
                <tbody>
                  {state.proxies.length ? state.proxies.map((p) => (
                    <tr key={p.id}>
                      <td title={p.url}>{p.name || `${p.host}:${p.port}`}</td>
                      <td>{p.protocol}</td>
                      <td>{p.lastIp ? `${p.lastCountry || "?"} · ${p.lastIp}` : "—"}</td>
                      <td><span className={`proxy-status ${p.status}`}>{p.status === "ok" ? t("pool.statusOk") : p.status === "fail" ? t("pool.statusFail") : t("pool.statusUnknown")}</span></td>
                      <td className="proxy-row-actions">
                        <button className="btn mini-btn" disabled={proxyBusyId === p.id} onClick={() => void checkProxyEntry(p.id)}>{proxyBusyId === p.id ? t("pool.checking") : t("pool.check")}</button>
                        <button className="btn mini-btn" onClick={() => void assignProxyToSelected(p.id)}>{t("pool.bind")}</button>
                        <button className="btn mini-btn danger" onClick={() => void deleteProxyEntry(p.id)}>{t("pool.delete")}</button>
                      </td>
                    </tr>
                  )) : <tr><td colSpan={5} className="muted">{t("pool.empty")}</td></tr>}
                </tbody>
              </table>
            </div>
            <p className="muted">{t("pool.bindHint")}</p>
          </div>
          <div className="dialog-actions">
            <button className="btn primary" onClick={() => setModal(null)}>{t("common.done")}</button>
          </div>
        </Modal>
      )}

      {modal === "about" && (
        <Modal title={`${t("about.title")} · Abencat Browser`} onClose={() => setModal(null)}>
          <div className="about-body">
            <p><strong>阿笨猫指纹浏览器 · Abencat Browser</strong> — {t("about.body")}</p>
            <ul className="about-list">
              <li>{t("about.list1")}</li>
              <li>{t("about.list2")}</li>
              <li>{t("about.list3")}</li>
            </ul>
          </div>
          <div className="dialog-actions">
            <button className="btn primary" onClick={() => setModal(null)}>{t("common.ok")}</button>
          </div>
        </Modal>
      )}

      {modal === "security" && (
        <SecurityModal
          status={security}
          onClose={() => setModal(null)}
          onSetMaster={setMasterPassword}
          onRemoveMaster={removeMasterPassword}
        />
      )}

      {contextMenu && (
        <div
          className="context-menu"
          style={{
            left: Math.min(contextMenu.x, window.innerWidth - (contextMenu.kind === "project" ? 170 : 180) - 12),
            top: Math.min(contextMenu.y, window.innerHeight - (contextMenu.kind === "project" ? 108 : 220) - 12),
          }}
          onClick={(event) => event.stopPropagation()}
          onContextMenu={(event) => event.preventDefault()}
        >
          {contextMenu.kind === "project" ? (
            <>
              <button onClick={() => { void selectProject(contextMenu.id); setContextMenu(null); }}>{t("ctx.switch")}</button>
              <button onClick={() => { void renameProjectById(contextMenu.id); setContextMenu(null); }}>{t("ctx.rename")}</button>
              <button className="danger" onClick={() => { void deleteProjectById(contextMenu.id); setContextMenu(null); }}>{t("ctx.delete")}</button>
            </>
          ) : (
            <>
              <button onClick={() => { runningSet.has(contextMenu.id) ? void stop(contextMenu.id) : void launch(contextMenu.id); setContextMenu(null); }}>
                {runningSet.has(contextMenu.id) ? t("ctx.stop") : t("ctx.start")}
              </button>
              <button onClick={() => { const profile = state.profiles.find((item) => item.id === contextMenu.id); if (profile) openEditor(profile); setContextMenu(null); }}>{t("ctx.edit")}</button>
              <button onClick={() => { cloneById(contextMenu.id); setContextMenu(null); }}>{t("ctx.clone")}</button>
              <button onClick={() => { moveProfileById(contextMenu.id); setContextMenu(null); }}>{t("ctx.moveTo")}</button>
              <button onClick={() => { void chooseScriptsForTargets([contextMenu.id]); setContextMenu(null); }}>{t("ctx.loadScripts")}</button>
              <button onClick={() => { openFolderById(contextMenu.id); setContextMenu(null); }}>{t("ctx.openFolder")}</button>
              <button className="danger" onClick={() => { void removeProfileById(contextMenu.id); setContextMenu(null); }}>{t("ctx.delete")}</button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
