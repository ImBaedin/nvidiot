// TypeScript types matching Rust structs

export interface DrsProfile {
  name: string;
  isPredefined: boolean;
  applicationCount: number;
}

export interface DrsApplication {
  name: string;
  executable: string;
  profileName: string;
  isPredefined: boolean;
  isBlacklisted: boolean;
}

export interface RunningProcess {
  processName: string;
  windowTitle: string;
  processId: number;
  executablePath: string | null;
  hasDrsProfile: boolean;
  profileName: string | null;
  isBlacklisted: boolean | null;
}

export interface FocusApplication {
  processName: string;
  windowTitle: string;
  processId: number;
  isInDrs: boolean;
  profileName: string | null;
  isBlacklisted: boolean | null;
}

export interface BlacklistResult {
  success: boolean;
  executable: string;
  message: string;
}

export interface NvApiStatus {
  available: boolean;
  error: string | null;
}
