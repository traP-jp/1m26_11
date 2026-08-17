# AGENTS.md

このファイルは、リポジトリ全体に適用するコーディングエージェント向けの共同作業ガイドです。
領域固有の規約は下位ディレクトリの `AGENTS.md`、セットアップや運用の詳細は各 `README.md` に
記載します。

## 指示の優先順位

判断に迷った場合は、次の順で確認します。

1. 現在のユーザー指示
2. 変更対象に最も近い `AGENTS.md`
3. 上位ディレクトリの `AGENTS.md`
4. 確定済みの設計資料とAPI契約
5. 各 `README.md` と補足資料

矛盾を見つけた場合は、独断で一方を採用せず、差分と影響範囲を報告して確認します。
ユーザーへの報告は原則として日本語のお嬢様言葉で行い、別の言語や文体を指定された場合は
その指定に従います。

## プロジェクトの不変条件

- フロントエンドは Vue、バックエンドは Rust/Axum、データベースは MariaDB を使用します。
- フロントエンドとバックエンドのテンプレートは導入済みです。再生成しません。
- Web Serial APIの生データはフロントエンドでゲーム操作へ変換します。バックエンドは生の
  シリアルデータを扱いません。
- キーボードまたは画面ボタンによる代替入力でも、実機入力と同じゲーム処理経路を確認できる
  状態を維持します。
- Aはフロントエンド、Bはバックエンドのレビュー担当です。API契約の変更はA・B双方、通し動作・
  統合・NeoShowcaseへの配置はCが確認します。
- A・B・Cは役割名です。実名やGitHub userとの対応を推測しません。
- Cには統合とデプロイの時間を残します。統合で見つかった不具合は、原則として該当領域の担当へ
  戻します。

確定・共有済みの事項を、新しい根拠や変更依頼なしに再議論しません。未確定事項を推測で固定せず、
実装判断に必要になった時点で確認します。

## 現在の作業ゲート

ユーザーから開発開始が明示されるまでは、次の操作を行いません。

- ソースコードの作成・変更
- プロジェクトの初期化やテンプレートの再生成
- 依存パッケージの導入・更新
- migrationの作成・適用
- デプロイや外部サービスの変更
- MariaDBの本番version、NeoShowcaseでの接続方法、シリアル仕様の確定

計画書、Issue案、設計メモなどの文書は、依頼された範囲で作成・更新できます。

## 正本と参照資料

| 対象 | 正本・参照先 | 扱い |
| --- | --- | --- |
| プロジェクト方針・日程 | `集会共有内容と開発スケジュール.md` | 確定済み事項を優先する |
| API契約 | `API定義案.md` | ファイル名に「案」とあるが、現在の確定済み契約として扱う |
| Issueの依存・優先度・担当 | `Issue分割案.md` | 実際のIssueや最新の割当資料がある場合は併せて確認する |
| 機械可読なAPI表現 | `openapi/openapi-v1.yaml` | 現行契約と同期させる。独断で契約を変更しない |
| 共有payload例 | `openapi/examples/**/*.json` | OpenAPIに適合するfixture。schemaを上書きしない |
| 前提条件・状態遷移 | `openapi/scenarios/p0-cases.yaml` | fixtureを使うscenario。API構造を上書きしない |
| tool version・開発task | `mise.toml` | 実在するversionとtask名の正本 |

OpenAPIを契約の正本へ移行する場合は、ユーザーの明示指示とA・Bの確認結果を記録します。それまでは
`API定義案.md` を仕様判断の正本とします。API定義、OpenAPI、fixture、scenarioに不一致がある場合は
作業を止め、契約差分として共有します。

## 作業開始前

1. 依頼の目的、変更対象、完了条件を確認します。
2. `git status --short` と対象ファイルを確認し、既存差分を保持します。
3. 対象に最も近い `AGENTS.md`、関連する正本、READMEを読みます。
4. 実在する構成、command、型を確認します。文書の例だけを根拠に新しい構成を作りません。
5. 資料に未確定と明記された事項が完了条件へ影響する場合は、実装前に確認します。

既存契約に定義済みのstatusやerrorを未確定扱いしません。現在の主な確認対象には、表示名の入力規則、
Cookieの詳細、最初・次のproblem IDの決定方法、asset配信、NeoShowcaseのDB条件、シリアル仕様があります。

調査、説明、レビューの依頼では、明示的な編集依頼がない限りファイルを変更しません。

## 変更の原則

- 依頼に必要な最小範囲だけを変更します。
- ユーザーや他の作業者の差分を保持し、無関係な整形、リファクタリング、依存更新を混ぜません。
- 依頼に必要な場合を除き、lockfile、生成物、build artifactを更新しません。
- `git reset --hard` や `git checkout --` などで既存差分を消しません。
- commit、push、PR作成、デプロイ、外部サービスへの公開は、明示的に依頼された場合だけ行います。
- 秘密情報、`.env.local`、実在するcredentialをcommitしません。
- setup、build、format、lint、testは、原則としてリポジトリルートから `mise` task経由で行います。
- `mise trust`、`mise install`、依存取得を伴うtaskは、setupまたは実装が許可された作業でのみ
  実行します。
- 自動修正を伴うformat・lint taskは、編集が許可された作業でのみ実行します。

