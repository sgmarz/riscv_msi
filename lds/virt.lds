OUTPUT_ARCH( "riscv" )
ENTRY(_start)
MEMORY
{
  ram  (rwx) : ORIGIN = 0x80000000, LENGTH = 32M
}

PHDRS
{
  dyndata PT_LOAD;
  text PT_LOAD;
  data PT_LOAD;
  rodata PT_LOAD;
  bss PT_NULL;
}

SECTIONS
{
  PROVIDE(_memory_start = ORIGIN(ram));
  PROVIDE(_memory_end = _memory_start + LENGTH(ram));

  .text : {
    PROVIDE(_text_start = .);
    *(.text.init) *(.text .text.*)
    PROVIDE(_text_end = .);
  } >ram AT>ram :text

  . = ALIGN(8);
  PROVIDE(__global_pointer$ = .);

  .bss : ALIGN(4096) {
    PROVIDE(_bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  } >ram AT>ram :bss
  
  .rodata : ALIGN(4096) {
    PROVIDE(_rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(_rodata_end = .);
  } >ram AT>ram :rodata

  .data : ALIGN(4096) {
    . = ALIGN(4096);
    PROVIDE(_data_start = .);
    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(_data_end = .);
  } >ram AT>ram :data

  .eh_hdr : {
    *(.eh*)
  } >ram AT>ram :data

  /* We need to make sure that the stack and heap are aligned by
   a page size, which for Risc-V (and most architectures) is 4096.
  */
  . = ALIGN(4096);

  PROVIDE(_stack_start = .);
  PROVIDE(_stack_end = _stack_start + 8K * 8);
  PROVIDE(_heap_start = _stack_end);
  PROVIDE(_heap_end = _memory_end);
}
