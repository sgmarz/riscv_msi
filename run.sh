#!/bin/bash

if [ $# -ne 1 -o ! -x $1 ]; then
    echo "Use cargo run instead of running this directly."
    exit 2
fi

KERNEL=$1

TRACES="pci_nvme*"

PARAMS+=" -nographic"
PARAMS+=" -machine virt,aclint=on,aia=aplic-imsic"
PARAMS+=" -cpu rv32"
PARAMS+=" -d guest_errors,unimp"
PARAMS+=" -smp 1"
PARAMS+=" -m 32M"
# PARAMS+=" -gdb unix:debug.pipe,server,nowait"
PARAMS+=" -serial mon:stdio"
PARAMS+=" -device pcie-root-port,id=bridge1,multifunction=off,chassis=0,slot=1,bus=pcie.0,addr=01.0"
PARAMS+=" -device pcie-root-port,id=bridge2,multifunction=off,chassis=1,slot=2,bus=pcie.0,addr=02.0"
# PARAMS+=" -device pcie-root-port,id=bridge3,multifunction=off,chassis=2,slot=3,bus=pcie.0,addr=03.0"
# PARAMS+=" -device pcie-root-port,id=bridge4,multifunction=off,chassis=3,slot=4,bus=pcie.0,addr=04.0"
PARAMS+=" -device qemu-xhci,bus=bridge1,id=xhci"
PARAMS+=" -device usb-tablet,id=usbtablet"
# PARAMS+=" -device virtio-rng-pci-non-transitional,bus=bridge1,id=rng"
# PARAMS+=" -device virtio-keyboard-pci,bus=bridge1,id=keyboard"
# PARAMS+=" -device virtio-tablet-pci,bus=bridge1,id=tablet"
# PARAMS+=" -device virtio-gpu-pci,bus=bridge2,id=gpu"
# PARAMS+=" -device vhost-vsock-pci-non-transitional,bus=bridge2,guest-cid=9,id=vsock"
# PARAMS+=" -device virtio-blk-pci-non-transitional,drive=hdd1,bus=bridge2,id=blk1"
# PARAMS+=" -device virtio-net-pci-non-transitional,netdev=net1,bus=bridge4,id=net"
PARAMS+=" -drive if=none,format=raw,file=hdd.dsk,id=hdd1"
PARAMS+=" -device nvme-subsys,id=nvmesubsys,nqn=1234"
PARAMS+=" -device nvme,serial=deadbeef,id=nvmehdd,subsys=nvmesubsys,bus=bridge2"
PARAMS+=" -device nvme-ns,drive=hdd1,bus=nvmehdd"
# PARAMS+=" -device nvme,serial=deadbeef,drive=hdd1,bus=bridge2,id=nvmehdd"
# PARAMS+=" -netdev user,id=net1,hostfwd=tcp::35555-:22"

T=""
for t in $TRACES; do
    T+="--trace $t "
done


exec qemu-system-riscv32 \
    ${PARAMS} \
    -bios none \
    $T \
    -kernel $KERNEL
