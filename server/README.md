# Rust backend

Axum、Tokio、SQLx、MariaDB、Serde で実装した API サーバーです。SQLx は MySQL protocol 互換 driver 経由で MariaDB に接続し、TLS を使う依存関係には rustls を利用します。

## API

- `GET /openapi.yaml` — ビルド時に埋め込んだ共有 OpenAPI 仕様
- `GET /api/v1/ping` — `text/plain` の `pong`

`build.rs` は `../openapi/openapi-v1.yaml` を YAML として読み込み、OpenAPI 3.1.0 と、契約配信用の`getOpenApi`およびP0認証・ゲームAPI 8本に必要な`operationId`を検証します。検証後の仕様は `OUT_DIR` にコピーされ、バイナリへ `include_str!` で埋め込まれます。P0 API handler本体の実装は、この共通契約・fixture作成とは別Issueです。

## 生成されたAPI境界

`generated/openapi/`は`../openapi/openapi-v1.yaml`から生成される独立crateで、`openapi_generated`という依存名で参照できます。request／response型は`openapi_generated::models`、生成Routerは`openapi_generated::server`にあります。

```rust
use openapi_generated::models::AnswerRequest;
```

生成crateは手書きserverのpath dependencyです。通常のbuildとClippyで依存としてコンパイルされ、formatはmise taskから生成crateも明示的に検査されます。親serverの回帰testでは生成された契約型も検査します。生成物へ直接変更を加えず、API契約を変更した場合はリポジトリルートで`mise run openapi-generate-server`を実行してください。handler、repositoryなどの実装本体は`src/`へ置きます。

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
| `RUST_LOG` | `server=info,tower_http=info` | tracing filter |
| `AUTH_MODE` | 既定値なし | `demo`または`neoshowcase`。起動時に必須 |
| `DEMO_COOKIE_SECURE` | `true` | demo session Cookieへ`Secure`属性を付けるか。localhostのHTTP開発時だけ`false` |
| `IMAGE_UPLOAD_ENABLED` | `false` | dev用画像upload APIを有効化する。`true`は`AUTH_MODE=demo`でのみ使用可能 |
| `S3_ENDPOINT` | 未設定 | upload先のS3互換endpoint。upload有効時は必須 |
| `S3_BUCKET` | 未設定 | upload先bucket。upload有効時は必須 |
| `AWS_ACCESS_KEY_ID` | 未設定 | storage access key。upload有効時は必須。repositoryやlogへ記録しない |
| `AWS_SECRET_ACCESS_KEY` | 未設定 | storage secret key。upload有効時は必須。repositoryやlogへ記録しない |
| `AWS_REGION` | 未設定 | storage署名に使用するregion。upload有効時は必須 |
| `S3_FORCE_PATH_STYLE` | 未設定 | S3 path-style accessを使うか。upload有効時は`true`または`false`が必須 |
| `ASSET_PUBLIC_BASE_URL` | 未設定 | responseの公開Asset URLを組み立てるbase URL。upload有効時は必須 |

画像upload APIはdev専用です。Composeではbackend container内を`0.0.0.0:8080`で待ち受けますが、host側の公開先を`127.0.0.1:8080`へ限定します。Composeを使わず直接起動する場合は`APP_ADDR=127.0.0.1:8080`を指定してください。`Host`、`X-Forwarded-For`、`X-Real-IP`によるlocalhost判定は行いません。

storage credentialはGit管理外の`server/.env.storage.local`で管理し、値をsource、fixture、HTTP request、frontend、logへ含めません。

起動時に `migrations/` の SQLx migration を自動適用します。
