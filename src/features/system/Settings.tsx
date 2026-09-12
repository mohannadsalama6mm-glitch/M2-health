import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  FeaturePage,
  Button,
  Card,
  SectionHeader,
  Input,
  Select,
  Alert,
  LinkButton,
} from "../../design-system";
import { settingsSections } from "../../mock/fixtures";
import { settingFields } from "./settings-fields";
import { useDemo } from "../../app/DemoContext";
export function Settings() {
  const [params, setParams] = useSearchParams();
  const section = settingsSections.includes(params.get("section") ?? "")
    ? params.get("section")!
    : "General";
  const [values, setValues] = useState<Record<string, string>>({});
  const { notify } = useDemo();
  return (
    <FeaturePage
      title="Settings"
      description="Configure the look and future operating preferences of your workspace."
    >
      <div className="settings-layout">
        <nav className="settings-menu" aria-label="Settings sections">
          {settingsSections.map((s) => (
            <Button
              key={s}
              variant={s === section ? "primary" : "ghost"}
              onClick={() => setParams({ section: s })}
            >
              {s}
            </Button>
          ))}
        </nav>
        <form
          className="content-stack"
          onSubmit={(e) => {
            e.preventDefault();
            notify(
              `${section} preferences saved in this screen only. No runtime policy changed.`,
            );
          }}
        >
          <Card>
            <SectionHeader
              title={section}
              subtitle="UI configuration preview"
            />
            <div className="form-grid">
              {settingFields[section].map((f) => {
                const key = `${section}:${f.label}`;
                return f.options ? (
                  <Select
                    key={key}
                    showLabel
                    label={f.label}
                    value={values[key] ?? f.options[0]}
                    onChange={(e) =>
                      setValues({ ...values, [key]: e.target.value })
                    }
                  >
                    {f.options.map((o) => (
                      <option key={o}>{o}</option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    key={key}
                    label={f.label}
                    type={f.type ?? "text"}
                    min={0}
                    disabled={f.disabled}
                    value={values[key] ?? f.value ?? ""}
                    onChange={(e) =>
                      setValues({ ...values, [key]: e.target.value })
                    }
                  />
                );
              })}
            </div>
          </Card>
          <Alert title="Preview preferences">
            Settings do not configure real devices, security, synchronization,
            printing, or storage in Phase 1.
          </Alert>
          <div className="row end">
            {section === "Users" && (
              <LinkButton to="/roles">Roles & permissions</LinkButton>
            )}
            {section === "Backup" && (
              <LinkButton to="/backup">Backup workspace</LinkButton>
            )}
            {section === "Sync" && (
              <LinkButton to="/sync">Sync workspace</LinkButton>
            )}
            <Button type="submit" variant="primary">
              Save settings preview
            </Button>
          </div>
        </form>
      </div>
    </FeaturePage>
  );
}
