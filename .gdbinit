define file-debug
file ./target/riscv64gc-unknown-none-elf/debug/rus_pintos
end
document file-debug
add symbol file of debug target
end

define file-release
file ./target/riscv64gc-unknown-none-elf/release/rus_pintos
end
document file-release
add symbol file of release target
end

define debug-qemu
target remote localhost:1234
end
document debug-qemu
attach to qemu (localhost:1234) and debug the kernel
end
