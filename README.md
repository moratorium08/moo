## 概要

golfっぽい使いやすさを多少考慮したesolang。妥協で作った。似た言語ありそう。適当に書いたので仕様どおりになっていなかったらごめんなさい

## 構造

stackが2つあり、データ操作に使える。mainスタックは、関数間で共通、sub stackは、関数に対して一つ割り当てられる

## 空白改行

無視される

## 関数

数字で名前付けし、呼び出しが可能。ネスト可能。ある種のscopeがある。

```
0{code}
1{code}
2{code0{code}}
```

のように `{` と `}` で囲う。

## 命令

| 命令 | 内容 |
| --- |  -- |
| t | main stackからpop|
| ) | main stackの先頭をpopしsub stackにpush |
| ( | sub stackの先頭をmain stackにpush |
| a | sub stackのすべての要素をmain stackにpush allする |
| r | main stackの先頭3つをrotateする |
| s | main stackの先頭2つをswapする |
| + | pop a; pop b; push a + b |
| - | pop a; pop b; push a - b |
| * | pop a; pop b; push a * b |
| / | pop a; pop b; push a / b |
| i | 1 byte入力。数字 |
| o | stackの先頭にある値を出力。改行はつかない。1 byteとは限らない |
| n | stackの先頭にある値を数字として出力 |
| c | stackの先頭にある値の番号の関数にジャンプする |
| > | pop a; pop b; if a > b then push 1 else push 0 |
| < | pop a; pop b; if a < b then push 1 else push 0 |
| = | pop a; pop b; if a = b then push 1 else push 0 |
| b | pop a; pop b; pop c; if c != 0 then call a else call b |
| 数字列 | 連続するできる限り長い数字をソースコードからとりその値をmain stackにpush |

--- まだ ---

| l | 空リストをpush |
| p | pop x; pop l; l.append(x) |
| f |  pop
| m | pop f; pop l; map(f, l) （分かって） (fは関数の名前）|

