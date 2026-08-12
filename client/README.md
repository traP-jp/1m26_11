# Client

Vue 3、Vite、TypeScript、pnpm で構成したフロントエンドです。タスクはリポジトリルートの `mise.toml` から実行します。

## Setup

```sh
mise install
mise run client-install
```

## Development

backend、MariaDB、Adminer と Vite dev server をまとめて起動します。

```sh
mise run dev
```

Vite のみ起動する場合は次を実行します。developmentではMSWが既定で有効になり、backendなしでAPIを利用できます。

```sh
mise run client-dev
```

mock responseは`../openapi/openapi-v1.yaml`、`../openapi/examples/`、`../openapi/scenarios/p0-cases.yaml`を直接読みます。response payloadをclient内に複製していないため、OpenAPIのexampleとscenarioがmockの正本です。

実backendへ接続する場合は`.env.example`を参考に`client/.env.local`を作成し、`VITE_ENABLE_MSW=false`を設定してください。その場合、Viteは`/api`と`/openapi.yaml`を`API_PROXY_TARGET`（既定は`http://127.0.0.1:8080`）へproxyします。特定の状態からmockを開始する場合は、`VITE_MSW_SCENARIO`へscenarioのcase id（例: `demo_login_and_logout`、`query_correct`）を指定します。production buildではMSWは常に無効です。

## Component preview (Histoire)

Histoireを使うと、アプリ全体を起動せずにVueコンポーネントを状態ごとに表示できます。

リポジトリルートで次を実行し、ターミナルに表示されたURLをブラウザで開きます。

```sh
mise run client-histoire
```

プレビューは対象コンポーネントと同じディレクトリに`*.story.vue`として作成します。`Story`がコンポーネントのまとまり、`Variant`が状態ごとの表示です。

```vue
<!-- src/components/ExampleButton.story.vue -->
<script setup lang="ts">
import ExampleButton from './ExampleButton.vue'
</script>

<template>
  <Story title="Components/ExampleButton">
    <Variant title="Default">
      <ExampleButton>Default</ExampleButton>
    </Variant>

    <Variant title="Disabled">
      <ExampleButton disabled>Disabled</ExampleButton>
    </Variant>
  </Story>
</template>
```

`src/histoire.setup.ts`が`src/assets/main.css`を読み込むため、アプリと同じTailwind CSSとグローバルスタイルが適用されます。PiniaやVue Routerなどのアプリプラグインが必要になった場合も、このsetupファイルでHistoireのVueアプリへ登録します。

静的サイトとしてのビルドと、その成果物の確認には次を使います。

```sh
mise run client-histoire-build
mise run client-histoire-preview
```

`mise run client-check`にもHistoireのproduction buildが含まれ、すべてのstoryをビルドできることを検証します。

現在はVite 8対応を取り込んだ正式版が未公開のため、Histoireを`1.0.0-beta.1`へ固定し、`pnpm-workspace.yaml`では同版に限ってVite 8のpeer依存警告を許容しています。この設定自体が互換性を保証するものではありませんが、現在の環境ではdevelopment server、build、previewの動作を確認済みです。build時にbeta版由来の`IMPORT_IS_UNDEFINED`警告が表示されることがありますが、末尾に`Built`と表示されればbuildは成功しています。Histoire更新時はこの許容設定と注意書きが不要になっていないか確認してください。

## Checks

```sh
mise run client-build
mise run client-typecheck
mise run client-lint
mise run client-format-check
mise run client-test
mise run client-check
```

自動修正には `mise run client-lint-fix` と `mise run client-format` を使用します。

## OpenAPI生成型

`src/generated/api.d.ts`は`../openapi/openapi-v1.yaml`から生成される通信境界の型です。直接編集せず、契約変更時はリポジトリルートで次を実行します。

```sh
mise run openapi-generate-client
```

API呼び出しでは`paths`または`components`からrequest／response型を参照し、mock payloadは`../openapi/examples/`の共通fixtureを使用してください。
