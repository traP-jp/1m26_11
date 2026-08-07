# 1m26_11

1Monthon 2026 11班のプロジェクトです。

## 構成

- `client/`: クライアントアプリケーション
- `openapi/`: クライアントとサーバーが共有する OpenAPI 3.1 契約
- `server/`: Rust 2024 edition のサーバーアプリケーション

## Requirements

通常のビルド、整形、Lint、テストには、プロジェクトで唯一のタスクランナーとして [mise](https://mise.jdx.dev/) を使用します。コンテナを使った開発には Docker と Docker Compose も必要です。

## Setup

```sh
mise trust
mise install
```

Rust stable は `mise.toml` の定義に従ってセットアップされます。

## Tasks

すべてリポジトリルートで実行します。Cargo コマンドは mise により `server/` で実行されます。

| Command | Description | Database |
| --- | --- | --- |
| `mise run build` | サーバーをビルド | 不要 |
| `mise run fmt` | Rust コードを整形 | 不要 |
| `mise run fmt-check` | Rust コードの整形を検査 | 不要 |
| `mise run lint` | Clippy を実行 | 不要 |
| `mise run test-unit` | DB 不要の server test を実行 | 不要 |
| `mise run test` | DB を必要としない通常テストを実行 | 不要 |
| `mise run test-integration` | ignored の API integration test を直列実行 | 必要 |
| `mise run check` | format、build、lint、DB 不要テストをまとめて実行 | 不要 |
| `mise run docker-build` | server のコンテナ image をビルド | 不要 |

Integration test をローカルで実行する場合は、MySQL を用意して `TEST_DATABASE_URL` を設定してください。CI では MySQL 8.4 service を起動して実行します。

## Docker development

Docker Compose の watch モードで開発環境を起動します。

```sh
mise run dev
```

バックグラウンドでの起動・停止も mise から実行できます。

```sh
mise run compose-up
mise run compose-down
```

## OpenAPI

API の共有契約は `openapi/openapi-v1.yaml` です。これはサーバーコード生成用のファイルではありません。`server/build.rs` がサーバービルド時に YAML、OpenAPI version、必須 path/method の `operationId` を検証して埋め込むため、契約を変更したら `mise run build` で契約の基本検証を行ってください。

詳細は `openapi/README.md` を参照してください。

## Members

- kaomojikun
- shimeji
- renkon
- solalyth
- Takenokomeshi
- SAH123
