## Improvement pending

- [x] error handling
- [-] handling gama value write and read
	- [x] write
	- [-] read — read 4 bytes as a decimal number is not bad at this point. So put it aside for now.
- [ ] add gama atom
- [x] info print (colr atom, gama value...) — It should not be our focus. For info printing, users can use Mediainfo instead.
- [ ] write tests
- [ ] improve the looking of `--help` message printing (tweak clap).
- [x] add option for not overwriting the original file but instead generate a copy, then do the colr atom, gama atom and frames modification things.

---

1-1-1 => 1-2-1:

- [-] add gama atom
	- [-] add gama value: 2.4, 2.2, etc.
- [x] change transfer function index to 2 (unspecified)

1-2-1 => 1-1-1 or others:

- [-] remove gama atom (if present).
- [x] change transfer function index to 1

## Behaviors

- -g 设为 0 => 去掉 gama atom（的影响），实际上没有完全去掉 gama atom。
- omit -g => program 将会使用默认的 gama value，-1.0。encode function 则会忽略 gama value 等于 -1.0 的情况，不对原始 gama value 做任何改动。leave it as it it.
- -g 支持负数，比如 `--gama-value=-2.4`，将会把 gamma 值设为 -2.4。只是应该没人会这么用。

```
cargo run --release -- -i <file_path> -p 1 -t 1 -m 1 -g 0
```

## Atom hierarchy

- ftyp
- wide
- mdat
- moov:
  - mvhd
  - trak:
    - tkhd (track header atom)
    - edts
      - elst
    - tref
      - tmcd
    - mdia (media atom):
      - mdhd
      - hdlr
      - minf:
        - vmhd
        - hdlr
        - dinf
          - dref
        - stbl:
          - stsd:
            - apcn / avc1 / hev1
              - fiel
              - colr
              - pasp
              - gama ()
          - stts
          - stsc
          - stsz
          - stco
  - trak:
  	- tkhd
  	- edts
  	- mdia
    	- mdhd
    	- hdlr
    	- minf
      	- gmhd:
        	- gmin
        	- text
        	- tmcd
      	- hdlr
      	- dinf:
          	- dref
        	- stbl:
          	- stsd
          	- stts
          	- stsc
          	- stsz
          	- stco
	- udta:
	- meta:
- free
