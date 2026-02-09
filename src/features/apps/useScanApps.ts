import { useCallback, useEffect, useMemo, useState } from "react";
import type { AppRecord, ScanProgress } from "../../types/apps";
import { listenScanEvents, startScanApps } from "../../lib/tauri/apps";

export function useScanApps() {
  const [rows, setRows] = useState<AppRecord[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [progress, setProgress] = useState<ScanProgress | null>(null);

  const stats = useMemo(() => {
    const categories: Record<string, number> = {};
    const KIND_MAP: Record<string, string> = {
      program: "InstallLocation",
      appDataLocal: "AppData",
      appDataRoaming: "AppData",
      appDataLocalLow: "AppData",
      programData: "ProgramData",
    };

    rows.forEach((app) => {
      app.breakdown.forEach((b) => {
        const mappedKind = KIND_MAP[b.kind] || "Unknown";
        categories[mappedKind] = (categories[mappedKind] || 0) + b.bytes;
      });
    });

    const categoryData = Object.entries(categories)
      .filter(([_, value]) => value > 0)
      .map(([name, value]) => ({ name, value }))
      .sort((a, b) => b.value - a.value);

    const topAppsData = [...rows]
      .sort((a, b) => b.totalBytes - a.totalBytes)
      .slice(0, 10)
      .map((app) => {
        const entry: Record<string, any> = { name: app.name, totalBytes: app.totalBytes };
        app.breakdown.forEach((b) => {
          const mappedKind = KIND_MAP[b.kind] || "Unknown";
          entry[mappedKind] = (entry[mappedKind] || 0) + b.bytes;
        });
        return entry;
      });

    return { categoryData, topAppsData };
  }, [rows]);

  const scan = useCallback(async () => {
    if (isScanning) return;
    setIsScanning(true);
    setProgress(null);
    setRows([]);
    setExpanded({});

    try {
      await startScanApps();
    } catch {
      setIsScanning(false);
    }
  }, [isScanning]);

  const toggleExpanded = useCallback((id: string) => {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      try {
        unlisten = await listenScanEvents({
          onProgress: (p) => setProgress(p),
          onRecord: (rec) => {
            setRows((prev) => {
              if (prev.some((r) => r.id === rec.id)) return prev;
              return [...prev, rec];
            });
          },
          onDone: () => setIsScanning(false),
        });
      } catch {
        unlisten = null;
      }
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  return {
    expanded,
    isScanning,
    progress,
    rows,
    scan,
    setExpanded,
    setIsScanning,
    setProgress,
    setRows,
    stats,
    toggleExpanded,
  };
}
