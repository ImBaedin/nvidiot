import { useState, useEffect, useCallback } from "react";
import "./App.css";
import type {
  RunningProcess,
  DrsApplication,
  FocusApplication,
  NvApiStatus,
} from "./types";
import {
  checkNvApiStatus,
  getRunningProcesses,
  getAllApplications,
  getFocusApplication,
  blacklistApplication,
  unblacklistApplication,
  createProfile,
  reloadSettings,
} from "./utils/invoke";

// Icons as SVG components
const SearchIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.35-4.35" />
  </svg>
);

const RefreshIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
    <path d="M3 3v5h5" />
    <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
    <path d="M16 21h5v-5" />
  </svg>
);

type Tab = "running" | "drs";

function App() {
  const [nvApiStatus, setNvApiStatus] = useState<NvApiStatus | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("running");
  const [searchQuery, setSearchQuery] = useState("");
  const [runningProcesses, setRunningProcesses] = useState<RunningProcess[]>([]);
  const [drsApplications, setDrsApplications] = useState<DrsApplication[]>([]);
  const [focusApp, setFocusApp] = useState<FocusApplication | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [pendingActions, setPendingActions] = useState<Set<string>>(new Set());

  // Check NVAPI status on mount
  useEffect(() => {
    checkNvApiStatus().then(setNvApiStatus).catch(() => {
      setNvApiStatus({ available: false, error: "Failed to connect" });
    });
  }, []);

  // Load data when NVAPI is available
  useEffect(() => {
    if (nvApiStatus?.available) {
      loadData();
    } else {
      setLoading(false);
    }
  }, [nvApiStatus?.available]);

  // Poll focus application every second
  useEffect(() => {
    if (!nvApiStatus?.available) return;

    const pollFocus = async () => {
      try {
        const focus = await getFocusApplication();
        setFocusApp(focus);
      } catch {
        // Ignore polling errors
      }
    };

    pollFocus();
    const interval = setInterval(pollFocus, 1000);
    return () => clearInterval(interval);
  }, [nvApiStatus?.available]);

  // Auto-refresh process list every 10 seconds
  useEffect(() => {
    if (!nvApiStatus?.available) return;

    const interval = setInterval(() => {
      loadRunningProcesses();
    }, 10000);
    return () => clearInterval(interval);
  }, [nvApiStatus?.available]);

  const loadData = async () => {
    setLoading(true);
    try {
      await Promise.all([
        loadRunningProcesses(),
        loadDrsApplications(),
      ]);
    } finally {
      setLoading(false);
    }
  };

  const loadRunningProcesses = async () => {
    try {
      const processes = await getRunningProcesses();
      setRunningProcesses(processes);
    } catch (e) {
      console.error("Failed to load processes:", e);
    }
  };

  const loadDrsApplications = async () => {
    try {
      const apps = await getAllApplications();
      setDrsApplications(apps);
    } catch (e) {
      console.error("Failed to load DRS applications:", e);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await reloadSettings();
      await loadData();
    } finally {
      setRefreshing(false);
    }
  };

  const handleToggleBlacklist = useCallback(async (executable: string, isCurrentlyBlacklisted: boolean | null) => {
    setPendingActions(prev => new Set(prev).add(executable));
    try {
      if (isCurrentlyBlacklisted) {
        await unblacklistApplication(executable);
      } else {
        await blacklistApplication(executable);
      }
      // Reload data to reflect changes
      await Promise.all([loadRunningProcesses(), loadDrsApplications()]);
    } catch (e) {
      console.error("Failed to toggle blacklist:", e);
    } finally {
      setPendingActions(prev => {
        const next = new Set(prev);
        next.delete(executable);
        return next;
      });
    }
  }, []);

  const handleCreateProfile = useCallback(async (executable: string) => {
    setPendingActions(prev => new Set(prev).add(executable));
    try {
      const profileName = executable.replace(/\.exe$/i, "");
      await createProfile(executable, profileName);
      // Reload data to reflect changes
      await Promise.all([loadRunningProcesses(), loadDrsApplications()]);
    } catch (e) {
      console.error("Failed to create profile:", e);
    } finally {
      setPendingActions(prev => {
        const next = new Set(prev);
        next.delete(executable);
        return next;
      });
    }
  }, []);

  const getInitials = (name: string) => {
    return name.slice(0, 2).toUpperCase();
  };

  const getBlacklistStatus = (isBlacklisted: boolean | null): "enabled" | "blacklisted" | "unknown" => {
    if (isBlacklisted === true) return "blacklisted";
    if (isBlacklisted === false) return "enabled";
    return "unknown";
  };

  // Filter functions
  const filteredProcesses = runningProcesses.filter(p =>
    p.processName.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.windowTitle.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredDrsApps = drsApplications.filter(a =>
    a.executable.toLowerCase().includes(searchQuery.toLowerCase()) ||
    a.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    a.profileName.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <div className="header-left">
          <div className="logo">
            <div className="logo-icon">N</div>
            <div className="logo-text">
              nvid<span>iot</span>
            </div>
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <button
            className={`refresh-btn ${refreshing ? "spinning" : ""}`}
            onClick={handleRefresh}
            disabled={refreshing || !nvApiStatus?.available}
            title="Refresh"
          >
            <RefreshIcon />
          </button>

          <div className="status-indicator">
            <div
              className={`status-dot ${
                nvApiStatus === null
                  ? "loading"
                  : nvApiStatus.available
                  ? "connected"
                  : "disconnected"
              }`}
            />
            <span>
              {nvApiStatus === null
                ? "Connecting..."
                : nvApiStatus.available
                ? "NVAPI Connected"
                : "Disconnected"}
            </span>
          </div>
        </div>
      </header>

      {/* Focus Section */}
      <section className="focus-section">
        <div className="focus-label">Current Focus</div>
        <div className="focus-card">
          {focusApp ? (
            <>
              <div className="focus-info">
                <div className="focus-process">{focusApp.processName}</div>
                <div className="focus-title">{focusApp.windowTitle}</div>
              </div>
              <div className="focus-status">
                <div className={`focus-profile ${!focusApp.profileName ? "no-profile" : ""}`}>
                  {focusApp.profileName || "No Profile"}
                </div>
                {focusApp.isInDrs && (
                  <button
                    className={`blacklist-toggle ${getBlacklistStatus(focusApp.isBlacklisted)}`}
                    onClick={() => handleToggleBlacklist(focusApp.processName, focusApp.isBlacklisted)}
                    disabled={pendingActions.has(focusApp.processName)}
                    title={focusApp.isBlacklisted ? "Click to enable recording" : "Click to disable recording"}
                  />
                )}
              </div>
            </>
          ) : (
            <div className="focus-empty">No application focused</div>
          )}
        </div>
      </section>

      {/* Main Content */}
      <main className="main-content">
        {/* Tabs */}
        <div className="tabs-header">
          <button
            className={`tab-button ${activeTab === "running" ? "active" : ""}`}
            onClick={() => setActiveTab("running")}
          >
            Running Processes
            <span className="tab-count">{runningProcesses.length}</span>
          </button>
          <button
            className={`tab-button ${activeTab === "drs" ? "active" : ""}`}
            onClick={() => setActiveTab("drs")}
          >
            DRS Applications
            <span className="tab-count">{drsApplications.length}</span>
          </button>
        </div>

        {/* Search Bar */}
        <div className="search-bar">
          <div className="search-input-wrapper">
            <span className="search-icon">
              <SearchIcon />
            </span>
            <input
              type="text"
              className="search-input"
              placeholder="Search applications..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* Error Banner */}
        {nvApiStatus && !nvApiStatus.available && (
          <div className="error-banner">
            <strong>NVAPI Unavailable:</strong> {nvApiStatus.error || "Unknown error"}
            <br />
            <small>Make sure you have an NVIDIA GPU and drivers installed.</small>
          </div>
        )}

        {/* Content */}
        {loading ? (
          <div className="loading-container">
            <div className="loading-spinner" />
            <div className="loading-text">Loading...</div>
          </div>
        ) : activeTab === "running" ? (
          <div className="app-list">
            {filteredProcesses.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">📋</div>
                <div className="empty-text">
                  {searchQuery ? "No matching processes found" : "No running processes detected"}
                </div>
              </div>
            ) : (
              filteredProcesses.map((process) => (
                <div
                  key={process.processId}
                  className={`app-card ${getBlacklistStatus(process.isBlacklisted)}`}
                >
                  <div className="app-icon">{getInitials(process.processName)}</div>
                  <div className="app-details">
                    <div className="app-name">{process.processName}</div>
                    <div className="app-title">{process.windowTitle}</div>
                  </div>
                  <div className="app-meta">
                    {process.hasDrsProfile ? (
                      <>
                        <div className="app-profile-badge has-profile">
                          {process.profileName || "Profile"}
                        </div>
                        <button
                          className={`blacklist-toggle ${getBlacklistStatus(process.isBlacklisted)}`}
                          onClick={() => handleToggleBlacklist(process.processName, process.isBlacklisted)}
                          disabled={pendingActions.has(process.processName)}
                          title={process.isBlacklisted ? "Enable recording" : "Disable recording"}
                        />
                      </>
                    ) : (
                      <button
                        className="create-profile-btn"
                        onClick={() => handleCreateProfile(process.processName)}
                        disabled={pendingActions.has(process.processName)}
                      >
                        + Create Profile
                      </button>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        ) : (
          <div className="app-list">
            {filteredDrsApps.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">📁</div>
                <div className="empty-text">
                  {searchQuery ? "No matching applications found" : "No DRS applications registered"}
                </div>
              </div>
            ) : (
              filteredDrsApps.map((app, index) => (
                <div
                  key={`${app.executable}-${index}`}
                  className={`app-card ${getBlacklistStatus(app.isBlacklisted)}`}
                >
                  <div className="app-icon">{getInitials(app.executable)}</div>
                  <div className="app-details">
                    <div className="app-name">{app.executable}</div>
                    <div className="app-title">{app.name || app.profileName}</div>
                  </div>
                  <div className="app-meta">
                    <div className="app-profile-badge has-profile">{app.profileName}</div>
                    <button
                      className={`blacklist-toggle ${getBlacklistStatus(app.isBlacklisted)}`}
                      onClick={() => handleToggleBlacklist(app.executable, app.isBlacklisted)}
                      disabled={pendingActions.has(app.executable)}
                      title={app.isBlacklisted ? "Enable recording" : "Disable recording"}
                    />
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