## Issueの扱い

Issueには、少なくとも分類、優先度、重さ、依存先、完了条件、期限、担当者、レビュー担当を記載します。

- 優先度はP0、P1、P2、重さはXS、S、Mを使用します。L相当は着手前に分割します。
- 依存先が完了しているIssueだけをReadyとし、Readyの中ではP0を優先します。
- 1人が同時に持つ実装Issueは原則1件までです。
- 2時間以上詰まった場合は、定例を待たず共有します。
- 14日目は最終動作確認に専念し、新機能を追加しません。

## APIと生成物

次の生成物は直接編集しません。

- `server/generated/openapi/**`
- `client/src/generated/api.d.ts`
- `client/public/mockServiceWorker.js`

API契約の変更は、ユーザーの明示指示とA・Bの確認がある場合だけ行います。影響するOpenAPI YAML、
JSON fixture、scenario、フロントエンドのmock、バックエンドのtestを同じ変更で同期し、既存taskで
境界コードを再生成します。

修正はOpenAPI YAML、generator設定、templateなどの生成元へ反映し、生成先だけを手作業で直しません。
フロントエンドのmockは、リポジトリで採用済みの方法から共有fixtureを利用し、同じpayloadを別の
TypeScript objectへ複製しません。バックエンドのHTTP境界では生成型を使用し、同等のrequest／
response型を重複定義しません。

`mise run openapi-generate-check` は、名前に `check` が含まれていても生成物を上書きします。編集が
許可され、生成先に保護すべき未コミット差分がない場合だけ実行します。実行後の差分を消すために
resetやcheckoutを使いません。

契約上のendpointが実装済みとは限りません。着手前にserverのRouter、フロントエンドのMSW handler、
OpenAPI契約を照合して実装範囲を確認します。

## 領域別の要点

### Server

- Rust edition、MSRV、lint設定は実際のtoolchain・manifest・`mise.toml` を正本とします。
- productionの通常経路では、外部入力、DB、I/Oの失敗に `unwrap` や `expect` を使わず、既存の
  error型で処理します。
- API testではstatus、content type、bodyを確認し、`204` はbodyが空であることも検証します。
- UUIDや時刻などの動的値も、固定、parse、正規化のいずれかで意味を検証します。
- main branchへ取り込まれたmigration、または共有DBへ適用済みのmigrationは書き換えず、新しい
  migrationで変更します。

### Client

- Vue SFC、TypeScript、test、story、import、formatは既存構成と設定ファイルに従います。
- TypeScriptのstrictな型検査を維持し、API型は生成された型から参照します。
- 見た目や状態を持つcomponentを変更した場合は、対応するtestとstoryへの影響を確認します。
- production buildでMSWを有効にしません。
- 意図的に固定された依存は、更新が依頼の目的でない限り変更しません。

詳細な配置・命名・環境変数・既知の警告は、`server/AGENTS.md`、`client/AGENTS.md`、
`openapi/AGENTS.md` と各READMEへ分けます。

## DB・ローカル環境の安全条件

- MariaDBとSQLxでは、UUIDを `BINARY(16)`、日時をUTCの `TIMESTAMP(3)`、JSON、InnoDB、utf8mb4を
  使用します。
- PostgreSQL向けの型、部分index、DDLや、MariaDB固有のENUMへ独断で置き換えません。
- ローカル・CIでMariaDB 11.8を使用していても、NeoShowcaseの本番versionとはみなしません。
- server起動時にmigrationが自動適用される構成では、起動taskもDBへの書込みを伴います。接続先を
  確認せずに実行しません。
- DB integration testは破棄可能な専用MariaDBだけに向けます。共有DBやproduction DBへ実行しません。
- MariaDBでもSQLxの接続URL schemeは `mysql://` です。
- `compose-down` がcontainerやローカルDBデータを削除する構成では、停止前に影響を確認します。
- 自分がこの作業で起動したComposeだけを停止し、既に起動していた環境は明示依頼なしに停止しません。

## 変更後の検証

taskが実在することを `mise.toml` で確認し、変更対象に対応する検証を実行します。

| 変更範囲 | 最低限の検証 |
| --- | --- |
| 文書のみ | diff、リンク、command、記載内容の確認 |
| serverのみ | `mise run server-check` |
| clientのみ | `mise run client-check` |
| DB repository・migration | server checkと、破棄可能なDBで `mise run test-integration` |
| OpenAPI契約 | 再生成、関連するserver/client test、条件を満たす場合のみ生成差分検査 |
| 横断変更 | `mise run check` と変更内容に必要な個別検証 |

`mise run check` にDB integration test、OpenAPI生成差分検査、Docker buildが含まれるとは限りません。
task定義を確認し、必要な検証を追加します。

完了前に `git status --short` と `git diff --check` を確認し、意図しない生成物、lockfile、build
artifactの差分がないことを確認します。

## 完了報告

完了時は、次を簡潔に報告します。

- 何を変更したか
- 主な変更ファイル
- 実行した検証commandと結果
- 未実行の検証と理由
- 残っている未確定事項、既知の制約、レビュー担当

変更を伴わない調査・レビューでは、確認した範囲、結論、その根拠を報告します。
