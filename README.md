# 1m26_11

1Monthon 2026 11班のプロジェクトです。

## 構成

- `client/`: Vue 3 + Vite + TypeScript のクライアントアプリケーション
- `openapi/`: クライアントとサーバーが共有する OpenAPI 3.1 契約
- `server/`: Rust 2024 edition のサーバーアプリケーション
- `Dockerfile`: Rust server の production image 定義
- `compose.yaml`: server、MariaDB、Adminer を束ねる開発環境定義

## Requirements

通常のビルド、整形、Lint、テストには、プロジェクトで唯一のタスクランナーとして [mise](https://mise.jdx.dev/) を使用します。コンテナを使った開発には Docker と Docker Compose も必要です。

## Setup

```sh
mise trust
mise install
```

Rust stable、Node.js 24、pnpm 11 は `mise.toml` の定義に従ってセットアップされます。

## Tasks

すべてリポジトリルートで実行します。Cargo と pnpm のコマンドは mise によりそれぞれ `server/` と `client/` で実行されます。

| Command | Description | Database |
| --- | --- | --- |
| `mise run build` | サーバーをビルド | 不要 |
| `mise run fmt` | Rust コードを整形 | 不要 |
| `mise run fmt-check` | Rust コードの整形を検査 | 不要 |
| `mise run lint` | Clippy を実行 | 不要 |
| `mise run test-unit` | DB 不要の server test を実行 | 不要 |
| `mise run test` | DB を必要としない通常テストを実行 | 不要 |
| `mise run test-integration` | ignored の API integration test を直列実行 | 必要 |
| `mise run client-install` | pnpm 依存関係を lockfile からインストール | 不要 |
| `mise run client-build` | client の typecheck と production build | 不要 |
| `mise run client-lint` | client の ESLint/Oxlint | 不要 |
| `mise run client-test` | client の Vitest | 不要 |
| `mise run client-check` | client の全チェック | 不要 |
| `mise run openapi-generate` | Rust・TypeScript の契約コードを再生成 | 不要 |
| `mise run openapi-generate-check` | 再生成後の差分がないことを検査 | 不要 |
| `mise run check` | server と client の全チェック | 不要 |
| `mise run docker-build` | server のコンテナ image をビルド | 不要 |

Integration test をローカルで実行する場合は、MariaDB を用意して `TEST_DATABASE_URL` を設定してください。CI と Compose では MariaDB 11.8 LTS 系を使用します。SQLx は MariaDB への接続にも MySQL protocol の `mysql://` URL を使用します。

## Docker development

`Dockerfile` と `compose.yaml` は、`server/` と `openapi/` の双方を参照するためリポジトリルートに配置しています。次のコマンドで backend の Compose Watch と frontend の Vite dev server を並列起動します。

```sh
mise run dev
```

Vite のみ起動する場合は `mise run client-dev`、backend のみ watch する場合は `mise run server-dev` を使用します。backend をバックグラウンドで起動・停止する場合も mise から実行できます。

```sh
mise run compose-up
mise run compose-down
```

## OpenAPI

API の共有契約は `openapi/openapi-v1.yaml` です。この1ファイルから`server/generated/openapi/`のRust契約クレートと`client/src/generated/api.d.ts`のTypeScript型を生成します。生成物へ直接変更を加えず、契約を変更したら`mise run openapi-generate`で更新してください。

`server/build.rs`はサーバービルド時にYAML、OpenAPI version、必須path/methodの`operationId`を検証して埋め込みます。`mise run check`では生成Rustクレートもbuildとformatの対象になり、Clippy時には依存としてコンパイルされます。親serverの回帰testで生成された契約型を検査します。

詳細は `openapi/README.md` を参照してください。

## Members

- kaomojikun
- shimeji
- renkon
- solalyth
- Takenokomeshi
- SAH123
