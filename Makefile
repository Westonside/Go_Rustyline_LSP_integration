all: target/debug/libcaller.a
	go run src/caller.go

target/debug/libcaller.a: src/lib.rs Cargo.toml
	cargo build

clean:
	rm -rf target
