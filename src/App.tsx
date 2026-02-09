import { HardDrive, Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { AuditPanel } from "./features/apps/components/AuditPanel";
import { AppsList } from "./features/apps/components/AppsList";
import { Dashboard } from "./features/apps/components/Dashboard";
import { DiskOverview } from "./features/apps/components/DiskOverview";
import { useAudit } from "./features/apps/useAudit";
import { useScanApps } from "./features/apps/useScanApps";

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
  const { expanded, isScanning, progress, rows, scan, stats, toggleExpanded } = useScanApps();
  const {
    audit,
    auditLoading,
    auditOpen,
    auditSizes,
    loadAudit,
    measureAuditFolder,
    resetAudit,
    setAuditOpen,
  } = useAudit();

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

  useEffect(() => {
    if (!isScanning) return;
    resetAudit();
  }, [isScanning, resetAudit]);

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
                扫描并分析软件及其关联数据占用的磁盘空间
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

        <DiskOverview formatBytes={formatBytes} />

        <div className="flex flex-col gap-8 lg:flex-row lg:items-start">
          <div className="min-w-0 flex-1 flex flex-col gap-6">
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

            <AppsList
              expanded={expanded}
              filtered={filtered}
              formatBytes={formatBytes}
              rows={rows}
              toggleExpanded={toggleExpanded}
            />

            <AuditPanel
              audit={audit}
              auditLoading={auditLoading}
              auditOpen={auditOpen}
              auditSizes={auditSizes}
              formatBytes={formatBytes}
              loadAudit={loadAudit}
              measureAuditFolder={measureAuditFolder}
              setAuditOpen={setAuditOpen}
            />
          </div>

          <div className="w-full shrink-0 lg:w-[360px]">
            <Dashboard
              stats={stats}
              totalApps={rows.length}
              formatBytes={formatBytes}
              isScanning={isScanning}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
