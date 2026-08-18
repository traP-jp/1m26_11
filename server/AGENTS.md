# AGENTS.md

このファイルは `server/` 配下の変更に適用します。リポジトリルートの `AGENTS.md` と併せて従い、
内容が競合する場合は、より対象に近いこのファイルを優先してください。

## 技術構成

- Rust 2024 edition
- Axum、Tokio
- SQLxのMySQL driverを介したMariaDB接続
- 共有OpenAPI契約から生成したRust crate
- `thiserror` によるerror定義
- `tracing` と `tower-http` によるlogging

Rustのeditionと最低対応versionは `Cargo.toml`、tool versionと標準taskはルートの `mise.toml` を
正本とします。

## ディレクトリと責務

- `src/main.rs`: 環境変数、DB pool、migration、listen、graceful shutdown
- `src/lib.rs`: `AppState`、Router構築、route登録、migration entry point
- `src/handler.rs`、`src/handler/`: HTTP requestの取出しとresponse構築
- `src/auth.rs`、`src/auth/`: demo CookieとNeoShowcase forwarded headerによる認証
- `src/repository.rs`: repository trait、SQLx query、DB record型
- `src/error.rs`: application errorからHTTP responseへの変換
- `migrations/`: 連番のSQLx migration
- `tests/`: Router、認証、共有fixture、生成型、DB integration test
- `generated/openapi/`: OpenAPIから生成される独立crate。直接編集しない

handlerへSQLを直接書かず、DB操作はrepositoryへ置きます。複数の更新を一体として成功させる必要が
ある処理にはtransactionを使用します。crate内部だけで使う項目は `pub(crate)` を優先します。

## OpenAPI契約

APIのpath、method、request／response schema、status、exampleは、ルートの次のファイルに従います。

- `openapi/openapi-v1.yaml`
- `openapi/examples/`
- `openapi/scenarios/p0-cases.yaml`

HTTP境界では `openapi_generated::models` の生成型を使用し、同じrequest／response DTOを手書きで
重複定義しません。内部domain modelが必要な場合は分離し、境界で明示的に変換します。

`build.rs` は共有OpenAPIを検証してserver binaryへ埋め込みます。共有契約にないpath、operationId、
response形式をserver側だけで追加・変更しません。

`generated/openapi/` を直接修正しません。変更が必要な場合は、目的に応じて次の生成元を修正します。

- 契約: `openapi/openapi-v1.yaml` と関連するexample・scenario
- Rust生成設定: `openapi/openapitools.json`
- Generator補正: `openapi/templates/rust-axum/`

契約を変更した場合はリポジトリルートで `mise run openapi-generate` を実行します。Rust生成物だけを
更新する場合は `mise run openapi-generate-server` を使用できます。

生成された `NullValue` は、OpenAPI 3.1の「必須かつ値がnull」を表します。`Option<T>` や任意の
`serde_json::Value` へ緩めません。

## 認証・設定・error

- `AUTH_MODE` は起動時に必須で、`demo` または `neoshowcase` だけを許可します。
- modeを暗黙にfallbackさせたり、両方の認証方式を同時に試したりしません。
- demo認証は `demo_session` Cookie、NeoShowcase認証は `x-forwarded-user` headerを使用します。
- 認証必須endpointでは `CurrentUser`、認証任意endpointでは `OptionalCurrentUser` extractorを
  使用します。
- `DATABASE_URL` が設定されている場合は個別の `DB_*` より優先されます。
- password、token、実環境の接続先をsource、fixture、logへ含めません。
- 内部errorは `AppError` へ変換します。詳細をlogへ残しても、DB errorや内部情報をHTTP responseへ
  露出させません。
- productionの通常経路では、外部入力、DB、I/Oの失敗に `unwrap` や `expect` を使いません。

## MariaDBとmigration

serverは起動時に `migrations/` を自動適用します。serverを起動する前に、接続先DBが意図した環境で
あることを確認してください。

共有・適用済みのmigrationは書き換えず、schema変更は新しい連番migrationとして追加します。
既存schemaでは次の表現を使用します。

- UUID: `BINARY(16)`
- 時刻: UTCの `TIMESTAMP(3)`
- JSON data: `JSON`
- storage engine: `InnoDB`
- character set: `utf8mb4`
- SQLx driver: MySQL
- parameter: `?` placeholderと `.bind(...)`

PostgreSQL固有の型、placeholder、部分index、DDLを持ち込みません。外部キー、unique制約、check制約、
statusと日時の整合性など、既存migrationの不変条件を保持します。

DB integration testはschemaへmigrationを適用し、recordを追加・削除します。`TEST_DATABASE_URL` には
必ず専用の破棄可能なDBを指定し、共有環境、staging、production、保存が必要なローカルDBへ向けません。
testが途中で失敗した場合はrecordが残る可能性があります。

## Test

- 通常のtestはDB不要とし、必要に応じてrepository stubを使用します。
- DBが必要なtestは `#[ignore]` とし、明示的なintegration testとして分離します。
- 共有OpenAPIにexampleがあるresponseは、`openapi/examples/` のJSONを `include_str!` で直接読み、
  responseと比較します。fixtureを `server/` 内へ複製しません。
- API testではstatus、`Content-Type`、JSON bodyを確認し、`204` はbodyが空であることも検証します。
- 認証mode、Cookieの設定・削除、主要なerror、repositoryへ渡した値と状態変更も変更範囲に応じて
  検証します。
- UUIDや時刻などの動的値を検証対象から外さず、固定、parse、正規化のいずれかで意味を確認します。
- repositoryやmigrationを変更した場合は、通常testに加えてintegration testを追加または更新します。

## コマンドと完了条件

通常のコマンドはリポジトリルートから `mise` 経由で実行します。

```sh
mise run build
mise run fmt
mise run fmt-check
mise run lint
mise run test-unit
mise run server-check
```

`mise run server-check` はDB不要のformat確認、build、Clippy、testをまとめて実行します。

変更内容に応じて、次も実行します。

- migration・SQLx query: 破棄可能なDBで `mise run test-integration`
- OpenAPI契約: `mise run openapi-generate` と関連test
- Dockerfile・起動設定: `mise run docker-build`

`Cargo.lock` は共有対象です。意図のない `cargo update` や広範囲の依存更新を行わず、通常のbuild、
lint、testではlockfileを尊重します。

実行できなかった検証がある場合は、commandと理由を報告します。
