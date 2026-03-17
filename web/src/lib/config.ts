const DEFAULT_ENV_LABEL = "local";

function readEnvLabel() {
  return import.meta.env.VITE_ENV_LABEL ?? DEFAULT_ENV_LABEL;
}

export const config = {
  envLabel: readEnvLabel(),
};

export type AppConfig = typeof config;
