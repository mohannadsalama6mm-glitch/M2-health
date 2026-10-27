import { useState, type FormEvent, type ReactNode } from "react";
import { Alert, Button, Logo, Textarea } from "../../design-system";
import { useLicense } from "../../app/LicenseContext";
import { release } from "../../lib/release";
import { useLocalization } from "../../localization/LocalizationContext";

export function LicenseGate({ children }: { children: ReactNode }) {
  const license = useLicense();
  const { t } = useLocalization();
  const [readOnlyAccepted, setReadOnlyAccepted] = useState(false);
  if (license.loading && !license.status)
    return <Frame><p role="status">{t("license.checking")}</p></Frame>;
  if (!license.status)
    return <Frame><Alert title={t("license.unavailable")} tone="danger">{license.error || t("license.restart")}</Alert></Frame>;
  if (license.status.canOperate || (license.status.readOnly && readOnlyAccepted))
    return children;
  return (
    <Activation
      restricted={license.status.readOnly}
      onReadOnly={() => setReadOnlyAccepted(true)}
    />
  );
}

function Frame({ children }: { children: ReactNode }) {
  return (
    <main className="auth-screen">
      <section className="auth-card license-card">
        <Logo variant="compact" />
        {children}
        <p className="auth-local-note">
          {release.channel} · v{release.version} · build {release.buildId}
        </p>
      </section>
    </main>
  );
}

function Activation({
  restricted,
  onReadOnly,
}: {
  restricted: boolean;
  onReadOnly: () => void;
}) {
  const license = useLicense();
  const { t } = useLocalization();
  const [code, setCode] = useState("");
  const [busy, setBusy] = useState(false);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      if (license.status?.state === "OFFLINE_GRACE") await license.revalidate(code);
      else await license.activate(code);
    } catch {
      // Safe text is owned by LicenseContext.
    } finally {
      setBusy(false);
    }
  };
  return (
    <Frame>
      <div>
        <h1>{restricted ? t("license.activationRequired") : t("license.start")}</h1>
        <p>{license.status?.reason}</p>
      </div>
      {license.error && <Alert title={t("license.couldNotActivate")} tone="danger">{license.error}</Alert>}
      {!restricted && (
        <Button
          variant="primary"
          disabled={busy}
          onClick={() => {
            setBusy(true);
            void license.startTrial().finally(() => setBusy(false));
          }}
        >
          {t("license.startTrial")}
        </Button>
      )}
      <form className="auth-form" onSubmit={submit}>
        <Textarea
          label={t("license.manualCode")}
          value={code}
          required
          onChange={(event) => setCode(event.target.value)}
        />
        <Button type="submit" disabled={busy || !code.trim()}>
          {busy ? t("license.verifying") : t("license.activateDevice")}
        </Button>
      </form>
      <p className="field-help">{t("license.deviceId")}: <code dir="ltr">{license.status?.deviceId}</code></p>
      {restricted && (
        <>
          <Alert title={t("license.dataSafe")} tone="warning">
            {t("license.dataSafeCopy")}
          </Alert>
          <Button onClick={onReadOnly}>{t("license.openReadOnly")}</Button>
        </>
      )}
    </Frame>
  );
}
