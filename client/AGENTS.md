# AGENTS.md

このファイルは `client/` 配下の変更に適用します。リポジトリルートの `AGENTS.md` と併せて従い、
内容が競合する場合は、より対象に近いこのファイルを優先してください。

## 技術構成

- Vue 3、Vite、TypeScriptによるESMアプリケーションです。
- package managerはpnpmです。ルートの `mise.toml` に定義されたNode.jsとpnpmを使用します。
- Tailwind CSSは `@tailwindcss/vite` と `src/assets/main.css` から読み込みます。
- `@/` aliasは `client/src/` を指します。

## 標準コマンド

コマンドは原則としてリポジトリルートから実行します。

```sh
mise run client-install
mise run client-dev
mise run client-typecheck
mise run client-lint
mise run client-format-check
mise run client-test
mise run client-build
mise run client-check
```

- フロントエンドだけを動かす場合は `mise run client-dev` を使用します。
- `mise run dev` はbackend、MariaDB、Adminerも起動します。
- 自動修正には `mise run client-lint-fix` と `mise run client-format` を使用します。
- `client-format-check` と `client-format` の対象は `src/` です。設定ファイルを変更した場合は、
  format taskだけでなく関連するlintやbuildも確認してください。

## Vue・TypeScript

- Vue SFCは既存どおり `<script setup lang="ts">` を基本とします。
- TypeScriptのstrict設定と `noUncheckedIndexedAccess` を維持します。配列・object lookupの
  `undefined` を扱い、無理なtype assertionでエラーを隠しません。
- 同じ機能の隣接ファイルには相対import、離れた `src/` 内のmoduleには必要に応じて `@/` aliasを
  使用します。
- 書式は既存設定に従います。UTF-8、LF、2 spaces、semicolonなし、single quote、原則100文字幅です。

## OpenAPIとAPI型

- API契約はルートの `openapi/openapi-v1.yaml` を使用します。
- `src/generated/api.d.ts` は生成物です。直接編集せず、契約更新後にリポジトリルートで
  `mise run openapi-generate-client` を実行します。
- API request／response型は、生成された `paths` または `components` から参照します。
- response payloadをclient内へ複製しません。mockとtestは `openapi/examples/` の共通fixtureを
  使用します。

## MSW

- developmentではMSWが既定で有効で、production buildでは常に無効です。
- mockの正本は `openapi/openapi-v1.yaml`、`openapi/examples/`、
  `openapi/scenarios/p0-cases.yaml` です。scenarioやpayloadをclient独自形式で重複管理しません。
- endpointのmock behaviorは `src/mocks/handlers.ts`、状態遷移は `src/mocks/state.ts`、契約読込みは
  `src/mocks/contract.ts` と `src/mocks/data.ts` で管理します。
- 実backendへ接続する場合は、追跡対象外の `client/.env.local` で `VITE_ENABLE_MSW=false` を設定し、
  必要に応じて `API_PROXY_TARGET` を設定します。Viteは `/api` と `/openapi.yaml` をproxyします。
- 特定状態からmockを開始する場合は、`VITE_MSW_SCENARIO` にscenarioのcase IDを指定します。
- `public/mockServiceWorker.js` は手作業で編集せず、production配布物へ含めない既存設定を維持します。
- endpointを追加・変更した場合は、OpenAPI example／scenario、MSW handler、関連testの整合を
  確認します。

## Component preview

- UI componentのstoryは対象componentと同じdirectoryへ `*.story.vue` として置きます。
- `Story` はcomponentのまとまり、`Variant` は状態ごとの表示に使用します。
- Histoireは `src/histoire.setup.ts` を通して `src/assets/main.css` を共有します。アプリpluginが
  必要な場合はpreview側にも登録します。
- UI componentを追加・変更した場合は、意味のある状態をstoryで確認できるようにします。
- Histoire `1.0.0-beta.1` と `pnpm-workspace.yaml` のVite 8 peer dependency設定は対になっています。
  更新する場合はdevelopment server、build、previewをまとめて再検証します。
- Histoire buildで `IMPORT_IS_UNDEFINED` 警告が出ても、末尾が `Built` なら既知の警告です。

## Test

- unit／component testにはVitest、Vue Test Utils、jsdomを使用します。
- testは対象に近い `src/**/__tests__/` へ `*.spec.ts` として置きます。
- MSW handlerのtestはbrowser workerではなく、`msw/node` の `setupServer` を使用します。
- 振る舞い、状態遷移、API連携を変更した場合は、共有fixtureとscenarioを使うtestを追加または
  更新します。

完了前には、変更に対応する個別taskを実行し、可能な限り次を通してください。

```sh
mise run client-check
```

実行できなかった検証がある場合は、commandと理由を報告します。
