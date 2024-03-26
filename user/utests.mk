INC_DIR := user/lib
BUILD_DIR := build
SRC_DIRS := user/userprogs user/vm

TOOLPREFIX := riscv64-unknown-elf-
CC := $(TOOLPREFIX)gcc
CFLAGS := -Wall -Werror -O
CFLAGS += -ffreestanding -fno-common -nostdlib -mno-relax
CFLAGS += -I $(INC_DIR)
LD := $(TOOLPREFIX)ld
LDFLAGS := -T $(INC_DIR)/user.ld
OBJDUMP := $(TOOLPREFIX)objdump

INCS := $(shell find $(INC_DIR) -name '*.c' -or -name '*.pl')
INCS := $(INCS:%.c=$(BUILD_DIR)/%.o)
INCS := $(INCS:%.pl=$(BUILD_DIR)/%.o)
SRCS := $(shell find $(SRC_DIRS) -name '*.c')
OBJS := $(SRCS:%.c=$(BUILD_DIR)/%.o)
TARGETS := $(OBJS:%.o=%)

TEST_DIR := $(BUILD_DIR)/user

$(BUILD_DIR)/%.S: %.pl
	perl $^ > $@

$(BUILD_DIR)/%.o: $(BUILD_DIR)/%.S
	$(CC) $(CFLAGS) -c $< -o $@

$(filter-out %/usys.o,$(INCS)):$(BUILD_DIR)/%.o:%.c
	@ mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJS):$(BUILD_DIR)/%.o:%.c
	@ mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

$(TARGETS):%:%.o $(INCS)
	$(LD) $(LDFLAGS) -o $@ $^
	$(OBJDUMP) -S $@ > $*.asm
	$(OBJDUMP) -t $@ | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > $*.sym

.PHONY: $(BUILD_DIR)
$(BUILD_DIR):
	@ mkdir -p $(BUILD_DIR)

$(TEST_DIR)/sample.txt:
	#! Ensure sample.txt is stored with LF in disk.img
	tr -d '\r' < user/userprogs/sample.txt > $(TEST_DIR)/sample.txt

$(TEST_DIR)/zeros:
	dd if=/dev/zero of=$(TEST_DIR)/zeros bs=8192 count=1

