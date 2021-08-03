A Rust implementation of
https://flownet.com/ron/papers/lisp-java/instructions.html.  As
recently popularised by @renatoathaydes.

Mainly exploring the performance gains possible through avoiding
recursion and keeping memory allocations fairly tightly controlled.

Seems to compare favourably to Renato's
`fast-rust-no-bigint-buffer-stdout` branch:

## Test setup

    $ curl -s 'https://raw.githubusercontent.com/dwyl/english-words/master/words.txt' | tail -100000 > words.txt

     $ wc -l words.txt
     100000 words.txt

     $ cat phones_1_len_29.txt
     91760687651618841752033181652

## Poorly controlled benchmark results

### fast-rust-no-bigint-buffer-stdout version (00:01:44)

     $ time ./phone_encoder words.txt phones_1_len_29.txt | wc -l
     201009600
     ./phone_encoder words.txt phones_1_len_29.txt  101.97s user 2.53s system 99% cpu 1:44.51 total

### This version (00:00:19)

     $ time ./rust-laatj words.txt phones_1_len_29.txt | wc -l
     201009600
     ./rust-laatj words.txt phones_1_len_29.txt  16.85s user 2.57s system 99% cpu 19.426 total
