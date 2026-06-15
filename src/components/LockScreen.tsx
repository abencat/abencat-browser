import { useState } from "react";
import { Lock } from "lucide-react";
import { useI18n } from "../i18n";

export function LockScreen({ onUnlock }: { onUnlock: (password: string) => Promise<void> }) {
  const { t } = useI18n();
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    if (!password) return;
    setBusy(true);
    setError("");
    try {
      await onUnlock(password);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="lock-screen">
      <div className="lock-card">
        <div className="lock-icon"><Lock size={28} /></div>
        <h2>{t("lock.title")}</h2>
        <p className="muted">{t("lock.hint")}</p>
        <input
          className="input"
          type="password"
          autoFocus
          value={password}
          placeholder={t("lock.password")}
          onChange={(e) => setPassword(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") void submit(); }}
        />
        {error && <div className="lock-error">{error}</div>}
        <button className="btn primary lock-btn" disabled={busy} onClick={() => void submit()}>
          {busy ? "…" : t("lock.unlock")}
        </button>
      </div>
    </div>
  );
}
