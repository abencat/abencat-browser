import type { ReactNode } from "react";
import { X } from "lucide-react";
import { useI18n } from "../i18n";

export function Modal({
  title,
  onClose,
  children,
  dialogClassName = "",
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
  dialogClassName?: string;
}) {
  return (
    <div className="modal show">
      <div className={`dialog ${dialogClassName}`.trim()}>
        <div className="modal-head">
          <h2>{title}</h2>
          <button className="close-btn" onClick={onClose}><X size={16} /></button>
        </div>
        {children}
      </div>
    </div>
  );
}

export function TextField({
  label,
  value,
  onChange,
  placeholder = "",
  type = "text",
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: string;
}) {
  return (
    <label>
      {label}
      <input className="input" type={type} value={value || ""} placeholder={placeholder} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

export function NumberField({ label, value, onChange }: { label: string; value: number; onChange: (value: number) => void }) {
  return <label>{label}<input className="input" type="number" value={value || 0} onChange={(event) => onChange(Number(event.target.value))} /></label>;
}

export function SelectField({ label, value, options, onChange }: { label: string; value: string; options: string[]; onChange: (value: string) => void }) {
  return (
    <label>{label}
      <select className="input" value={value || options[0]} onChange={(event) => onChange(event.target.value)}>
        {options.map((option) => <option key={option} value={option}>{option}</option>)}
      </select>
    </label>
  );
}

export function TextAreaField({ label, value, onChange, placeholder = "", className = "" }: { label: string; value: string; onChange: (value: string) => void; placeholder?: string; className?: string }) {
  return <label className="span-2">{label}<textarea className={`textarea ${className}`} value={value || ""} placeholder={placeholder} onChange={(event) => onChange(event.target.value)} /></label>;
}

export function CopyRow({ label, value, onCopy }: { label: string; value: string; onCopy: (value: string) => void | Promise<void> }) {
  const { t } = useI18n();
  return (
    <div className="copy-row">
      <span className="copy-label">{label}</span>
      <code className="copy-value">{value}</code>
      <button className="btn mini-btn" type="button" onClick={() => void onCopy(value)}>{t("common.copy")}</button>
    </div>
  );
}
