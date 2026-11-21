.RECIPEPREFIX := >
GO        ?= go
CARGO     ?= cargo
BINDGEN   ?= wasm-bindgen
ADDR      ?= :8080

STATIC    := static
OUT       := build

CRATE     := apps/gallery
WASM      := $(CRATE)/target/wasm32-unknown-unknown/release/gallery.wasm
CRATE_SRC := $(shell find $(CRATE) -path $(CRATE)/target -prune -o -type f -print)

# Hand-written web files, copied as they are.
ASSETS    := $(patsubst $(STATIC)/%,$(OUT)/%,$(wildcard $(STATIC)/*))

.PHONY: run build vet clean help

## Build everything, then serve.
run: build
>$(GO) run ./apps/server -addr $(ADDR) -dir $(OUT)

## Build outputs only.
build: $(OUT)/gallery.js $(ASSETS)
>@echo build complete: $(OUT)/

## Compile the client. Cargo runs inside the crate so that its
## rust-toolchain.toml picks the pinned toolchain.
$(WASM): $(CRATE_SRC)
>cd $(CRATE) && $(CARGO) build --locked --release --target wasm32-unknown-unknown

## Generate the JS glue (gallery.js) and final gallery_bg.wasm.
$(OUT)/gallery.js: $(WASM)
>$(BINDGEN) --target web --no-typescript --out-dir $(@D) $<

## Copy hand-written static files.
$(OUT)/%: $(STATIC)/%
>@mkdir -p $(@D)
>cp "$<" $@

## Vet the Go server, then lint the Rust client.
vet:
>$(GO) vet ./...
>cd $(CRATE) && $(CARGO) fmt --check
>cd $(CRATE) && $(CARGO) clippy --locked --release --target wasm32-unknown-unknown -- -D warnings

## Remove all generated output.
clean:
>rm -rf $(OUT) $(CRATE)/target

help:
>@grep -E "^## " $(MAKEFILE_LIST) | sed "s/^## //"
