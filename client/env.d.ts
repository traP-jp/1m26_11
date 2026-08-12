/// <reference types="vite/client" />
/// <reference types="@histoire/plugin-vue/components" />

interface ImportMetaEnv {
  readonly VITE_ENABLE_MSW?: string
  readonly VITE_MSW_SCENARIO?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
