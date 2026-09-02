# AGENTS.md

このファイルは、リポジトリ全体に適用するコーディングエージェント向けの共同作業ガイドです。
`client/` と `server/` には領域固有の `AGENTS.md` があります。変更対象に近い指示を優先し、
セットアップや運用の詳細は各 `README.md` も確認してください。

## 作業原則

- 原則として日本語で簡潔に報告します。ユーザーが別の言語を指定した場合は従います。
- 調査・説明・レビューの依頼では、明示的な編集依頼がない限りファイルを変更しません。
- 作業前に `git status --short`、対象ファイル、関連する `AGENTS.md` とREADMEを確認します。
- ユーザーや他の作業者による既存差分を保持し、依頼と無関係な変更を混ぜません。
- 依頼に必要な場合を除き、依存関係、lockfile、生成物、build artifactを更新しません。
- commit、push、PR作成、デプロイ、外部サービスの変更は、明示的に依頼された場合だけ行います。
- 未確定事項を推測で契約や実装へ追加しません。`openapi/README.md` の「未確定事項」も確認します。
- 秘密情報、実在するcredential、ローカル専用の `.env` ファイルをcommitしません。

## リポジトリ構成

- `client/`: Vue 3、Vite、TypeScriptによるフロントエンド
- `device/`: Raspberry Pi Pico H向けMicroPython firmwareと実機確認資料
- `server/`: Rust 2024、Axum、Tokio、SQLxによるAPIサーバー
- `openapi/`: フロントエンドとサーバーが共有するOpenAPI 3.1契約、JSON fixture、scenario
- `mise.toml`: tool versionと標準taskの正本
- `compose.yaml`: server、開発用MariaDB、AdminerのCompose定義
- `Dockerfile`: serverのproduction image。build contextはリポジトリルート

## 標準コマンド

通常のsetup、build、format、lint、testはリポジトリルートから `mise` 経由で実行します。
個別の `cargo` や `pnpm` コマンドより、`mise.toml` に定義されたtaskを優先してください。

```sh
mise trust
mise install

mise run server-check
mise run client-check
mise run device-test
mise run check
```

- `mise run server-check` はDB不要のserver検証をまとめて実行します。
- `mise run client-check` はformat確認、typecheck、build、Histoire build、lint、unit testを実行します。
- `mise run device-test` はhardware不要のfirmware状態machine unit testを実行します。
- `mise run check` は上記3つを実行しますが、DB integration test、OpenAPI生成差分検査、Docker buildは
  含みません。
- 自動修正を伴うformat・lint taskは、編集が許可された作業でのみ実行します。

## OpenAPIと生成物

API契約と共有テストデータの役割は次のとおりです。

- path、method、schema、status: `openapi/openapi-v1.yaml`
- request／response payload: `openapi/examples/**/*.json`
- 前提条件と状態遷移: `openapi/scenarios/p0-cases.yaml`

これらに不一致がある場合は独断でどれかへ合わせず、差分を明示して確認します。API契約を変更する
場合は、関連するYAML、JSON fixture、scenario、client mock、server testを同じ変更で同期します。

次の生成物は直接編集しません。

- `server/generated/openapi/**`
- `client/src/generated/api.d.ts`
- `client/public/mockServiceWorker.js`

契約や生成方法の変更は、`openapi/openapi-v1.yaml`、`openapi/openapitools.json`、
`openapi/templates/` などの生成元へ反映し、リポジトリルートで再生成します。

```sh
mise run openapi-generate
```

`mise run openapi-generate-check` は生成物を上書きしてからGit差分を検査するCI向けtaskです。
生成先に保護すべき未コミット差分がない、差分なしを期待する状態でのみ実行してください。
既存差分を消すためにresetやcheckoutを使用しません。

契約上のendpointが実装済みとは限りません。変更前にOpenAPI、serverのRouter、clientのMSW handlerを
照合して実装範囲を確認します。

## DB・Composeの安全条件

- 開発用ComposeとCIはMariaDB 11.8を使用します。別環境向けの互換性は対象環境で確認します。
- SQLxはMariaDBにも `mysql://` URLを使用します。
- serverは起動時にmigrationを自動適用します。起動前に接続先DBを確認してください。
- DB integration testはmigrationとデータ変更を行うため、破棄可能な専用DBだけに向けます。
- `compose.yaml` には永続volumeがありません。containerを削除すると開発用DBのデータも失われます。
- 自分が起動したものではない開発環境を、明示的な依頼なしに停止しません。

## 変更後の検証

変更内容に応じて、少なくとも次を実行します。

| 変更範囲 | 最低限の検証 |
| --- | --- |
| 文書のみ | diff、リンク、command、記載内容の確認 |
| `server/` | `mise run server-check` |
| `client/` | `mise run client-check` |
| `device/` | `mise run device-test` と、変更内容に対応する実機確認 |
| repository・migration | server checkと、破棄可能なDBで `mise run test-integration` |
| OpenAPI契約 | `mise run openapi-generate` と関連するclient／server test |
| Dockerfile・起動構成 | `mise run docker-build` |
| 横断変更 | `mise run check` と変更内容に必要な個別検証 |

完了前に次を確認します。

```sh
git status --short
git diff --check
```

完了報告には、変更内容、主な変更ファイル、実行した検証と結果、未実行の検証と理由、残っている
制約を記載します。
