import { useCallback, useEffect, useState } from "react";
import type { AppRecord, ScanProgress } from "../../types/apps";
import { listenScanEvents, startScanApps } from "../../lib/tauri/apps";

export function useScanApps() {
  const [rows, setRows] = useState<AppRecord[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [progress, setProgress] = useState<ScanProgress | null>(null);

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
    toggleExpanded,
  };
}
