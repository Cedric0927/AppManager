import { HardDrive, Info } from "lucide-react";
import { useEffect, useState } from "react";
import { getDiskInfo } from "../../../lib/tauri/apps";
import type { DiskInfo } from "../../../types/apps";

interface DiskOverviewProps {
  formatBytes: (bytes: number) => string;
}

export function DiskOverview({ formatBytes }: DiskOverviewProps) {
  const [disks, setDisks] = useState<DiskInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadDisks = async () => {
      try {
        const data = await getDiskInfo();
        // 优先显示 C 盘
        const sorted = [...data].sort((a, b) => {
          if (a.mountPoint === "C:\\") return -1;
          if (b.mountPoint === "C:\\") return 1;
          return 0;
        });
        setDisks(sorted);
      } catch (err) {
        console.error("Failed to load disks", err);
      } finally {
        setLoading(false);
      }
    };
    loadDisks();
  }, []);

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 animate-pulse">
        <div className="h-32 rounded-2xl bg-zinc-900/50 ring-1 ring-white/10" />
        <div className="h-32 rounded-2xl bg-zinc-900/50 ring-1 ring-white/10" />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2 text-sm font-medium text-zinc-400">
        <HardDrive className="h-4 w-4" />
        <span>磁盘空间状态</span>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {disks.map((disk) => {
          const usedSpace = disk.totalSpace - disk.availableSpace;
          const usedPercent = Math.round((usedSpace / disk.totalSpace) * 100);
          const isLowSpace = usedPercent > 90;

          return (
            <div 
              key={disk.mountPoint}
              className="group relative flex flex-col gap-3 rounded-2xl bg-zinc-900/40 p-4 ring-1 ring-white/10 transition hover:bg-zinc-900/60"
            >
              <div className="flex items-start justify-between">
                <div className="flex flex-col">
                  <span className="text-sm font-bold text-zinc-100">
                    本地磁盘 ({disk.mountPoint.replace("\\", "")})
                  </span>
                  <span className="text-[10px] text-zinc-500 uppercase tracking-wider">
                    {disk.name || "固定磁盘"}
                  </span>
                </div>
                <div className={`flex h-8 w-8 items-center justify-center rounded-lg ${isLowSpace ? 'bg-red-500/10 text-red-500' : 'bg-zinc-950/40 text-zinc-400'}`}>
                  <HardDrive className="h-4 w-4" />
                </div>
              </div>

              <div className="flex flex-col gap-1.5">
                <div className="flex justify-between text-[11px]">
                  <span className="text-zinc-500">
                    已用 {formatBytes(usedSpace)}
                  </span>
                  <span className={isLowSpace ? "text-red-400 font-medium" : "text-zinc-400"}>
                    {usedPercent}%
                  </span>
                </div>
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-zinc-950/40 ring-1 ring-white/5">
                  <div 
                    className={`h-full transition-all duration-500 ${isLowSpace ? 'bg-red-500' : 'bg-indigo-500'}`}
                    style={{ width: `${usedPercent}%` }}
                  />
                </div>
                <div className="flex justify-between text-[10px] text-zinc-500">
                  <span>共 {formatBytes(disk.totalSpace)}</span>
                  <span>剩余 {formatBytes(disk.availableSpace)}</span>
                </div>
              </div>
              
              {isLowSpace && (
                <div className="mt-1 flex items-center gap-1.5 rounded-lg bg-red-500/5 px-2 py-1 text-[10px] text-red-400 ring-1 ring-red-500/10">
                  <Info className="h-3 w-3" />
                  <span>空间极低，建议清理</span>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
