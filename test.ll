; ModuleID = 'test'
source_filename = "test"

@0 = private unnamed_addr constant [13 x i8] c"Ol\C3\A1, mundo!\00", align 1

declare i32 @puts(ptr)

define i32 @main() {
entry:
  %0 = call i32 @puts(ptr @0)
  ret i32 0
}
