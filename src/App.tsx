import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown, HardDrive, Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

type AppBreakdownEntry = {
  kind: string;
  label: string;
  bytes: number;
  paths: string[];
};

type AppRecord = {
  id: string;
  name: string;
  publisher?: string;
  totalBytes: number;
  breakdown: AppBreakdownEntry[];
};

type ScanProgress = {
  phase: string;
  current: number;
  total: number;
  message: string;
};

type AuditRootSummary = {
  kind: string;
  assignedFolders: number;
  unassignedFolders: number;
};

type AuditDuplicateInstallLocation = {
  installDir: string;
  apps: string[];
};

type AuditUnassignedFolder = {
  kind: string;
  folder: string;
  path: string;
};

type AuditOverview = {
  appCount: number;
  unknownProgramSizeCount: number;
  roots: AuditRootSummary[];
  duplicateInstallLocations: AuditDuplicateInstallLocation[];
  unassignedFolders: AuditUnassignedFolder[];
};

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  const digits = unitIndex === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[unitIndex]}`;
}

function App() {
  const [query, setQuery] = useState("");
  const [rows, setRows] = useState<AppRecord[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [audit, setAudit] = useState<AuditOverview | null>(null);
  const [auditLoading, setAuditLoading] = useState(false);
  const [auditSizes, setAuditSizes] = useState<Record<string, number>>({});
  const [auditOpen, setAuditOpen] = useState(false);

  const filtered = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    const base = normalized
      ? rows.filter((r) => {
          const haystack = `${r.name} ${r.publisher ?? ""}`.toLowerCase();
          return haystack.includes(normalized);
        })
      : rows;

    return [...base].sort((a, b) => b.totalBytes - a.totalBytes);
  }, [query, rows]);

  async function scan() {
    if (isScanning) return;
    setIsScanning(true);
    setProgress(null);
    setRows([]);
    setExpanded({});
    setAudit(null);
    setAuditSizes({});
    try {
      await invoke("start_scan_apps");
    } catch {
      setIsScanning(false);
    }
  }

  async function loadAudit() {
    if (auditLoading) return;
    setAuditLoading(true);
    try {
      const result = (await invoke("get_audit_overview")) as AuditOverview;
      setAudit(result);
      setAuditOpen(true);
    } finally {
      setAuditLoading(false);
    }
  }

  async function measureAuditFolder(kind: string, folder: string) {
    const key = `${kind}:${folder}`;
    if (auditSizes[key] != null) return;
    const bytes = (await invoke("measure_audit_folder_size", {
      kind,
      folder,
    })) as number;
    setAuditSizes((prev) => ({ ...prev, [key]: bytes }));
  }

  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;
    let unlistenResult: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;

    (async () => {
      unlistenProgress = await listen<ScanProgress>("scan_progress", (event) => {
        setProgress(event.payload);
      });
      unlistenResult = await listen<AppRecord>("scan_result", (event) => {
        const rec = event.payload;
        setRows((prev) => {
          if (prev.some((r) => r.id === rec.id)) return prev;
          return [...prev, rec];
        });
      });
      unlistenDone = await listen("scan_done", () => {
        setIsScanning(false);
      });
    })();

    return () => {
      unlistenProgress?.();
      unlistenResult?.();
      unlistenDone?.();
    };
  }, []);

  return (
    <div className="min-h-dvh bg-zinc-950 text-zinc-100">
      <div className="mx-auto flex w-full max-w-5xl flex-col gap-6 px-6 py-8">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-zinc-900 ring-1 ring-white/10">
              <HardDrive className="h-5 w-5 text-zinc-200" />
            </div>
            <div className="flex flex-col">
              <div className="text-xl font-semibold tracking-tight">AppManager</div>
              <div className="text-sm text-zinc-400">
                扫描并归因安装目录、AppData 与 ProgramData 占用
              </div>
            </div>
          </div>

          <button
            type="button"
            onClick={scan}
            disabled={isScanning}
            className="inline-flex h-10 items-center justify-center rounded-xl bg-white px-4 text-sm font-medium text-zinc-950 shadow-sm ring-1 ring-white/10 transition enabled:hover:bg-zinc-200 disabled:opacity-60"
          >
            {isScanning ? "扫描中…" : "开始扫描"}
          </button>
        </div>

        <div className="flex flex-col gap-3 rounded-2xl bg-zinc-900/30 p-4 ring-1 ring-white/10">
          <div className="flex items-center justify-between gap-3">
            <div className="text-sm font-medium text-zinc-200">校验漏掉 / 重复</div>
            <button
              type="button"
              onClick={loadAudit}
              disabled={auditLoading}
              className="inline-flex h-9 items-center justify-center rounded-xl bg-zinc-950/40 px-3 text-xs font-medium text-zinc-100 ring-1 ring-white/10 transition enabled:hover:bg-white/5 disabled:opacity-60"
            >
              {auditLoading ? "校验中…" : "运行校验"}
            </button>
          </div>

          {audit ? (
            <div className="flex flex-col gap-3">
              <div className="grid grid-cols-2 gap-3">
                <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                  <div className="text-xs text-zinc-500">应用条目数</div>
                  <div className="mt-1 text-sm tabular-nums text-zinc-100">{audit.appCount}</div>
                </div>
                <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                  <div className="text-xs text-zinc-500">程序本身未知数</div>
                  <div className="mt-1 text-sm tabular-nums text-zinc-100">
                    {audit.unknownProgramSizeCount}
                  </div>
                </div>
              </div>

              <button
                type="button"
                onClick={() => setAuditOpen((v) => !v)}
                className="flex items-center justify-between rounded-xl bg-zinc-950/40 px-3 py-2 text-left text-xs text-zinc-300 ring-1 ring-white/10 transition hover:bg-white/5"
              >
                <span>展开校验详情</span>
                <ChevronDown
                  className={[
                    "h-4 w-4 text-zinc-400 transition-transform",
                    auditOpen ? "rotate-180" : "rotate-0",
                  ].join(" ")}
                />
              </button>

              <AnimatePresence initial={false}>
                {auditOpen ? (
                  <motion.div
                    key="audit-details"
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="overflow-hidden"
                  >
                    <div className="flex flex-col gap-3">
                      <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                        <div className="text-xs font-medium text-zinc-200">归因覆盖（目录数）</div>
                        <div className="mt-2 grid grid-cols-1 gap-2">
                          {audit.roots.map((r) => (
                            <div
                              key={r.kind}
                              className="flex items-center justify-between gap-3 text-xs text-zinc-400"
                            >
                              <div className="truncate">{r.kind}</div>
                              <div className="tabular-nums">
                                已归因 {r.assignedFolders} / 未归因 {r.unassignedFolders}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>

                      {audit.duplicateInstallLocations.length > 0 ? (
                        <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                          <div className="text-xs font-medium text-zinc-200">
                            重复安装目录（可能为组件/共享目录）
                          </div>
                          <div className="mt-2 flex flex-col gap-2">
                            {audit.duplicateInstallLocations.slice(0, 12).map((d) => (
                              <div key={d.installDir} className="flex flex-col gap-1">
                                <div className="truncate font-mono text-xs text-zinc-500" title={d.installDir}>
                                  {d.installDir}
                                </div>
                                <div className="text-xs text-zinc-400">
                                  {d.apps.slice(0, 6).join("、")}
                                  {d.apps.length > 6 ? ` 等 ${d.apps.length} 个` : ""}
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      ) : null}

                      {audit.unassignedFolders.length > 0 ? (
                        <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                          <div className="text-xs font-medium text-zinc-200">未归因目录（抽样）</div>
                          <div className="mt-2 flex flex-col gap-2">
                            {audit.unassignedFolders.slice(0, 40).map((u) => {
                              const key = `${u.kind}:${u.folder}`;
                              const bytes = auditSizes[key];
                              return (
                                <div
                                  key={u.path}
                                  className="flex items-start justify-between gap-3 rounded-lg bg-black/20 px-2 py-2"
                                >
                                  <div className="min-w-0">
                                    <div className="text-xs text-zinc-300">
                                      {u.kind} / {u.folder}
                                    </div>
                                    <div className="truncate font-mono text-[11px] text-zinc-500" title={u.path}>
                                      {u.path}
                                    </div>
                                  </div>
                                  <div className="flex shrink-0 items-center gap-2">
                                    {bytes != null ? (
                                      <div className="text-xs tabular-nums text-zinc-200">
                                        {formatBytes(bytes)}
                                      </div>
                                    ) : (
                                      <button
                                        type="button"
                                        onClick={() => measureAuditFolder(u.kind, u.folder)}
                                        className="inline-flex h-8 items-center justify-center rounded-lg bg-zinc-950/40 px-2 text-[11px] text-zinc-200 ring-1 ring-white/10 transition hover:bg-white/5"
                                      >
                                        计算大小
                                      </button>
                                    )}
                                  </div>
                                </div>
                              );
                            })}
                          </div>
                        </div>
                      ) : null}
                    </div>
                  </motion.div>
                ) : null}
              </AnimatePresence>
            </div>
          ) : (
            <div className="text-xs text-zinc-500">
              用它来判断是否存在“同一安装目录多条记录”、以及有哪些数据目录尚未归因到具体应用。
            </div>
          )}
        </div>

        <div className="flex flex-col gap-3 rounded-2xl bg-zinc-900/50 p-4 ring-1 ring-white/10">
          <div className="flex items-center gap-2 text-sm text-zinc-300">
            <Search className="h-4 w-4" />
            <span>搜索软件</span>
            {rows.length > 0 ? (
              <span className="ml-auto text-xs text-zinc-500">
                已识别 {rows.length} 个
              </span>
            ) : null}
          </div>
          {isScanning ? (
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between gap-3 text-xs text-zinc-500">
                <div className="truncate">{progress?.message ?? "准备扫描…"}</div>
                {progress && progress.total > 0 ? (
                  <div className="tabular-nums">
                    {progress.current}/{progress.total}
                  </div>
                ) : null}
              </div>
              <div className="h-2 w-full overflow-hidden rounded-full bg-zinc-950/40 ring-1 ring-white/10">
                <div
                  className="h-full bg-white/70 transition-[width]"
                  style={{
                    width:
                      progress && progress.total > 0
                        ? `${Math.min(100, (progress.current / progress.total) * 100)}%`
                        : "20%",
                  }}
                />
              </div>
            </div>
          ) : null}
          <input
            value={query}
            onChange={(e) => setQuery(e.currentTarget.value)}
            placeholder="按名称或厂商过滤…"
            className="h-11 w-full rounded-xl bg-zinc-950/40 px-4 text-sm text-zinc-100 placeholder:text-zinc-500 ring-1 ring-white/10 outline-none focus:ring-2 focus:ring-white/20"
          />
        </div>

        <div className="overflow-hidden rounded-2xl bg-zinc-900/30 ring-1 ring-white/10">
          <div className="grid grid-cols-[1fr_auto] gap-4 border-b border-white/10 px-5 py-3 text-xs font-medium text-zinc-400">
            <div>应用</div>
            <div>占用</div>
          </div>

          <AnimatePresence initial={false}>
            {filtered.length === 0 ? (
              <motion.div
                key="empty"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="px-5 py-10 text-sm text-zinc-400"
              >
                {rows.length === 0 ? "点击“开始扫描”获取列表。" : "没有匹配结果。"}
              </motion.div>
            ) : (
              <motion.div
                key="list"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="divide-y divide-white/10"
              >
                {filtered.map((r) => {
                  const isOpen = Boolean(expanded[r.id]);

                  return (
                    <div key={r.id}>
                      <button
                        type="button"
                        onClick={() =>
                          setExpanded((prev) => ({ ...prev, [r.id]: !prev[r.id] }))
                        }
                        className="grid w-full grid-cols-[1fr_auto] items-center gap-4 px-5 py-4 text-left transition hover:bg-white/5"
                      >
                        <div className="min-w-0">
                          <div className="truncate text-sm font-medium text-zinc-100">
                            {r.name}
                          </div>
                          {r.publisher ? (
                            <div className="truncate text-xs text-zinc-500">{r.publisher}</div>
                          ) : null}
                        </div>
                        <div className="flex items-center gap-3">
                          <div className="text-sm tabular-nums text-zinc-200">
                            {formatBytes(r.totalBytes)}
                          </div>
                          <ChevronDown
                            className={[
                              "h-4 w-4 text-zinc-400 transition-transform",
                              isOpen ? "rotate-180" : "rotate-0",
                            ].join(" ")}
                          />
                        </div>
                      </button>

                      <AnimatePresence initial={false}>
                        {isOpen ? (
                          <motion.div
                            key="details"
                            initial={{ height: 0, opacity: 0 }}
                            animate={{ height: "auto", opacity: 1 }}
                            exit={{ height: 0, opacity: 0 }}
                            transition={{ duration: 0.2 }}
                            className="overflow-hidden border-t border-white/10 bg-black/20"
                          >
                            <div className="flex flex-col gap-3 px-5 py-4">
                              {r.breakdown.map((b) => (
                                <div
                                  key={b.kind}
                                  className="flex flex-col gap-1 rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10"
                                >
                                  <div className="flex items-center justify-between gap-3">
                                    <div className="text-sm text-zinc-200">{b.label}</div>
                                    <div className="text-sm tabular-nums text-zinc-100">
                                      {formatBytes(b.bytes)}
                                    </div>
                                  </div>
                                  {b.paths.length > 0 ? (
                                    <div className="flex flex-col gap-1">
                                      {b.paths.map((p) => (
                                        <div
                                          key={p}
                                          className="truncate font-mono text-xs text-zinc-500"
                                          title={p}
                                        >
                                          {p}
                                        </div>
                                      ))}
                                    </div>
                                  ) : null}
                                </div>
                              ))}
                            </div>
                          </motion.div>
                        ) : null}
                      </AnimatePresence>
                    </div>
                  );
                })}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}

export default App;
