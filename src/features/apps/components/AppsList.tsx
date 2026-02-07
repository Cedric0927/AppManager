import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown } from "lucide-react";
import type { AppRecord } from "../../../types/apps";

export function AppsList(props: {
  expanded: Record<string, boolean>;
  filtered: AppRecord[];
  formatBytes: (bytes: number) => string;
  rows: AppRecord[];
  toggleExpanded: (id: string) => void;
}) {
  const { expanded, filtered, formatBytes, rows, toggleExpanded } = props;

  return (
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
                    onClick={() => toggleExpanded(r.id)}
                    className="grid w-full grid-cols-[1fr_auto] items-center gap-4 px-5 py-4 text-left transition hover:bg-white/5"
                  >
                    <div className="min-w-0">
                      <div className="truncate text-sm font-medium text-zinc-100">{r.name}</div>
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
  );
}
