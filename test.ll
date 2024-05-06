; ModuleID = 'test'
source_filename = "test"

@0 = private unnamed_addr constant [7 x i8] c"Ol\C3\A1, \00", align 1
@1 = private unnamed_addr constant [7 x i8] c"mundo!\00", align 1

declare i32 @puts(ptr)

define i32 @main() {
entry:
  %0 = call i32 @puts(ptr @0)
  %1 = call i32 @puts(ptr @1)
  ret i32 %1
}
