import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppRecord, AuditOverview, ScanProgress } from "../../types/apps";

export type Unlisten = () => void;

export async function startScanApps(): Promise<void> {
  await invoke("start_scan_apps");
}

export async function listenScanEvents(options: {
  onProgress: (progress: ScanProgress) => void;
  onRecord: (record: AppRecord) => void;
  onDone: () => void;
}): Promise<Unlisten> {
  const unlistenProgress = await listen<ScanProgress>("scan_progress", (event) => {
    options.onProgress(event.payload);
  });

  const unlistenResult = await listen<AppRecord>("scan_result", (event) => {
    options.onRecord(event.payload);
  });

  const unlistenDone = await listen("scan_done", () => {
    options.onDone();
  });

  return () => {
    unlistenProgress();
    unlistenResult();
    unlistenDone();
  };
}

export async function getAuditOverview(): Promise<AuditOverview> {
  return (await invoke("get_audit_overview")) as AuditOverview;
}

export async function measureAuditFolderSize(kind: string, folder: string): Promise<number> {
  return (await invoke("measure_audit_folder_size", { kind, folder })) as number;
}
