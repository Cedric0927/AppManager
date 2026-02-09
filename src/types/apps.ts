export type AppBreakdownEntry = {
  kind: string;
  label: string;
  bytes: number;
  paths: string[];
};

export type AppRecord = {
  id: string;
  name: string;
  publisher?: string;
  totalBytes: number;
  breakdown: AppBreakdownEntry[];
};

export type ScanProgress = {
  phase: string;
  current: number;
  total: number;
  message: string;
};

export type AuditRootSummary = {
  kind: string;
  assignedFolders: number;
  unassignedFolders: number;
};

export type AuditDuplicateInstallLocation = {
  installDir: string;
  apps: string[];
};

export type AuditUnassignedFolder = {
  kind: string;
  folder: string;
  path: string;
};

export type AuditOverview = {
  appCount: number;
  unknownProgramSizeCount: number;
  roots: AuditRootSummary[];
  duplicateInstallLocations: AuditDuplicateInstallLocation[];
  unassignedFolders: AuditUnassignedFolder[];
};

export type DiskInfo = {
  name: string;
  mountPoint: string;
  totalSpace: number;
  availableSpace: number;
  isRemovable: boolean;
};
