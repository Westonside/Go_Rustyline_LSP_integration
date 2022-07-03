package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
)

// {"jsonrpc":"2.0","method":"textDocument/didChange","params":{"contentChanges":[{"text":"x"}],"textDocument":{"uri":"file:///foo.flux","version":0}}}
type Params struct {
	Changes Changes `json:"params"`
}

type Changes struct {
	Inputs []Input `json:"contentChanges"`
}

type Input struct {
	Text string `json:"text"`
}

func main() {
	print("hello world!")
	reader := bufio.NewReader(os.Stdin)

	for true {
		line, _, _ := reader.ReadLine()
		if !json.Valid(line) {
			println("Not valid json!")
			continue
		}
		println("valid")

		println(string(line))
		var res Params
		json.Unmarshal(line, &res)
		fmt.Printf("%#v", res)
		fmt.Println(res.Changes.Inputs[0].Text)
		//now send the input to the go run time

		// println(params.Text, "testing")
	}
}
