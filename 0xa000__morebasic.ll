; ModuleID = '0xa000__morebasic'
source_filename = "./language/polkavm/examples/basic/sources/morebasic.move"
target datalayout = "e-m:e-p:32:32-i64:64-n32-S128"
target triple = "riscv32-unknown-none-elf"

module asm ".pushsection .polkavm_exports,\22R\22,@note"
module asm ".byte 1"
module asm ".4byte _ZN170xa000__morebasic3sum8METADATA17h9891cd975c6c747eE"
module asm ".4byte _ZN170xa000__morebasic3sum17h5e64ca7b8cc01bf6E"
module asm ".popsection"

@alloc_91fd7f9047e119a6 = private unnamed_addr constant { [3 x i8] } { [3 x i8] c"sum" }, section ".rodata..Lalloc_91fd7f9047e119a6", align 1
@_ZN170xa000__morebasic3sum8METADATA17h9891cd975c6c747eE = internal constant <{ [9 x i8], ptr, [2 x i8] }> <{ [9 x i8] c"\01\00\00\00\00\03\00\00\00", ptr bitcast (ptr @alloc_91fd7f9047e119a6 to ptr), [2 x i8] c"\02\01" }>, section ".polkavm_metadata", align 1

define i64 @_ZN170xa000__morebasic3sum17h5e64ca7b8cc01bf6E(i64 %0, i64 %1) {
entry:
  %local_0 = alloca i64, align 8
  %local_1 = alloca i64, align 8
  %local_2 = alloca i64, align 8
  %local_3 = alloca i64, align 8
  %local_4 = alloca i64, align 8
  store i64 %0, ptr %local_0, align 8
  store i64 %1, ptr %local_1, align 8
  %load_store_tmp = load i64, ptr %local_0, align 8
  store i64 %load_store_tmp, ptr %local_2, align 8
  %load_store_tmp1 = load i64, ptr %local_1, align 8
  store i64 %load_store_tmp1, ptr %local_3, align 8
  %add_src_0 = load i64, ptr %local_2, align 8
  %add_src_1 = load i64, ptr %local_3, align 8
  %add_dst = add i64 %add_src_0, %add_src_1
  store i64 %add_dst, ptr %local_4, align 8
  %retval = load i64, ptr %local_4, align 8
  ret i64 %retval
}
