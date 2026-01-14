import { invoke } from "@tauri-apps/api/core";
import type {
  DrsProfile,
  DrsApplication,
  RunningProcess,
  FocusApplication,
  BlacklistResult,
  NvApiStatus,
} from "../types";

export async function getProfiles(): Promise<DrsProfile[]> {
  return invoke<DrsProfile[]>("get_profiles");
}

export async function getAllApplications(): Promise<DrsApplication[]> {
  return invoke<DrsApplication[]>("get_all_applications");
}

export async function getRunningProcesses(): Promise<RunningProcess[]> {
  return invoke<RunningProcess[]>("get_running_processes");
}

export async function getFocusApplication(): Promise<FocusApplication | null> {
  return invoke<FocusApplication | null>("get_focus_application");
}

export async function createProfile(
  executable: string,
  profileName: string
): Promise<void> {
  return invoke("create_profile", { executable, profileName });
}

export async function blacklistApplication(
  executable: string
): Promise<BlacklistResult> {
  return invoke<BlacklistResult>("blacklist_application", { executable });
}

export async function unblacklistApplication(
  executable: string
): Promise<BlacklistResult> {
  return invoke<BlacklistResult>("unblacklist_application", { executable });
}

export async function checkNvApiStatus(): Promise<NvApiStatus> {
  return invoke<NvApiStatus>("check_nvapi_status");
}

export async function reloadSettings(): Promise<void> {
  return invoke("reload_settings");
}
