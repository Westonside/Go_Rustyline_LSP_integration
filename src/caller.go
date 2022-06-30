package main

//#cgo LDFLAGS: -L${SRCDIR}/../target/debug -lcaller -ldl
//#include <stdint.h>
//extern void calling_func();
import "C"
import "fmt"

func main() {
	fmt.Println("starting the ffi")
	C.calling_func()

	//grpc listener: on go side connect with grpc client rust side
	//spawn process set go stdin to rust stdin and stdout to out
	//spawn calling func in a process


	// input := 2
	// double := C.double_input(C.int32_t(input))
	// fmt.Printf("%d * 2 = %d\n", input, double)
}
