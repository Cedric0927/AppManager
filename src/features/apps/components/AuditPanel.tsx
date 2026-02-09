import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown } from "lucide-react";
import type { AuditOverview } from "../../../types/apps";

export function AuditPanel(props: {
  audit: AuditOverview | null;
  auditLoading: boolean;
  auditOpen: boolean;
  auditSizes: Record<string, number>;
  formatBytes: (bytes: number) => string;
  loadAudit: () => void | Promise<void>;
  measureAuditFolder: (kind: string, folder: string) => void | Promise<void>;
  setAuditOpen: (value: boolean | ((prev: boolean) => boolean)) => void;
}) {
  const {
    audit,
    auditLoading,
    auditOpen,
    auditSizes,
    formatBytes,
    loadAudit,
    measureAuditFolder,
    setAuditOpen,
  } = props;

  return (
    <div className="flex flex-col gap-3 rounded-2xl bg-zinc-900/30 p-4 ring-1 ring-white/10">
      <div className="flex items-center justify-between gap-3">
        <div className="text-sm font-medium text-zinc-200">深度发现 (扫描未知数据)</div>
        <button
          type="button"
          onClick={loadAudit}
          disabled={auditLoading}
          className="inline-flex h-9 items-center justify-center rounded-xl bg-zinc-950/40 px-3 text-xs font-medium text-zinc-100 ring-1 ring-white/10 transition enabled:hover:bg-white/5 disabled:opacity-60"
        >
          {auditLoading ? "正在深度扫描…" : "开始深度扫描"}
        </button>
      </div>

      {audit ? (
        <div className="flex flex-col gap-3">
          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
              <div className="text-xs text-zinc-500">已识别软件</div>
              <div className="mt-1 text-sm tabular-nums text-zinc-100">{audit.appCount} 个</div>
            </div>
            <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
              <div className="text-xs text-zinc-500">无法确定的占用</div>
              <div className="mt-1 text-sm tabular-nums text-zinc-100">
                {audit.unknownProgramSizeCount} 处
              </div>
            </div>
          </div>

          <button
            type="button"
            onClick={() => setAuditOpen((v) => !v)}
            className="flex items-center justify-between rounded-xl bg-zinc-950/40 px-3 py-2 text-left text-xs text-zinc-300 ring-1 ring-white/10 transition hover:bg-white/5"
          >
            <span>{auditOpen ? "收起详细报告" : "查看详细发现报告"}</span>
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
                    <div className="text-xs font-medium text-zinc-200">目录匹配覆盖率</div>
                    <div className="mt-2 grid grid-cols-1 gap-2">
                      {audit.roots.map((r) => (
                        <div
                          key={r.kind}
                          className="flex items-center justify-between gap-3 text-xs text-zinc-400"
                        >
                          <div className="truncate">{r.kind}</div>
                          <div className="tabular-nums">
                            已关联 {r.assignedFolders} / 待确定 {r.unassignedFolders}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  {audit.duplicateInstallLocations.length > 0 ? (
                    <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                      <div className="text-xs font-medium text-zinc-200">
                        共享安装目录（多个软件共用）
                      </div>
                      <div className="mt-2 flex flex-col gap-2">
                        {audit.duplicateInstallLocations.slice(0, 12).map((d) => (
                          <div key={d.installDir} className="flex flex-col gap-1">
                            <div
                              className="truncate font-mono text-xs text-zinc-500"
                              title={d.installDir}
                            >
                              {d.installDir}
                            </div>
                            <div className="text-xs text-zinc-400">
                              涉及：{d.apps.slice(0, 6).join("、")}
                              {d.apps.length > 6 ? ` 等 ${d.apps.length} 个` : ""}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  ) : null}

                  {audit.unassignedFolders.length > 0 ? (
                    <div className="rounded-xl bg-zinc-950/40 p-3 ring-1 ring-white/10">
                      <div className="text-xs font-medium text-zinc-200">未关联到软件的文件夹 (前 40 个)</div>
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
                                <div
                                  className="truncate font-mono text-[11px] text-zinc-500"
                                  title={u.path}
                                >
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
                                    分析大小
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
        <div className="text-xs text-zinc-500 leading-relaxed">
          除了已识别的软件，系统里还潜伏着一些“无主”文件夹。点击上方按钮进行深度扫描，找出那些躲在角落里偷吃空间的未知数据。
        </div>
      )}
    </div>
  );
}
