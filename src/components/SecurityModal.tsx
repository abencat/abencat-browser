import { useState } from "react";
import { Modal } from "./ui";
import { useI18n } from "../i18n";
import type { SecurityStatus } from "../types";

interface Props {
  status: SecurityStatus | null;
  onClose: () => void;
  onSetMaster: (password: string) => Promise<void>;
  onRemoveMaster: () => Promise<void>;
}

export function SecurityModal({ status, onClose, onSetMaster, onRemoveMaster }: Props) {
  const { t } = useI18n();
  const [pw, setPw] = useState("");
  const [pw2, setPw2] = useState("");
  const [pwError, setPwError] = useState("");
  const hasMaster = status?.hasMasterPassword ?? false;

  const applyMaster = async () => {
    if (pw.length < 4) return setPwError(t("sec.newPassword"));
    if (pw !== pw2) return setPwError(t("sec.mismatch"));
    setPwError("");
    await onSetMaster(pw);
    setPw("");
    setPw2("");
  };

  return (
    <Modal title={t("sec.title")} onClose={onClose}>
      <div className="security-body">
        <section className="sec-section">
          <h4>{t("sec.master")}</h4>
          <p className="muted">{hasMaster ? t("sec.masterOn") : t("sec.masterOff")}</p>
          <div className="form-grid compact-form">
            <label>{t("sec.newPassword")}<input className="input" type="password" value={pw} onChange={(e) => setPw(e.target.value)} /></label>
            <label>{t("sec.confirmPassword")}<input className="input" type="password" value={pw2} onChange={(e) => setPw2(e.target.value)} /></label>
          </div>
          {pwError && <div className="lock-error">{pwError}</div>}
          <div className="sec-actions">
            <button className="btn primary" onClick={() => void applyMaster()}>{hasMaster ? t("sec.changeMaster") : t("sec.setMaster")}</button>
            {hasMaster && <button className="btn danger" onClick={() => void onRemoveMaster()}>{t("sec.removeMaster")}</button>}
          </div>
        </section>
      </div>
      <div className="dialog-actions">
        <button className="btn primary" onClick={onClose}>{t("common.done")}</button>
      </div>
    </Modal>
  );
}
