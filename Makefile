.RECIPEPREFIX := >
GO        ?= go
CARGO     ?= cargo
BINDGEN   ?= wasm-bindgen
ADDR      ?= :8080

STATIC    := static
OUT       := build

RS_DIR    := apps/gallery-rs
RS_WASM   := $(RS_DIR)/target/wasm32-unknown-unknown/release/gallery_rs.wasm
RS_SRC    := $(shell find $(RS_DIR) -path $(RS_DIR)/target -prune -o -type f -print)

# Hand-written web files: the shared ones in static/ plus static/rust/.
ASSETS    := $(patsubst $(STATIC)/%,$(OUT)/%,$(filter-out $(STATIC)/rust,$(wildcard $(STATIC)/*))) \
             $(patsubst $(STATIC)/rust/%,$(OUT)/%,$(wildcard $(STATIC)/rust/*))

.PHONY: run build vet clean help

## Build everything, then serve.
run: build
>$(GO) run ./apps/server -addr $(ADDR) -dir $(OUT)

## Build outputs only.
build: $(OUT)/gallery_rs.js $(ASSETS)
>@echo build complete: $(OUT)/

## Compile the client. Cargo runs inside the crate so that its
## rust-toolchain.toml picks the pinned toolchain.
$(RS_WASM): $(RS_SRC)
>cd $(RS_DIR) && $(CARGO) build --locked --release --target wasm32-unknown-unknown

## Generate the JS glue (gallery_rs.js) and final gallery_rs_bg.wasm.
$(OUT)/gallery_rs.js: $(RS_WASM)
>$(BINDGEN) --target web --no-typescript --out-dir $(@D) $<

## Copy hand-written static files.
$(OUT)/%: $(STATIC)/rust/%
>@mkdir -p $(@D)
>cp "$<" $@

$(OUT)/%: $(STATIC)/%
>@mkdir -p $(@D)
>cp "$<" $@

## Vet the Go server, then lint the Rust client.
vet:
>$(GO) vet ./...
>cd $(RS_DIR) && $(CARGO) fmt --check
>cd $(RS_DIR) && $(CARGO) clippy --locked --release --target wasm32-unknown-unknown -- -D warnings

## Remove all generated output.
clean:
>rm -rf $(OUT) $(RS_DIR)/target

help:
>@grep -E "^## " $(MAKEFILE_LIST) | sed "s/^## //"
