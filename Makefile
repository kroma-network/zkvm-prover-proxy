build-prover-proxy:
	cargo build --release --bin prover-proxy

run-prover-proxy:
	END_POINT ?= "0.0.0.0:3031"
	PROVER_DB ?= "/tmp/prover_db"

	$(MAKE) build-prover-proxy
	./target/release/prover-proxy --endpoint $(END_POINT) --data $(PROVER_DB)