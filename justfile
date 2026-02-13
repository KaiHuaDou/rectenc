alias te := test-encode
alias td := test-decode
alias ta := test-all

test-encode CODE:
	cargo run --release -- encode BAS.mkv tests/BAS_{{CODE}}.rcts -r {{CODE}} -p -a

test-decode CODE:
	cargo run --release -- decode tests/BAS_{{CODE}}.rcts tests/BAS_{{CODE}}.mkv

test-all CODE:
	just test-encode {{CODE}}
	just test-decode {{CODE}}
