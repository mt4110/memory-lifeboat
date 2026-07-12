# Memory Lifeboat

AIサービスの記憶やブラウザ文脈を、自分で持ち運べる形に救出するためのローカルファーストなMac向けツールです。別サービスが生成した「内部メモリ」を正本とはみなさず、利用者が選んだデータだけを暗号化して保管します。

最初の版で扱うものは、ChatGPTの公式エクスポートZIP、利用者が選んで貼り付けたAtlas Browser Memoriesのテキスト、ブックマーク、URL一覧です。保存された候補はすべて「観測データ」であり、ツールへの命令として実行されません。

これはしません。

- Cookie、セッション、キーチェーン内のブラウザ秘密情報、Atlas内部DB、ブラウザプロファイルを読まない
- `~/.codex/memories` を含むネイティブメモリに直接書き込まない
- ネットワークへデータを送らない

詳しい救出手順は [日本語ガイド](docs/ja/MIGRATION.md)、英語版は [English guide](docs/en/MIGRATION.md) を参照してください。

## はじめかた

```bash
cargo run -- doctor
cargo run -- import chatgpt-export ~/Downloads/chatgpt-export.zip
pbpaste | cargo run -- import atlas-memory-text
cargo run -- import bookmarks ~/Downloads/AtlasBookmarks.html
cargo run -- report
```

By default, the store lives at:

```text
~/Library/Application Support/Memory Lifeboat/
```

Use `--store ./store` for local testing.

## 保管場所と設計

標準の保管場所は `~/Library/Application Support/Memory Lifeboat/` です。元ファイルと候補レコードはmacOSキーチェーンの鍵で暗号化します。マニフェストと監査レポートには、どの種類のデータを何件観測したかだけを残します。

AIサービスの記憶はキャッシュとして扱い、利用者が管理する暗号化ストアを正本にします。将来のアダプタは、承認済みデータだけを `AGENTS.md`、フック、読み取り専用MCPのような安定した接点へ投影します。

## English

Memory Lifeboat is a local-first Mac tool for rescuing AI product memories and browser context into a user-owned encrypted store. It only imports data you explicitly select, never treats imported text as executable instruction, and never reads browser secrets or writes to native memory stores.

See the [English migration guide](docs/en/MIGRATION.md) and [architecture notes](docs/ARCHITECTURE.md).
