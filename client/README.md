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

Vite のみ起動する場合は次を実行します。

```sh
mise run client-dev
```

Vite は `/api` と `/openapi.yaml` を既定で `http://127.0.0.1:8080` に proxy します。転送先を変更する場合は `.env.example` を参考に `client/.env.local` を作成し、`API_PROXY_TARGET` を設定してください。

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
