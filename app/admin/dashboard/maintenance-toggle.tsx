"use client";

import { useState, useEffect } from "react";
import { getChatMaintenanceMode, setChatMaintenanceModeAction } from "@/app/actions";
import { Bot, AlertCircle } from "lucide-react";

export default function MaintenanceToggle() {
  const [isMaintenance, setIsMaintenance] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    getChatMaintenanceMode().then((isMaintenanceMode) => {
      setIsMaintenance(isMaintenanceMode);
      setLoading(false);
    });
  }, []);

  async function handleToggle() {
    setSaving(true);
    setError(null);
    setSuccess(false);

    const newValue = !isMaintenance;
    const result = await setChatMaintenanceModeAction(newValue);

    if (result.success) {
      setIsMaintenance(newValue);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } else {
      setError(result.error || "Failed to update maintenance mode");
    }

    setSaving(false);
  }

  if (loading) {
    return (
      <div className="rounded-lg border border-border bg-bg-light p-6 mb-8">
        <div className="flex items-center gap-2 text-text-muted">
          <div className="h-4 w-4 animate-spin rounded-full border-2 border-action-primary border-t-transparent"></div>
          <span>Loading maintenance settings...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-border bg-bg-light p-6 mb-8">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-action-primary/10">
            <Bot className="h-5 w-5 text-action-primary" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-text">AI Chat Maintenance</h2>
            <p className="text-sm text-text-muted">
              {isMaintenance
                ? "Chat feature is currently disabled for maintenance"
                : "Chat feature is enabled (beta version)"}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-4">
          {error && (
            <div className="flex items-center gap-2 text-sm text-alert-danger">
              <AlertCircle className="h-4 w-4" />
              <span>{error}</span>
            </div>
          )}
          {success && (
            <div className="text-sm text-alert-success">Settings updated!</div>
          )}
          <button
            onClick={handleToggle}
            disabled={saving}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-action-primary focus:ring-offset-2 ${
              isMaintenance ? "bg-action-primary" : "bg-text-muted"
            } ${saving ? "opacity-50 cursor-not-allowed" : ""}`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                isMaintenance ? "translate-x-6" : "translate-x-1"
              }`}
            />
          </button>
        </div>
      </div>
    </div>
  );
}
