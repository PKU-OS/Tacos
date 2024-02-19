MAKEFLAGS := --jobs=$(shell nproc)

CARGO := cargo run -q
FILTER := grep -E "[0-9]+ ms" --color=never
BUILD_DIR := build

all: $(BUILD_DIR)/disk.img

include user/*.mk

$(BUILD_DIR)/mkfs: mkfs.c
	gcc -o $@ mkfs.c

$(BUILD_DIR)/disk.img: $(TARGETS) $(BUILD_DIR)/mkfs $(BUILD_DIR)/sample.txt
	cd $(BUILD_DIR)/ && ./mkfs

run: all
	$(CARGO) --release -F test | $(FILTER)

run-gdb: all
	$(CARGO) -- -s -S

test-%: all
	$(CARGO) --features $@ | $(FILTER)

gdb-%: all
	$(CARGO) --features test-$* -- -s -S

clean:
	rm -rf $(BUILD_DIR)

format:
	cargo fmt
	find . -type f -name "*.c" -o -name "*.h" -exec clang-format -i {} +

test-user-%: $(TARGETS) $(BUILD_DIR)/disk.img
	$(CARGO) --features test-user -- -append "$*"

gdb-user-%: $(TARGETS) $(BUILD_DIR)/disk.img
	$(CARGO) --features test-user -- -append "$*" -s -S

test-user-debug-%: $(TARGETS) $(BUILD_DIR)/disk.img
	$(CARGO) --features test-user,debug -- -append "$*"
