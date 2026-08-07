# Rust backend

Axum、Tokio、SQLx、MariaDB、Serde、Reqwest で実装した API サーバーです。SQLx は MySQL protocol 互換 driver 経由で MariaDB に接続し、TLS を使う依存関係には rustls を利用します。

## API

- `GET /openapi.yaml` — ビルド時に埋め込んだ共有 OpenAPI 仕様
- `GET /api/v1/ping` — `text/plain` の `pong`
- `GET /api/v1/users`
- `POST /api/v1/users`
- `GET /api/v1/users/{userID}`

`build.rs` は `../openapi/openapi-v1.yaml` を YAML として読み込み、OpenAPI 3.1.0 と各 path/method に必要な `operationId` (`getOpenApi`、`ping`、`getUsers`、`createUser`、`getUser`) を検証します。検証後の仕様は `OUT_DIR` にコピーされ、バイナリへ `include_str!` で埋め込まれます。

## 必要なもの

- [mise](https://mise.jdx.dev/)
- Docker / Docker Compose 2.22 以降（開発用 MariaDB と Compose Watch を使う場合）

コマンドはリポジトリルートで実行します。

```sh
mise install
mise run build
mise run test
mise run lint
mise run check
```

`mise run test` は DB 不要のテストだけを実行します。フォーマットは `mise run fmt`、確認のみなら `mise run fmt-check` です。

## 開発環境

コンテナ定義はリポジトリルートの `Dockerfile` と `compose.yaml` にあります。`server/` と共有契約の `openapi/` の両方を build context に含めます。

```sh
mise run dev
```

API は <http://localhost:8080>、Adminer は <http://localhost:8081> で起動します。通常の Compose 起動／停止には `mise run compose-up` と `mise run compose-down` を使えます。

Docker build はリポジトリルートを context にします。

```sh
mise run docker-build
```

## 環境変数

| 変数 | 既定値 | 説明 |
| --- | --- | --- |
| `APP_ADDR` | `0.0.0.0:8080` | listen address（`:8080` 形式も可） |
| `DATABASE_URL` | 未設定 | 設定時は個別の `DB_*` より優先する MariaDB URL（`mysql://` scheme） |
| `DB_USER` | `root` | MariaDB user |
| `DB_PASS` | `pass` | MariaDB password |
| `DB_HOST` | `localhost` | MariaDB host |
| `DB_PORT` | `3306` | MariaDB port |
| `DB_NAME` | `app` | MariaDB database |
| `PHOTO_API_URL` | `https://jsonplaceholder.typicode.com/photos` | photo API の base URL |
| `RUST_LOG` | `server=info,tower_http=info` | tracing filter |

起動時に `migrations/` の SQLx migration を自動適用します。

## MariaDB 結合テスト

MariaDB を使う user flow は `#[ignore]` です。`TEST_DATABASE_URL` を設定するか、上記の `DB_USER`、`DB_PASS`、`DB_HOST`、`DB_PORT`、`DB_NAME` を設定して実行します。テストは migration を適用し、開始前後に `users` を削除します。

```sh
TEST_DATABASE_URL=mysql://root:pass@127.0.0.1:3306/app \
  mise run test-integration
```

`mysql://` は SQLx が MariaDB との通信に使用する MySQL protocol の scheme です。


