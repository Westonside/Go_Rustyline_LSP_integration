package main

//#cgo LDFLAGS: -L${SRCDIR}/../target/debug -lcaller -ldl
//#include <stdint.h>
//extern void calling_func();
import "C"
import "fmt"

func main() {
	fmt.Println("starting the ffi")
	C.calling_func()


	// input := 2
	// double := C.double_input(C.int32_t(input))
	// fmt.Printf("%d * 2 = %d\n", input, double)
}
