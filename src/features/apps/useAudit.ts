import { useCallback, useState } from "react";
import type { AuditOverview } from "../../types/apps";
import { getAuditOverview, measureAuditFolderSize } from "../../lib/tauri/apps";

export function useAudit() {
  const [audit, setAudit] = useState<AuditOverview | null>(null);
  const [auditLoading, setAuditLoading] = useState(false);
  const [auditSizes, setAuditSizes] = useState<Record<string, number>>({});
  const [auditOpen, setAuditOpen] = useState(false);

  const loadAudit = useCallback(async () => {
    if (auditLoading) return;
    setAuditLoading(true);
    try {
      const result = await getAuditOverview();
      setAudit(result);
      setAuditOpen(true);
    } finally {
      setAuditLoading(false);
    }
  }, [auditLoading]);

  const measureAuditFolder = useCallback(
    async (kind: string, folder: string) => {
    const key = `${kind}:${folder}`;
    if (auditSizes[key] != null) return;
    const bytes = await measureAuditFolderSize(kind, folder);
    setAuditSizes((prev) => ({ ...prev, [key]: bytes }));
    },
    [auditSizes],
  );

  const resetAudit = useCallback(() => {
    setAudit(null);
    setAuditSizes({});
    setAuditOpen(false);
  }, []);

  return {
    audit,
    auditLoading,
    auditOpen,
    auditSizes,
    loadAudit,
    measureAuditFolder,
    resetAudit,
    setAuditOpen,
  };
}
