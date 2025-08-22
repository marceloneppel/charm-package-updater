example-rock:
	cargo-sort && cargo fmt && cargo run -- \
		--ppa neppel/ppa \
		--package patroni=4.0.6-0ubuntu0.24.04.1~ppa1 \
		--snaprepo https://github.com/marceloneppel/charmed-postgresql-snap \
		--snaprepobranch 16/edge \
		--rockrepo https://github.com/marceloneppel/charmed-postgresql-rock \
        --rockrepobranch 16-24.04 \
		--charmrepo https://github.com/marceloneppel/postgresql-k8s-operator \
		--charmrepobranch 16/edge
example-snap:
	cargo-sort && cargo fmt && cargo run -- \
		--ppa neppel/ppa \
		--package patroni=4.0.6-0ubuntu0.24.04.1~ppa1 \
		--snaprepo https://github.com/marceloneppel/charmed-postgresql-snap \
		--snaprepobranch 16/edge \
		--charmrepo https://github.com/marceloneppel/postgresql-operator \
		--charmrepobranch 16/edge
help:
	cargo-sort && cargo fmt && cargo run -- --help
run:
	cargo-sort && cargo fmt && cargo run
