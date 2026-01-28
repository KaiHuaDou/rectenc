alias te := test-encode
alias td := test-decode
alias ta := test-all

test-encode:
	cargo run --release -- encode BA.webm BA.rcts --preview

test-decode:
	cargo run --release -- decode BA.rcts BA.mkv

test-all:
	just test-encode
	just test-decode
