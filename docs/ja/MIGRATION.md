# 救出ガイド

## まず確認すること

救出する対象は、次の四つから選べます。

- ChatGPTの公式データエクスポートZIP
- Atlas Browser Memories画面で、自分で選択してコピーしたテキスト
- ブラウザからエクスポートしたブックマークHTML
- 自分で作ったURL一覧テキスト

ログインCookie、ブラウザ履歴DB、キーチェーン、アプリの内部DBは対象外です。そこを直接読む実装は、移行の再現性と安全性を壊すため追加しません。

## 実行

```bash
cargo run -- doctor
cargo run -- import chatgpt-export ~/Downloads/chatgpt-export.zip
pbpaste | cargo run -- import atlas-memory-text
cargo run -- import bookmarks ~/Downloads/bookmarks.html
cargo run -- import urls ~/Downloads/urls.txt
cargo run -- report
```

各取り込みの直後に `audit/latest-report.md` が更新されます。さらに `audit/runs/` に、取り込み単位の実行ログがMarkdownで蓄積されます。ログには時刻、ソース種別、件数、カバレッジ、安全確認を記録しますが、本文・URL・元ファイルの場所は書きません。

別タブでは次のコマンドで進行状況を追えます。`Ctrl-C` は監視を止めるだけで、救出データには影響しません。

```bash
./scripts/watch-audit.sh
```

## 今は移さないもの

この初期版は救出と監査までです。候補をCodexのネイティブメモリへ自動投入しません。公式に安定したインポートAPIがない状態で直接書くと、将来の更新で失われたり、命令として誤作動したりするためです。
